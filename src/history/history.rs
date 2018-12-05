use bash_history;
use rusqlite::{Connection, MappedRows, Row, NO_PARAMS};
use std::fmt;
use std::fs;
use std::io;
use std::io::Write;
use std::path::PathBuf;
//use std::time::Instant;
use history::db_extensions;
use history::schema;
use simplified_command::SimplifiedCommand;
use path_update_helpers;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use network::Network;
use settings::Settings;
use rusqlite::types::ToSql;

#[derive(Debug, Clone, Default)]
pub struct Features {
    pub age_factor: f64,
    pub length_factor: f64,
    pub exit_factor: f64,
    pub recent_failure_factor: f64,
    pub selected_dir_factor: f64,
    pub dir_factor: f64,
    pub overlap_factor: f64,
    pub immediate_overlap_factor: f64,
    pub selected_occurrences_factor: f64,
    pub occurrences_factor: f64,
}

#[derive(Debug, Clone, Default)]
pub struct Command {
    pub id: i64,
    pub cmd: String,
    pub cmd_tpl: String,
    pub session_id: String,
    pub rank: f64,
    pub when_run: Option<i64>,
    pub exit_code: Option<i32>,
    pub selected: bool,
    pub dir: Option<String>,
    pub features: Features,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.cmd.fmt(f)
    }
}

impl From<Command> for String {
    fn from(command: Command) -> Self {
        command.cmd
    }
}

#[derive(Debug)]
pub struct History {
    pub connection: Connection,
    pub network: Network,
}

const IGNORED_COMMANDS: [&str; 7] = ["pwd", "ls", "cd", "cd ..", "clear", "history", "mcfly search"];

impl History {
    pub fn load() -> History {
        let db_path = Settings::mcfly_db_path();
        let history = if db_path.exists() {
            History::from_db_path(db_path)
        } else {
            History::from_bash_history()
        };
        schema::migrate(&history.connection);
        history
    }

    pub fn should_add(&self, command: &String) -> bool {
        // Ignore empty commands.
        if command.is_empty() {
            return false;
        }

        // Ignore commands added via a ctrl-r search.
        if command.starts_with("#mcfly:") {
            return false;
        }

        // Ignore commands with a leading space.
        if command.starts_with(' ') {
            return false;
        }

        // Ignore blacklisted commands.
        if IGNORED_COMMANDS.contains(&command.as_str()) {
            return false;
        }

        // Ignore the previous command (independent of Session ID) so that opening a new terminal
        // window won't replay the last command in the history.
        let last_command = self.last_command(&None);
        if last_command.is_none() {
            return true;
        }
        !command.eq(&last_command.unwrap().cmd)
    }

    pub fn add(
        &self,
        command: &String,
        session_id: &String,
        dir: &String,
        when_run: &Option<i64>,
        exit_code: &Option<i32>,
        old_dir: &Option<String>,
    ) {
        self.possibly_update_paths(command, exit_code);
        let selected = self.determine_if_selected_from_ui(command, session_id, dir);
        let simplified_command = SimplifiedCommand::new(command.as_str(), true);
        self.connection.execute_named("INSERT INTO commands (cmd, cmd_tpl, session_id, when_run, exit_code, selected, dir, old_dir) VALUES (:cmd, :cmd_tpl, :session_id, :when_run, :exit_code, :selected, :dir, :old_dir)",
                                      &[
                                          (":cmd", &command.to_owned()),
                                          (":cmd_tpl", &simplified_command.result.to_owned()),
                                          (":session_id", &session_id.to_owned()),
                                          (":when_run", &when_run.to_owned()),
                                          (":exit_code", &exit_code.to_owned()),
                                          (":selected", &selected),
                                          (":dir", &dir.to_owned()),
                                          (":old_dir", &old_dir.to_owned()),
                                      ]).expect("Insert into commands to work");
    }

