use file_buffer::BiBufReader;

use termion::screen::AlternateScreen;
use termion::terminal_size;
use unicode_segmentation::UnicodeSegmentation;

use std::io::{Read, Seek, Write};
use std::str;

use string_util;
use util;

pub struct Printer<W: Write> {
    pub out: AlternateScreen<W>,
}

impl<W: Write> Printer<W> {
    pub fn flush(&mut self) {
        self.out.flush().unwrap();
    }

    pub fn print_screen(&mut self, page: &Vec<u8>) -> Result<(), ()> {
        self.clear_screen();

        let mut screen_line_number: u16 = 1;
        let (screen_width, screen_height) = util::screen_width_height();

        let page_string = match str::from_utf8(&page[..]) {
            Ok(s) => s,
            Err(e) => return Err(()),
        };

        write!(self.out, "{}", termion::cursor::Goto(1, 1));

        let mut grapheme_count = 0;
        for grapheme in UnicodeSegmentation::graphemes(&page_string[..], true) {
            if screen_line_number >= screen_height - 1 {
                break;
            }

            if grapheme_count >= screen_width as usize {
                grapheme_count = 0;
                screen_line_number += 1;
                writeln!(self.out);
                write!(self.out, "\r");
            }

            if string_util::is_newline(grapheme) {
                grapheme_count = 0;
                screen_line_number += 1;
                writeln!(self.out);
                write!(self.out, "\r");
            } else {
                grapheme_count += 1;
                write!(self.out, "{}", grapheme);
            }
        }

        self.write_command_line(screen_line_number);
        self.flush();

        Ok(())
    }

    fn write_command_line(&mut self, line_num: u16) {
        write!(self.out, "\n\r");
        write!(self.out, ":");
        write!(self.out, "{}", termion::cursor::Goto(2, line_num + 1));
    }

    pub fn print2<R: Read + Seek>(
        &mut self,
        reader: &mut BiBufReader<R>,
    ) -> Result<(), ()> {

        write!(self.out, "{}", termion::cursor::Goto(1, 1));
        self.clear_screen();
        self.flush();

        Ok(())
    }


    fn clear_screen(&mut self) {
        write!(self.out, "{}", termion::clear::All);
    }
}
