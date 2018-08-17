use settings::Settings;
use history::History;
use command_input::{CommandInput, Move};
use unicode_segmentation::UnicodeSegmentation;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{cursor, clear, terminal_size};
use std::io::{Write, stdout, stdin};
use termion::screen::AlternateScreen;
use history::Command;
use termion::color;

#[derive(Debug)]
pub struct Interface<'a> {
    settings: &'a Settings,
    history: &'a History,
    input: CommandInput,
    cursor: usize
}

pub trait GraphemeStrings {
    fn push_grapheme_str<S: Into<String>>(&mut self, s: S, max_width: usize);
}

impl GraphemeStrings for String {
    fn push_grapheme_str<S: Into<String>>(&mut self, s: S, max_width: usize) {
        let initial_length = self.graphemes(true).count();
        for (index, grapheme) in s.into().graphemes(true).enumerate() {
            if initial_length + index >= max_width {
                return;
            }
            self.push_str(grapheme);
        }
    }
}

impl <'a> Interface<'a> {
    pub fn new(settings: &'a Settings, history: &'a History) -> Interface<'a> {
        Interface { settings, history, input: CommandInput::from(settings.command.to_owned()), cursor: 0 }
    }

    pub fn prompt<W: Write>(&'a self, screen: &mut W) {
        write!(screen, "{}{}$ {}",
               cursor::Goto(1, 1),
               clear::CurrentLine,
               self.input
        ).unwrap();
        write!(screen, "{}{}",
               cursor::Goto(self.input.cursor as u16 + 3, 1),
               cursor::Show
        ).unwrap();
        screen.flush().unwrap();
    }

    fn truncate_for_display(command: &Command, search: &str, width: u16) -> String {
        let mut out = String::new();

        let mut prev = 0;

        if !search.is_empty() {
            for (index, _) in command.cmd.match_indices(search) {
                if prev != index {
                    out.push_grapheme_str(&command.cmd[prev..index], width as usize);
                }
                out.push_str(&color::Fg(color::Green).to_string());
                out.push_grapheme_str(search, width as usize);
                out.push_str(&color::Fg(color::Reset).to_string());
                prev = index + search.len();
            }
        }

        if prev != command.cmd.len() {
            out.push_grapheme_str(&command.cmd[prev..], width as usize);
        }

        out
    }

    pub fn results<W: Write>(&'a self, screen: &mut W) {
        write!(screen, "{}{}{}", cursor::Hide, cursor::Goto(1, 3), clear::All).unwrap();
        let (width, height): (u16, u16) = terminal_size().unwrap();
        for (index, command) in self.history.find_matches(&self.input.command).iter().enumerate() {
            write!(screen, "{}{}",
                   cursor::Goto(1, index as u16 + 3),
                   Interface::truncate_for_display(command, &self.input.command, width)
            ).unwrap();
        }
        screen.flush().unwrap();
    }

    fn debug<W: Write, S: Into<String>>(&self, screen: &mut W, s: S) {
        write!(screen, "{}{}{}", cursor::Goto(1, 2), clear::CurrentLine, s.into()).unwrap();
        screen.flush().unwrap();
    }

    pub fn select(&'a mut self) -> String {
        let stdin = stdin();
//        let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());
        let mut screen = stdout().into_raw_mode().unwrap();
        write!(screen, "{}", clear::All).unwrap();

        self.results(&mut screen);
        self.prompt(&mut screen);

        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('\n') | Key::Char('\r') | Key::Char('\t') | Key::Ctrl('j') => break,
                Key::Ctrl('c') | Key::Ctrl('d') | Key::Ctrl('g') | Key::Ctrl('z') => {
                    self.input.clear();
                    break
                },
                Key::Ctrl('b') => self.input.move_cursor(Move::Backward),
                Key::Ctrl('f') => self.input.move_cursor(Move::Forward),
                Key::Ctrl('a') => self.input.move_cursor(Move::BOL),
                Key::Ctrl('e') => self.input.move_cursor(Move::EOL),
                Key::Ctrl('w') | Key::Alt('\x08') | Key::Alt('\x7f') => self.input.delete(Move::BackwardWord),
                Key::Alt('d') => self.input.delete(Move::ForwardWord),
                Key::Alt('b') => self.input.move_cursor(Move::BackwardWord),
                Key::Alt('f') => self.input.move_cursor(Move::ForwardWord),
                Key::Left => self.input.move_cursor(Move::Backward),
                Key::Right => self.input.move_cursor(Move::Forward),
                Key::Up | Key::PageUp => {},
                Key::Down | Key::PageDown => {},
                Key::Ctrl('k') => self.input.delete(Move::EOL),
                Key::Ctrl('u') => self.input.delete(Move::BOL),
                Key::Backspace | Key::Ctrl('h') => self.input.delete(Move::Backward),
                Key::Delete => self.input.delete(Move::Forward),
                Key::Home => self.input.move_cursor(Move::BOL),
                Key::End => self.input.move_cursor(Move::EOL),
                Key::Char(c) => self.input.insert(c),
                Key::Ctrl(c) => {
//                    self.debug(&mut screen, format!("Ctrl({})", c))
                },
                Key::Alt(c) => {
//                    self.debug(&mut screen, format!("Alt({})", c))
                },
                Key::F(k) => {
//                    self.debug(&mut screen, format!("F({})", k))
                },
                Key::Insert | Key::Null | Key::__IsNotComplete | Key::Esc => {}
            }

            self.results(&mut screen);
            self.prompt(&mut screen);
        }

        write!(screen, "{}{}", clear::All, cursor::Show).unwrap();

        self.input.command.to_owned()
    }
}

// TODO:
// Ctrl('X') + Ctrl('U') => undo
// Ctrl('X') + Ctrl('G') => abort
// Meta('c') => capitalize word
// Meta('l') => downcase word
// Meta('t') => transpose words
// Meta('u') => upcase word
// Meta('y') => yank pop
// Ctrl('r') => reverse history search
// Ctrl('s') => forward history search
// Ctrl('t') => transpose characters
// Ctrl('q') | Ctrl('v') => quoted insert
// Ctrl('y') => yank
// Ctrl('_') => undo