    fn determine_if_selected_from_ui(
        &self,
        command: &String,
        session_id: &String,
        dir: &String,
    ) -> bool {
        let rows_affected = self.connection
            .execute_named(
                "DELETE FROM selected_commands \
                 WHERE cmd = :cmd \
                 AND session_id = :session_id \
                 AND dir = :dir",
                &[
                    (":cmd", &command.to_owned()),
                    (":session_id", &session_id.to_owned()),
                    (":dir", &dir.to_owned()),
                ],
            )
            .expect("DELETE from selected_commands to work");

        // Delete any other pending selected commands for this session -- they must have been aborted or edited.
        self.connection
            .execute_named(
                "DELETE FROM selected_commands WHERE session_id = :session_id",
                &[(":session_id", &session_id.to_owned())],
            )
            .expect("DELETE from selected_commands to work");

        rows_affected > 0
    }

    pub fn record_selected_from_ui(&self, command: &String, session_id: &String, dir: &String) {
        self.connection.execute_named("INSERT INTO selected_commands (cmd, session_id, dir) VALUES (:cmd, :session_id, :dir)",
                                      &[
                                          (":cmd", &command.to_owned()),
                                          (":session_id", &session_id.to_owned()),
                                          (":dir", &dir.to_owned())
                                      ]).expect("Insert into selected_commands to work");
    }

    // Update historical paths in our database if a directory has been renamed or moved.
    pub fn possibly_update_paths(&self, command: &String, exit_code: &Option<i32>) {
        if exit_code.is_none() || exit_code.unwrap() == 0 {
            if command.to_lowercase().starts_with("mv ") && !command.contains("*") && !command.contains("?") {
                let parts = path_update_helpers::parse_mv_command(command);
                if parts.len() == 2 {
                    let normalized_from = path_update_helpers::normalize_path(&parts[0]);
                    let normalized_to = path_update_helpers::normalize_path(&parts[1]);

                    // If $to/$(base_name($from)) exists, and is a directory, assume we've moved $from into $to.
                    // If not, assume we've renamed $from to $to.

                    if let Some(basename) = PathBuf::from(&normalized_from).file_name() {
                        if let Some(utf8_basename) = basename.to_str() {
                            if utf8_basename.contains(".") {
                                // It was probably a file.
                                return;
                            }
                            let maybe_moved_directory = PathBuf::from(&normalized_to).join(utf8_basename);
                            if maybe_moved_directory.exists() {
                                if maybe_moved_directory.is_dir() {
                                    self.update_paths(&normalized_from, maybe_moved_directory.to_str().unwrap(), false);
                                } else {
                                    // The source must have been a file, so ignore it.
                                }
                                return;
                            }
                        } else {
                            // Don't try to handle non-utf8 filenames, at least for now.
                            return;
                        }
                    }

                    let to_pathbuf = PathBuf::from(&normalized_to);
                    if to_pathbuf.exists() && to_pathbuf.is_dir() {
                        self.update_paths(&normalized_from, &normalized_to, false);
                    }
                }
            }
        }
    }

