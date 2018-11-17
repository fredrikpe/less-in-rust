
use file_buffer::BiBufReader;

use termion::screen::AlternateScreen;
use termion::terminal_size;
use unicode_segmentation::UnicodeSegmentation;

use std::io::{Read, Seek, Write};

use string_util;

pub struct Printer<W: Write> {
    pub out: AlternateScreen<W>,
}

impl<W: Write> Printer<W> {
    pub fn flush(&mut self) {
        self.out.flush().unwrap();
    }

    pub fn print_screen<R: Read + Seek>(
        &mut self,
        reader: &mut BiBufReader<R>,
    ) -> Result<(), ()> {
        self.clear_screen();

        let mut screen_line_number: u16 = 1;
        let (screen_width, screen_height) = terminal_size().unwrap();

        let page = match reader.page() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {}", e);
                return Ok(());
            }
        };

        write!(self.out, "{}", termion::cursor::Goto(1, screen_line_number));

        let mut grapheme_count = 0;
        for (_, grapheme) in UnicodeSegmentation::grapheme_indices(page, true) {
            if screen_line_number >= screen_height {
                break;
            }

            if grapheme_count >= screen_width as usize {
                grapheme_count = 0;
                screen_line_number += 1;
                writeln!(self.out);
                write!(self.out, "{}", termion::cursor::Goto(1, screen_line_number));
            }

            if string_util::is_newline(grapheme) {
                grapheme_count = 0;
                screen_line_number += 1;
                writeln!(self.out);
                write!(self.out, "{}", termion::cursor::Goto(1, screen_line_number));
            } else {
                grapheme_count += 1;
                write!(self.out, "{}", grapheme);
            }
        }

        writeln!(self.out);
        write!(self.out, "{}", termion::cursor::Goto(1, screen_line_number));

        Ok(())
    }

    fn clear_screen(&mut self) {
        write!(self.out, "{}", termion::clear::All);
    }
}
