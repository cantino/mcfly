use clap::{Arg, App, SubCommand};
use clap::AppSettings;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use std::env;
use bash_history;

#[derive(Debug)]
pub enum Mode {
    Add,
    Search
}

#[derive(Debug)]
pub struct Settings {
    pub mode: Mode,
    pub command: String,
    pub when: Option<i64>,
    pub exit_code: Option<i32>,
    pub dir: Option<String>,
    pub old_dir: Option<String>
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            mode: Mode::Add,
            command: String::new(),
            when: None,
            exit_code: None,
            dir: None,
            old_dir: None
        }
    }
}

impl Settings {
    pub fn parse_args() -> Settings {
        let matches = App::new("Bash Wizard")
            .version(crate_version!())
            .author(crate_authors!())
            .about("Wizardly Bash history")
            .setting(AppSettings::SubcommandRequiredElseHelp)
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
                .arg(Arg::with_name("exit")
                    .short("e")
                    .long("exit")
                    .value_name("EXIT_CODE")
                    .help("Exit code of command")
                    .takes_value(true))
                .arg(Arg::with_name("within")
                    .short("w")
                    .long("within")
                    .value_name("SECONDS")
                    .help("Number of seconds ago that the command must have been run")
                    .takes_value(true))
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
            .get_matches();


        let mut settings = Settings::default();

        match matches.subcommand() {
            ("add", Some(add_matches)) =>{
                settings.mode = Mode::Add;

                settings.when = Some(value_t!(add_matches, "when", i64).unwrap_or(
                    SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as i64
                ));

                if let Some(_) = add_matches.value_of("exit") {
                    settings.exit_code = Some(value_t!(add_matches, "exit", i32).unwrap_or_else(|e| e.exit()));
                }

                if let Some(dir) = add_matches.value_of("directory") {
                    settings.dir = Some(dir.to_string());
                } else {
                    settings.dir = env::var("PWD").ok();
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
                    // CD shows PWD as the resulting directory, but we want it from the source directory.
                    if settings.command.starts_with("cd ") || settings.command.starts_with("pushd ") {
                        settings.dir = settings.old_dir.to_owned();
                    }
                }
            },

            ("search", Some(search_matches)) =>{
                settings.mode = Mode::Search;
                if let Some(_) = search_matches.value_of("within") {
                    settings.when = Some(value_t!(search_matches, "within", i64).unwrap_or_else(|e| e.exit()));
                }
                if let Some(_) = search_matches.value_of("exit") {
                    settings.exit_code = Some(value_t!(search_matches, "exit", i32).unwrap_or_else(|e| e.exit()));
                }
                if let Some(dir) = search_matches.value_of("directory") {
                    settings.dir = Some(dir.to_string());
                }
                if let Some(values) = search_matches.values_of("command") {
                    settings.command = values.collect::<Vec<_>>().join(" ");
                } else {
                    settings.command = bash_history::last_history_line(&bash_history::bash_history_file_path()).expect("Command is required if ~/.bash_history is empty").trim_left_matches("#").to_string();
                    bash_history::delete_last_history_entry_if_comment(&bash_history::bash_history_file_path());
                }
            },
            ("", None)   => println!("No subcommand was used"), // If no subcommand was used it'll match the tuple ("", None)
            _            => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
        }

        settings
    }
}
