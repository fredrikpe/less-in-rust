use termion::event::Event;
use termion::event::Key;
use termion::event::MouseButton;
use termion::event::MouseEvent;
use termion::input::TermRead;

use std::io::stdin;

#[derive(Debug)]
pub enum LEvent {
    UpOneLine,
    UpHalfScreen,
    UpOneScreen,

    DownOneLine,
    DownHalfScreen,
    DownOneScreen,

    JumpBeginning,
    JumpEnd,

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

            Event::Mouse(MouseEvent::Press(MouseButton::WheelDown, _, _))
            | Event::Key(Key::Char('j')) => LEvent::DownOneLine,

            Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _))
            | Event::Key(Key::Char('k')) => LEvent::UpOneLine,

            Event::Key(Key::Char('g')) => LEvent::JumpBeginning,
            Event::Key(Key::Char('G')) => LEvent::JumpEnd,

            _ => LEvent::NoOp,
        };
    }
    LEvent::NoOp
}
