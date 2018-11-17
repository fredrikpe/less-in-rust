
use termion::terminal_size;


pub fn height_half_screen() -> usize {
    let (_, screen_height) = terminal_size().unwrap();
    eprintln!("half screen {}", screen_height as usize / 2);
    screen_height as usize / 2
}
