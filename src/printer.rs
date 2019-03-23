use termion::color;
use termion::screen::AlternateScreen;
use unicode_segmentation::UnicodeSegmentation;

use std::io::Write;
use std::str;

use grep::matcher::Match;

use string_util;
use util;

struct ColoredString {
    string: String,
    color: bool,
}

impl ColoredString {
    pub fn new(s: &str, color: bool) -> ColoredString {
        ColoredString {
            string: s.to_string(),
            color: color,
        }
    }
}

pub struct Printer<W: Write> {
    pub out: AlternateScreen<W>,
    output_buffer: Vec<ColoredString>,
}

impl<W: Write> Printer<W> {
    pub fn new(output: W) -> Printer<W> {
        Printer {
            out: AlternateScreen::from(output),
            output_buffer: Vec::new(),
        }
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

        self.output_buffer.push(ColoredString::new("", false));

        unsafe {
            let page_string = str::from_utf8_unchecked(&page[..]);

            write(&mut self.out, &termion::cursor::Goto(1, 1));

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
                    self.push_newline();
                }

                if string_util::is_newline(grapheme) {
                    grapheme_count = 0;
                    screen_line_number += 1;
                    self.push_newline();
                } else {
                    grapheme_count += 1;
                    self.push_str(
                        &grapheme,
                        search_offsets.contains(&(index as u64)),
                    );
                }
            }

            for _ in screen_line_number..(screen_height - 1) {
                self.push_tilde_newline();
            }

            self.write_output_buffer();
            self.output_buffer.clear();
        }

        Ok(())
    }

    fn print_command_line(&mut self, command_line_text: String) {
        let (_screen_width, screen_height) = util::screen_width_height();
        write(&mut self.out, &"\n\r");
        write(&mut self.out, &command_line_text);
        write(
            &mut self.out,
            &termion::cursor::Goto(
                (string_util::grapheme_count(&command_line_text[..]) + 1)
                    as u16,
                screen_height + 1,
            ),
        );
    }

    fn clear_screen(&mut self) {
        write(&mut self.out, &termion::clear::All);
    }

    fn flush(&mut self) {
        self.out.flush().unwrap();
    }

    fn write_output_buffer(&mut self) {
        for colored_string in &self.output_buffer {
            if colored_string.color {
                write_higlight(&mut self.out, &colored_string.string);
            } else {
                write(&mut self.out, &colored_string.string);
            }
        }
    }

    fn push_tilde_newline(&mut self) {
        self.push_str(&"~\n\r", false);
    }

    fn push_newline(&mut self) {
        self.push_str(&"\n\r", false);
    }

    fn push_str(&mut self, grapheme: &str, colored: bool) {
        let last_index = self.output_buffer.len() - 1;

        if self.output_buffer[last_index].color == colored {
            self.output_buffer[last_index].string.push_str(grapheme);
        } else {
            self.output_buffer
                .push(ColoredString::new(grapheme, colored));
        }
    }
}

fn write_higlight<D: std::fmt::Display, W: Write>(
    out: &mut AlternateScreen<W>,
    text: &D,
) {
    let _ = write!(
        out,
        "{}{}{}",
        color::Fg(color::Black),
        color::Bg(color::White),
        text
    );
    let _ = write!(
        out,
        "{}{}",
        color::Fg(color::Reset),
        color::Bg(color::Reset)
    );
}

fn write<D: std::fmt::Display, W: Write>(
    out: &mut AlternateScreen<W>,
    text: &D,
) {
    let _ = write!(out, "{}", text);
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
        // Are X mathces enough?
        if res.len() > 3000 {
            break;
        }
    }
    res
}
