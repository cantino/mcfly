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
    selection: usize,
    matches: Vec<Command>
}

#[derive(Debug)]
pub enum MoveSelection {
    Up,
    Down
}

pub trait GraphemeStrings {
    fn push_grapheme_str<S: Into<String>>(&mut self, s: S, max_width: u16);
}

impl GraphemeStrings for String {
    fn push_grapheme_str<S: Into<String>>(&mut self, s: S, max_width: u16) {
        let initial_length = self.graphemes(true).count();
        for (index, grapheme) in s.into().graphemes(true).enumerate() {
            if initial_length + index >= max_width as usize {
                return;
            }
            self.push_str(grapheme);
        }
    }
}

impl <'a> Interface<'a> {
    pub fn new(settings: &'a Settings, history: &'a History) -> Interface<'a> {
        Interface { settings, history, input: CommandInput::from(settings.command.to_owned()), selection: 0, matches: Vec::new() }
    }

    pub fn prompt<W: Write>(&self, screen: &mut W) {
        write!(screen, "{}{}{}$ {}",
               color::Fg(color::LightWhite).to_string(),
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

    pub fn results<W: Write>(&mut self, screen: &mut W) {
        write!(screen, "{}{}{}", cursor::Hide, cursor::Goto(1, 3), clear::All).unwrap();
        let (width, _height): (u16, u16) = terminal_size().unwrap();

        if self.matches.len() > 0 && self.selection > self.matches.len() - 1 {
            self.selection = self.matches.len() - 1;
        }

        for (index, command) in self.matches.iter().enumerate() {
            let mut fg = color::Fg(color::LightWhite).to_string();
            let mut bg = color::Bg(color::Reset).to_string();

            if index == self.selection {
                fg = color::Fg(color::Black).to_string();
                bg = color::Bg(color::LightWhite).to_string();
            }

            write!(screen, "{}{}", fg, bg).unwrap();

            write!(screen, "{}{}",
                   cursor::Goto(1, index as u16 + 3),
                   Interface::truncate_for_display(command,
                                                   &self.input.command,
                                                   width,
                                                   color::Fg(color::Green).to_string(),
                                                   fg,
                                                   self.settings.debug
                   )
            ).unwrap();

            write!(screen, "{}", color::Bg(color::Reset)).unwrap();
            write!(screen, "{}", color::Fg(color::Reset)).unwrap();
        }
        screen.flush().unwrap();
    }

    #[allow(unused)]
    fn debug<W: Write, S: Into<String>>(&self, screen: &mut W, s: S) {
        write!(screen, "{}{}{}", cursor::Goto(1, 2), clear::CurrentLine, s.into()).unwrap();
        screen.flush().unwrap();
    }

    fn move_selection(&mut self, direction: MoveSelection) {
        match direction {
            MoveSelection::Up => {
                if self.selection > 0 {
                    self.selection -= 1;
                }
            },
            MoveSelection::Down => {
                self.selection += 1;
            }
        }
    }
    
    fn accept_selection(&mut self) {
        if self.matches.len() > 0 {
            self.input.set(&self.matches[self.selection].cmd);
        }
    }

    fn refresh_matches(&mut self) {
        self.selection = 0;
        self.matches = self.history.find_matches(&self.input.command, Some(10));
    }

    pub fn select(&'a mut self) -> String {
        let stdin = stdin();
        let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());
//        let mut screen = stdout().into_raw_mode().unwrap();
        write!(screen, "{}", clear::All).unwrap();

        self.refresh_matches();
        self.results(&mut screen);
        self.prompt(&mut screen);

        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('\n') | Key::Char('\r') | Key::Char('\t') | Key::Ctrl('j') => {
                    self.accept_selection();
                    break;
                },
                Key::Ctrl('c') | Key::Ctrl('d') | Key::Ctrl('g') | Key::Ctrl('z') => {
                    self.input.clear();
                    break
                },
                Key::Ctrl('b') => self.input.move_cursor(Move::Backward),
                Key::Ctrl('f') => self.input.move_cursor(Move::Forward),
                Key::Ctrl('a') => self.input.move_cursor(Move::BOL),
                Key::Ctrl('e') => self.input.move_cursor(Move::EOL),
                Key::Ctrl('w') | Key::Alt('\x08') | Key::Alt('\x7f') => {
                    self.input.delete(Move::BackwardWord);
                    self.refresh_matches();
                },
                Key::Alt('d') => {
                    self.input.delete(Move::ForwardWord);
                    self.refresh_matches();
                },
                Key::Alt('b') => self.input.move_cursor(Move::BackwardWord),
                Key::Alt('f') => self.input.move_cursor(Move::ForwardWord),
                Key::Left => self.input.move_cursor(Move::Backward),
                Key::Right => self.input.move_cursor(Move::Forward),
                Key::Up | Key::PageUp => self.move_selection(MoveSelection::Up),
                Key::Down | Key::PageDown => self.move_selection(MoveSelection::Down),
                Key::Ctrl('k') => {
                    self.input.delete(Move::EOL);
                    self.refresh_matches();
                },
                Key::Ctrl('u') => {
                    self.input.delete(Move::BOL);
                    self.refresh_matches();
                },
                Key::Backspace | Key::Ctrl('h') => {
                    self.input.delete(Move::Backward);
                    self.refresh_matches();
                },
                Key::Delete => {
                    self.input.delete(Move::Forward);
                    self.refresh_matches();
                },
                Key::Home => self.input.move_cursor(Move::BOL),
                Key::End => self.input.move_cursor(Move::EOL),
                Key::Char(c) => {
                    self.input.insert(c);
                    self.refresh_matches();
                },
                Key::Ctrl(_c) => {
//                    self.debug(&mut screen, format!("Ctrl({})", c))
                },
                Key::Alt(_c) => {
//                    self.debug(&mut screen, format!("Alt({})", c))
                },
                Key::F(_c) => {
//                    self.debug(&mut screen, format!("F({})", c))
                },
                Key::Insert | Key::Null | Key::__IsNotComplete | Key::Esc => {}
            }

            self.results(&mut screen);
            self.prompt(&mut screen);
        }

        write!(screen, "{}{}", clear::All, cursor::Show).unwrap();

        self.input.command.to_owned()
    }

    fn truncate_for_display(command: &Command, search: &str, width: u16, highlight_color: String, base_color: String, debug: bool) -> String {
        let mut out = String::new();
        let mut prev = 0;
        let debug_space = if debug { 60 } else { 0 };
        let available_width = if width > debug_space { width - debug_space } else { 2 };

        if !search.is_empty() {
            for (index, _) in command.cmd.match_indices(search) {
                if prev != index {
                    out.push_grapheme_str(&command.cmd[prev..index], available_width);
                }
                out.push_str(&highlight_color);
                out.push_grapheme_str(search, available_width);
                out.push_str(&base_color);
                prev = index + search.len();
            }
        }

        if prev != command.cmd.len() {
            out.push_grapheme_str(&command.cmd[prev..], available_width);
        }

        if debug {
            out.push_grapheme_str(format!("  {}rnk: {} age: {} ext: {} ls: {} ovlp: {} occ: {}{}", color::Fg(color::LightBlue).to_string(), command.rank, command.age_factor, command.exit_factor, command.dir_factor, command.overlap_factor, command.occurrences_factor, &base_color), available_width + debug_space);
        }

        out
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