    pub fn find_matches(&self, cmd: &String, num: i16) -> Vec<Command> {
        let mut like_query = "%".to_string();
        like_query.push_str(cmd);
        like_query.push_str("%");

        let query = "SELECT id, cmd, cmd_tpl, session_id, when_run, exit_code, selected, dir, rank,
                                  age_factor, length_factor, exit_factor, recent_failure_factor,
                                  selected_dir_factor, dir_factor, overlap_factor, immediate_overlap_factor,
                                  selected_occurrences_factor, occurrences_factor
                           FROM contextual_commands
                           WHERE cmd LIKE (:like)
                           ORDER BY rank DESC LIMIT :limit";
        let mut statement = self.connection.prepare(query).expect("Prepare to work");
        let command_iter = statement
            .query_map_named(
                &[(":like", &like_query), (":limit", &num)],
                |row| Command {
                    id: row.get_checked(0).expect("id to be readable"),
                    cmd: row.get_checked(1).expect("cmd to be readable"),
                    cmd_tpl: row.get_checked(2).expect("cmd_tpl to be readable"),
                    session_id: row.get_checked(3).expect("session_id to be readable"),
                    when_run: row.get_checked(4).expect("when_run to be readable"),
                    exit_code: row.get_checked(5).expect("exit_code to be readable"),
                    selected: row.get_checked(6).expect("selected to be readable"),
                    dir: row.get_checked(7).expect("dir to be readable"),
                    rank: row.get_checked(8).expect("rank to be readable"),
                    features: Features {
                        age_factor: row.get_checked(9).expect("age_factor to be readable"),
                        length_factor: row.get_checked(10).expect("length_factor to be readable"),
                        exit_factor: row.get_checked(11).expect("exit_factor to be readable"),
                        recent_failure_factor: row.get_checked(12)
                            .expect("recent_failure_factor to be readable"),
                        selected_dir_factor: row.get_checked(13)
                            .expect("selected_dir_factor to be readable"),
                        dir_factor: row.get_checked(14).expect("dir_factor to be readable"),
                        overlap_factor: row.get_checked(15).expect("overlap_factor to be readable"),
                        immediate_overlap_factor: row.get_checked(16)
                            .expect("immediate_overlap_factor to be readable"),
                        selected_occurrences_factor: row.get_checked(17)
                            .expect("selected_occurrences_factor to be readable"),
                        occurrences_factor: row.get_checked(18)
                            .expect("occurrences_factor to be readable"),
                    },
                },
            )
            .expect("Query Map to work");

        let mut names = Vec::new();
        for command in command_iter {
            names.push(command.expect("Unable to load command from DB"));
        }

        names
    }

