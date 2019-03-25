use std::fs::File;
use std::io::{
    self, stdin, BufRead, Cursor, Error, ErrorKind, Initializer, Read, Result,
    Seek, SeekFrom, Stdin,
};
use std::str;

use grep::matcher::Match;
use grep::regex::RegexMatcher;

use searcher;
use standard::StandardSink;
use string_util;
use utf8_validation;
use util;

const DEFAULT_BUF_SIZE: usize = 4096;

pub trait Search {
    fn search(&mut self, matches: &mut Vec<(u64, Match)>, pattern: &str);
}

pub trait FileSwitcher {
    fn search(&mut self, matches: &mut Vec<(u64, Match)>, pattern: &str);
}

pub struct BiBufReader<R> {
    inner: R,
    buf: Box<[u8]>,
    pos: usize,
    cap: usize,
}

impl<R: Read + Seek> BiBufReader<R> {
    pub fn new(inner: R) -> BiBufReader<R> {
        unsafe {
            let mut buffer = Vec::with_capacity(DEFAULT_BUF_SIZE);
            buffer.set_len(DEFAULT_BUF_SIZE);
            BiBufReader {
                inner,
                buf: buffer.into_boxed_slice(),
                pos: 0,
                cap: 0,
            }
        }
    }

    pub fn jump_offset(&mut self, offset: u64) -> Result<()> {
        self.seek(SeekFrom::Start(offset))?;
        Ok(())
    }

    pub fn jump_percentage(&mut self, percent: u64) -> Result<()> {
        let _ = self.seek_percent(percent)?;
        self.up_n_lines(1)
    }

    pub fn jump_end(&mut self) -> Result<()> {
        let _ = self.seek_percent(100)?;
        self.up_n_lines(util::screen_height() - 1)
    }

    pub fn up_n_lines(&mut self, n: usize) -> Result<()> {
        let buf = self.make_buf_up()?;

        unsafe {
            let size = buf.len();

            let (screen_width, _) = util::screen_width_height();

            let offset = size as i64
                - string_util::nth_last_newline_wrapped(
                    n + 1,
                    str::from_utf8_unchecked(&buf[..]),
                    screen_width as usize,
                ) as i64;
            self.seek(SeekFrom::Current(-(offset as i64)))?;
        }

        Ok(())
    }

    pub fn down_n_lines(&mut self, n: usize) -> Result<()> {
        eprint!("down, ");
        self.pcur_pos()?;
        let (buf, size) = self.make_buf_down()?;

        let (screen_width, _) = util::screen_width_height();

        unsafe {
            let newline_offset = string_util::nth_newline_wrapped(
                n,
                str::from_utf8_unchecked(&buf[..size]),
                screen_width as usize,
            );
            self.seek(SeekFrom::Current(newline_offset as i64))?;
        }

        Ok(())
    }

    pub fn page(&mut self) -> Result<(u64, Vec<u8>)> {
        let size = self.page_size();

        let mut page_buf = Vec::with_capacity(size);
        page_buf.resize(size, 0);

        let bytes_read = match self.read(&mut page_buf[..]) {
            Err(e) => {
                eprintln!("errorrrr {}", e);
                0
            }
            Ok(s) => s as i64,
        };

        let offset = self.seek(SeekFrom::Current(-bytes_read))?;

        Ok((offset, page_buf[..bytes_read as usize].to_vec()))
    }

    fn make_buf_up(&mut self) -> Result<Vec<u8>> {
        let cur_pos = self.seek(SeekFrom::Current(0))?;

        let size = std::cmp::min(
            self.search_buf_size(),
            self.seek(SeekFrom::Current(0))? as usize,
        );

        let mut buf = vec![0; size];
        let _ = self.seek(SeekFrom::Current(-(size as i64)))?;

        let bytes_read = self.read(&mut buf[..size])?;

        self.seek(SeekFrom::Current((size - bytes_read) as i64))?;

        assert_eq!(cur_pos, self.seek(SeekFrom::Current(0))?);
        Ok(string_util::make_valid(buf))
    }

    fn make_buf_down(&mut self) -> Result<(Vec<u8>, usize)> {
        let cur_pos = self.seek(SeekFrom::Current(0))?;

        let size = self.search_buf_size();
        let mut buf = vec![0; size as usize];

        let bytes_read = self.read(&mut buf)?;

        if bytes_read == 0 {
            return Err(Error::new(
                ErrorKind::Other,
                "Reached EOF. Can't make down buffer.",
            ));
        }

        self.seek(SeekFrom::Current(-(bytes_read as i64)))?;

        assert_eq!(cur_pos, self.seek(SeekFrom::Current(0))?);
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
        let offset = std::cmp::min(size, (size * percent / 100) as u64);

        self.seek(SeekFrom::Start(offset))
    }

    pub fn current_offset(&mut self) -> Result<u64> {
        self.seek(SeekFrom::Current(0))
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
        debug_assert!(utf8_validation::first_valid_pos(buf).unwrap() == 0);
        Ok(nread)
    }

    // we can't skip unconditionally because of the large buffer case in read.
    unsafe fn initializer(&self) -> Initializer {
        self.inner.initializer()
    }
}

impl<R: Search+Seek> Search for BiBufReader<R> {
    fn search(&mut self, matches: &mut Vec<(u64, Match)>, pattern: &str) {
        // Searching will seek from current position to end, so first have
        // to remeber the current position, go to the beginning (we want to
        // search the entire file), then go back to the original position.
        let cur_pos = self.seek(SeekFrom::Current(0)).unwrap();
        self.inner.seek(SeekFrom::Start(0));
        self.inner.search(matches, pattern);
        self.inner.seek(SeekFrom::Start(cur_pos));
    }
}

