use termion::terminal_size;

use std::io::BufReader;
use std::io::{Read, Result, Seek, SeekFrom};
use std::str;

use input::LEvent;
use line_num_cache::{LineNumCache, LineNum};
use string_util;


pub struct BiBufReader<R> {
    pub reader: BufReader<R>,
    pub page_string: String,
    line_num_cache: LineNumCache,
}

impl<R: Read + Seek> BiBufReader<R> {
    pub fn new(r: BufReader<R>) -> BiBufReader<R> {
        let mut line_num_cache = LineNumCache::new();
        line_num_cache.add(LineNum { num: 0, offset: 0 });

        BiBufReader {
            reader: r,
            page_string: String::with_capacity(1024),
            line_num_cache: line_num_cache,
        }
    }

    pub fn move_to_command(&mut self, action: &LEvent) -> Result<()> {
        match action {
            LEvent::NoOp => (),
            LEvent::DownOneLine => match self.down_one_line() {
                Ok(_) => (),
                Err(e) => eprintln!("{}", e),
            },
            LEvent::UpOneLine => match self.up_one_line() {
                Ok(_) => (),
                Err(e) => eprintln!("{}", e),
            },
            _ => (),
        }

        Ok(())
    }

    fn pcur_pos(&mut self) -> Result<()> {
        if let Ok(cur_pos) = self.reader.seek(SeekFrom::Current(0)) {
            eprintln!("cur_pos: {}", cur_pos);
        }
        Ok(())
    }

    fn back_offset(&mut self) -> Result<i64> {
        let cur_pos = self.reader.seek(SeekFrom::Current(0))?;

        return match self.line_num_cache.prev(cur_pos as usize) {
            Some((LineNum, distance)) => Ok(distance),
            None => Ok(0),
        }
    }

    fn fill_back_buf(&mut self) -> Result<Vec<u8>> {
        let size = std::cmp::min(self.search_buf_size(),
                                 self.reader.seek(SeekFrom::Current(0))? as usize);

        let mut buf = vec![0; size];
        let new_pos = self.reader.seek(SeekFrom::Current(-(size as i64)))?;

        let bytes_read = self.reader.read(&mut buf[..size])?;
        debug_assert!(bytes_read as u64 == size as u64);

        Ok(buf)
    }

    fn up_one_line(&mut self) -> Result<()> {
        let mut buf = self.fill_back_buf()?;
        let size = buf.len();

        unsafe {
            //let start = match string_util::last_newline_offset(
                //str::from_utf8_unchecked(&buf)) {
                //Some(ofs) => {
                    //eprintln!("prev newline offset{}", ofs);
                    //search_len - ofs
                //}
                //None => 0,
            //};
            //eprintln!("start {}", start);

            let (screen_width, _) = terminal_size().unwrap();

            let start = 0;
            let offset = size as i64 - start as i64 - 
                string_util::snd_last_newline_wrapped(
                str::from_utf8_unchecked(&buf[start..]),
                screen_width as usize,
            ) as i64;
            eprintln!("nlo {}", offset);
            self.reader.seek(SeekFrom::Current(-(offset as i64)))?;
        }

        Ok(())
    }

    fn down_one_line(&mut self) -> Result<()> {
        let size: i64 = 1024;
        let mut buf = vec![0; size as usize];

        let bytes_read = self.reader.read(&mut buf)?;

        if bytes_read == 0 {
            return Ok(());
        }

        self.reader.seek(SeekFrom::Current(-(bytes_read as i64)))?;

        let (screen_width, _) = terminal_size().unwrap();

        unsafe {
            if let Some(newline_offset) = string_util::first_newline_wrapped(
                str::from_utf8_unchecked(&buf),
                screen_width as usize,
            ) {
                self.reader
                    .seek(SeekFrom::Current(newline_offset as i64))?;
            }
        }

        Ok(())
    }

    fn seek_back(&mut self, size: i64) -> Result<u64> {
        return if let Ok(cur_pos) = self.reader.seek(SeekFrom::Current(0)) {
            if cur_pos == 0 {
                Ok(0)
            } else if (cur_pos as i64) < size {
                let _ = self.reader.seek(SeekFrom::Start(0));
                Ok(cur_pos)
            } else {
                self.reader.seek(SeekFrom::Current(-size))?;
                Ok(size as u64)
            }
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "returned"))
        };
    }

    fn seek_forward(&mut self, size: i64) -> Result<u64> {
        return if let Ok(cur_pos) = self.reader.seek(SeekFrom::Current(0)) {
            if cur_pos == 0 {
                Ok(0)
            } else if (cur_pos as i64) < size {
                let _ = self.reader.seek(SeekFrom::Start(0));
                Ok(cur_pos)
            } else {
                self.reader.seek(SeekFrom::Current(-size))?;
                Ok(size as u64)
            }
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "returned"))
        };
    }


    pub fn page(&mut self) -> Result<&str> {
        self.pcur_pos()?;
        let page_size: i64 = self.page_size();
        self.page_string.clear();
        self.page_string = String::from_utf8_lossy(&vec![0; page_size as usize]).to_string();

        let mut bytes_read: i64 = 0;
        unsafe {
            bytes_read = match self.reader.read(self.page_string.as_mut_vec()) {
                Err(e) => {
                    eprintln!("errorrrr {}", e);
                    0
                }
                Ok(s) => s as i64,
            };
        }
        self.reader.seek(SeekFrom::Current(-bytes_read))?;

        Ok(&self.page_string[..])
    }

    pub fn search_buf_size(&self) -> usize {
        4096
    }
    
    pub fn page_size(&mut self) -> i64 {
        let (screen_width, screen_height) = terminal_size().unwrap();
        screen_width as i64 * screen_height as i64 * 4 // 4 is max utf8 char size
    }
}