    pub fn build_cache_table(
        &self,
        dir: &String,
        session_id: &Option<String>,
        start_time: Option<i64>,
        end_time: Option<i64>,
        now: Option<i64>,
    ) {
        let lookback: u16 = 3;

        let mut last_commands = self.last_command_templates(session_id, lookback as i16, 0);
        if last_commands.len() < lookback as usize {
            last_commands = self.last_command_templates(&None, lookback as i16, 0);
            while last_commands.len() < lookback as usize {
                last_commands.push(String::from(""));
            }
        }

        self.connection
            .execute("DROP TABLE IF EXISTS temp.contextual_commands;", NO_PARAMS)
            .expect("Removal of temp table to work");

        let (mut when_run_min, when_run_max): (f64, f64) = self.connection
            .query_row(
                "SELECT MIN(when_run), MAX(when_run) FROM commands",
                NO_PARAMS,
                |row| (row.get(0), row.get(1)),
            )
            .expect("Query to work");

        if when_run_min == when_run_max {
            when_run_min -= 60.0 * 60.0;
        }

        let max_occurrences: f64 = self.connection
            .query_row(
                "SELECT COUNT(*) AS c FROM commands GROUP BY cmd ORDER BY c DESC LIMIT 1",
                NO_PARAMS,
                |row| row.get(0),
            )
            .unwrap_or(1.0);

        let max_selected_occurrences: f64 = self.connection
            .query_row("SELECT COUNT(*) AS c FROM commands WHERE selected = 1 GROUP BY cmd ORDER BY c DESC LIMIT 1", NO_PARAMS,
                       |row| row.get(0)).unwrap_or(1.0);

        let max_length: f64 = self.connection
            .query_row("SELECT MAX(LENGTH(cmd)) FROM commands", NO_PARAMS, |row| {
                row.get(0)
            })
            .unwrap_or(100.0);

        #[allow(unused_variables)]
        let beginning_of_execution = Instant::now();
        self.connection.execute_named(
            "CREATE TEMP TABLE contextual_commands AS SELECT
                  id, cmd, cmd_tpl, session_id, when_run, exit_code, selected, dir,

                  /* to be filled in later */
                  0.0 AS rank,

                  /* length of the command string */
                  LENGTH(c.cmd) / :max_length AS length_factor,

                  /* age of the last execution of this command (0.0 is new, 1.0 is old) */
                  MIN((:when_run_max - when_run) / :history_duration) AS age_factor,

                  /* average error state (1: always successful, 0: always errors) */
                  SUM(CASE WHEN exit_code = 0 THEN 1.0 ELSE 0.0 END) / COUNT(*) as exit_factor,

                  /* recent failure (1 if failed recently, 0 if not) */
                  MAX(CASE WHEN exit_code != 0 AND :now - when_run < 120 THEN 1.0 ELSE 0.0 END) AS recent_failure_factor,

                  /* percentage run in this directory (1: always run in this directory, 0: never run in this directory) */
                  SUM(CASE WHEN dir = :directory THEN 1.0 ELSE 0.0 END) / COUNT(*) as dir_factor,

                  /* percentage of time selected in this directory (1: only selected in this dir, 0: only selected elsewhere) */
                  SUM(CASE WHEN dir = :directory AND selected = 1 THEN 1.0 ELSE 0.0 END) / (SUM(CASE WHEN selected = 1 THEN 1.0 ELSE 0.0 END) + 1) as selected_dir_factor,

                  /* average contextual overlap of this command (0: none of the last 3 commands has ever overlapped with this command, 1: all of the last three commands always overlap with this command) */
                  SUM((
                    SELECT COUNT(DISTINCT c2.cmd_tpl) FROM commands c2
                    WHERE c2.id >= c.id - :lookback AND c2.id < c.id AND c2.cmd_tpl IN (:last_commands0, :last_commands1, :last_commands2)
                  ) / :lookback_f64) / COUNT(*) AS overlap_factor,

                  /* average overlap with the last command (0: this command never follows the last command, 1: this command always follows the last command) */
                  SUM((SELECT COUNT(*) FROM commands c2 WHERE c2.id = c.id - 1 AND c2.cmd_tpl = :last_commands0)) / COUNT(*) AS immediate_overlap_factor,

                  /* percentage selected (1: this is the most commonly selected command, 0: this command is never selected) */
                  SUM(CASE WHEN selected = 1 THEN 1.0 ELSE 0.0 END) / :max_selected_occurrences AS selected_occurrences_factor,

                  /* percentage of time this command is run relative to the most common command (1: this is the most common command, 0: this is the least common command) */
                  COUNT(*) / :max_occurrences AS occurrences_factor

                  FROM commands c WHERE when_run > :start_time AND when_run < :end_time GROUP BY cmd ORDER BY id DESC;",
            &[
                (":when_run_max", &when_run_max),
                (":history_duration", &(when_run_max - when_run_min)),
                (":directory", &dir.to_owned()),
                (":max_occurrences", &max_occurrences),
                (":max_length", &max_length),
                (":max_selected_occurrences", &max_selected_occurrences),
                (":lookback", &lookback),
                (":lookback_f64", &(lookback as f64)),
                (":last_commands0", &last_commands[0].to_owned()),
                (":last_commands1", &last_commands[1].to_owned()),
                (":last_commands2", &last_commands[2].to_owned()),
                (":start_time", &start_time.unwrap_or(0).to_owned()),
                (":end_time", &end_time.unwrap_or(SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as i64).to_owned()),
                (":now", &now.unwrap_or(SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as i64).to_owned())
            ]).expect("Creation of temp table to work");

        self.connection
            .execute(
                "UPDATE contextual_commands
                 SET rank = nn_rank(age_factor, length_factor, exit_factor,
                                    recent_failure_factor, selected_dir_factor, dir_factor,
                                    overlap_factor, immediate_overlap_factor,
                                    selected_occurrences_factor, occurrences_factor);",
                NO_PARAMS,
            )
            .expect("Ranking of temp table to work");

        self.connection
            .execute(
                "CREATE INDEX temp.MyIndex ON contextual_commands(id);",
                NO_PARAMS,
            )
            .expect("Creation of index on temp table to work");

        // println!("Seconds: {}", (beginning_of_execution.elapsed().as_secs() as f64) + (beginning_of_execution.elapsed().subsec_nanos() as f64 / 1000_000_000.0));
    }

