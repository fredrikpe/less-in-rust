use termion::terminal_size;

pub fn screen_height_half() -> usize {
    let (_, screen_height) = terminal_size().unwrap();
    eprintln!("half screen {}", screen_height as usize / 2);
    (screen_height as usize - 1) / 2
}

pub fn screen_height() -> usize {
    let (_, screen_height) = terminal_size().unwrap();
    screen_height as usize - 1
}
