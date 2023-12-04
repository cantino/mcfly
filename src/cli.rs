use clap::{Parser, Subcommand, ValueEnum};
use regex::Regex;
use std::path::PathBuf;

/// Fly through your shell history
#[derive(Parser)]
#[command(author, version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: SubCommand,

    /// Debug
    #[arg(short, long)]
    pub debug: bool,

    /// Session ID to record or search under (defaults to $MCFLY_SESSION_ID)
    #[arg(long = "session_id")]
    pub session_id: Option<String>,

    /// Shell history file to read from when adding or searching (defaults to $MCFLY_HISTORY)
    #[arg(long = "mcfly_history")]
    pub mcfly_history: Option<PathBuf>,

    /// Shell history file format
    #[arg(
        value_name = "FORMAT",
        value_enum,
        long = "history_format",
        default_value_t
    )]
    pub history_format: HistoryFormat,
}

#[derive(Subcommand)]
pub enum SubCommand {
    /// Add commands to the history
    #[command(alias = "a")]
    Add {
        /// The command that was run (default last line of $MCFLY_HISTORY file)
        command: Vec<String>,

        /// Exit code of command
        #[arg(value_name = "EXIT_CODE", short, long)]
        exit: Option<i32>,

        /// Also append command to the given file (e.q., .bash_history)
        #[arg(value_name = "HISTFILE", short, long)]
        append_to_histfile: Option<String>,

        /// The time that the command was run (default now)
        #[arg(value_name = "UNIX_EPOCH", short, long)]
        when: Option<i64>,

        /// Directory where command was run (default $PWD)
        #[arg(value_name = "PATH", short, long = "dir")]
        directory: Option<String>,

        /// The previous directory the user was in before running the command (default $OLDPWD)
        #[arg(value_name = "PATH", short, long = "old-dir")]
        old_directory: Option<String>,
    },

    /// Search the history
    #[command(alias = "s")]
    Search {
        /// The command search term(s)
        command: Vec<String>,

        /// Directory where command was run (default $PWD)
        #[arg(value_name = "PATH", short, long = "dir")]
        directory: Option<String>,

        /// Number of results to return
        #[arg(value_name = "NUMBER", short, long)]
        results: Option<u16>,

        /// Fuzzy-find results. 0 is off; higher numbers weight shorter/earlier matches more. Try 2
        #[arg(short, long)]
        fuzzy: Option<i16>,

        /// Delete entry without confirm
        #[arg(long = "delete_without_confirm")]
        delete_without_confirm: bool,

        /// Write results to file, including selection mode, new commandline, and any shell-specific requests
        #[arg(value_name = "PATH", short, long)]
        output_selection: Option<String>,
    },

    /// Record a directory having been moved; moves command records from the old path to the new one
    Move {
        /// The old directory path
        old_dir_path: String,

        /// The new directory path
        new_dir_path: String,
    },

    /// Train the suggestion engine (developer tool)
    Train {
        /// Directory where command was run
        #[arg(short, long = "refresh_cache")]
        refresh_cache: bool,
    },

    /// Prints the shell code used to execute mcfly
    Init {
        /// Shell to init
        #[arg(value_enum)]
        shell: InitMode,
    },

    /// Dump history into stdout; the results are sorted by timestamp
    Dump {
        /// Select all commands ran since the point
        #[arg(long)]
        since: Option<String>,

        /// Select all commands ran before the point
        #[arg(long)]
        before: Option<String>,

        /// Sort order [case ignored]
        #[arg(
            long,
            short,
            value_name = "ORDER",
            value_enum,
            default_value_t,
            ignore_case = true
        )]
        sort: SortOrder,

        /// Require commands to match the pattern
        #[arg(long, short)]
        regex: Option<Regex>,

        /// The format to dump in
        #[arg(long, short, value_enum, default_value_t)]
        format: DumpFormat,
    },
}

#[derive(Clone, Copy, ValueEnum, Default)]
pub enum HistoryFormat {
    #[default]
    Bash,
    Zsh,
    ZshExtended,
    Fish,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum InitMode {
    Bash,
    Zsh,
    Fish,
    Powershell,
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
#[value(rename_all = "UPPER")]
pub enum SortOrder {
    #[default]
    #[value(alias = "asc")]
    Asc,
    #[value(alias = "desc")]
    Desc,
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum DumpFormat {
    #[default]
    Json,
    Csv,
}

impl Cli {
    pub fn is_init(&self) -> bool {
        matches!(self.command, SubCommand::Init { .. })
    }
}

impl SortOrder {
    #[inline]
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}
