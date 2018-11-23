
use std::io::{self, Seek, Read, SeekFrom, Error, ErrorKind};
use std::str;

use unicode_segmentation::GraphemeCursor;

use utf8_validation;

pub struct ValidReader<R> {
    inner: R,
}

impl<R: Read> ValidReader<R> {
    pub fn new(reader: R) -> ValidReader<R> {
        ValidReader {
            inner: reader,
        }
    }
}

impl<R: Read> Read for ValidReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<R: Read + Seek> Seek for ValidReader<R> {
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
            None => return Err(Error::new(
                ErrorKind::Other,
                "No valid position found.",
            )),
        };

        self.inner.seek(SeekFrom::Start(pos))
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
        assert_eq!(reader.seek(SeekFrom::End(0)).unwrap(),
                   reader.seek(SeekFrom::Current(0)).unwrap());
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
