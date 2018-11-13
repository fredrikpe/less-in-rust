

use std::cmp::min;
use std::io::{Seek, SeekFrom, Read, Result};
use std::io::BufReader;
use std::io::BufRead;

static LF_BYTE: u8 = '\n' as u8;
static CR_BYTE: u8 = '\r' as u8;

pub struct BiBufReader<R> {
    pub reader: BufReader<R>,
    pub position: usize,
    pub size: usize,
}

impl<R: Read + Seek> BiBufReader<R> {
    pub fn read_line(&mut self, buf: &mut String) -> Result<usize> {
        self.position += self.reader.read_line(buf)?;
        Ok(self.position as usize)
    }

    pub fn move_up(&mut self, lines: i64) {
        for i in 0..lines {
            self.move_up_one();
        }
    }

    pub fn move_offset(&mut self, line_offset: i64) {
        if line_offset > 0 {
            let mut s = String::new();
            for _ in 0..line_offset {
                
                self.position += 1;
                self.reader.read_line(&mut s);
            }
        } else if line_offset < 0 {
            self.move_up(-line_offset);
        }
    }

    pub fn seek(&mut self, offset: i64) -> Result<u64> {
        self.position += offset as usize;
        self.reader.seek(SeekFrom::Current(offset))
    }

    fn read_to_buffer(&mut self, size: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0; size as usize];
        let offset = -(size as i64);

        self.reader.seek(SeekFrom::Current(offset))?;
        self.reader.read_exact(&mut buf[0..(size as usize)])?;
        self.reader.seek(SeekFrom::Current(offset))?;

        self.position -= size as usize;

        Ok(buf)
    }

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
                                Ok(_)  => {
                                    self.position += offset as usize;

                                    break 'outer;
                                },

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
                    return ()
                }
            }
        }
    }
}


