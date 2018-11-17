use termion::terminal_size;

use std::io::BufReader;
use std::io::{Read, Result, Seek, SeekFrom};
use std::str;

use input::LEvent;
use line_num_cache::{LineNumCache, LineNum};
use string_util;
use screen;


pub struct BiBufReader<R> {
    pub reader: BufReader<R>,
    pub page_buf: Vec<u8>,
    line_num_cache: LineNumCache,
}

impl<R: Read + Seek> BiBufReader<R> {
    pub fn new(r: BufReader<R>) -> BiBufReader<R> {
        let mut line_num_cache = LineNumCache::new();
        line_num_cache.add(LineNum { num: 0, offset: 0 });

        BiBufReader {
            reader: r,
            page_buf: Vec::with_capacity(1024),
            line_num_cache: line_num_cache,
        }
    }

    pub fn move_to_command(&mut self, action: &LEvent) -> Result<()> {
        match action {
            LEvent::NoOp => (),
            LEvent::DownOneLine => match self.down_n_lines(1) {
                Ok(_) => (),
                Err(e) => eprintln!("{}", e),
            },
            LEvent::DownHalfScreen => match self.down_n_lines(screen::height_half_screen()) {
                Ok(_) => (),
                Err(e) => eprintln!("{}", e),
            },
            LEvent::UpHalfScreen => match self.up_n_lines(screen::height_half_screen()) {
                Ok(_) => (),
                Err(e) => eprintln!("{}", e),
            },
            LEvent::UpOneLine => match self.up_n_lines(1) {
                Ok(_) => (),
                Err(e) => eprintln!("{}", e),
            },
            _ => (),
        }

        Ok(())
    }

    fn up_n_lines(&mut self, n: usize) -> Result<()> {
        let buf = self.make_buf_up()?;
        let size = buf.len();

        unsafe {
            let (screen_width, _) = terminal_size().unwrap();

            let start = 0;
            let offset = size as i64 - start as i64 - 
                string_util::nth_last_newline_wrapped(
                    n + 1,
                    str::from_utf8_unchecked(&buf[start..]),
                    screen_width as usize,
            ) as i64;
            eprintln!("{}, {}", size, offset);
            self.reader.seek(SeekFrom::Current(-(offset as i64)))?;
        }

        Ok(())
    }

    fn down_n_lines(&mut self, n: usize) -> Result<()> {
        let (buf, size) = self.make_buf_down()?;

        let (screen_width, _) = terminal_size().unwrap();

        unsafe {
            if let Some(newline_offset) = string_util::nth_newline_wrapped(
                n,
                str::from_utf8_unchecked(&buf),
                screen_width as usize,
            ) {
                self.reader
                    .seek(SeekFrom::Current(newline_offset as i64))?;
            }
        }

        Ok(())
    }

    pub fn page(&mut self) -> Result<&str> {
        self.pcur_pos()?;
        let size = self.page_size();

        self.page_buf.clear();
        self.page_buf.resize(size, 0);

        let mut bytes_read: i64 = 0;
        bytes_read = match self.reader.read(&mut self.page_buf[..size]) {
            Err(e) => {
                eprintln!("errorrrr {}", e);
                0
            }
            Ok(s) => s as i64,
        };

        self.reader.seek(SeekFrom::Current(-bytes_read))?;

        // Could be unchecked?
        return match str::from_utf8(& self.page_buf[..size]) {
            Ok(s) => Ok(s),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, "from tfu8")),
        }
    }

    fn make_buf_up(&mut self) -> Result<Vec<u8>> {
        let size = std::cmp::min(self.search_buf_size(),
                                 self.reader.seek(SeekFrom::Current(0))? as usize);

        let mut buf = vec![0; size];
        let new_pos = self.reader.seek(SeekFrom::Current(-(size as i64)))?;

        let bytes_read = self.reader.read(&mut buf[..size])?;
        //debug_assert!(bytes_read as u64 == size as u64);

        Ok(buf)
    }

    fn make_buf_down(&mut self) -> Result<(Vec<u8>, usize)> {
        let size = self.search_buf_size();
        let mut buf = vec![0; size as usize];

        let bytes_read = self.reader.read(&mut buf)?;

        if bytes_read == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "from tfu8"));
        }

        self.reader.seek(SeekFrom::Current(-(bytes_read as i64)))?;

        Ok((buf, bytes_read))
    }

    fn search_buf_size(&self) -> usize {
        self.page_size()
    }
    
    fn page_size(&self) -> usize {
        let (screen_width, screen_height) = terminal_size().unwrap();
        eprintln!("page size {}", 
        screen_width as usize * screen_height as usize * 4); // 4 is max utf8 char sizebb
        screen_width as usize * screen_height as usize * 4 // 4 is max utf8 char size
    }

    fn pcur_pos(&mut self) -> Result<()> {
        if let Ok(cur_pos) = self.reader.seek(SeekFrom::Current(0)) {
            eprintln!("cur_pos: {}", cur_pos);
        }
        Ok(())
    }
}
