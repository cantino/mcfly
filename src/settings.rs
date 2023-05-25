use crate::cli::{Cli, SubCommand};
use crate::shell_history;
use clap::Parser;
use etcetera::base_strategy::Xdg;
use etcetera::BaseStrategy;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use std::{env, fs};

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

#[derive(Debug, PartialEq, Eq)]
pub enum InterfaceView {
    Top,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResultSort {
    Rank,
    LastRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResultFilter {
    Global,
    CurrentDirectory,
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
    pub append_to_histfile: Option<String>,
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
    pub result_filter: ResultFilter,
    pub disable_menu: bool,
    pub prompt: String,
    pub disable_run_command: bool,
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
            append_to_histfile: None,
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
            result_filter: ResultFilter::Global,
            disable_menu: false,
            prompt: String::from("$"),
            disable_run_command: false,
        }
    }
}

impl Settings {
    pub fn parse_args() -> Settings {
        let cli = Cli::parse();

        let mut settings = Settings {
            skip_environment_check: cli.is_init(),
            ..Default::default()
        };

        settings.debug = cli.debug || is_env_var_truthy("MCFLY_DEBUG");
        settings.limit = env::var("MCFLY_HISTORY_LIMIT")
            .ok()
            .and_then(|o| o.parse::<i64>().ok());

        settings.interface_view = match env::var("MCFLY_INTERFACE_VIEW") {
            Ok(val) => match val.to_uppercase().as_str() {
                "TOP" => InterfaceView::Top,
                "BOTTOM" => InterfaceView::Bottom,
                _ => InterfaceView::Top,
            },
            _ => InterfaceView::Top,
        };

        settings.result_sort = match env::var("MCFLY_RESULTS_SORT") {
            Ok(val) => match val.to_uppercase().as_str() {
                "RANK" => ResultSort::Rank,
                "LAST_RUN" => ResultSort::LastRun,
                _ => ResultSort::Rank,
            },
            _ => ResultSort::Rank,
        };

        settings.result_filter = match env::var("MCFLY_RESULTS_FILTER") {
            Ok(val) => match val.to_uppercase().as_str() {
                "GLOBAL" => ResultFilter::Global,
                "CURRENT_DIRECTORY" => ResultFilter::CurrentDirectory,
                _ => ResultFilter::Global,
            },
            _ => ResultFilter::Global,
        };

        settings.session_id = cli.session_id.unwrap_or_else(||
            env::var("MCFLY_SESSION_ID")
                .unwrap_or_else(|err| {
                    if !settings.skip_environment_check {
                        panic!(
                            "McFly error: Please ensure that MCFLY_SESSION_ID contains a random session ID ({})",
                            err
                        )
                    } else {
                        String::new()
                    }
                }
            )
        );

        settings.mcfly_history = cli.mcfly_history.unwrap_or_else(|| {
            {
                env::var("MCFLY_HISTORY").unwrap_or_else(|err| {
                    if !settings.skip_environment_check {
                        panic!(
                            "McFly error: Please ensure that MCFLY_HISTORY is set ({})",
                            err
                        )
                    } else {
                        String::new()
                    }
                })
            }
            .into()
        });

        {
            use crate::cli::HistoryFormat::*;
            settings.history_format = match cli.history_format {
                Bash => HistoryFormat::Bash,
                Zsh => HistoryFormat::Zsh {
                    extended_history: false,
                },
                ZshExtended => HistoryFormat::Zsh {
                    extended_history: true,
                },
                Fish => HistoryFormat::Fish,
            };
        }

        match cli.command {
            SubCommand::Add {
                command,
                exit,
                append_to_histfile,
                when,
                directory,
                old_directory,
            } => {
                settings.mode = Mode::Add;

                settings.when_run = when.or_else(|| {
                    Some(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_else(|err| {
                                panic!("McFly error: Time went backwards ({})", err)
                            })
                            .as_secs() as i64,
                    )
                });

                settings.append_to_histfile = append_to_histfile;

                settings.exit_code = exit;

                settings.dir = directory.unwrap_or_else(pwd);

                settings.old_dir = old_directory.or_else(|| env::var("OLDPWD").ok());

                if !command.is_empty() {
                    settings.command = command.join(" ");
                } else {
                    settings.command = shell_history::last_history_line(
                        &settings.mcfly_history,
                        settings.history_format,
                    )
                    .unwrap_or_default();
                }

                // CD shows PWD as the resulting directory, but we want it from the source directory.
                if settings.command.starts_with("cd ")
                    || settings.command.starts_with("pushd ")
                    || settings.command.starts_with("j ")
                {
                    settings.dir = settings.old_dir.clone().unwrap_or(settings.dir);
                }
            }

            SubCommand::Search {
                command,
                directory,
                results,
                fuzzy,
                delete_without_confirm,
                output_selection,
            } => {
                settings.mode = Mode::Search;

                settings.dir = directory.unwrap_or_else(pwd);

                if let Ok(results) = env::var("MCFLY_RESULTS") {
                    if let Ok(results) = u16::from_str(&results) {
                        settings.results = results;
                    }
                }

                if let Some(results) = results {
                    settings.results = results;
                }

                if let Ok(fuzzy) = env::var("MCFLY_FUZZY") {
                    if let Ok(fuzzy) = i16::from_str(&fuzzy) {
                        settings.fuzzy = fuzzy;
                    } else if fuzzy.to_lowercase() != "false" {
                        settings.fuzzy = 2;
                    }
                }

                if let Some(fuzzy) = fuzzy {
                    settings.fuzzy = fuzzy;
                }

                settings.delete_without_confirm =
                    delete_without_confirm || is_env_var_truthy("MCFLY_DELETE_WITHOUT_CONFIRM");

                settings.output_selection = output_selection;

                if !command.is_empty() {
                    settings.command = command.join(" ");
                } else {
                    settings.command = shell_history::last_history_line(
                        &settings.mcfly_history,
                        settings.history_format,
                    )
                    .unwrap_or_default()
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

            SubCommand::Train { refresh_cache } => {
                settings.mode = Mode::Train;

                settings.refresh_training_cache = refresh_cache;
            }

            SubCommand::Move {
                old_dir_path,
                new_dir_path,
            } => {
                settings.mode = Mode::Move;

                settings.old_dir = Some(old_dir_path);
                settings.dir = new_dir_path;
            }

            SubCommand::Init { shell } => {
                settings.mode = Mode::Init;

                use crate::cli::InitMode::*;
                settings.init_mode = match shell {
                    Bash => InitMode::Bash,
                    Zsh => InitMode::Zsh,
                    Fish => InitMode::Fish,
                };
            }
        }

        settings.lightmode = is_env_var_truthy("MCFLY_LIGHT");

        settings.disable_menu = is_env_var_truthy("MCFLY_DISABLE_MENU");

        settings.disable_run_command = is_env_var_truthy("MCFLY_DISABLE_RUN_COMMAND");

        settings.key_scheme = match env::var("MCFLY_KEY_SCHEME").as_ref().map(String::as_ref) {
            Ok("vim") => KeyScheme::Vim,
            _ => KeyScheme::Emacs,
        };

        if let Ok(prompt) = env::var("MCFLY_PROMPT") {
            if prompt.chars().count() == 1 {
                settings.prompt = prompt;
            }
        }

        settings
    }

    // Use or create the 'mcfly' folder in `$XDG_CACHE_DIR`
    pub fn mcfly_training_cache_path() -> PathBuf {
        Settings::xdg_base_strategy()
            .cache_dir()
            .join(PathBuf::from("mcfly/training-cache.v1.csv"))
    }

    // Use or migrate to the 'mcfly' folder in `$XDG_STATE_DIR`
    pub fn mcfly_db_path() -> PathBuf {
        let basedirs = Settings::xdg_base_strategy();
        let path = basedirs
            .state_dir()
            .unwrap()
            .join(PathBuf::from("mcfly/history.db"));
        if !path.exists() {
            println!("McFly: history.db not found: `{}`", path.display());
            let old = basedirs.data_dir().join(PathBuf::from("mcfly/history.db"));
            if !Settings::migrate_old_path(old, &path) {
                let old = basedirs.home_dir().join(PathBuf::from(".mcfly/history.db"));
                if !Settings::migrate_old_path(old, &path) {
                    #[cfg(target_os = "macos")]
                    {
                        let old = etcetera::base_strategy::Apple::new()
                            .unwrap()
                            .data_dir()
                            .join(PathBuf::from("McFly/history.db"));
                        Settings::migrate_old_path(old, &path);
                    }
                }
            }
        }
        path
    }

    fn xdg_base_strategy() -> impl BaseStrategy {
        Xdg::new().unwrap()
    }

    fn migrate_old_path(old: PathBuf, new: &Path) -> bool {
        println!("McFly: Checking old history.db: `{}`", old.display());
        let ret = old.exists();
        if ret {
            fs::create_dir_all(new.parent().unwrap()).unwrap();
            fs::copy(&old, new).unwrap();
            println!(
                "McFly: history.db migrated.\nYou can now delete the old directory: `rm -r '{}'`",
                old.parent().unwrap().display()
            );
        }
        ret
    }
}

fn pwd() -> String {
    env::var("PWD").unwrap_or_else(|err| {
        panic!(
            "McFly error: Unable to determine current directory ({})",
            err
        )
    })
}

fn is_env_var_truthy(name: &str) -> bool {
    match env::var(name) {
        Ok(val) => {
            val != "F"
                && val != "f"
                && val != "false"
                && val != "False"
                && val != "FALSE"
                && val != "0"
        }
        Err(_) => false,
    }
}
