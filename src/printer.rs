use file_buffer::BiBufReader;

use termion::screen::AlternateScreen;
use termion::terminal_size;

use std::io::{Read, Seek, Write};

pub struct Printer<W: Write> {
    pub out: AlternateScreen<W>,
}

impl<W: Write> Printer<W> {
    pub fn flush(&mut self) {
        self.out.flush().unwrap();
    }

    pub fn print_screen<R: Read + Seek>(
        &mut self,
        byte_offset: i64,
        reader: &mut BiBufReader<R>,
    ) -> Result<(), ()> {

        eprintln!("{}", line_offset);
        reader.move_offset(line_offset);

        //let mut file_line_number = line_number;
        let mut screen_line_number: u16 = 1;

        self.clear_screen();

        let (_screen_width, screen_height) = terminal_size().unwrap();

        let mut line = String::new();

        while screen_line_number < screen_height {
            let _ = reader.read_line(&mut line);

            write!(self.out, "{}", termion::cursor::Goto(1, screen_line_number));
            self.write_line(&line, _screen_width, &mut screen_line_number);

            //file_line_number += 1;
            screen_line_number += 1;
            line.clear();
        }

        reader.move_up((screen_line_number - 2) as i64);

        writeln!(self.out);

        Ok(())
    }

    fn write_line(&mut self, line: &str, _screen_width: u16, screen_line_number: &mut u16) {
        let mut char_count = 0;
        for c in line.chars() {
            char_count += 1;
            if char_count >= _screen_width {
                *screen_line_number += 1;
                char_count = 0;
                writeln!(self.out);
                write!(
                    self.out,
                    "{}",
                    termion::cursor::Goto(1, *screen_line_number)
                );
            }
            write!(self.out, "{}", c);
        }
    }

    fn clear_screen(&mut self) {
        write!(self.out, "{}", termion::clear::All);
    }
}
