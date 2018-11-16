use termion::terminal_size;

use std::cmp::min;
use std::io::BufRead;
use std::io::BufReader;
use std::io::{Read, Result, Seek, SeekFrom};
use std::str;
use std::cmp;

use input::LEvent;

static LF_BYTE: u8 = '\n' as u8;
static CR_BYTE: u8 = '\r' as u8;

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
            LEvent::DownOneLine => self.down_one_line()?,
            _ => (),
        }

        Ok(())
    }

    fn down_one_line(&mut self) -> Result<()> {
        let size: i64 = 1024;
        let mut buf = vec![0; size as usize];

        self.reader.read_exact(&mut buf)?;
        self.reader.seek(SeekFrom::Current(-size))?;

        unsafe {
            let bytes_to_new_line = self.first_new_line_pos(str::from_utf8_unchecked(&buf));
            eprintln!("btnl {}", bytes_to_new_line);
            self.reader.seek(SeekFrom::Current(bytes_to_new_line + 1))?;
        }

        Ok(())
    }

    fn first_new_line_pos(&mut self, s: &str) -> i64 {
        let (screen_width, _) = terminal_size().unwrap();

        match find_newline(&s) {
            Some(p) => cmp::min(p as i64, screen_width_as_bytes(screen_width) as i64),
            _ => screen_width_as_bytes(screen_width) as i64,
        }
    }

    pub fn page(&mut self) -> Result<&str> {
        let page_size: i64 = self.page_size();
        self.page_string.clear();
        self.page_string = String::from_utf8_lossy(&vec![0; page_size as usize]).to_string();

        unsafe {
            match self.reader.read_exact(self.page_string.as_mut_vec()) {
                Err(_) => self.reader.read_to_end(self.page_string.as_mut_vec()),
                _ => Ok(0)
            };
        }
        self.reader.seek(SeekFrom::Current(-page_size))?;

        Ok(&self.page_string[..])
    }


    pub fn page_size(&mut self) -> i64 {
        2000
    }

    /*
    fn move_up_one(&mut self) {
        'outer: loop {
            if self.position < 1 {
                return ();
            }

            // Read the of minimum between the desired
            // buffer size or remaining length of the reader
            let size = min(self.size, self.position);

            match self.read_to_buffer(size) {
                Ok(buf) => {
                    for (idx, ch) in (&buf).iter().enumerate().rev() {
                        // Found a new line character to break on
                        if *ch == LF_BYTE {
                            let mut offset = idx as u64;

                            // Add an extra byte cause of CR character
                            if idx > 1 && buf[idx - 1] == CR_BYTE {
                                offset -= 1;
                            }

                            match self.reader.seek(SeekFrom::Current(offset as i64)) {
                                Ok(_) => {
                                    self.position += offset as usize;

                                    break 'outer;
                                }

                                Err(_) => {
                                    println!("asdf");
                                    return ();
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    println!("ERROr line");
                    return ();
                }
            }
        }
    }
    */
}

pub fn is_new_line(c: char) -> bool {
    return c == '\n'
}

pub fn find_newline(s: &str) -> Option<usize> {
    s.find('\n')
}

pub fn screen_width_as_bytes(screen_width: u16) -> usize {
    2 * screen_width as usize
}



