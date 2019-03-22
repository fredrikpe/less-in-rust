use termion::color;
use termion::screen::AlternateScreen;
use unicode_segmentation::UnicodeSegmentation;

use std::io::Write;
use std::str;

use grep::matcher::Match;

use string_util;
use util;


pub struct Printer<W: Write> {
    pub out: AlternateScreen<W>,
}

impl<W: Write> Printer<W> {
    pub fn flush(&mut self) {
        self.out.flush().unwrap();
    }

    pub fn render(
        &mut self,
        page: &(u64, Vec<u8>),
        matches: &Vec<(u64, Match)>,
        command_line_text: String,
    ) -> Result<(), ()> {
        self.clear_screen();

        self.print_page(&page.1, search_offsets(page.0, matches))?;
        self.print_command_line(command_line_text);
        self.flush();

        Ok(())
    }

    pub fn print_page(
        &mut self,
        page: &Vec<u8>,
        search_offsets: Vec<u64>,
    ) -> Result<(), ()> {
        let mut screen_line_number: u16 = 1;
        let (screen_width, screen_height) = util::screen_width_height();

        let mut bb: Vec<String> = Vec::new();

        unsafe {
            let page_string = str::from_utf8_unchecked(&page[..]);

            self.write(&termion::cursor::Goto(1, 1));

            let mut grapheme_count = 0;
            for (index, grapheme) in
                UnicodeSegmentation::grapheme_indices(&page_string[..], true)
            {
                if screen_line_number >= screen_height - 1 {
                    break;
                }

                if grapheme_count >= screen_width as usize {
                    grapheme_count = 0;
                    screen_line_number += 1;
                    bb.push_str(&"\n\r");
                    //self.write(&"\n\r");
                }

                if string_util::is_newline(grapheme) {
                    grapheme_count = 0;
                    screen_line_number += 1;
                    bb.push_str(&"\n\r");
                    //self.write(&"\n\r");
                } else {
                    grapheme_count += 1;
                    bb.push_str(&grapheme);
                    //self.write_grapheme(
                        //&grapheme,
                        //search_offsets.contains(&(index as u64)),
                    //);
                }
            }

            for _ in screen_line_number..(screen_height - 1) {
                bb.push_str(&"~\n\r");
                //self.write(&"~\r\n");
            }

            self.write(&bb);
        }

        Ok(())
    }

    fn print_command_line(&mut self, command_line_text: String) {
        let (_screen_width, screen_height) = util::screen_width_height();
        self.write(&"\n\r");
        self.write(&command_line_text);
        self.write(&termion::cursor::Goto(
            (string_util::grapheme_count(&command_line_text[..]) + 1) as u16,
            screen_height + 1,
        ));
    }

    fn clear_screen(&mut self) {
        self.write(&termion::clear::All);
    }

    fn write_grapheme(&mut self, grapheme: &str, highlight: bool) {
        if highlight {
            let _ = write!(self.out, "{}{}", color::Bg(color::White), grapheme);
            let _ = write!(self.out, "{}", color::Bg(color::Reset));
        } else {
            let _ = write!(self.out, "{}", grapheme);
        }
    }

    fn write<S: std::fmt::Display>(&mut self, s: &S) {
        let _ = write!(self.out, "{}", s);
    }
}

fn search_offsets(start: u64, matches: &Vec<(u64, Match)>) -> Vec<u64> {
    let mut res = Vec::new();
    for (offset, mat) in matches {
        if (*offset + mat.start() as u64) < start {
            continue;
        }
        let s = *offset + mat.start() as u64 - start;
        let e = *offset + mat.end() as u64 - start;
        for i in s..e {
            res.push(i);
        }
    }

    res
}
