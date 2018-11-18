
use std::io::BufRead;
use std::io::{self, Initializer, Error, ErrorKind};
use std::io::{Read, Result, Seek, SeekFrom};
use std::str;

use input::LEvent;
use line_num_cache::{LineNum, LineNumCache};
use util;
use string_util;


const DEFAULT_BUF_SIZE: usize = 4096;

pub struct BiBufReader<R> {
    inner: R,
    buf: Box<[u8]>,
    pos: usize,
    cap: usize,
}

impl<R: Read + Seek> BiBufReader<R> {
    pub fn new(inner: R) -> BiBufReader<R> {
        BiBufReader::with_capacity(DEFAULT_BUF_SIZE, inner)
    }

    pub fn with_capacity(cap: usize, inner: R) -> BiBufReader<R> {
        unsafe {
            let mut buffer = Vec::with_capacity(cap);
            buffer.set_len(cap);
            inner.initializer().initialize(&mut buffer);
            BiBufReader {
                inner,
                buf: buffer.into_boxed_slice(),
                pos: 0,
                cap: 0,
            }
        }
    }

    pub fn jump_percentage(&mut self, percent: u64) -> Result<()> {
        let pos = self.seek_percent(percent)?;

        self.down_n_lines(1)
    }

    pub fn up_n_lines(&mut self, n: usize) -> Result<()> {
        let buf = self.make_buf_up()?;
        let size = buf.len();

        unsafe {
            let (screen_width, _) = util::screen_width_height();

            let start = 0;
            let offset = size as i64 - start as i64 - string_util::nth_last_newline_wrapped(
                n + 1,
                str::from_utf8_unchecked(&buf[start..]),
                screen_width as usize,
            ) as i64;
            self.seek(SeekFrom::Current(-(offset as i64)))?;
        }

        Ok(())
    }

    pub fn down_n_lines(&mut self, n: usize) -> Result<()> {
        let (buf, size) = self.make_buf_down()?;

        let (screen_width, _) = util::screen_width_height();

        unsafe {
            if let Some(newline_offset) = string_util::nth_newline_wrapped(
                n,
                str::from_utf8_unchecked(&buf),
                screen_width as usize,
            ) {
                self.seek(SeekFrom::Current(newline_offset as i64))?;
            }
        }

        Ok(())
    }

    pub fn page(&mut self) -> Result<Vec<u8>> {
        let size = self.page_size();

        let mut page_buf = Vec::with_capacity(size);
        page_buf.resize(size, 0);

        let mut bytes_read: i64 = 0;
        bytes_read = match self.read(&mut page_buf[..]) {
            Err(e) => {
                eprintln!("errorrrr {}", e);
                0
            }
            Ok(s) => s as i64,
        };

        self.seek(SeekFrom::Current(-bytes_read))?;

        Ok(page_buf)
    }

    fn make_buf_up(&mut self) -> Result<Vec<u8>> {
        let size = std::cmp::min(
            self.search_buf_size(),
            self.seek(SeekFrom::Current(0))? as usize,
        );

        let mut buf = vec![0; size];
        let new_pos = self.seek(SeekFrom::Current(-(size as i64)))?;

        let bytes_read = self.read(&mut buf[..size])?;
        //debug_assert!(bytes_read as u64 == size as u64);

        Ok(buf)
    }

    fn make_buf_down(&mut self) -> Result<(Vec<u8>, usize)> {
        let size = self.search_buf_size();
        let mut buf = vec![0; size as usize];

        let bytes_read = self.read(&mut buf)?;

        if bytes_read == 0 {
            return Err(Error::new(ErrorKind::Other,
                                  "Reached EOF. Can't make down buffer."));
        }

        self.seek(SeekFrom::Current(-(bytes_read as i64)))?;

        Ok((buf, bytes_read))
    }

    fn search_buf_size(&self) -> usize {
        self.page_size()
    }