    pub fn commands(&self, session_id: &Option<String>, num: i16, offset: u16, random: bool) -> Vec<Command> {
        let order = if random { "RANDOM()" } else { "id" };
        let query = if session_id.is_none() {
            format!("SELECT id, cmd, cmd_tpl, session_id, when_run, exit_code, selected, dir FROM commands ORDER BY {} DESC LIMIT :limit OFFSET :offset", order)
        } else {
            format!("SELECT id, cmd, cmd_tpl, session_id, when_run, exit_code, selected, dir FROM commands WHERE session_id = :session_id ORDER BY {} DESC LIMIT :limit OFFSET :offset", order)
        };

        if session_id.is_none() {
            self.run_query(&query, &[
                (":limit", &num),
                (":offset", &offset),
            ])
        } else {
            self.run_query(&query, &[
                (":session_id", &session_id.to_owned().unwrap()),
                (":limit", &num),
                (":offset", &offset),
            ])
        }
    }

    fn run_query(&self, query: &str, params: &[(&str, &ToSql)]) -> Vec<Command> {
        let mut statement = self.connection.prepare(query).unwrap();

        let closure: fn(&Row) -> Command = |row| Command {
            id: row.get(0),
            cmd: row.get(1),
            cmd_tpl: row.get(2),
            session_id: row.get(3),
            when_run: row.get(4),
            exit_code: row.get(5),
            selected: row.get(6),
            dir: row.get(7),
            ..Command::default()
        };

        let command_iter: MappedRows<_> = statement.
            query_map_named(params, closure).
            expect("Query Map to work");

        let mut vec = Vec::new();
        for result in command_iter {
            if let Ok(command) = result {
                vec.push(command);
            }
        }

        vec
    }

    pub fn last_command(&self, session_id: &Option<String>) -> Option<Command> {
        self.commands(session_id, 1, 0, false)
            .get(0)
            .map(|cmd| cmd.clone())
    }

    pub fn last_command_templates(
        &self,
        session_id: &Option<String>,
        num: i16,
        offset: u16,
    ) -> Vec<String> {
        self.commands(session_id, num, offset, false)
            .iter()
            .map(|command| command.cmd_tpl.to_owned())
            .collect()
    }

    pub fn delete_command(&self, command: &str) {
        self.connection
            .execute_named(
                "DELETE FROM selected_commands WHERE cmd = :command",
                &[(":command", &command)],
            )
            .expect("DELETE from selected_commands to work");

        self.connection
            .execute_named(
                "DELETE FROM commands WHERE cmd = :command",
                &[(":command", &command)],
            )
            .expect("DELETE from commands to work");
    }

    pub fn update_paths(&self, old_path: &str, new_path: &str, print_output: bool) {
        let normalized_old_path = path_update_helpers::normalize_path(old_path);
        let normalized_new_path = path_update_helpers::normalize_path(new_path);

        if normalized_old_path.len() > 1 && normalized_new_path.len() > 1 {
            let like_query = normalized_old_path.to_string() + "/%";

            let mut dir_update_statement = self.connection.prepare(
                "UPDATE commands SET dir = :new_dir || SUBSTR(dir, :length) WHERE dir = :exact OR dir LIKE (:like)"
            ).unwrap();

            let mut old_dir_update_statement = self.connection.prepare(
                "UPDATE commands SET old_dir = :new_dir || SUBSTR(old_dir, :length) WHERE old_dir = :exact OR old_dir LIKE (:like)"
            ).unwrap();

            let affected = dir_update_statement.execute_named(&[
                (":like", &like_query),
                (":exact", &normalized_old_path),
                (":new_dir", &normalized_new_path),
                (":length", &(normalized_old_path.chars().count() as u32 + 1)),
            ]).expect("dir UPDATE to work");

            old_dir_update_statement.execute_named(&[
                (":like", &like_query),
                (":exact", &normalized_old_path),
                (":new_dir", &normalized_new_path),
                (":length", &(normalized_old_path.chars().count() as u32 + 1)),
            ]).expect("old_dir UPDATE to work");

            if print_output {
                println!("McFly: Command database paths renamed from {} to {} (affected {} commands)", normalized_old_path, normalized_new_path, affected);
            }
        } else {
            if print_output {
                println!("McFly: Not updating paths due to invalid options.");
            }
        }
    }

