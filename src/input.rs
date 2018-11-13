
use termion::input::TermRead;
use termion::event::Event;
use termion::event::Key;
use termion::event::MouseButton;
use termion::event::MouseEvent;

use std::io::stdin;


pub enum LEvent {
    LineUp,
    LineDown,
    Quit,
    NoOp,
}


pub fn get_input() -> LEvent {
    let stdin = stdin();
    for c in stdin.events() {
        return match c.unwrap() {
            Event::Key(Key::Char('q')) => {
                println!("quitpressesd");
                LEvent::Quit
            }

            Event::Mouse(MouseEvent::Press(MouseButton::WheelDown, _, _)) | 
            Event::Key(Key::Char('j')) => LEvent::LineDown,

            Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _)) |
            Event::Key(Key::Char('k')) => LEvent::LineUp,

            _ => LEvent::NoOp,
        }
    }
    LEvent::NoOp
}
