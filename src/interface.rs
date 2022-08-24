
use crate::command_input::{CommandInput, Move};
use crate::history::History;

use crate::fixed_length_grapheme_string::FixedLengthGraphemeString;
use crate::history::Command;
use crate::history_cleaner;
use crate::settings::Settings;
use crate::settings::{InterfaceView, KeyScheme};
use crossterm::cursor;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::queue;
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal;
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use std::io::{stdout, Write};
use std::str::FromStr;
use chrono::{Duration, TimeZone, Utc};
use humantime::format_duration;




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

#[derive(PartialEq)]
pub enum MenuMode {
    Normal,
    ConfirmDelete,
}

impl MenuMode {
    fn text(&self, interface: &Interface) -> &str {
        match *self {
            MenuMode::Normal => match interface.settings.key_scheme {
                KeyScheme::Emacs => "McFly | ESC - Exit | ⏎ - Run | TAB - Edit | F2 - Delete",
                KeyScheme::Vim => {
                    if interface.in_vim_insert_mode {
                        "McFly (Ins) | ESC - Cmd  | ⏎ - Run | TAB - Edit | F2 - Delete"
                    } else {
                        "McFly (Cmd) | ESC - Exit | ⏎ - Run | TAB - Edit | F2 - Delete"
                    }
                }
            },
            MenuMode::ConfirmDelete => "Delete selected command from the history? (Y/N)",
        }
    }

    fn bg(&self, interface: &Interface) -> Color {
        match *self {
            MenuMode::Normal => Color::from_str(&interface.settings.colors.menu_bg).unwrap(),
            MenuMode::ConfirmDelete => {
                Color::from_str(&interface.settings.colors.menu_deleting_bg).unwrap()
            }
        }
    }

    fn fg(&self, interface: &Interface) -> Color {
        match *self {
            MenuMode::Normal => Color::from_str(&interface.settings.colors.menu_fg).unwrap(),
            MenuMode::ConfirmDelete => {
                Color::from_str(&interface.settings.colors.menu_deleting_fg).unwrap()
            }
        }
    }
}

