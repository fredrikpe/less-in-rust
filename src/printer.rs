use termion::screen::AlternateScreen;
use unicode_segmentation::UnicodeSegmentation;

use std::io::Write;
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

    pub fn render(&mut self, page: &Vec<u8>, command_line_text: String) -> Result<(), ()> {
        self.clear_screen();

        self.print_page(page)?;
        self.print_command_line(command_line_text);
        self.flush();

        Ok(())
    }

    pub fn print_page(&mut self, page: &Vec<u8>) -> Result<(), ()> {
        let mut screen_line_number: u16 = 1;
        let (screen_width, screen_height) = util::screen_width_height();

        let page_string = match str::from_utf8(&page[..]) {
            Ok(s) => s,
            Err(e) => return Err(()),
        };

        self.write(&termion::cursor::Goto(1, 1));

        let mut grapheme_count = 0;
        for grapheme in UnicodeSegmentation::graphemes(&page_string[..], true) {
            if screen_line_number >= screen_height - 1 {
                break;
            }

            if grapheme_count >= screen_width as usize {
                grapheme_count = 0;
                screen_line_number += 1;
                self.write(&"\n\r");
            }

            if string_util::is_newline(grapheme) {
                grapheme_count = 0;
                screen_line_number += 1;
                self.write(&"\n\r");
            } else {
                grapheme_count += 1;
                self.write(&grapheme);
            }
        }

        for i in screen_line_number..(screen_height - 1) {
            self.write(&"~\r\n");
        }

        Ok(())
    }

    fn print_command_line(&mut self, command_line_text: String) {
        let (screen_width, screen_height) = util::screen_width_height();
        self.write(&"\n\r");
        self.write(&command_line_text);
        self.write(&termion::cursor::Goto(
            (command_line_text.len() + 1) as u16,
            screen_height + 1,
        ));
    }

    fn clear_screen(&mut self) {
        self.write(&termion::clear::All);
    }

    fn write<S: std::fmt::Display>(&mut self, s: &S) {
        let _ = write!(self.out, "{}", s);
    }
}
