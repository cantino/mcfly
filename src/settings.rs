use std::env;

use clap::{Arg, App, SubCommand};
use clap::AppSettings;
use std::num::ParseIntError;

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
    pub dir: Option<String>
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            mode: Mode::Add,
            command: String::new(),
            when: None,
            exit_code: None,
            dir: None
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
                    .value_name("EXIT_STATUS")
                    .help("Exit status of command")
                    .takes_value(true))
                .arg(Arg::with_name("time")
                    .short("t")
                    .long("time")
                    .value_name("TIME")
                    .help("Time command was run as UNIX epoch")
                    .takes_value(true))
                .arg(Arg::with_name("directory")
                    .short("d")
                    .long("dir")
                    .value_name("DIR")
                    .help("Directory where command was run")
                    .takes_value(true))
                .arg(Arg::with_name("command")
                    .help("The command that was run")
                    .value_name("COMMAND")
                    .multiple(true)
                    .required(true)
                    .index(1)))
            .subcommand(SubCommand::with_name("search")
                .about("Search the history")
                .aliases(&["s"])
                .arg(Arg::with_name("exit")
                    .short("e")
                    .long("exit")
                    .value_name("EXIT_STATUS")
                    .help("Exit status of command")
                    .takes_value(true))
                .arg(Arg::with_name("time")
                    .short("t")
                    .long("time")
                    .value_name("TIME")
                    .help("Time window command was run during (in seconds)")
                    .takes_value(true))
                .arg(Arg::with_name("directory")
                    .short("d")
                    .long("dir")
                    .value_name("DIR")
                    .help("Directory where command was run")
                    .takes_value(true))
                .arg(Arg::with_name("command")
                    .help("The command search term(s)")
                    .value_name("COMMAND")
                    .multiple(true)
                    .required(true)
                    .index(1)))
            .get_matches();


        let mut settings = Settings::default();

        match matches.subcommand() {
            ("add", Some(add_matches)) =>{
                settings.mode = Mode::Add;
                if let Some(time) = add_matches.value_of("time") {
                    settings.when = Some(value_t!(add_matches, "time", i64).unwrap_or_else(|e| e.exit()));
                }
                if let Some(exit_code) = add_matches.value_of("exit") {
                    settings.exit_code = Some(value_t!(add_matches, "exit", i32).unwrap_or_else(|e| e.exit()));
                }
                if let Some(dir) = add_matches.value_of("directory") {
                    settings.dir = Some(dir.to_string());
                }
                settings.command = add_matches.values_of("command").unwrap().collect::<Vec<_>>().join(" ");
            },
            ("search", Some(search_matches)) =>{
                settings.mode = Mode::Search;
                if let Some(time) = search_matches.value_of("time") {
                    settings.when = Some(value_t!(search_matches, "time", i64).unwrap_or_else(|e| e.exit()));
                }
                if let Some(exit_code) = search_matches.value_of("exit") {
                    settings.exit_code = Some(value_t!(search_matches, "exit", i32).unwrap_or_else(|e| e.exit()));
                }
                if let Some(dir) = search_matches.value_of("directory") {
                    settings.dir = Some(dir.to_string());
                }
                settings.command = search_matches.values_of("command").unwrap().collect::<Vec<_>>().join(" ");
            },
            ("", None)   => println!("No subcommand was used"), // If no subcommand was used it'll match the tuple ("", None)
            _            => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
        }

        settings
    }
}
