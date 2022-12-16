use crate::command_input::{CommandInput, Move};
use crate::history::History;

use crate::fixed_length_grapheme_string::FixedLengthGraphemeString;
use crate::history::Command;
use crate::history_cleaner;
use crate::settings::{InterfaceView, KeyScheme};
use crate::settings::{ResultSort, Settings};
use chrono::{Duration, TimeZone, Utc};
use humantime::format_duration;
use std::io::{stdin, stdout, Write};
use termion::color;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::{clear, cursor, terminal_size};

pub struct Interface<'a> {
    history: &'a History,
    settings: &'a Settings,
    input: CommandInput,
    selection: usize,
    matches: Vec<Command>,
    debug: bool,
    run: bool,
    delete_requests: Vec<String>,
    menu_mode: MenuMode,
    in_vim_insert_mode: bool,
    result_sort: ResultSort,
}

pub struct SelectionResult {
    /// Whether the user requested to run the resulting command immediately.
    pub run: bool,
    /// The command string the user selected, if any.
    pub selection: Option<String>,
    /// Commands the user has requested be deleted from shell history.
    pub delete_requests: Vec<String>,
}

pub enum MoveSelection {
    Up,
    Down,
}

#[derive(PartialEq, Eq)]
pub enum MenuMode {
    Normal,
    ConfirmDelete,
}

impl MenuMode {
    fn text(&self, interface: &Interface) -> String {
        let mut menu_text = String::from("McFly");
        match *self {
            MenuMode::Normal => match interface.settings.key_scheme {
                KeyScheme::Emacs => menu_text.push_str(" | ESC - Exit | "),
                KeyScheme::Vim => {
                    if interface.in_vim_insert_mode {
                        menu_text.push_str(" (Ins) | ESC - Cmd | ");
                    } else {
                        menu_text.push_str(" (Cmd) | ESC - Exit | ");
                    }
                }
            },
            MenuMode::ConfirmDelete => {
                return String::from("Delete selected command from the history? (Y/N)")
            }
        }

        menu_text.push_str("⏎ - Run | TAB - Edit | ");

        match interface.result_sort {
            ResultSort::Rank => menu_text.push_str("F1 - Switch Sort to Time | "),
            ResultSort::LastRun => menu_text.push_str("F1 - Switch Sort to Rank | "),
        }

        menu_text.push_str("F2 - Delete");
        menu_text
    }

    fn bg(&self) -> String {
        match *self {
            MenuMode::Normal => color::Bg(color::LightBlue).to_string(),
            MenuMode::ConfirmDelete => color::Bg(color::Red).to_string(),
        }
    }
}

const PROMPT_LINE_INDEX: u16 = 3;
const INFO_LINE_INDEX: u16 = 1;
const RESULTS_TOP_INDEX: u16 = 5;