#[derive(Debug)]
pub struct StdinCursor {
    cursor: Cursor<Vec<u8>>,
    pub file: File,
}

impl StdinCursor {
    pub fn new(mut stdin_file: File) -> StdinCursor {
        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        // TODO: Currently reads the whole thing at the beginning. Could instead read while
        // scrolling down.
        stdin_file.read_to_end(cursor.get_mut()).unwrap();
        StdinCursor {
            cursor: cursor,
            file: stdin_file,
        }
    }
}

impl Seek for StdinCursor {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.cursor.seek(pos)
    }
}

impl Read for StdinCursor {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.cursor.read(buf)
    }
}

#[derive(Debug)]
pub enum InputReader {
    Stdin(StdinCursor),
    Files(Vec<File>),
}

impl Read for InputReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        return match self {
            InputReader::Stdin(stdin_cursor) => stdin_cursor.read(buf),
            InputReader::Files(files) => (&files[0]).read(buf),
        };
    }
}

impl Seek for InputReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        return match self {
            InputReader::Stdin(stdin_cursor) => stdin_cursor.seek(pos),
            InputReader::Files(files) => (&files[0]).seek(pos),
        };
    }
}

impl Search for InputReader {
    fn search(&mut self, matches: &mut Vec<(u64, Match)>, pattern: &str) {
        let matcher = match RegexMatcher::new(&pattern[..]) {
            Err(_) => return,
            Ok(m) => m,
        };

        let mut sink = StandardSink {
            matcher: matcher,
            matches: matches,
            match_count: 0,
        };

        match self {
            InputReader::Stdin(stdin_cursor) => {
                match searcher::search_reader(&mut sink, stdin_cursor) {
                    Err(_) => (),
                    Ok(_) => (),
                }
            }
            InputReader::Files(files) => {
                match searcher::search_file(&mut sink, &files[0]) {
                    Err(_) => (),
                    Ok(_) => (),
                }
            }
        }
    }
}

/// This reader ensures that the position is always at a valid utf-8 code point, i.e., when seeking
/// to a pos it will stop at a nearby valid point if the original is in the middle of a character.
///
/// The start position is assumed valid.
pub struct ValidReader<R> {
    inner: R,
}

impl<R: Read> ValidReader<R> {
    pub fn new(reader: R) -> ValidReader<R> {
        ValidReader { inner: reader }
    }
}

impl<R: Read> Read for ValidReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<R: Read + Seek> Seek for ValidReader<R> {

    /// When seeking to a posi
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let mut pos = self.inner.seek(pos)?;

        // We assume start is always valid
        if pos == 0 {
            return Ok(pos);
        }

        // Me thinks 8 is enough?
        let mut buf: [u8; 8] = [0; 8];
        let r = self.read(&mut buf)?;

        pos += match utf8_validation::first_valid_pos(&buf[..r]) {
            Some(offset) => offset as u64,
            None => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "No valid position found.",
                ))
            }
        };

        self.inner.seek(SeekFrom::Start(pos))
    }
}

impl<S: Search> Search for ValidReader<S> {
    fn search(&mut self, matches: &mut Vec<(u64, Match)>, pattern: &str) {
        self.inner.search(matches, pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::BufReader;

    fn thai_file() -> File {
        File::open("tests/resources/thai.txt").unwrap()
    }

    #[test]
    fn test_seek_to_end() {
        let mut reader = ValidReader::new(thai_file());
        assert_eq!(
            reader.seek(SeekFrom::End(0)).unwrap(),
            reader.seek(SeekFrom::Current(0)).unwrap()
        );
    }

    #[test]
    fn test_seek_0_returns_0() {
        let mut reader = ValidReader::new(thai_file());
        assert_eq!(reader.seek(SeekFrom::Start(0)).unwrap(), 0);
    }

    #[test]
    fn test_valid_from_1() {
        let mut reader = ValidReader::new(thai_file());

        // Seek to invalid pos
        let pos = reader.seek(SeekFrom::Start(1)).unwrap();
        assert_eq!(pos, 3);

        let mut buf: [u8; 1000] = [0; 1000];
        let b = reader.read(&mut buf[..]).unwrap();
        assert!(std::str::from_utf8(&buf[..b]).is_ok());
    }

    #[test]
    fn test_err_at_end() {
        let mut reader = ValidReader::new(thai_file());
        let mut buf: [u8; 10] = [0; 10];
        let b = reader.read(&mut buf[..]).unwrap();
        eprintln!("b {}", b);
        assert!(std::str::from_utf8(&buf[..b]).is_err());
    }

    #[test]
    fn test_bufreader_fails_start() {
        let mut reader = BufReader::new(thai_file());

        // Seek to invalid position
        reader.seek(SeekFrom::Start(1)).unwrap();

        let mut buf: [u8; 1000] = [0; 1000];
        let b = reader.read(&mut buf[..]).unwrap();
        assert!(std::str::from_utf8(&buf[..b]).is_err());
    }

    #[test]
    fn test_bufreader_fails_end() {
        let mut reader = BufReader::new(thai_file());

        // 10 bytes in happens to be in the middle of a grapheme
        let mut buf: [u8; 10] = [0; 10];
        let b = reader.read(&mut buf[..]).unwrap();
        assert!(std::str::from_utf8(&buf[..b]).is_err());
    }
}
