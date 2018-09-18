use clap::{Arg, App, SubCommand};
use clap::AppSettings;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use std::env;
use bash_history;

#[derive(Debug)]
pub enum Mode {
    Add,
    Search,
    Train,
    Export
}

#[derive(Debug)]
pub struct Settings {
    pub mode: Mode,
    pub debug: bool,
    pub session_id: String,
    pub command: String,
    pub dir: String,
    pub when_run: Option<i64>,
    pub exit_code: Option<i32>,
    pub old_dir: Option<String>,
    pub file: Option<String>
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            mode: Mode::Add,
            command: String::new(),
            session_id: String::new(),
            dir: String::new(),
            when_run: None,
            exit_code: None,
            old_dir: None,
            file: None,
            debug: false
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
                .short("s")
                .long("session_id")
                .help("Session ID to record or search under (defaults to $MCFLY_SESSION_ID)")
                .value_name("SESSION_ID")
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
                    .help("The command that was run (default last line of ~/.bash_history)")
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
            .subcommand(SubCommand::with_name("train")
                .about("Train the suggestion engine"))
            .subcommand(SubCommand::with_name("export")
                .about("Export training data")
                .arg(Arg::with_name("file")
                    .short("f")
                    .long("file")
                    .value_name("PATH")
                    .help("Output file path")
                    .required(true)
                    .takes_value(true)))
            .get_matches();

        let mut settings = Settings::default();

        settings.debug = matches.is_present("debug");
        settings.session_id = matches
            .value_of("session_id")
            .map(|s| s.to_string())
            .unwrap_or(env::var("MCFLY_SESSION_ID").expect("Please ensure that MCFLY_SESSION_ID contains a random session ID."));

        match matches.subcommand() {
            ("add", Some(add_matches)) =>{
                settings.mode = Mode::Add;

                settings.when_run = Some(value_t!(add_matches, "when", i64).unwrap_or(
                    SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as i64
                ));

                if let Some(_) = add_matches.value_of("exit") {
                    settings.exit_code = Some(value_t!(add_matches, "exit", i32).unwrap_or_else(|e| e.exit()));
                }

                if let Some(dir) = add_matches.value_of("directory") {
                    settings.dir = dir.to_string();
                } else {
                    settings.dir = env::var("PWD").expect("Unable to determine current directory");
                }

                if let Some(old_dir) = add_matches.value_of("old_directory") {
                    settings.old_dir = Some(old_dir.to_string());
                } else {
                    settings.old_dir = env::var("OLDPWD").ok();
                }

                if let Some(commands) = add_matches.values_of("command") {
                    settings.command = commands.collect::<Vec<_>>().join(" ");
                } else {
                    settings.command = bash_history::last_history_line(&bash_history::bash_history_file_path()).expect("Command is required if ~/.bash_history is empty");
                }

                // CD shows PWD as the resulting directory, but we want it from the source directory.
                if settings.command.starts_with("cd ") || settings.command.starts_with("pushd ") || settings.command.starts_with("j ") {
                    settings.dir = settings.old_dir.to_owned().unwrap_or(settings.dir);
                }
            },

            ("search", Some(search_matches)) =>{
                settings.mode = Mode::Search;
                if let Some(dir) = search_matches.value_of("directory") {
                    settings.dir = dir.to_string();
                } else {
                    settings.dir = env::var("PWD").expect("Unable to determine current directory");
                }
                if let Some(values) = search_matches.values_of("command") {
                    settings.command = values.collect::<Vec<_>>().join(" ");
                } else {
                    settings.command = bash_history::last_history_line(&bash_history::bash_history_file_path()).expect("Command is required if ~/.bash_history is empty").trim_left_matches("#").to_string();
                    bash_history::delete_last_history_entry_if_comment(&bash_history::bash_history_file_path());
                }
            },

            ("train", Some(_train_matches)) => {
                settings.mode = Mode::Train;
            },
            ("export", Some(export_matches)) => {
                settings.mode = Mode::Export;
                settings.file = Some(export_matches.value_of("file").unwrap().to_string());
            },
            ("", None)   => println!("No subcommand was used"), // If no subcommand was used it'll match the tuple ("", None)
            _            => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
        }

        settings
    }
}