impl<'a> Interface<'a> {
    pub fn new(settings: &'a Settings, history: &'a History) -> Interface<'a> {
        Interface {
            history,
            settings,
            input: CommandInput::from(settings.command.to_owned()),
            selection: 0,
            matches: Vec::new(),
            debug: settings.debug,
            run: false,
            delete_requests: Vec::new(),
            menu_mode: MenuMode::Normal,
            in_vim_insert_mode: true,
            result_sort: settings.result_sort.to_owned(),
        }
    }

    pub fn display(&mut self) -> SelectionResult {
        self.build_cache_table();
        self.select();

        let command = self.input.command.to_owned();

        if command.chars().any(|c| !c.is_whitespace()) {
            self.history.record_selected_from_ui(
                &command,
                &self.settings.session_id,
                &self.settings.dir,
            );
            SelectionResult {
                run: self.run,
                selection: Some(command),
                // Remove delete_requests from the Interface, in case it's used to display() again.
                delete_requests: self.delete_requests.split_off(0),
            }
        } else {
            SelectionResult {
                run: self.run,
                selection: None,
                delete_requests: self.delete_requests.split_off(0),
            }
        }
    }

    fn build_cache_table(&self) {
        self.history.build_cache_table(
            &self.settings.dir.to_owned(),
            &Some(self.settings.session_id.to_owned()),
            None,
            None,
            None,
            self.settings.limit.to_owned(),
        );
    }

    fn menubar<W: Write>(&self, screen: &mut W) {
        if !self.settings.disable_menu {
            let (width, _height): (u16, u16) = terminal_size().unwrap();
            write!(
                screen,
                "{hide}{cursor}{clear}{fg}{bg}{text:width$}{reset_bg}",
                hide = cursor::Hide,
                fg = color::Fg(color::LightWhite),
                bg = self.menu_mode.bg(),
                cursor = cursor::Goto(1, self.info_line_index()),
                clear = clear::CurrentLine,
                text = self.menu_mode.text(self),
                reset_bg = color::Bg(color::Reset),
                width = width as usize
            )
            .unwrap();
            screen.flush().unwrap();
        }
    }

    fn prompt<W: Write>(&self, screen: &mut W) {
        let prompt_line_index = self.prompt_line_index();
        write!(
            screen,
            "{}{}{}$ {}",
            if self.settings.lightmode {
                color::Fg(color::Black).to_string()
            } else {
                color::Fg(color::LightWhite).to_string()
            },
            cursor::Goto(1, self.prompt_line_index()),
            clear::CurrentLine,
            self.input
        )
        .unwrap();
        write!(
            screen,
            "{}{}",
            cursor::Goto(self.input.cursor as u16 + 3, prompt_line_index),
            cursor::Show
        )
        .unwrap();
        screen.flush().unwrap();
    }

    fn debug_cursor<W: Write>(&self, screen: &mut W) {
        let result_top_index = self.result_top_index();
        write!(
            screen,
            "{}{}",
            cursor::Hide,
            cursor::Goto(0, result_top_index + self.settings.results + 1)
        )
        .unwrap();
        screen.flush().unwrap();
    }

    fn results<W: Write>(&mut self, screen: &mut W) {
        let result_top_index = self.result_top_index();
        write!(
            screen,
            "{}{}{}",
            cursor::Hide,
            cursor::Goto(1, result_top_index),
            clear::All
        )
        .unwrap();
        let (width, _height): (u16, u16) = terminal_size().unwrap();

        if !self.matches.is_empty() && self.selection > self.matches.len() - 1 {
            self.selection = self.matches.len() - 1;
        }

        for (index, command) in self.matches.iter().enumerate() {
            let mut fg = if self.settings.lightmode {
                color::Fg(color::Black).to_string()
            } else {
                color::Fg(color::LightWhite).to_string()
            };

            let mut highlight = if self.settings.lightmode {
                color::Fg(color::Blue).to_string()
            } else {
                color::Fg(color::Green).to_string()
            };

            let mut bg = color::Bg(color::Reset).to_string();

            if index == self.selection {
                if self.settings.lightmode {
                    fg = color::Fg(color::LightWhite).to_string();
                    bg = color::Bg(color::LightBlack).to_string();
                    highlight = color::Fg(color::White).to_string();
                } else {
                    fg = color::Fg(color::Black).to_string();
                    bg = color::Bg(color::LightWhite).to_string();
                    highlight = color::Fg(color::Green).to_string();
                }
            }

            write!(screen, "{}{}", fg, bg).unwrap();

            let command_line_index = self.command_line_index(index as i16);

            write!(
                screen,
                "{}{}",
                cursor::Goto(1, (command_line_index + result_top_index as i16) as u16),
                Interface::truncate_for_display(
                    command,
                    &self.input.command,
                    width,
                    highlight,
                    fg,
                    self.debug
                )
            )
            .unwrap();

            if command.last_run.is_some() {
                write!(
                    screen,
                    "{}",
                    cursor::Goto(
                        width - 9,
                        (command_line_index + result_top_index as i16) as u16
                    )
                )
                .unwrap();

                let duration = &format_duration(
                    Duration::minutes(
                        Utc::now()
                            .signed_duration_since(
                                Utc.timestamp_opt(command.last_run.unwrap(), 0).unwrap(),
                            )
                            .num_minutes(),
                    )
                    .to_std()
                    .unwrap(),
                )
                .to_string()
                .split(' ')
                .take(2)
                .map(|s| {
                    s.replace("years", "y")
                        .replace("year", "y")
                        .replace("months", "mo")
                        .replace("month", "mo")
                        .replace("days", "d")
                        .replace("day", "d")
                        .replace("hours", "h")
                        .replace("hour", "h")
                        .replace("minutes", "m")
                        .replace("minute", "m")
                        .replace("0s", "< 1m")
                })
                .collect::<Vec<String>>()
                .join(" ");

                let highlight = if self.settings.lightmode {
                    color::Fg(color::Blue).to_string()
                } else {
                    color::Fg(color::LightBlue).to_string()
                };

                write!(screen, "{}", highlight).unwrap();
                write!(screen, "{:>9}", duration).unwrap();
            }

            write!(screen, "{}", color::Bg(color::Reset)).unwrap();
            write!(screen, "{}", color::Fg(color::Reset)).unwrap();
        }
        screen.flush().unwrap();
    }

    #[allow(unused)]
    fn debug<W: Write, S: Into<String>>(&self, screen: &mut W, s: S) {
        write!(
            screen,
            "{}{}{}",
            cursor::Goto(1, 2),
            clear::CurrentLine,
            s.into()
        )
        .unwrap();
        screen.flush().unwrap();
    }

    fn move_selection(&mut self, direction: MoveSelection) {
        if self.is_screen_view_bottom() {
            match direction {
                MoveSelection::Up => {
                    self.selection += 1;
                }
                MoveSelection::Down => {
                    if self.selection > 0 {
                        self.selection -= 1;
                    }
                }
            }
        } else {
            match direction {
                MoveSelection::Up => {
                    if self.selection > 0 {
                        self.selection -= 1;
                    }
                }
                MoveSelection::Down => {
                    self.selection += 1;
                }
            }
        }
    }

    fn accept_selection(&mut self) {
        if !self.matches.is_empty() {
            self.input.set(&self.matches[self.selection].cmd);
        }
    }

    fn confirm(&mut self, confirmation: bool) {
        if confirmation {
            if let MenuMode::ConfirmDelete = self.menu_mode {
                self.delete_selection()
            }
        }
        self.menu_mode = MenuMode::Normal;
    }

    fn delete_selection(&mut self) {
        if !self.matches.is_empty() {
            {
                let command = &self.matches[self.selection];
                history_cleaner::clean(self.settings, self.history, &command.cmd);
                self.delete_requests.push(command.cmd.clone());
            }
            self.build_cache_table();
            self.refresh_matches(false);
        }
    }

    fn refresh_matches(&mut self, reset_selection: bool) {
        if reset_selection {
            self.selection = 0;
        }
        self.matches = self.history.find_matches(
            &self.input.command,
            self.settings.results as i16,
            self.settings.fuzzy,
            &self.result_sort,
        );
    }

    fn switch_result_sort(&mut self) {
        match self.result_sort {
            ResultSort::Rank => self.result_sort = ResultSort::LastRun,
            ResultSort::LastRun => self.result_sort = ResultSort::Rank,
        }
    }

    fn select(&mut self) {
        let stdin = stdin();
        let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());
        //        let mut screen = stdout().into_raw_mode().unwrap();
        write!(screen, "{}", clear::All).unwrap();

        self.refresh_matches(true);
        self.results(&mut screen);
        self.menubar(&mut screen);
        self.prompt(&mut screen);

        for c in stdin.keys() {
            self.debug_cursor(&mut screen);

            if self.menu_mode != MenuMode::Normal {
                match c.unwrap() {
                    Key::Ctrl('c')
                    | Key::Ctrl('d')
                    | Key::Ctrl('g')
                    | Key::Ctrl('z')
                    | Key::Ctrl('r') => {
                        self.run = false;
                        self.input.clear();
                        break;
                    }
                    Key::Char('y') | Key::Char('Y') => {
                        self.confirm(true);
                    }
                    Key::Char('n') | Key::Char('N') | Key::Esc => {
                        self.confirm(false);
                    }
                    _ => {}
                }
            } else {
                let early_out = match self.settings.key_scheme {
                    KeyScheme::Emacs => self.select_with_emacs_key_scheme(c.unwrap()),
                    KeyScheme::Vim => self.select_with_vim_key_scheme(c.unwrap()),
                };

                if early_out {
                    break;
                }
            }

            self.results(&mut screen);
            self.menubar(&mut screen);
            self.prompt(&mut screen);
        }

        write!(screen, "{}{}", clear::All, cursor::Show).unwrap();
    }

    fn select_with_emacs_key_scheme(&mut self, k: Key) -> bool {
        match k {
            Key::Char('\n') | Key::Char('\r') | Key::Ctrl('j') => {
                self.run = true;
                self.accept_selection();
                return true;
            }
            Key::Char('\t') => {
                self.run = false;
                self.accept_selection();
                return true;
            }
            Key::Ctrl('c') | Key::Ctrl('g') | Key::Ctrl('z') | Key::Esc | Key::Ctrl('r') => {
                self.run = false;
                self.input.clear();
                return true;
            }
            Key::Ctrl('b') => self.input.move_cursor(Move::Backward),
            Key::Ctrl('f') => self.input.move_cursor(Move::Forward),
            Key::Ctrl('a') => self.input.move_cursor(Move::BOL),
            Key::Ctrl('e') => self.input.move_cursor(Move::EOL),
            Key::Ctrl('w') | Key::Alt('\x08') | Key::Alt('\x7f') => {
                self.input.delete(Move::BackwardWord);
                self.refresh_matches(true);
            }
            Key::Alt('d') => {
                self.input.delete(Move::ForwardWord);
                self.refresh_matches(true);
            }
            Key::Ctrl('v') => {
                self.debug = !self.debug;
            }
            Key::Alt('b') => self.input.move_cursor(Move::BackwardWord),
            Key::Alt('f') => self.input.move_cursor(Move::ForwardWord),
            Key::Left => self.input.move_cursor(Move::Backward),
            Key::Right => self.input.move_cursor(Move::Forward),
            Key::Up | Key::PageUp | Key::Ctrl('p') => self.move_selection(MoveSelection::Up),
            Key::Down | Key::PageDown | Key::Ctrl('n') => self.move_selection(MoveSelection::Down),
            Key::Ctrl('k') => {
                self.input.delete(Move::EOL);
                self.refresh_matches(true);
            }
            Key::Ctrl('u') => {
                self.input.delete(Move::BOL);
                self.refresh_matches(true);
                self.selection = 0;
            }
            Key::Backspace | Key::Ctrl('h') => {
                self.input.delete(Move::Backward);
                self.refresh_matches(true);
            }
            Key::Delete | Key::Ctrl('d') => {
                self.input.delete(Move::Forward);
                self.refresh_matches(true);
            }
            Key::Home => self.input.move_cursor(Move::BOL),
            Key::End => self.input.move_cursor(Move::EOL),
            Key::Char(c) => {
                self.input.insert(c);
                self.refresh_matches(true);
            }
            Key::F(1) => {
                self.switch_result_sort();
                self.refresh_matches(true);
            }
            Key::F(2) => {
                if !self.matches.is_empty() {
                    if self.settings.delete_without_confirm {
                        self.delete_selection();
                    } else {
                        self.menu_mode = MenuMode::ConfirmDelete;
                    }
                }
            }
            _ => {}
        }

        false
    }

    fn select_with_vim_key_scheme(&mut self, k: Key) -> bool {
        if self.in_vim_insert_mode {
            match k {
                Key::Char('\n') | Key::Char('\r') | Key::Ctrl('j') => {
                    self.run = true;
                    self.accept_selection();
                    return true;
                }
                Key::Char('\t') => {
                    self.run = false;
                    self.accept_selection();
                    return true;
                }
                Key::Ctrl('c') | Key::Ctrl('g') | Key::Ctrl('z') | Key::Ctrl('r') => {
                    self.run = false;
                    self.input.clear();
                    return true;
                }
                Key::Left => self.input.move_cursor(Move::Backward),
                Key::Right => self.input.move_cursor(Move::Forward),
                Key::Up | Key::PageUp | Key::Ctrl('u') | Key::Ctrl('p') => {
                    self.move_selection(MoveSelection::Up)
                }
                Key::Down | Key::PageDown | Key::Ctrl('d') | Key::Ctrl('n') => {
                    self.move_selection(MoveSelection::Down)
                }
                Key::Esc => self.in_vim_insert_mode = false,
                Key::Backspace => {
                    self.input.delete(Move::Backward);
                    self.refresh_matches(true);
                }
                Key::Delete => {
                    self.input.delete(Move::Forward);
                    self.refresh_matches(true);
                }
                Key::Ctrl('w') => {
                    self.input.delete(Move::BackwardWord);
                    self.refresh_matches(true);
                }
                Key::Home => self.input.move_cursor(Move::BOL),
                Key::End => self.input.move_cursor(Move::EOL),
                Key::Char(c) => {
                    self.input.insert(c);
                    self.refresh_matches(true);
                }
                Key::F(1) => {
                    self.switch_result_sort();
                    self.refresh_matches(true);
                }
                Key::F(2) => {
                    if !self.matches.is_empty() {
                        if self.settings.delete_without_confirm {
                            self.delete_selection();
                        } else {
                            self.menu_mode = MenuMode::ConfirmDelete;
                        }
                    }
                }
                _ => {}
            }
        } else {
            match k {
                Key::Char('\n') | Key::Char('\r') | Key::Ctrl('j') => {
                    self.run = true;
                    self.accept_selection();
                    return true;
                }
                Key::Char('\t') => {
                    self.run = false;
                    self.accept_selection();
                    return true;
                }
                Key::Ctrl('c')
                | Key::Ctrl('g')
                | Key::Ctrl('z')
                | Key::Esc
                | Key::Char('q')
                // TODO add ZZ as shortcut
                | Key::Ctrl('r') => {
                    self.run = false;
                    self.input.clear();
                    return true;
                }
                Key::Left | Key::Char('h') => self.input.move_cursor(Move::Backward),
                Key::Right | Key::Char('l') => self.input.move_cursor(Move::Forward),
                Key::Up | Key::PageUp | Key::Char('k') | Key::Ctrl('u') => self.move_selection(MoveSelection::Up),
                Key::Down | Key::PageDown | Key::Char('j') | Key::Ctrl('d') => self.move_selection(MoveSelection::Down),
                Key::Char('b') | Key::Char('e') => self.input.move_cursor(Move::BackwardWord),
                Key::Char('w') => self.input.move_cursor(Move::ForwardWord),
                Key::Char('0') | Key::Char('^') => self.input.move_cursor(Move::BOL),
                Key::Char('$') => self.input.move_cursor(Move::EOL),
                Key::Char('i') => self.in_vim_insert_mode = true,
                Key::Char('I') => {
                    self.input.move_cursor(Move::BOL);
                    self.in_vim_insert_mode = true;
                }
                Key::Char('a') => {
                    self.input.move_cursor(Move::Forward);
                    self.in_vim_insert_mode = true;
                }
                Key::Char('A') => {
                    self.input.move_cursor(Move::EOL);
                    self.in_vim_insert_mode = true;
                }
                Key::Backspace => {
                    self.input.delete(Move::Backward);
                    self.refresh_matches(true);
                }
                Key::Delete | Key::Char('x') => {
                    self.input.delete(Move::Forward);
                    self.refresh_matches(true);
                }
                Key::Home => self.input.move_cursor(Move::BOL),
                Key::End => self.input.move_cursor(Move::EOL),
                Key::Char(_c) => {

                }
                Key::F(1) => {
                    self.switch_result_sort();
                    self.refresh_matches(true);
                },
                Key::F(2) => {
                    if !self.matches.is_empty() {
                        if self.settings.delete_without_confirm {
                            self.delete_selection();
                        }else{
                            self.menu_mode = MenuMode::ConfirmDelete;
                        }
                    }
                }
                _ => {}
            }
        }

        false
    }

    fn truncate_for_display(
        command: &Command,
        search: &str,
        width: u16,
        highlight_color: String,
        base_color: String,
        debug: bool,
    ) -> String {
        let mut prev: usize = 0;
        let debug_space = if debug { 90 } else { 0 };
        let max_grapheme_length = if width > debug_space {
            width - debug_space - 9
        } else {
            11
        };
        let mut out = FixedLengthGraphemeString::empty(max_grapheme_length);

        if !search.is_empty() {
            for (start, end) in &command.match_bounds {
                if prev != *start {
                    out.push_grapheme_str(&command.cmd[prev..*start]);
                }

                out.push_str(&highlight_color);
                out.push_grapheme_str(&command.cmd[*start..*end]);
                out.push_str(&base_color);
                prev = *end;
            }
        }

        if prev != command.cmd.len() {
            out.push_grapheme_str(&command.cmd[prev..]);
        }

        if debug {
            out.max_grapheme_length += debug_space;
            out.push_grapheme_str("  ");
            out.push_str(&format!("{}", color::Fg(color::LightBlue)));
            out.push_grapheme_str(format!("rnk: {:.*} ", 2, command.rank));
            out.push_grapheme_str(format!("age: {:.*} ", 2, command.features.age_factor));
            out.push_grapheme_str(format!("lng: {:.*} ", 2, command.features.length_factor));
            out.push_grapheme_str(format!("ext: {:.*} ", 0, command.features.exit_factor));
            out.push_grapheme_str(format!(
                "r_ext: {:.*} ",
                0, command.features.recent_failure_factor
            ));
            out.push_grapheme_str(format!("dir: {:.*} ", 3, command.features.dir_factor));
            out.push_grapheme_str(format!(
                "s_dir: {:.*} ",
                3, command.features.selected_dir_factor
            ));
            out.push_grapheme_str(format!("ovlp: {:.*} ", 3, command.features.overlap_factor));
            out.push_grapheme_str(format!(
                "i_ovlp: {:.*} ",
                3, command.features.immediate_overlap_factor
            ));
            out.push_grapheme_str(format!(
                "occ: {:.*}",
                2, command.features.occurrences_factor
            ));
            out.push_grapheme_str(format!(
                "s_occ: {:.*} ",
                2, command.features.selected_occurrences_factor
            ));
            out.push_str(&base_color);
        }

        out.string
    }

    fn result_top_index(&self) -> u16 {
        let (_width, height): (u16, u16) = terminal_size().unwrap();

        if self.is_screen_view_bottom() {
            return height - RESULTS_TOP_INDEX;
        }
        RESULTS_TOP_INDEX
    }

    fn prompt_line_index(&self) -> u16 {
        let (_width, height): (u16, u16) = terminal_size().unwrap();
        if self.is_screen_view_bottom() {
            return height - PROMPT_LINE_INDEX;
        }
        PROMPT_LINE_INDEX
    }

    fn info_line_index(&self) -> u16 {
        let (_width, height): (u16, u16) = terminal_size().unwrap();
        if self.is_screen_view_bottom() {
            return height;
        }
        INFO_LINE_INDEX
    }

    fn command_line_index(&self, index: i16) -> i16 {
        if self.is_screen_view_bottom() {
            return -index;
        }
        index
    }

    fn is_screen_view_bottom(&self) -> bool {
        self.settings.interface_view == InterfaceView::Bottom
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
