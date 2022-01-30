use crate::shell_history;
use clap::AppSettings;
use clap::{crate_authors, crate_version, value_t};
use clap::{App, Arg, SubCommand};
use dirs::home_dir;
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

#[derive(Debug)]
pub enum Mode {
    Add,
    Search,
    Train,
    Move,
    Init,
}

#[derive(Debug)]
pub enum KeyScheme {
    Emacs,
    Vim,
}

#[derive(Debug)]
pub enum InitMode {
    Bash,
    Zsh,
    Fish,
}

#[derive(Debug, PartialEq)]
pub enum InterfaceView {
    Top,
    Bottom,
}

#[derive(Debug)]
pub enum ResultSort {
    Rank,
    LastRun,
}

#[derive(Debug, Clone, Copy)]
pub enum HistoryFormat {
    /// bash format - commands in plain text, one per line, with multi-line commands joined.
    /// HISTTIMEFORMAT is assumed to be empty.
    Bash,

    /// zsh format - commands in plain text, with multiline commands on multiple lines.
    /// McFly does not currently handle joining these lines; they're treated as separate commands.
    /// If --zsh-extended-history was given, `extended_history` will be true, and we'll strip the
    /// timestamp from the beginning of each command.
    Zsh { extended_history: bool },

    /// fish's pseudo-yaml, with commands stored as 'cmd' with multiple lines joined into one with
    /// '\n', and with timestamps stored as 'when'.  ('paths' is ignored.)
    /// (Some discussion of changing format: https://github.com/fish-shell/fish-shell/pull/6493)
    Fish,
}

#[derive(Debug)]
pub struct Settings {
    pub mode: Mode,
    pub debug: bool,
    pub fuzzy: i16,
    pub session_id: String,
    pub mcfly_history: PathBuf,
    pub output_selection: Option<String>,
    pub command: String,
    pub dir: String,
    pub results: u16,
    pub when_run: Option<i64>,
    pub exit_code: Option<i32>,
    pub old_dir: Option<String>,
    pub append_to_histfile: bool,
    pub refresh_training_cache: bool,
    pub lightmode: bool,
    pub key_scheme: KeyScheme,
    pub history_format: HistoryFormat,
    pub limit: Option<i64>,
    pub skip_environment_check: bool,
    pub init_mode: InitMode,
    pub delete_without_confirm: bool,
    pub interface_view: InterfaceView,
    pub result_sort: ResultSort,
    pub disable_menu: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            mode: Mode::Add,
            output_selection: None,
            command: String::new(),
            session_id: String::new(),
            mcfly_history: PathBuf::new(),
            dir: String::new(),
            results: 10,
            when_run: None,
            exit_code: None,
            old_dir: None,
            refresh_training_cache: false,
            append_to_histfile: false,
            debug: false,
            fuzzy: 0,
            lightmode: false,
            key_scheme: KeyScheme::Emacs,
            history_format: HistoryFormat::Bash,
            limit: None,
            skip_environment_check: false,
            init_mode: InitMode::Bash,
            delete_without_confirm: false,
            interface_view: InterfaceView::Top,
            result_sort: ResultSort::Rank,
            disable_menu: false,
        }
    }
}

