use crate::cli::{Cli, DumpFormat, SortOrder, SubCommand};
use crate::shell_history;
use crate::time::parse_timestamp;
use clap::Parser;
use config::Source;
use config::Value;
use crossterm::style::Color;
use directories_next::{ProjectDirs, UserDirs};
use regex::Regex;
use std::collections::HashMap;
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
    Dump,
    Stats,
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
    Powershell,
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
    /// `McFly` does not currently handle joining these lines; they're treated as separate commands.
    /// If --zsh-extended-history was given, `extended_history` will be true, and we'll strip the
    /// timestamp from the beginning of each command.
    Zsh { extended_history: bool },

    /// fish's pseudo-yaml, with commands stored as 'cmd' with multiple lines joined into one with
    /// '\n', and with timestamps stored as 'when'.  ('paths' is ignored.)
    /// (Some discussion of changing format: https://github.com/fish-shell/fish-shell/pull/6493)
    Fish,
}

/// Time range, it can be:
/// - `..`
/// - `since..before`
/// - `since..`
/// - `..before`
#[derive(Debug, Clone, Default)]
pub struct TimeRange {
    pub since: Option<i64>,
    pub before: Option<i64>,
}

#[derive(Debug)]
pub struct Colors {
    pub menubar_bg: Color,
    pub menubar_fg: Color,
    pub darkmode_colors: DarkModeColors,
    pub lightmode_colors: LightModeColors,
}

#[derive(Debug)]
pub struct DarkModeColors {
    pub prompt: Color,
    pub timing: Color,
    pub results_fg: Color,
    pub results_bg: Color,
    pub results_hl: Color,
    pub results_selection_fg: Color,
    pub results_selection_bg: Color,
    pub results_selection_hl: Color,
}

