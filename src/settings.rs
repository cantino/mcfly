use crate::bash_history;
use clap::{crate_version, crate_authors, value_t};
use clap::AppSettings;
use clap::{App, Arg, SubCommand};
use std::env;
use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use dirs::home_dir;

#[derive(Debug)]
pub enum Mode {
    Add,
    Search,
    Train,
    Move
}

#[derive(Debug)]
pub struct Settings {
    pub mode: Mode,
    pub debug: bool,
    pub session_id: String,
    pub mcfly_history: PathBuf,
    pub command: String,
    pub dir: String,
    pub when_run: Option<i64>,
    pub exit_code: Option<i32>,
    pub old_dir: Option<String>,
    pub append_to_histfile: bool,
    pub refresh_training_cache: bool,
    pub lightmode: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            mode: Mode::Add,
            command: String::new(),
            session_id: String::new(),
            mcfly_history: PathBuf::new(),
            dir: String::new(),
            when_run: None,
            exit_code: None,
            old_dir: None,
            refresh_training_cache: false,
            append_to_histfile: false,
            debug: false,
            lightmode: false,
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
                    .help("Also append new history to $HISTFILE (e.q., .bash_history)"))
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
            .get_matches();

        let mut settings = Settings::default();

        settings.debug = matches.is_present("debug");
        settings.session_id = matches
            .value_of("session_id")
            .map(|s| s.to_string())
            .unwrap_or(
                env::var("MCFLY_SESSION_ID")
                    .expect("McFly error: Please ensure that MCFLY_SESSION_ID contains a random session ID."),
            );
        settings.mcfly_history = PathBuf::from(
            matches
                .value_of("mcfly_history")
                .map(|s| s.to_string())
                .unwrap_or(
                    env::var("MCFLY_HISTORY").expect("McFly error: Please ensure that MCFLY_HISTORY is set."),
                ),
        );

        match matches.subcommand() {
            ("add", Some(add_matches)) => {
                settings.mode = Mode::Add;

                settings.when_run = Some(
                    value_t!(add_matches, "when", i64).unwrap_or(SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("McFly error: Time went backwards")
                        .as_secs()
                        as i64),
                );

                settings.append_to_histfile = add_matches.is_present("append_to_histfile");

                if let Some(_) = add_matches.value_of("exit") {
                    settings.exit_code =
                        Some(value_t!(add_matches, "exit", i32).unwrap_or_else(|e| e.exit()));
                }

                if let Some(dir) = add_matches.value_of("directory") {
                    settings.dir = dir.to_string();
                } else {
                    settings.dir = env::var("PWD").expect("McFly error: Unable to determine current directory");
                }

                if let Some(old_dir) = add_matches.value_of("old_directory") {
                    settings.old_dir = Some(old_dir.to_string());
                } else {
                    settings.old_dir = env::var("OLDPWD").ok();
                }

                if let Some(commands) = add_matches.values_of("command") {
                    settings.command = commands.collect::<Vec<_>>().join(" ");
                } else {
                    settings.command = bash_history::last_history_line(&settings.mcfly_history)
                        .unwrap_or(String::from(""));
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
                    settings.dir = env::var("PWD").expect("McFly error: Unable to determine current directory");
                }
                if let Some(values) = search_matches.values_of("command") {
                    settings.command = values.collect::<Vec<_>>().join(" ");
                } else {
                    settings.command = bash_history::last_history_line(&settings.mcfly_history)
                        .unwrap_or(String::from(""))
                        .trim_left_matches("#mcfly: ")
                        .trim_left_matches("#mcfly:")
                        .to_string();
                    bash_history::delete_last_history_entry_if_search(&settings.mcfly_history);
                }
            }

            ("train", Some(train_matches)) => {
                settings.mode = Mode::Train;
                settings.refresh_training_cache = train_matches.is_present("refresh_cache");
            }

            ("move", Some(move_matches)) => {
                settings.mode = Mode::Move;
                settings.old_dir = Some(String::from(move_matches.value_of("old_dir_path").expect("McFly error: Value for old_dir_path")));
                settings.dir = String::from(move_matches.value_of("new_dir_path").expect("McFly error: Value for new_dir_path"));
            }

            ("", None) => println!("No subcommand was used"), // If no subcommand was used it'll match the tuple ("", None)
            _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
        }

        settings.lightmode = match env::var_os("MCFLY_LIGHT") {
            Some(_val) => true,
            None => false
        };
        settings
    }

    pub fn mcfly_training_cache_path() -> PathBuf {
        Settings::storage_dir_path().join(PathBuf::from("training-cache.v1.csv"))
    }

    pub fn storage_dir_path() -> PathBuf {
        home_dir()
            .expect("McFly error: Unable to access home directory")
            .join(PathBuf::from(".mcfly"))
    }

    pub fn mcfly_db_path() -> PathBuf {
        Settings::storage_dir_path().join(PathBuf::from("history.db"))
    }
}
