use crate::command_input::{CommandInput, Move};
use crate::history::History;

use crate::fixed_length_grapheme_string::FixedLengthGraphemeString;
use crate::history::Command;
use crate::history_cleaner;
use crate::settings::{InterfaceView, KeyScheme, ResultFilter};
use crate::settings::{ResultSort, Settings};
use chrono::{Duration, TimeZone, Utc};
use crossterm::event::KeyCode::Char;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::style::{Color, Print, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{self, LeaveAlternateScreen};
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen};
use crossterm::{cursor, execute, queue};
use humantime::format_duration;
use std::io::{stdout, Write};
use std::string::String;

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
    result_filter: ResultFilter,
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

        if interface.settings.disable_run_command {
            menu_text.push_str("⏎, TAB - Edit | ");
        } else {
            menu_text.push_str("⏎ - Run | TAB - Edit | ");
        }

        match interface.result_sort {
            ResultSort::Rank => menu_text.push_str("F1 - Rank Sort | "),
            ResultSort::LastRun => menu_text.push_str("F1 - Time Sort | "),
        }

        menu_text.push_str("F2 - Delete | ");

        match interface.result_filter {
            ResultFilter::Global => menu_text.push_str("F3 - All Directories"),
            ResultFilter::CurrentDirectory => menu_text.push_str("F3 - This Directory"),
        }

        menu_text
    }

    fn bg(&self, normal: Color) -> Color {
        match *self {
            MenuMode::Normal => normal,
            MenuMode::ConfirmDelete => Color::Red,
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
            input: CommandInput::from(settings.command.clone()),
            selection: 0,
            matches: Vec::new(),
            debug: settings.debug,
            run: false,
            delete_requests: Vec::new(),
            menu_mode: MenuMode::Normal,
            in_vim_insert_mode: true,
            result_sort: settings.result_sort.clone(),
            result_filter: settings.result_filter.clone(),
        }
    }

    pub fn display(&mut self) -> SelectionResult {
        self.build_cache_table();
        self.select();

        let command = self.input.command.clone();

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
            &self.settings.dir.clone(),
            &self.result_filter,
            &Some(self.settings.session_id.clone()),
            None,
            None,
            None,
            self.settings.limit,
        );
    }

    fn menubar<W: Write>(&self, screen: &mut W) {
        if !self.settings.disable_menu {
            let (width, _height): (u16, u16) = terminal::size().unwrap();

            queue!(
                screen,
                cursor::Hide,
                cursor::MoveTo(0, self.info_line_index()),
                Clear(ClearType::CurrentLine),
                SetBackgroundColor(self.menu_mode.bg(self.settings.colors.menubar_bg)),
                SetForegroundColor(self.settings.colors.menubar_fg),
                cursor::MoveTo(1, self.info_line_index()),
                Print(format!(
                    "{text:width$}",
                    text = self.menu_mode.text(self),
                    width = width as usize - 1
                )),
                SetBackgroundColor(Color::Reset)
            )
            .unwrap();
        }
    }

    fn prompt<W: Write>(&self, screen: &mut W) {
        let prompt_line_index = self.prompt_line_index();
        let fg = if self.settings.lightmode {
            self.settings.colors.lightmode_colors.prompt
        } else {
            self.settings.colors.darkmode_colors.prompt
        };
        queue!(
            screen,
            cursor::MoveTo(1, prompt_line_index),
            SetForegroundColor(fg),
            Clear(ClearType::CurrentLine),
            Print(format!("{} {}", self.settings.prompt, self.input)),
            cursor::MoveTo(self.input.cursor as u16 + 3, prompt_line_index),
            cursor::Show
        )
        .unwrap();
    }

    fn debug_cursor<W: Write>(&self, screen: &mut W) {
        let result_top_index = self.result_top_index();
        queue!(
            screen,
            cursor::Hide,
            cursor::MoveTo(0, result_top_index + self.settings.results + 1)
        )
        .unwrap();
    }

    fn results<W: Write>(&mut self, screen: &mut W) {
        let result_top_index = self.result_top_index();
        queue!(screen, cursor::Hide, cursor::MoveTo(1, result_top_index)).unwrap();

        let (width, _height): (u16, u16) = terminal::size().unwrap();

        if !self.matches.is_empty() && self.selection > self.matches.len() - 1 {
            self.selection = self.matches.len() - 1;
        }

        let mut index: usize = 0;
        for command in &self.matches {
            let mut fg = if self.settings.lightmode {
                self.settings.colors.lightmode_colors.results_fg
            } else {
                self.settings.colors.darkmode_colors.results_fg
            };

            let mut highlight = if self.settings.lightmode {
                self.settings.colors.lightmode_colors.results_hl
            } else {
                self.settings.colors.darkmode_colors.results_hl
            };

            let mut bg = Color::Reset;

            if index == self.selection {
                if self.settings.lightmode {
                    fg = self.settings.colors.lightmode_colors.results_selection_fg;
                    bg = self.settings.colors.lightmode_colors.results_selection_bg;
                    highlight = self.settings.colors.lightmode_colors.results_selection_hl;
                } else {
                    fg = self.settings.colors.darkmode_colors.results_selection_fg;
                    bg = self.settings.colors.darkmode_colors.results_selection_bg;
                    highlight = self.settings.colors.darkmode_colors.results_selection_hl;
                }
            }

            let command_line_index = self.command_line_index(index as i16);
            queue!(
                screen,
                cursor::MoveTo(1, (command_line_index + result_top_index as i16) as u16),
                Clear(ClearType::CurrentLine),
                SetBackgroundColor(bg),
                SetForegroundColor(fg),
                Print(Interface::truncate_for_display(
                    command,
                    &self.input.command,
                    width,
                    highlight,
                    fg,
                    self.debug
                ))
            )
            .unwrap();

            if command.last_run.is_some() {
                queue!(
                    screen,
                    cursor::MoveTo(
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

                let timing_color = if self.settings.lightmode {
                    self.settings.colors.lightmode_colors.timing
                } else {
                    self.settings.colors.darkmode_colors.timing
                };
                queue!(
                    screen,
                    cursor::MoveTo(
                        width - 9,
                        (command_line_index + self.result_top_index() as i16) as u16
                    ),
                    SetForegroundColor(timing_color),
                    Print(format!("{duration:>9}")),
                    SetForegroundColor(Color::Reset),
                    SetBackgroundColor(Color::Reset)
                )
                .unwrap();
            }
            index += 1;
        }
        // Since we only clear by line instead of clearing the screen each update,
        //  we need to clear all the lines that may have previously had a command
        for i in index..(self.settings.results as usize) {
            let command_line_index = self.command_line_index(i as i16);
            queue!(
                screen,
                cursor::MoveTo(1, (command_line_index + result_top_index as i16) as u16),
                Clear(ClearType::CurrentLine)
            )
            .unwrap();
        }
    }

    #[allow(unused)]
    fn debug<W: Write, S: Into<String>>(&self, screen: &mut W, s: S) {
        queue!(
            screen,
            cursor::MoveTo(0, 0),
            Clear(ClearType::CurrentLine),
            Print(s.into())
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
                self.delete_selection();
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

    fn switch_result_filter(&mut self) {
        self.result_filter = match self.result_filter {
            ResultFilter::Global => ResultFilter::CurrentDirectory,
            ResultFilter::CurrentDirectory => ResultFilter::Global,
        };
        self.build_cache_table();
    }

    fn select(&mut self) {
        let mut screen = stdout();
        terminal::enable_raw_mode().unwrap();
        queue!(screen, EnterAlternateScreen, Clear(ClearType::All)).unwrap();

        self.refresh_matches(true);
        self.results(&mut screen);
        self.menubar(&mut screen);
        self.prompt(&mut screen);

        screen.flush().unwrap();

        loop {
            let event =
                read().unwrap_or_else(|e| panic!("McFly error: failed to read input {:?}", &e));
            self.debug_cursor(&mut screen);

            match self.menu_mode {
                MenuMode::Normal => {
                    let early_out = match self.settings.key_scheme {
                        KeyScheme::Emacs => self.select_with_emacs_key_scheme(event),
                        KeyScheme::Vim => {
                            if let Event::Key(key_event) = event {
                                self.select_with_vim_key_scheme(key_event)
                            } else {
                                false
                            }
                        }
                    };

                    if early_out {
                        break;
                    }
                }
                MenuMode::ConfirmDelete => {
                    if let Event::Key(key_event) = event {
                        match key_event {
                            KeyEvent {
                                modifiers: KeyModifiers::CONTROL,
                                code: Char('c' | 'd' | 'g' | 'z' | 'r'),
                                ..
                            } => {
                                self.run = false;
                                self.input.clear();
                                break;
                            }
                            KeyEvent {
                                code: Char('y' | 'Y'),
                                ..
                            } => {
                                self.confirm(true);
                            }
                            KeyEvent {
                                code: Char('n' | 'N') | KeyCode::Esc,
                                ..
                            } => {
                                self.confirm(false);
                            }
                            _ => {}
                        }
                    }
                }
            }

            self.results(&mut screen);
            self.menubar(&mut screen);
            self.prompt(&mut screen);
            screen.flush().unwrap();
        }

        queue!(
            screen,
            Clear(ClearType::All),
            cursor::Show,
            LeaveAlternateScreen
        )
        .unwrap();
        terminal::disable_raw_mode().unwrap();
    }

    fn select_with_emacs_key_scheme(&mut self, event: Event) -> bool {
        match event {
            Event::Key(event) => self.handle_emacs_keyevent(event),
            Event::Paste(s) => {
                for i in s.chars() {
                    self.input.insert(i);
                }
                self.refresh_matches(true);
                false
            }
            _ => false,
        }
    }

    fn handle_emacs_keyevent(&mut self, event: KeyEvent) -> bool {
        if event.kind != KeyEventKind::Press {
            return false;
        }
        match event {
            KeyEvent {
                code: KeyCode::Enter | Char('\r' | '\n'),
                ..
            }
            | KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: Char('j'),
                ..
            } => {
                self.run = !self.settings.disable_run_command;
                self.accept_selection();
                return true;
            }

            KeyEvent {
                code: KeyCode::Tab, ..
            } => {
                self.run = false;
                self.accept_selection();
                return true;
            }

            KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: Char('c' | 'g' | 'z' | 'r'),
                ..
            }
            | KeyEvent {
                code: KeyCode::Esc, ..
            } => {
                self.run = false;
                self.input.clear();
                return true;
            }

            KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code,
                ..
            } => match code {
                Char('b') => self.input.move_cursor(Move::Backward),
                Char('f') => self.input.move_cursor(Move::Forward),
                Char('a') => self.input.move_cursor(Move::BOL),
                Char('e') => self.input.move_cursor(Move::EOL),
                Char('v') => self.debug = !self.debug,
                Char('k') => {
                    self.input.delete(Move::EOL);
                    self.refresh_matches(true);
                }
                Char('u') => {
                    self.input.delete(Move::BOL);
                    self.refresh_matches(true);
                }
                Char('w') => {
                    self.input.delete(Move::BackwardWord);
                    self.refresh_matches(true);
                }
                Char('p') => self.move_selection(MoveSelection::Up),
                Char('n') => self.move_selection(MoveSelection::Down),
                Char('h') => {
                    self.input.delete(Move::Backward);
                    self.refresh_matches(true);
                }
                Char('d') => {
                    self.input.delete(Move::Forward);
                    self.refresh_matches(true);
                }
                _ => {}
            },

            KeyEvent {
                modifiers: KeyModifiers::ALT,
                code: Char('\x08' | '\x7f'),
                ..
            } => {
                self.input.delete(Move::BackwardWord);
                self.refresh_matches(true);
            }

            KeyEvent {
                modifiers: KeyModifiers::ALT,
                code,
                ..
            } => match code {
                Char('b') => self.input.move_cursor(Move::BackwardWord),
                Char('f') => self.input.move_cursor(Move::ForwardWord),
                Char('d') => {
                    self.input.delete(Move::ForwardWord);
                    self.refresh_matches(true);
                }
                _ => {}
            },

            KeyEvent {
                code: KeyCode::Left,
                ..
            } => self.input.move_cursor(Move::Backward),

            KeyEvent {
                code: KeyCode::Right,
                ..
            } => self.input.move_cursor(Move::Forward),

            KeyEvent {
                code: KeyCode::Up | KeyCode::PageUp,
                ..
            } => self.move_selection(MoveSelection::Up),

            KeyEvent {
                code: KeyCode::Down | KeyCode::PageDown,
                ..
            } => self.move_selection(MoveSelection::Down),

            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => {
                self.input.delete(Move::Backward);
                self.refresh_matches(true);
            }

            KeyEvent {
                code: KeyCode::Delete,
                ..
            } => {
                self.input.delete(Move::Forward);
                self.refresh_matches(true);
            }

            KeyEvent {
                code: KeyCode::Home,
                ..
            } => self.input.move_cursor(Move::BOL),

            KeyEvent {
                code: KeyCode::End, ..
            } => self.input.move_cursor(Move::EOL),

            KeyEvent { code: Char(c), .. } => {
                self.input.insert(c);
                self.refresh_matches(true);
            }

            KeyEvent {
                code: KeyCode::F(1),
                ..
            } => {
                self.switch_result_sort();
                self.refresh_matches(true);
            }

            KeyEvent {
                code: KeyCode::F(2),
                ..
            } => {
                if !self.matches.is_empty() {
                    if self.settings.delete_without_confirm {
                        self.delete_selection();
                    } else {
                        self.menu_mode = MenuMode::ConfirmDelete;
                    }
                }
            }

            KeyEvent {
                code: KeyCode::F(3),
                ..
            } => {
                self.switch_result_filter();
                self.refresh_matches(true);
            }
            _ => {}
        }

        false
    }

    fn select_with_vim_key_scheme(&mut self, event: KeyEvent) -> bool {
        if event.kind != KeyEventKind::Press {
            return false;
        }
        if self.in_vim_insert_mode {
            match event {
                KeyEvent {
                    code: KeyCode::Tab, ..
                } => {
                    self.run = false;
                    self.accept_selection();
                    return true;
                }

                KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }
                | KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: Char('j'),
                    ..
                } => {
                    self.run = !self.settings.disable_run_command;
                    self.accept_selection();
                    return true;
                }

                KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: Char('c' | 'g' | 'z' | 'r'),
                    ..
                } => {
                    self.run = false;
                    self.input.clear();
                    return true;
                }

                KeyEvent {
                    code: KeyCode::Left,
                    ..
                } => self.input.move_cursor(Move::Backward),
                KeyEvent {
                    code: KeyCode::Right,
                    ..
                } => self.input.move_cursor(Move::Forward),

                KeyEvent {
                    code: KeyCode::Up | KeyCode::PageUp,
                    ..
                }
                | KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: Char('u' | 'p'),
                    ..
                } => self.move_selection(MoveSelection::Up),

                KeyEvent {
                    code: KeyCode::Down | KeyCode::PageDown,
                    ..
                }
                | KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: Char('d' | 'n'),
                    ..
                } => self.move_selection(MoveSelection::Down),

                KeyEvent {
                    code: KeyCode::Esc, ..
                } => self.in_vim_insert_mode = false,
                KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                } => {
                    self.input.delete(Move::Backward);
                    self.refresh_matches(true);
                }
                KeyEvent {
                    code: KeyCode::Delete,
                    ..
                } => {
                    self.input.delete(Move::Forward);
                    self.refresh_matches(true);
                }
                KeyEvent {
                    code: KeyCode::Home,
                    ..
                } => self.input.move_cursor(Move::BOL),
                KeyEvent {
                    code: KeyCode::End, ..
                } => self.input.move_cursor(Move::EOL),
                KeyEvent { code: Char(c), .. } => {
                    self.input.insert(c);
                    self.refresh_matches(true);
                }
                KeyEvent {
                    code: KeyCode::F(1),
                    ..
                } => {
                    self.switch_result_sort();
                    self.refresh_matches(true);
                }
                KeyEvent {
                    code: KeyCode::F(2),
                    ..
                } => {
                    if !self.matches.is_empty() {
                        if self.settings.delete_without_confirm {
                            self.delete_selection();
                        } else {
                            self.menu_mode = MenuMode::ConfirmDelete;
                        }
                    }
                }
                KeyEvent {
                    code: KeyCode::F(3),
                    ..
                } => {
                    self.switch_result_filter();
                    self.refresh_matches(true);
                }
                _ => {}
            }
        } else {
            match event {
                KeyEvent {
                    code: KeyCode::Tab, ..
                } => {
                    self.run = false;
                    self.accept_selection();
                    return true;
                }

                KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }
                | KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: Char('j'),
                    ..
                } => {
                    self.run = !self.settings.disable_run_command;
                    self.accept_selection();
                    return true;
                }

                KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: Char('c' | 'g' | 'z' | 'r'),
                    ..
                }
                | KeyEvent {
                    code: KeyCode::Esc, ..
                } => {
                    self.run = false;
                    self.input.clear();
                    return true;
                }

                KeyEvent {
                    code: KeyCode::Left | Char('h'),
                    ..
                } => self.input.move_cursor(Move::Backward),
                KeyEvent {
                    code: KeyCode::Right | Char('l'),
                    ..
                } => self.input.move_cursor(Move::Forward),

                KeyEvent {
                    code: KeyCode::Up | KeyCode::PageUp | Char('k'),
                    ..
                }
                | KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: Char('u'),
                    ..
                } => self.move_selection(MoveSelection::Up),

                KeyEvent {
                    code: KeyCode::Down | KeyCode::PageDown | Char('j'),
                    ..
                }
                | KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: Char('d'),
                    ..
                } => self.move_selection(MoveSelection::Down),

                KeyEvent {
                    code: Char('b' | 'e'),
                    ..
                } => self.input.move_cursor(Move::BackwardWord),
                KeyEvent {
                    code: Char('w'), ..
                } => self.input.move_cursor(Move::ForwardWord),
                KeyEvent {
                    code: Char('0' | '^'),
                    ..
                } => self.input.move_cursor(Move::BOL),
                KeyEvent {
                    code: Char('$'), ..
                } => self.input.move_cursor(Move::EOL),

                KeyEvent {
                    code: Char('i' | 'a'),
                    ..
                } => self.in_vim_insert_mode = true,

                KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                } => {
                    self.input.delete(Move::Backward);
                    self.refresh_matches(true);
                }
                KeyEvent {
                    code: KeyCode::Delete | Char('x'),
                    ..
                } => {
                    self.input.delete(Move::Forward);
                    self.refresh_matches(true);
                }
                KeyEvent {
                    code: KeyCode::Home,
                    ..
                } => self.input.move_cursor(Move::BOL),
                KeyEvent {
                    code: KeyCode::End, ..
                } => self.input.move_cursor(Move::EOL),

                KeyEvent {
                    code: KeyCode::F(1),
                    ..
                } => {
                    self.switch_result_sort();
                    self.refresh_matches(true);
                }
                KeyEvent {
                    code: KeyCode::F(2),
                    ..
                } => {
                    if !self.matches.is_empty() {
                        if self.settings.delete_without_confirm {
                            self.delete_selection();
                        } else {
                            self.menu_mode = MenuMode::ConfirmDelete;
                        }
                    }
                }
                KeyEvent {
                    code: KeyCode::F(3),
                    ..
                } => {
                    self.switch_result_filter();
                    self.refresh_matches(true);
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
        highlight_color: Color,
        base_color: Color,
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

                execute!(out, SetForegroundColor(highlight_color)).unwrap();
                out.push_grapheme_str(&command.cmd[*start..*end]);
                execute!(out, SetForegroundColor(base_color)).unwrap();
                prev = *end;
            }
        }

        if prev != command.cmd.len() {
            out.push_grapheme_str(&command.cmd[prev..]);
        }

        if debug {
            out.max_grapheme_length += debug_space;
            out.push_grapheme_str("  ");
            execute!(out, SetForegroundColor(Color::Blue)).unwrap();
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
            execute!(out, SetForegroundColor(base_color)).unwrap();
        }

        out.string
    }

    fn result_top_index(&self) -> u16 {
        let (_width, height): (u16, u16) = terminal::size().unwrap();

        if self.is_screen_view_bottom() {
            return height - RESULTS_TOP_INDEX;
        }
        RESULTS_TOP_INDEX
    }

    fn prompt_line_index(&self) -> u16 {
        let (_width, height): (u16, u16) = terminal::size().unwrap();
        if self.is_screen_view_bottom() {
            return height - PROMPT_LINE_INDEX;
        }
        PROMPT_LINE_INDEX
    }

    fn info_line_index(&self) -> u16 {
        let (_width, height): (u16, u16) = terminal::size().unwrap();
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