#[derive(Debug)]
pub struct LightModeColors {
    pub prompt: Color,
    pub timing: Color,
    pub results_fg: Color,
    pub results_bg: Color,
    pub results_hl: Color,
    pub results_selection_fg: Color,
    pub results_selection_bg: Color,
    pub results_selection_hl: Color,
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
    pub time_range: TimeRange,
    pub sort_order: SortOrder,
    pub pattern: Option<Regex>,
    pub dump_format: DumpFormat,
    pub colors: Colors,
    pub stats_min_cmd_length: i16,
    pub stats_cmds: i16,
    pub stats_dirs: i16,
    pub stats_global_commands_to_ignore: i16,
    pub stats_only_dir: Option<String>,
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
            time_range: TimeRange::default(),
            sort_order: SortOrder::default(),
            pattern: None,
            dump_format: DumpFormat::default(),
            colors: Colors {
                menubar_bg: Color::Blue,
                menubar_fg: Color::White,
                darkmode_colors: DarkModeColors {
                    prompt: Color::White,
                    timing: Color::Blue,
                    results_fg: Color::White,
                    results_bg: Color::Black,
                    results_hl: Color::Blue,
                    results_selection_fg: Color::Black,
                    results_selection_bg: Color::White,
                    results_selection_hl: Color::DarkGreen,
                },
                lightmode_colors: LightModeColors {
                    prompt: Color::Black,
                    timing: Color::DarkBlue,
                    results_fg: Color::Black,
                    results_bg: Color::White,
                    results_hl: Color::Blue,
                    results_selection_fg: Color::White,
                    results_selection_bg: Color::DarkGrey,
                    results_selection_hl: Color::Grey,
                },
            },
            stats_min_cmd_length: 0,
            stats_cmds: 10,
            stats_dirs: 0,
            stats_global_commands_to_ignore: 10,
            stats_only_dir: None,
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
                            "McFly error: Please ensure that MCFLY_SESSION_ID contains a random session ID ({err})"
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
                        panic!("McFly error: Please ensure that MCFLY_HISTORY is set ({err})")
                    } else {
                        String::new()
                    }
                })
            }
            .into()
        });

        {
            use crate::cli::HistoryFormat::{Bash, Fish, Zsh, ZshExtended};
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
                                panic!("McFly error: Time went backwards ({err})")
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

                use crate::cli::InitMode::{Bash, Fish, Powershell, Zsh};
                settings.init_mode = match shell {
                    Bash => InitMode::Bash,
                    Zsh => InitMode::Zsh,
                    Fish => InitMode::Fish,
                    Powershell => InitMode::Powershell,
                };
            }

            SubCommand::Dump {
                since,
                before,
                sort,
                regex,
                format,
            } => {
                settings.mode = Mode::Dump;

                settings.time_range.since = since.as_ref().map(|s| parse_timestamp(s));
                settings.time_range.before = before.as_ref().map(|s| parse_timestamp(s));
                settings.sort_order = sort;
                settings.pattern = regex;
                settings.dump_format = format;
            }

            SubCommand::Stats {
                min_cmd_length,
                cmds,
                dirs,
                global_commands_to_ignore,
                only_dir,
            } => {
                settings.mode = Mode::Stats;
                settings.stats_min_cmd_length = min_cmd_length;
                settings.stats_cmds = cmds;
                settings.stats_dirs = dirs;
                settings.stats_global_commands_to_ignore = global_commands_to_ignore;
                settings.stats_only_dir = only_dir;
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

    pub fn load_config(&mut self) {
        let config_path = Settings::mcfly_config_path();
        if config_path.exists() {
            let config = config::File::from(config_path);
            if let Ok(config_map) = config.collect() {
                self.merge_config(config_map);
            }
        };
    }

    pub fn merge_config(&mut self, config_map: HashMap<String, Value>) {
        let color_config = config_map.get("colors");

        let menubar_config = color_config
            .and_then(|v| v.clone().into_table().ok())
            .and_then(|v| v.get("menubar").and_then(|v| v.clone().into_table().ok()));

        if let Some(menubar_config) = menubar_config {
            if let Some(menubar_bg) = menubar_config
                .get("bg")
                .and_then(|v| v.clone().into_string().ok())
                .and_then(|v| Color::from_str(v.as_str()).ok())
            {
                self.colors.menubar_bg = menubar_bg;
            }
            if let Some(menubar_fg) = menubar_config
                .get("fg")
                .and_then(|v| v.clone().into_string().ok())
                .and_then(|v| Color::from_str(v.as_str()).ok())
            {
                self.colors.menubar_fg = menubar_fg;
            }
        }

        let darkmode_config = color_config
            .and_then(|v| v.clone().into_table().ok())
            .and_then(|v| v.get("darkmode").and_then(|v| v.clone().into_table().ok()));

        if let Some(darkmode_config) = darkmode_config {
            if let Some(prompt) = darkmode_config
                .get("prompt")
                .and_then(|v| v.clone().into_string().ok())
                .and_then(|v| Color::from_str(v.as_str()).ok())
            {
                self.colors.darkmode_colors.prompt = prompt;
            }
            if let Some(timing) = darkmode_config
                .get("timing")
                .and_then(|v| v.clone().into_string().ok())
                .and_then(|v| Color::from_str(v.as_str()).ok())
            {
                self.colors.darkmode_colors.timing = timing;
            }
            if let Some(results_fg) = darkmode_config
                .get("results_fg")
                .and_then(|v| v.clone().into_string().ok())
                .and_then(|v| Color::from_str(v.as_str()).ok())
            {
                self.colors.darkmode_colors.results_fg = results_fg;
            }
            if let Some(results_bg) = darkmode_config
                .get("results_bg")
                .and_then(|v| v.clone().into_string().ok())
                .and_then(|v| Color::from_str(v.as_str()).ok())
            {
                self.colors.darkmode_colors.results_bg = results_bg;
            }
            if let Some(results_hl) = darkmode_config
                .get("results_hl")
                .and_then(|v| v.clone().into_string().ok())
                .and_then(|v| Color::from_str(v.as_str()).ok())
            {
                self.colors.darkmode_colors.results_hl = results_hl;
            }
            if let Some(results_selection_fg) = darkmode_config
                .get("results_selection_fg")
                .and_then(|v| v.clone().into_string().ok())
                .and_then(|v| Color::from_str(v.as_str()).ok())
            {
                self.colors.darkmode_colors.results_selection_fg = results_selection_fg;
            }
            if let Some(results_selection_bg) = darkmode_config
                .get("results_selection_bg")
                .and_then(|v| v.clone().into_string().ok())
                .and_then(|v| Color::from_str(v.as_str()).ok())
            {
                self.colors.darkmode_colors.results_selection_bg = results_selection_bg;
            }
            if let Some(results_selection_hl) = darkmode_config
                .get("results_selection_hl")
                .and_then(|v| v.clone().into_string().ok())
                .and_then(|v| Color::from_str(v.as_str()).ok())
            {
                self.colors.darkmode_colors.results_selection_hl = results_selection_hl;
            }
        }

        let lightmode_config = color_config
            .and_then(|v| v.clone().into_table().ok())
            .and_then(|v| v.get("lightmode").and_then(|v| v.clone().into_table().ok()));

        if let Some(lightmode_config) = lightmode_config {
            if let Some(prompt) = lightmode_config
                .get("prompt")
                .and_then(|v| v.clone().into_string().ok())
                .and_then(|v| Color::from_str(v.as_str()).ok())
            {
                self.colors.lightmode_colors.prompt = prompt;
            }
            if let Some(timing) = lightmode_config
                .get("timing")
                .and_then(|v| v.clone().into_string().ok())
                .and_then(|v| Color::from_str(v.as_str()).ok())
            {
                self.colors.lightmode_colors.timing = timing;
            }
            if let Some(results_fg) = lightmode_config
                .get("results_fg")
                .and_then(|v| v.clone().into_string().ok())
                .and_then(|v| Color::from_str(v.as_str()).ok())
            {
                self.colors.lightmode_colors.results_fg = results_fg;
            }
            if let Some(results_bg) = lightmode_config
                .get("results_bg")
                .and_then(|v| v.clone().into_string().ok())
                .and_then(|v| Color::from_str(v.as_str()).ok())
            {
                self.colors.lightmode_colors.results_bg = results_bg;
            }
            if let Some(results_hl) = lightmode_config
                .get("results_hl")
                .and_then(|v| v.clone().into_string().ok())
                .and_then(|v| Color::from_str(v.as_str()).ok())
            {
                self.colors.lightmode_colors.results_hl = results_hl;
            }
            if let Some(results_selection_fg) = lightmode_config
                .get("results_selection_fg")
                .and_then(|v| v.clone().into_string().ok())
                .and_then(|v| Color::from_str(v.as_str()).ok())
            {
                self.colors.lightmode_colors.results_selection_fg = results_selection_fg;
            }
            if let Some(results_selection_bg) = lightmode_config
                .get("results_selection_bg")
                .and_then(|v| v.clone().into_string().ok())
                .and_then(|v| Color::from_str(v.as_str()).ok())
            {
                self.colors.lightmode_colors.results_selection_bg = results_selection_bg;
            }
            if let Some(results_selection_hl) = lightmode_config
                .get("results_selection_hl")
                .and_then(|v| v.clone().into_string().ok())
                .and_then(|v| Color::from_str(v.as_str()).ok())
            {
                self.colors.lightmode_colors.results_selection_hl = results_selection_hl;
            }
        }
    }

    // Use ~/.mcfly only if it already exists, otherwise create 'mcfly' folder in XDG_CACHE_DIR
    #[must_use]
    pub fn mcfly_training_cache_path() -> PathBuf {
        let cache_dir = Settings::mcfly_xdg_dir().cache_dir().to_path_buf();

        Settings::mcfly_base_path(cache_dir).join(PathBuf::from("training-cache.v1.csv"))
    }

    // Use ~/.mcfly only if it already exists, otherwise create 'mcfly' folder in XDG_DATA_DIR
    #[must_use]
    pub fn mcfly_db_path() -> PathBuf {
        let data_dir = Settings::mcfly_xdg_dir().data_dir().to_path_buf();
        if data_dir.exists() {
            return Settings::mcfly_base_path(data_dir).join(PathBuf::from("history.db"));
        };

        let data_local_dir = Settings::mcfly_xdg_dir().data_local_dir().to_path_buf();
        Settings::mcfly_base_path(data_local_dir).join(PathBuf::from("history.db"))
    }

    // Use ~/.mcfly only if it already exists, otherwise create 'mcfly' folder in XDG_DATA_DIR
    #[must_use]
    pub fn mcfly_config_path() -> PathBuf {
        let data_dir = Settings::mcfly_xdg_dir().data_dir().to_path_buf();

        Settings::mcfly_base_path(data_dir).join(PathBuf::from("config.toml"))
    }

    fn mcfly_xdg_dir() -> ProjectDirs {
        ProjectDirs::from("", "", "McFly").unwrap()
    }

    fn mcfly_base_path(base_dir: PathBuf) -> PathBuf {
        Settings::mcfly_dir_in_home().unwrap_or(base_dir)
    }

    fn mcfly_dir_in_home() -> Option<PathBuf> {
        let user_dirs_file = UserDirs::new()
            .unwrap()
            .home_dir()
            .join(PathBuf::from(".mcfly"));

        user_dirs_file.exists().then_some(user_dirs_file)
    }
}

#[cfg(not(windows))]
#[must_use]
pub fn pwd() -> String {
    env::var("PWD")
        .unwrap_or_else(|err| panic!("McFly error: Unable to determine current directory ({err})"))
}

#[cfg(windows)]
pub fn pwd() -> String {
    env::current_dir()
        .unwrap_or_else(|err| {
            panic!(
                "McFly error: Unable to determine current directory ({})",
                err
            )
        })
        .display()
        .to_string()
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

impl TimeRange {
    /// Determine the range is full (`..`)
    #[inline]
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.since.is_none() && self.before.is_none()
    }
}
