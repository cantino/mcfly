use settings::Settings;
use history::History;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{cursor, clear};
use std::io::{Write, stdout, stdin};
use unicode_segmentation::UnicodeSegmentation;
use core::mem;
use termion::screen::AlternateScreen;

#[derive(Debug)]
pub struct Interface<'a> {
    settings: &'a Settings,
    history: &'a History,
    command: String,
    cursor: usize
}

impl <'a> Interface<'a> {
    pub fn new(settings: &'a Settings, history: &'a History) -> Interface<'a> {
        let mut interface = Interface { settings, history, command: settings.command.to_owned(), cursor: 0 };
        interface.cursor = interface.command_length();
        interface
    }

    pub fn prompt<W: Write>(&'a self, screen: &mut W) {
        write!(screen, "{}{}$ {}",
               cursor::Goto(1, 1),
               clear::CurrentLine,
               self.command
        ).unwrap();
        write!(screen, "{}{}",
               cursor::Goto(self.cursor as u16 + 3, 1),
               cursor::Show
        ).unwrap();
        screen.flush().unwrap();
    }

    fn command_length(&self) -> usize {
        self.command.graphemes(true).count()
    }

    fn move_cursor(&mut self, amt: isize) {
        let mut tmp: isize = self.cursor as isize;
        tmp += amt;
        let length = self.command_length();
        if tmp < 0 {
            tmp = 0;
        } else if tmp > length as isize {
            tmp = length as isize;
        }
        self.cursor = tmp as usize;
    }

    fn debug<W: Write, S: Into<String>>(&self, screen: &mut W, s: S) {
        write!(screen, "{}{}{}", cursor::Goto(1, 10), clear::CurrentLine, s.into()).unwrap();
        screen.flush().unwrap();
    }

    fn delete(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.move_cursor(-1);
        let mut new_command = String::with_capacity(self.command.len());
        {
            let vec = self.command.graphemes(true);
            let mut count = 0;
            for item in vec {
                if count != self.cursor {
                    new_command.push_str(item);
                }
                count += 1;
            }
        }
        mem::replace(&mut self.command, new_command);
    }

    fn insert(&mut self, c: char) {
        let mut new_command = String::with_capacity(self.command.len());
        {
            let vec = self.command.graphemes(true);
            let mut count = 0;
            let mut pushed = false;
            for item in vec {
                if count == self.cursor {
                    pushed = true;
                    new_command.push(c);
                }
                new_command.push_str(item);
                count += 1;
            }
            if !pushed {
                new_command.push(c);
            }
        }
        mem::replace(&mut self.command, new_command);
        self.move_cursor(1);
    }

    pub fn select(&'a mut self) -> String {
        let stdin = stdin();
        let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());
//        let mut screen = stdout().into_raw_mode().unwrap();
        write!(screen, "{}", clear::All).unwrap();

        self.prompt(&mut screen);

        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('\n') | Key::Char('\r') => break,
                Key::Char(c) => self.insert(c),
                Key::Alt(c) => println!("^{}", c),
                Key::Ctrl('c') | Key::Ctrl('d') | Key::Ctrl('z') => break,
                Key::Ctrl(_c) => {},
                Key::Esc => break,
                Key::Left => self.move_cursor(-1),
                Key::Right => self.move_cursor(1),
                Key::Up => {},
                Key::Down => {},
                Key::Backspace => self.delete(),
                what => {
                    println!("Got: {:?}", what);
                }
            }

            self.prompt(&mut screen);
        }

        write!(screen, "{}{}", clear::All, cursor::Show).unwrap();

        self.command.to_owned()
    }
}
