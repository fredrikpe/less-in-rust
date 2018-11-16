use termion::terminal_size;

use std::cmp;
use std::io::BufReader;
use std::io::{Read, Result, Seek, SeekFrom};
use std::str;

use input::LEvent;
use string_util;

pub struct BiBufReader<R> {
    pub reader: BufReader<R>,
    pub byte_offset: usize,
    pub page_string: String,
}

impl<R: Read + Seek> BiBufReader<R> {
    pub fn new(r: BufReader<R>) -> BiBufReader<R> {
        BiBufReader {
            reader: r,
            byte_offset: 0,
            page_string: String::with_capacity(1024),
        }
    }

    pub fn move_to_command(&mut self, action: &LEvent) -> Result<()> {
        match action {
            LEvent::NoOp => (),
            LEvent::DownOneLine => match self.down_one_line() {
                Ok(_) => (),
                Err(e) => eprintln!("{}", e),
            },
            _ => (),
        }

        Ok(())
    }

    //fn up_one_line(&mut self) -> Result<()> {

    fn down_one_line(&mut self) -> Result<()> {
        let size: i64 = 1024;
        let mut buf = vec![0; size as usize];

        self.reader.read(&mut buf)?;
        self.reader.seek(SeekFrom::Current(-size))?;

        let (screen_width, _) = terminal_size().unwrap();

        unsafe {
            if let Some(newline_offset) =
                string_util::first_newline_offset(str::from_utf8_unchecked(&buf),
                                                  screen_width as usize)
            {
                self.reader
                    .seek(SeekFrom::Current(newline_offset as i64 + 1))?;
            }
        }

        Ok(())
    }

    pub fn page(&mut self) -> Result<&str> {
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

    pub fn page_size(&mut self) -> i64 {
        2000
    }
}