const PROMPT_LINE_INDEX: u16 = 2;
const INFO_LINE_INDEX: u16 = 0;
const RESULTS_TOP_INDEX: u16 = 4;

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
        let (width, _height): (u16, u16) = terminal::size().unwrap();

        let _ = queue!(screen, cursor::MoveTo(0, self.info_line_index()));
        let _ = queue!(screen, SetBackgroundColor(self.menu_mode.bg(self)));
        let _ = queue!(screen, SetForegroundColor(self.menu_mode.fg(self)));
        let _ = queue!(
            screen,
            Print(format!(
                "{text:width$}",
                text = self.menu_mode.text(self),
                width = width as usize
            ))
        );
        let _ = queue!(screen, ResetColor);

        screen.flush().unwrap();
    }

    fn prompt<W: Write>(&self, screen: &mut W) {
        let _ = queue!(
            screen,
            SetForegroundColor(Color::from_str(&self.settings.colors.prompt_fg).unwrap())
        );
        let _ = queue!(screen, cursor::MoveTo(0, self.prompt_line_index()));
        let _ = queue!(screen, Clear(ClearType::CurrentLine));
        let _ = queue!(screen, Print(format!("$ {}", self.input)));
        let _ = queue!(
            screen,
            cursor::MoveTo(self.input.cursor as u16 + 2, self.prompt_line_index())
        );

        if self.in_vim_insert_mode {
            let _ = queue!(screen, cursor::EnableBlinking);
        } else {
            let _ = queue!(screen, cursor::DisableBlinking);
        }

        let _ = queue!(screen, cursor::Show);

        screen.flush().unwrap();
    }

    fn results<W: Write>(&mut self, screen: &mut W) {
        let _ = queue!(screen, cursor::Hide);
        let _ = queue!(screen, cursor::MoveTo(0, self.result_top_index()));
        let _ = queue!(screen, Clear(ClearType::All));

        let (width, _height): (u16, u16) = terminal::size().unwrap();

        if !self.matches.is_empty() && self.selection > self.matches.len() - 1 {
            self.selection = self.matches.len() - 1;
        }

        for (index, command) in self.matches.iter().enumerate() {
            let mut fg = Color::from_str(&self.settings.colors.fg).unwrap();
            let mut bg = Color::Reset;
            let mut highlight = Color::from_str(&self.settings.colors.highlight).unwrap();
            let mut timing_color = Color::from_str(&self.settings.colors.timing).unwrap();

            if index == self.selection {
                fg = Color::from_str(&self.settings.colors.cursor_fg).unwrap();
                bg = Color::from_str(&self.settings.colors.cursor_bg).unwrap();
                highlight = Color::from_str(&self.settings.colors.cursor_highlight).unwrap();
                timing_color = Color::from_str(&self.settings.colors.timing).unwrap();
            }

            let command_line_index = self.command_line_index(index as i16);
            let _ = queue!(screen, cursor::MoveTo(0, (command_line_index as i16 + self.result_top_index() as i16) as u16));
            let _ = queue!(screen, SetBackgroundColor(bg));
            let _ = queue!(screen, SetForegroundColor(fg));
            Interface::truncate_for_display(screen, command, &self.input.command, width, highlight, fg, self.debug);
            // let _ = queue!(screen, Print(command));
            // let tmp_str = format!("{:width$}", Interface::truncate_for_display(
            //         command,
            //         &self.input.command,
            //         width,
            //         highlight,
            //         fg,
            //         self.debug
            //     ), width=(width + 10) as usize);
            // let _ = queue!(
            //     screen,
            //     Print(&tmp_str),
            // );

            if command.last_run.is_some() {
                let duration = &format_duration(
                    Duration::minutes(
                        Utc::now()
                            .signed_duration_since(Utc.timestamp(command.last_run.unwrap(), 0))
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

                let _ = queue!(screen, cursor::MoveTo(width - 9, (command_line_index as i16 + self.result_top_index() as i16) as u16));
                let _ = queue!(screen, SetForegroundColor(timing_color));
                let tmp_str = format!("{:>9}", duration);
                let _ = queue!(screen, Print(&tmp_str));
                let _ = queue!(screen, SetForegroundColor(fg));
            }
        }
        screen.flush().unwrap();
    }

    #[allow(unused)]
    fn debug<W: Write, S: Into<String>>(&self, screen: &mut W, s: S) {
        let _ = queue!(screen, cursor::MoveTo(0, 0));
        let _ = queue!(screen, Clear(ClearType::CurrentLine));
        let _ = queue!(screen, Print(s.into()));

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
            self.refresh_matches();
        }
    }

    fn refresh_matches(&mut self) {
        self.selection = 0;
        self.matches = self.history.find_matches(
            &self.input.command,
            self.settings.results as i16,
            self.settings.fuzzy,
            &self.settings.result_sort,
        );
    }

    fn select(&mut self) {
        let _ = terminal::enable_raw_mode();

        let mut screen = stdout();

        let _ = queue!(screen, EnterAlternateScreen);
        let _ = queue!(screen, Clear(ClearType::All));

        self.refresh_matches();
        self.results(&mut screen);
        self.menubar(&mut screen);
        self.prompt(&mut screen);

        loop {
            let event =
                read().unwrap_or_else(|e| panic!("McFly error: failed to read input {:?}", &e));

            if self.menu_mode != MenuMode::Normal {
                match event {
                    Event::Key(KeyEvent {
                        modifiers: KeyModifiers::CONTROL,
                        code:
                            KeyCode::Char('c')
                            | KeyCode::Char('d')
                            | KeyCode::Char('g')
                            | KeyCode::Char('z')
                            | KeyCode::Char('r'),
                    }) => {
                        self.run = false;
                        self.input.clear();
                        break;
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('y'),
                        ..
                    }) => {
                        self.confirm(true);
                    }
                    Event::Key(
                        KeyEvent {
                            code: KeyCode::Char('n'),
                            ..
                        }
                        | KeyEvent {
                            code: KeyCode::Esc, ..
                        },
                    ) => {
                        self.confirm(false);
                    }
                    _ => {}
                };
            } else {
                let early_out = match self.settings.key_scheme {
                    KeyScheme::Emacs => self.select_with_emacs_key_scheme(event),
                    KeyScheme::Vim => self.select_with_vim_key_scheme(event),
                };

                if early_out {
                    break;
                }
            }

            self.results(&mut screen);
            self.menubar(&mut screen);
            self.prompt(&mut screen);
        }

        let _ = queue!(screen, Clear(ClearType::All));
        let _ = queue!(screen, cursor::Show);
        let _ = queue!(screen, LeaveAlternateScreen);

        let _ = terminal::disable_raw_mode();
    }

    fn select_with_emacs_key_scheme(&mut self, event: Event) -> bool {
        match event {
            Event::Key(
                KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code:
                        KeyCode::Char('c')
                        | KeyCode::Char('g')
                        | KeyCode::Char('z')
                        | KeyCode::Char('r'),
                }
                | KeyEvent {
                    code: KeyCode::Esc, ..
                },
            ) => {
                self.run = false;
                self.input.clear();
                return true;
            }

            Event::Key(KeyEvent {
                code: KeyCode::Tab, ..
            }) => {
                self.run = false;
                self.accept_selection();
                return true;
            }

            Event::Key(
                KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }
                | KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: KeyCode::Char('j'),
                },
            ) => {
                self.run = true;
                self.accept_selection();
                return true;
            }

            Event::Key(
                KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: KeyCode::Char('w'),
                }
                | KeyEvent {
                    modifiers: KeyModifiers::ALT,
                    code: KeyCode::Char('\x08') | KeyCode::Char('\x7f'),
                },
            ) => {
                self.input.delete(Move::BackwardWord);
                self.refresh_matches();
            }

            Event::Key(KeyEvent {
                code: KeyCode::Left,
                ..
            }) => self.input.move_cursor(Move::Backward),
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                ..
            }) => self.input.move_cursor(Move::Forward),

            Event::Key(
                KeyEvent {
                    code: KeyCode::Up | KeyCode::PageUp,
                    ..
                }
                | KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: KeyCode::Char('p'),
                },
            ) => self.move_selection(MoveSelection::Up),

            Event::Key(
                KeyEvent {
                    code: KeyCode::Down | KeyCode::PageDown,
                    ..
                }
                | KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: KeyCode::Char('n'),
                },
            ) => self.move_selection(MoveSelection::Down),

            Event::Key(
                KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                }
                | KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: KeyCode::Char('h'),
                },
            ) => {
                self.input.delete(Move::Backward);
                self.refresh_matches();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Delete,
                ..
            }
            | KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('d')
            }) => {
                self.input.delete(Move::Forward);
                self.refresh_matches();
            }

            Event::Key(KeyEvent {
                code: KeyCode::Home,
                ..
            }) => self.input.move_cursor(Move::BOL),
            Event::Key(KeyEvent {
                code: KeyCode::End, ..
            }) => self.input.move_cursor(Move::EOL),
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            }) => {
                self.input.insert(c);
                self.refresh_matches();
            }
            Event::Key(KeyEvent {
                code: KeyCode::F(2),
                ..
            }) => {
                if !self.matches.is_empty() {
                    if self.settings.delete_without_confirm {
                        self.delete_selection();
                    } else {
                        self.menu_mode = MenuMode::ConfirmDelete;
                    }
                }
            }

            Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code,
            }) => match code {
                KeyCode::Char('v') => self.debug = !self.debug,
                KeyCode::Char('b') => self.input.move_cursor(Move::Backward),
                KeyCode::Char('f') => self.input.move_cursor(Move::Forward),
                KeyCode::Char('a') => self.input.move_cursor(Move::BOL),
                KeyCode::Char('e') => self.input.move_cursor(Move::EOL),
                KeyCode::Char('k') => {
                    self.input.delete(Move::EOL);
                    self.refresh_matches();
                }
                KeyCode::Char('u') => {
                    self.input.delete(Move::BOL);
                    self.refresh_matches();
                }
                _ => {}
            },

            Event::Key(KeyEvent {
                modifiers: KeyModifiers::ALT,
                code,
            }) => match code {
                KeyCode::Char('b') => self.input.move_cursor(Move::BackwardWord),
                KeyCode::Char('f') => self.input.move_cursor(Move::ForwardWord),
                KeyCode::Char('d') => {
                    self.input.delete(Move::ForwardWord);
                    self.refresh_matches();
                }
                _ => {}
            },

            _ => {}
        }

        false
    }

    fn select_with_vim_key_scheme(&mut self, event: Event) -> bool {
        if self.in_vim_insert_mode {
            match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Tab, ..
                }) => {
                    self.run = false;
                    self.accept_selection();
                    return true;
                }

                Event::Key(
                    KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    }
                    | KeyEvent {
                        modifiers: KeyModifiers::CONTROL,
                        code: KeyCode::Char('j'),
                    },
                ) => {
                    self.run = true;
                    self.accept_selection();
                    return true;
                }

                Event::Key(KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code:
                        KeyCode::Char('c')
                        | KeyCode::Char('g')
                        | KeyCode::Char('z')
                        | KeyCode::Char('r'),
                }) => {
                    self.run = false;
                    self.input.clear();
                    return true;
                }

                Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    ..
                }) => self.input.move_cursor(Move::Backward),
                Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    ..
                }) => self.input.move_cursor(Move::Forward),

                Event::Key(
                    KeyEvent {
                        code: KeyCode::Up | KeyCode::PageUp,
                        ..
                    }
                    | KeyEvent {
                        modifiers: KeyModifiers::CONTROL,
                        code: KeyCode::Char('u') | KeyCode::Char('p'),
                    },
                ) => self.move_selection(MoveSelection::Up),

                Event::Key(
                    KeyEvent {
                        code: KeyCode::Down | KeyCode::PageDown,
                        ..
                    }
                    | KeyEvent {
                        modifiers: KeyModifiers::CONTROL,
                        code: KeyCode::Char('d') | KeyCode::Char('n'),
                    },
                ) => self.move_selection(MoveSelection::Down),

                Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                }) => self.in_vim_insert_mode = false,
                Event::Key(KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                }) => {
                    self.input.delete(Move::Backward);
                    self.refresh_matches();
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Delete,
                    ..
                }) => {
                    self.input.delete(Move::Forward);
                    self.refresh_matches();
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Home,
                    ..
                }) => self.input.move_cursor(Move::BOL),
                Event::Key(KeyEvent {
                    code: KeyCode::End, ..
                }) => self.input.move_cursor(Move::EOL),
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                }) => {
                    self.input.insert(c);
                    self.refresh_matches();
                }
                Event::Key(KeyEvent {
                    code: KeyCode::F(2),
                    ..
                }) => {
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
            match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Tab, ..
                }) => {
                    self.run = false;
                    self.accept_selection();
                    return true;
                }

                Event::Key(
                    KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    }
                    | KeyEvent {
                        modifiers: KeyModifiers::CONTROL,
                        code: KeyCode::Char('j'),
                    },
                ) => {
                    self.run = true;
                    self.accept_selection();
                    return true;
                }

                Event::Key(
                    KeyEvent {
                        modifiers: KeyModifiers::CONTROL,
                        code:
                            KeyCode::Char('c')
                            | KeyCode::Char('g')
                            | KeyCode::Char('z')
                            | KeyCode::Char('r'), // TODO add ZZ as shortcut
                    }
                    | KeyEvent {
                        code: KeyCode::Esc, ..
                    },
                ) => {
                    self.run = false;
                    self.input.clear();
                    return true;
                }

                Event::Key(KeyEvent {
                    code: KeyCode::Left | KeyCode::Char('h'),
                    ..
                }) => self.input.move_cursor(Move::Backward),
                Event::Key(KeyEvent {
                    code: KeyCode::Right | KeyCode::Char('l'),
                    ..
                }) => self.input.move_cursor(Move::Forward),

                Event::Key(
                    KeyEvent {
                        code: KeyCode::Up | KeyCode::PageUp | KeyCode::Char('k'),
                        ..
                    }
                    | KeyEvent {
                        modifiers: KeyModifiers::CONTROL,
                        code: KeyCode::Char('u'),
                    },
                ) => self.move_selection(MoveSelection::Up),

                Event::Key(
                    KeyEvent {
                        code: KeyCode::Down | KeyCode::PageDown | KeyCode::Char('j'),
                        ..
                    }
                    | KeyEvent {
                        modifiers: KeyModifiers::CONTROL,
                        code: KeyCode::Char('d'),
                    },
                ) => self.move_selection(MoveSelection::Down),

                Event::Key(KeyEvent {
                    code: KeyCode::Char('b') | KeyCode::Char('e'),
                    ..
                }) => self.input.move_cursor(Move::BackwardWord),
                Event::Key(KeyEvent {
                    code: KeyCode::Char('w'),
                    ..
                }) => self.input.move_cursor(Move::ForwardWord),
                Event::Key(KeyEvent {
                    code: KeyCode::Char('0') | KeyCode::Char('^'),
                    ..
                }) => self.input.move_cursor(Move::BOL),
                Event::Key(KeyEvent {
                    code: KeyCode::Char('$'),
                    ..
                }) => self.input.move_cursor(Move::EOL),

                Event::Key(KeyEvent {
                    code: KeyCode::Char('i') | KeyCode::Char('a'),
                    ..
                }) => self.in_vim_insert_mode = true,

                Event::Key(KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                }) => {
                    self.input.delete(Move::Backward);
                    self.refresh_matches();
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Delete | KeyCode::Char('x'),
                    ..
                }) => {
                    self.input.delete(Move::Forward);
                    self.refresh_matches();
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Home,
                    ..
                }) => self.input.move_cursor(Move::BOL),
                Event::Key(KeyEvent {
                    code: KeyCode::End, ..
                }) => self.input.move_cursor(Move::EOL),

                Event::Key(KeyEvent {
                    code: KeyCode::F(2),
                    ..
                }) => {
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
        screen: &mut Write,
        command: &Command,
        search: &str,
        width: u16,
        highlighted_text_color: Color,
        text_color: Color,
        debug: bool,
    ) -> String {
        let mut prev: usize = 0;
        let debug_space = if debug { 90 } else { 0 };
        // let max_grapheme_length = if width > debug_space {
        //     width - debug_space - 9
        // } else {
        //     11
        // };
        // let mut out = FixedLengthGraphemeString::empty(max_grapheme_length);

        if !search.is_empty() {
            for (start, end) in &command.match_bounds {
                if prev != *start {
                    let _ = queue!(screen, Print(&command.cmd[prev..*start]));
                }

                let _ = queue!(screen, SetForegroundColor(highlighted_text_color));
                let _ = queue!(screen, Print(&command.cmd[*start..*end]));
                let _ = queue!(screen, SetForegroundColor(text_color));
                prev = *end;
            }
        }

        if prev != command.cmd.len() {
            let _ = queue!(screen, Print(&command.cmd[prev..]));
        }

        if debug {
            // out.max_grapheme_length += debug_space;
            let _ = queue!(screen, Print("  "));
            let _ = queue!(screen, SetForegroundColor(Color::Blue));
            let _ = queue!(screen, Print(format!("rnk: {:.*} ", 2, command.rank)));
            let _ = queue!(screen, Print(format!("age: {:.*} ", 2, command.features.age_factor)));
            let _ = queue!(screen, Print(format!("lng: {:.*} ", 2, command.features.length_factor)));
            let _ = queue!(screen, Print(format!("ext: {:.*} ", 0, command.features.exit_factor)));
            let _ = queue!(screen, Print(format!(
                "r_ext: {:.*} ",
                0, command.features.recent_failure_factor
            )));
            // out.push_grapheme_str(format!("dir: {:.*} ", 3, command.features.dir_factor));
            // out.push_grapheme_str(format!(
            //     "s_dir: {:.*} ",
            //     3, command.features.selected_dir_factor
            // ));
            // out.push_grapheme_str(format!("ovlp: {:.*} ", 3, command.features.overlap_factor));
            // out.push_grapheme_str(format!(
            //     "i_ovlp: {:.*} ",
            //     3, command.features.immediate_overlap_factor
            // ));
            // out.push_grapheme_str(format!(
            //     "occ: {:.*}",
            //     2, command.features.occurrences_factor
            // ));
            // out.push_grapheme_str(format!(
            //     "s_occ: {:.*} ",
            //     2, command.features.selected_occurrences_factor
            // ));

            // out.push_str(&format!("{}", SetForegroundColor(text_color)));
        }

        // out.string
        "".to_string()
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




