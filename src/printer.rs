use file_buffer::BiBufReader;

use termion::screen::AlternateScreen;
use termion::terminal_size;

use std::io::{Read, Seek, Write};

use file_buffer;

pub struct Printer<W: Write> {
    pub out: AlternateScreen<W>,
}

impl<W: Write> Printer<W> {
    pub fn flush(&mut self) {
        self.out.flush().unwrap();
    }

    pub fn print_screen<R: Read + Seek>(&mut self, reader: &mut BiBufReader<R>)
        -> Result<(), ()>
    {
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
        let mut char_count = 0;
        for c in page.chars() {
            if screen_line_number >= screen_height {
                break;
            }

            char_count += 1;
            // An estimate
            if char_count >= screen_width {
                screen_line_number += 1;
                char_count = 0;
                writeln!(self.out);
                write!(self.out, "{}", termion::cursor::Goto(1, screen_line_number));
            }

            if file_buffer::is_new_line(c) {
                screen_line_number += 1;
                char_count = 0;
                writeln!(self.out);
                write!(self.out, "{}", termion::cursor::Goto(1, screen_line_number));
            } else {
                write!(self.out, "{}", c);
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
