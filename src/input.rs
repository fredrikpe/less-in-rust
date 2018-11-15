
use termion::input::TermRead;
use termion::event::Event;
use termion::event::Key;
use termion::event::MouseButton;
use termion::event::MouseEvent;

use std::io::stdin;


pub enum LEvent {
    UpOneLine,
    UpHalfScreen,
    UpOneScreen,

    DownOneLine,
    DownHalfScreen,
    DownOneScreen,

    Quit,
    NoOp,
}

pub fn get_input() -> LEvent {
    let stdin = stdin();
    for c in stdin.events() {
        return match c.unwrap() {
            Event::Key(Key::Char('q')) => LEvent::Quit,

            Event::Key(Key::Ctrl('d')) => LEvent::DownHalfScreen,
            Event::Key(Key::Ctrl('u')) => LEvent::UpHalfScreen,

            Event::Key(Key::Ctrl('f')) => LEvent::DownOneScreen,
            Event::Key(Key::Ctrl('b')) => LEvent::UpOneScreen,

            Event::Mouse(MouseEvent::Press(MouseButton::WheelDown, _, _)) | 
            Event::Key(Key::Char('j')) => LEvent::DownOneLine,

            Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _)) |
            Event::Key(Key::Char('k')) => LEvent::UpOneLine,

            _ => LEvent::NoOp,
        }
    }
    LEvent::NoOp
}
