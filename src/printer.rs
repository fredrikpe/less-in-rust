
use file_buffer::BiBufReader;

use termion::terminal_size;

use std::io::BufReader;
use std::io::BufRead;
use std::io::Read;
use std::io::Seek;


pub fn print_screen<R: Read + Seek>(line_offset: i64, reader: &mut BiBufReader<R>)
        -> Result<(), ()> {

    print!("{}", line_offset);
    reader.move_offset(line_offset);

    //let mut file_line_number = line_number;
    let mut screen_line_number: u16 = 1;

    clear_screen();

    let (screen_width, screen_height) = terminal_size().unwrap();

    let mut line = String::new();

    while screen_line_number < screen_height {

        let _ = reader.read_line(&mut line);

        print!("{}", termion::cursor::Goto(1, screen_line_number));
        println!("{}", line);

        //file_line_number += 1;
        screen_line_number += 1;
        line.clear();
    }
    
    reader.move_up((screen_line_number - 1) as i64);

    println!();

    Ok(())
}


fn clear_screen() {
    print!("{}", termion::clear::All);
}