    fn page_size(&self) -> usize {
        let (screen_width, screen_height) = util::screen_width_height();
        screen_width as usize * screen_height as usize * 4 // 4 is max utf8 char size
    }

    fn pcur_pos(&mut self) -> Result<()> {
        if let Ok(cur_pos) = self.seek(SeekFrom::Current(0)) {
            eprintln!("cur_pos: {}", cur_pos);
        }
        Ok(())
    }

    pub fn seek_percent(&mut self, percent: u64) -> Result<u64> {
        let size = self.seek(SeekFrom::End(0))?;
        eprintln!("size {}", size);

        let offset = (size * percent / 100) as u64;
        eprintln!("ofst {}", offset);

        self.seek(SeekFrom::Start(offset))
    }
}

impl<R: Read> BufRead for BiBufReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        // If we've reached the end of our internal buffer then we need to fetch
        // some more data from the underlying reader.
        // Branch using `>=` instead of the more correct `==`
        // to tell the compiler that the pos..cap slice is always valid.
        if self.pos >= self.cap {
            debug_assert!(self.pos == self.cap);
            self.cap = self.inner.read(&mut self.buf)?;
            self.pos = 0;
        }
        Ok(&self.buf[self.pos..self.cap])
    }

    fn consume(&mut self, amt: usize) {
        self.pos = std::cmp::min(self.pos + amt, self.cap);
    }
}

impl<R: Seek> Seek for BiBufReader<R> {
    /// Seek to an offset, in bytes, in the underlying reader.
    ///
    /// The position used for seeking with `SeekFrom::Current(_)` is the
    /// position the underlying reader would be at if the `BufReader` had no
    /// internal buffer.
    ///
    /// Seeking always discards the internal buffer, even if the seek position
    /// would otherwise fall within it. This guarantees that calling
    /// `.into_inner()` immediately after a seek yields the underlying reader
    /// at the same position.
    ///
    /// To seek without discarding the internal buffer, use [`seek_relative`].
    ///
    /// See `std::io::Seek` for more details.
    ///
    /// Note: In the edge case where you're seeking with `SeekFrom::Current(n)`
    /// where `n` minus the internal buffer length overflows an `i64`, two
    /// seeks will be performed instead of one. If the second seek returns
    /// `Err`, the underlying reader will be left at the same position it would
    /// have if you called `seek` with `SeekFrom::Current(0)`.
    ///
    /// [`seek_relative`]: #method.seek_relative
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let result: u64;
        if let SeekFrom::Current(n) = pos {
            let remainder = (self.cap - self.pos) as i64;
            // it should be safe to assume that remainder fits within an i64 as the alternative
            // means we managed to allocate 8 exbibytes and that's absurd.
            // But it's not out of the realm of possibility for some weird underlying reader to
            // support seeking by i64::min_value() so we need to handle underflow when subtracting
            // remainder.
            if let Some(offset) = n.checked_sub(remainder) {
                result = self.inner.seek(SeekFrom::Current(offset))?;
            } else {
                // seek backwards by our remainder, and then by the offset
                self.inner.seek(SeekFrom::Current(-remainder))?;
                self.pos = self.cap; // empty the buffer
                result = self.inner.seek(SeekFrom::Current(n))?;
            }
        } else {
            // Seeking with Start/End doesn't care about our buffer length.
            result = self.inner.seek(pos)?;
        }
        self.pos = self.cap; // empty the buffer
        Ok(result)
    }
}

impl<R: Read> Read for BiBufReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // If we don't have any buffered data and we're doing a massive read
        // (larger than our internal buffer), bypass our internal buffer
        // entirely.
        if self.pos == self.cap && buf.len() >= self.buf.len() {
            return self.inner.read(buf);
        }
        let nread = {
            let mut rem = self.fill_buf()?;
            rem.read(buf)?
        };
        self.consume(nread);
        Ok(nread)
    }

    // we can't skip unconditionally because of the large buffer case in read.
    unsafe fn initializer(&self) -> Initializer {
        self.inner.initializer()
    }
}