    fn from_bash_history() -> History {
        print!(
            "McFly: Importing Bash history for the first time. This may take a minute or two..."
        );
        io::stdout().flush().expect("STDOUT flush should work");

        // Load this first to make sure it works before we create the DB.
        let bash_history = bash_history::full_history(&bash_history::bash_history_file_path());

        // Make ~/.mcfly
        fs::create_dir_all(Settings::storage_dir_path())
            .expect(format!("Unable to create {:?}", Settings::storage_dir_path()).as_str());

        // Make ~/.mcfly/history.db
        let connection = Connection::open(Settings::mcfly_db_path()).expect(
            format!(
                "Unable to create history DB at {:?}",
                Settings::mcfly_db_path()
            ).as_str(),
        );
        db_extensions::add_db_functions(&connection);

        connection.execute_batch(
            "CREATE TABLE commands( \
                      id INTEGER PRIMARY KEY AUTOINCREMENT, \
                      cmd TEXT NOT NULL, \
                      cmd_tpl TEXT, \
                      session_id TEXT NOT NULL, \
                      when_run INTEGER NOT NULL, \
                      exit_code INTEGER NOT NULL, \
                      selected INTEGER NOT NULL, \
                      dir TEXT, \
                      old_dir TEXT \
                  ); \
                  CREATE INDEX command_cmds ON commands (cmd);\
                  CREATE INDEX command_session_id ON commands (session_id);\
                  CREATE INDEX command_dirs ON commands (dir);\
                  \
                  CREATE TABLE selected_commands( \
                      id INTEGER PRIMARY KEY AUTOINCREMENT, \
                      cmd TEXT NOT NULL, \
                      session_id TEXT NOT NULL, \
                      dir TEXT NOT NULL \
                  ); \
                  CREATE INDEX selected_command_session_cmds ON selected_commands (session_id, cmd);"
        ).expect("Unable to initialize history db");

        {
            let mut statement = connection
                .prepare("INSERT INTO commands (cmd, cmd_tpl, session_id, when_run, exit_code, selected) VALUES (:cmd, :cmd_tpl, :session_id, :when_run, :exit_code, :selected)")
                .expect("Unable to prepare insert");
            let epoch = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as i64;
            for command in &bash_history {
                if !IGNORED_COMMANDS.contains(&command.as_str()) {
                    let simplified_command = SimplifiedCommand::new(command.as_str(), true);
                    statement
                        .execute_named(&[
                            (":cmd", command),
                            (":cmd_tpl", &simplified_command.result.to_owned()),
                            (":session_id", &"IMPORTED"),
                            (":when_run", &epoch),
                            (":exit_code", &0),
                            (":selected", &0),
                        ])
                        .expect("Insert to work");
                }
            }
        }

        schema::first_time_setup(&connection);

        println!("done.");

        History {
            connection,
            network: Network::default(),
        }
    }

    fn from_db_path(path: PathBuf) -> History {
        let connection = Connection::open(path).expect("Unable to open history database");
        db_extensions::add_db_functions(&connection);
        History {
            connection,
            network: Network::default(),
        }
    }
}
