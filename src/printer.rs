
use file_buffer::BiBufReader;

use termion::terminal_size;
use termion::screen::AlternateScreen;

use std::io::BufReader;
use std::io::{Read, Seek, Write};

pub struct Printer<W: Write> {
    pub out: AlternateScreen<W>,
}

impl<W: Write> Printer<W> {
    pub fn flush(&mut self) {
        self.out.flush().unwrap();
    }

    pub fn print_screen<R: Read + Seek>(&mut self, line_offset: i64, reader: &mut BiBufReader<R>)
            -> Result<(), ()> {

        reader.move_offset(line_offset);

        //let mut file_line_number = line_number;
        let mut screen_line_number: u16 = 1;

        self.clear_screen();

        let (_screen_width, screen_height) = terminal_size().unwrap();

        let mut line = String::new();

        while screen_line_number < screen_height {

            let _ = reader.read_line(&mut line);

            write!(self.out, "{}", termion::cursor::Goto(1, screen_line_number));
            writeln!(self.out, "{}", line);

            //file_line_number += 1;
            screen_line_number += 1;
            line.clear();
        }
        
        reader.move_up((screen_line_number - 1) as i64);

        writeln!(self.out);

        Ok(())
    }

    fn clear_screen(&mut self) {
        write!(self.out, "{}", termion::clear::All);
    }
}