impl Settings {
    pub fn parse_args() -> Settings {
        let matches = App::new("McFly")
            .version(crate_version!())
            .author(crate_authors!())
            .about("Fly through your shell history")
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .arg(Arg::with_name("debug")
                .short("d")
                .long("debug")
                .help("Debug"))
            .arg(Arg::with_name("session_id")
                .long("session_id")
                .help("Session ID to record or search under (defaults to $MCFLY_SESSION_ID)")
                .value_name("SESSION_ID")
                .takes_value(true))
            .arg(Arg::with_name("mcfly_history")
                .long("mcfly_history")
                .help("Shell history file to read from when adding or searching (defaults to $MCFLY_HISTORY)")
                .value_name("MCFLY_HISTORY")
                .takes_value(true))
            .arg(Arg::with_name("history_format")
                .long("history_format")
                .help("Shell history file format, 'bash', 'zsh', 'zsh-extended' or 'fish' (defaults to 'bash')")
                .value_name("FORMAT")
                .takes_value(true))
            .subcommand(SubCommand::with_name("add")
                .about("Add commands to the history")
                .aliases(&["a"])
                .arg(Arg::with_name("exit")
                    .short("e")
                    .long("exit")
                    .value_name("EXIT_CODE")
                    .help("Exit code of command")
                    .takes_value(true))
                .arg(Arg::with_name("append_to_histfile")
                    .long("append-to-histfile")
                    .help("Also append new history to $HISTFILE/$MCFLY_HISTFILE (e.q., .bash_history)"))
                .arg(Arg::with_name("when")
                    .short("w")
                    .long("when")
                    .value_name("UNIX_EPOCH")
                    .help("The time that the command was run (default now)")
                    .takes_value(true))
                .arg(Arg::with_name("directory")
                    .short("d")
                    .long("dir")
                    .value_name("PATH")
                    .help("Directory where command was run (default $PWD)")
                    .takes_value(true))
                .arg(Arg::with_name("old_directory")
                    .short("o")
                    .long("old-dir")
                    .value_name("PATH")
                    .help("The previous directory the user was in before running the command (default $OLDPWD)")
                    .takes_value(true))
                .arg(Arg::with_name("command")
                    .help("The command that was run (default last line of $MCFLY_HISTORY file)")
                    .value_name("COMMAND")
                    .multiple(true)
                    .required(false)
                    .index(1)))
            .subcommand(SubCommand::with_name("search")
                .about("Search the history")
                .aliases(&["s"])
                .arg(Arg::with_name("directory")
                    .short("d")
                    .long("dir")
                    .value_name("PATH")
                    .help("Directory where command was run")
                    .takes_value(true))
                .arg(Arg::with_name("results")
                    .short("r")
                    .long("results")
                    .value_name("NUMBER")
                    .help("Number of results to return")
                    .takes_value(true))
                .arg(Arg::with_name("fuzzy")
                    .short("f")
                    .long("fuzzy")
                    .help("Fuzzy-find results. 0 is off; higher numbers weight shorter/earlier matches more. Try 2"))
                .arg(Arg::with_name("delete_without_confirm")
                    .long("delete_without_confirm")
                    .help("Delete entry without confirm"))
                .arg(Arg::with_name("output_selection")
                    .short("o")
                    .long("output-selection")
                    .value_name("PATH")
                    .help("Write results to file, including selection mode, new commandline, and any shell-specific requests")
                    .takes_value(true))
                .arg(Arg::with_name("command")
                    .help("The command search term(s)")
                    .value_name("COMMAND")
                    .multiple(true)
                    .required(false)
                    .index(1)))
            .subcommand(SubCommand::with_name("move")
                .about("Record a directory having been moved; moves command records from the old path to the new one")
                .arg(Arg::with_name("old_dir_path")
                    .help("The old directory path")
                    .value_name("OLD_DIR_PATH")
                    .multiple(false)
                    .required(true)
                    .index(1))
                .arg(Arg::with_name("new_dir_path")
                    .help("The new directory path")
                    .value_name("NEW_DIR_PATH")
                    .multiple(false)
                    .required(true)
                    .index(2)))
            .subcommand(SubCommand::with_name("train")
                .about("Train the suggestion engine (developer tool)")
                .arg(Arg::with_name("refresh_cache")
                    .short("r")
                    .long("refresh_cache")
                    .help("Directory where command was run")
                    .required(false)))
            .subcommand(SubCommand::with_name("init")
                .about("Prints the shell code used to execute mcfly")
                .arg(Arg::with_name("shell")
                    .help("Shell to init — one of bash, zsh, or fish")
                    .possible_values(&["bash", "zsh", "fish"])
                    .required(true))
            )
            .get_matches();

        let mut settings = Settings::default();
        if matches.is_present("init") {
            settings.skip_environment_check = true;
        }

        settings.debug = matches.is_present("debug") || env::var("MCFLY_DEBUG").is_ok();
        settings.limit = env::var("MCFLY_HISTORY_LIMIT")
            .ok()
            .and_then(|o| o.parse::<i64>().ok());

        settings.interface_view = match env::var("MCFLY_INTERFACE_VIEW") {
            Ok(val) => match val.as_str() {
                "TOP" => InterfaceView::Top,
                "BOTTOM" => InterfaceView::Bottom,
                _ => InterfaceView::Top,
            },
            _ => InterfaceView::Top,
        };

        settings.result_sort = match env::var("MCFLY_RESULTS_SORT") {
            Ok(val) => match val.as_str() {
                "RANK" => ResultSort::Rank,
                "LAST_RUN" => ResultSort::LastRun,
                _ => ResultSort::Rank,
            },
            _ => ResultSort::Rank,
        };

        settings.session_id = matches
            .value_of("session_id")
            .map(|s| s.to_string())
            .unwrap_or_else( ||
                env::var("MCFLY_SESSION_ID")
                    .unwrap_or_else(|err| {
                        if !settings.skip_environment_check
                        {
                            panic!(
                            "McFly error: Please ensure that MCFLY_SESSION_ID contains a random session ID ({})",
                            err)
                        }
                        else {
                            std::string::String::new()
                        }
                    }));
        settings.mcfly_history = PathBuf::from(
            matches
                .value_of("mcfly_history")
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    env::var("MCFLY_HISTORY").unwrap_or_else(|err| {
                        if !settings.skip_environment_check {
                            panic!(
                                "McFly error: Please ensure that MCFLY_HISTORY is set ({})",
                                err
                            )
                        } else {
                            std::string::String::new()
                        }
                    })
                }),
        );
        settings.history_format = match matches.value_of("history_format") {
            None => HistoryFormat::Bash,
            Some("bash") => HistoryFormat::Bash,
            Some("zsh") => HistoryFormat::Zsh {
                extended_history: false,
            },
            Some("zsh-extended") => HistoryFormat::Zsh {
                extended_history: true,
            },
            Some("fish") => HistoryFormat::Fish,
            Some(format) => panic!("McFly error: unknown history format '{}'", format),
        };

        match matches.subcommand() {
            ("add", Some(add_matches)) => {
                settings.mode = Mode::Add;

                settings.when_run = Some(
                    value_t!(add_matches, "when", i64).unwrap_or(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_else(|err| {
                                panic!("McFly error: Time went backwards ({})", err)
                            })
                            .as_secs() as i64,
                    ),
                );

                settings.append_to_histfile = add_matches.is_present("append_to_histfile");

                if add_matches.value_of("exit").is_some() {
                    settings.exit_code =
                        Some(value_t!(add_matches, "exit", i32).unwrap_or_else(|e| e.exit()));
                }

                if let Some(dir) = add_matches.value_of("directory") {
                    settings.dir = dir.to_string();
                } else {
                    settings.dir = env::var("PWD").unwrap_or_else(|err| {
                        panic!(
                            "McFly error: Unable to determine current directory ({})",
                            err
                        )
                    });
                }

                if let Some(old_dir) = add_matches.value_of("old_directory") {
                    settings.old_dir = Some(old_dir.to_string());
                } else {
                    settings.old_dir = env::var("OLDPWD").ok();
                }

                if let Some(commands) = add_matches.values_of("command") {
                    settings.command = commands.collect::<Vec<_>>().join(" ");
                } else {
                    settings.command = shell_history::last_history_line(
                        &settings.mcfly_history,
                        settings.history_format,
                    )
                    .unwrap_or_else(String::new);
                }

                // CD shows PWD as the resulting directory, but we want it from the source directory.
                if settings.command.starts_with("cd ")
                    || settings.command.starts_with("pushd ")
                    || settings.command.starts_with("j ")
                {
                    settings.dir = settings.old_dir.to_owned().unwrap_or(settings.dir);
                }
            }

            ("search", Some(search_matches)) => {
                settings.mode = Mode::Search;
                if let Some(dir) = search_matches.value_of("directory") {
                    settings.dir = dir.to_string();
                } else {
                    settings.dir = env::var("PWD").unwrap_or_else(|err| {
                        panic!(
                            "McFly error: Unable to determine current directory ({})",
                            err
                        )
                    });
                }

                if let Ok(results) = env::var("MCFLY_RESULTS") {
                    if let Ok(results) = u16::from_str(&results) {
                        settings.results = results;
                    }
                }
                if let Ok(results) = value_t!(search_matches.value_of("results"), u16) {
                    settings.results = results;
                }

                if let Ok(fuzzy) = env::var("MCFLY_FUZZY") {
                    if let Ok(fuzzy) = i16::from_str(&fuzzy) {
                        settings.fuzzy = fuzzy;
                    } else if fuzzy.to_lowercase() != "false" {
                        settings.fuzzy = 2;
                    }
                }
                if let Ok(fuzzy) = value_t!(search_matches.value_of("fuzzy"), i16) {
                    settings.fuzzy = fuzzy;
                } else if search_matches.is_present("fuzzy") {
                    settings.fuzzy = 2;
                }

                settings.delete_without_confirm = search_matches
                    .is_present("delete_without_confirm")
                    || env::var("MCFLY_DELETE_WITHOUT_CONFIRM").is_ok();
                settings.output_selection = search_matches
                    .value_of("output_selection")
                    .map(|s| s.to_owned());

                if let Some(values) = search_matches.values_of("command") {
                    settings.command = values.collect::<Vec<_>>().join(" ");
                } else {
                    settings.command = shell_history::last_history_line(
                        &settings.mcfly_history,
                        settings.history_format,
                    )
                    .unwrap_or_else(String::new)
                    .trim_start_matches("#mcfly: ")
                    .trim_start_matches("#mcfly:")
                    .to_string();
                    shell_history::delete_last_history_entry_if_search(
                        &settings.mcfly_history,
                        settings.history_format,
                        settings.debug,
                    );
                }
            }

            ("train", Some(train_matches)) => {
                settings.mode = Mode::Train;
                settings.refresh_training_cache = train_matches.is_present("refresh_cache");
            }

            ("move", Some(move_matches)) => {
                settings.mode = Mode::Move;
                settings.old_dir = Some(String::from(
                    move_matches
                        .value_of("old_dir_path")
                        .unwrap_or_else(|| panic!("McFly error: Expected value for old_dir_path")),
                ));
                settings.dir = String::from(
                    move_matches
                        .value_of("new_dir_path")
                        .unwrap_or_else(|| panic!("McFly error: Expected value for new_dir_path")),
                );
            }

            ("init", Some(init_matches)) => {
                settings.mode = Mode::Init;
                match init_matches.value_of("shell").unwrap() {
                    "bash" => {
                        settings.init_mode = InitMode::Bash;
                    }
                    "zsh" => {
                        settings.init_mode = InitMode::Zsh;
                    }
                    "fish" => {
                        settings.init_mode = InitMode::Fish;
                    }
                    _ => unreachable!(),
                }
            }

            ("", None) => println!("No subcommand was used"), // If no subcommand was used it'll match the tuple ("", None)
            _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
        }

        settings.lightmode = match env::var_os("MCFLY_LIGHT") {
            Some(_val) => true,
            None => false,
        };

        settings.disable_menu = match env::var_os("MCFLY_DISABLE_MENU") {
            Some(_val) => true,
            None => false,
        };

        settings.key_scheme = match env::var("MCFLY_KEY_SCHEME").as_ref().map(String::as_ref) {
            Ok("vim") => KeyScheme::Vim,
            _ => KeyScheme::Emacs,
        };

        settings
    }

    pub fn mcfly_training_cache_path() -> PathBuf {
        Settings::storage_dir_path().join(PathBuf::from("training-cache.v1.csv"))
    }

    pub fn storage_dir_path() -> PathBuf {
        home_dir()
            .unwrap_or_else(|| panic!("McFly error: Unable to access home directory"))
            .join(PathBuf::from(".mcfly"))
    }

    pub fn mcfly_db_path() -> PathBuf {
        Settings::storage_dir_path().join(PathBuf::from("history.db"))
    }
}
