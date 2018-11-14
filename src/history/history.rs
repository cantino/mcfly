use std::path::PathBuf;
use dirs::home_dir;
use rusqlite::Connection;
use std::fs;
use bash_history;
use std::fmt;
use std::io;
use std::io::Write;
//use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use weights::Weights;

use history::schema;
use simplified_command::SimplifiedCommand;
use rusqlite::Row;
use rusqlite::MappedRows;

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
    pub age_factor: f64,
    pub length_factor: f64,
    pub exit_factor: f64,
    pub recent_failure_factor: f64,
    pub selected_dir_factor: f64,
    pub dir_factor: f64,
    pub overlap_factor: f64,
    pub immediate_overlap_factor: f64,
    pub selected_occurrences_factor: f64,
    pub occurrences_factor: f64
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
    pub weights: Weights
}

const IGNORED_COMMANDS: [&str; 7] = ["pwd", "ls", "cd", "cd ..", "clear", "history", "mcfly search"];

impl History {
    pub fn load() -> History {
        let db_path = History::mcfly_db_path();
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

    pub fn add(&self,
               command: &String,
               session_id: &String,
               dir: &String,
               when_run: &Option<i64>,
               exit_code: &Option<i32>,
               old_dir: &Option<String>) {
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

    fn determine_if_selected_from_ui(&self, command: &String, session_id: &String, dir: &String) -> bool {
        let rows_affected = self.connection
            .execute_named(
                "DELETE FROM selected_commands \
                WHERE cmd = :cmd \
                AND session_id = :session_id \
                AND dir = :dir",
                &[
                    (":cmd", &command.to_owned()),
                    (":session_id", &session_id.to_owned()),
                    (":dir", &dir.to_owned())
                ]).expect("DELETE from selected_commands to work");

        // Delete any other pending selected commands for this session -- they must have been aborted or edited.
        self.connection
            .execute_named("DELETE FROM selected_commands WHERE session_id = :session_id",
                &[(":session_id", &session_id.to_owned())]).expect("DELETE from selected_commands to work");

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

    pub fn find_matches(&self, cmd: &String, num: Option<u16>) -> Vec<Command> {
        let mut like_query = "%".to_string();
        like_query.push_str(cmd);
        like_query.push_str("%");

        let query = "SELECT id, cmd, cmd_tpl, session_id, when_run, exit_code, selected, dir, rank,
                                  age_factor, length_factor, exit_factor, recent_failure_factor,
                                  selected_dir_factor, dir_factor, overlap_factor, immediate_overlap_factor,
                                  selected_occurrences_factor, occurrences_factor
                           FROM contextual_commands
                           WHERE cmd LIKE (?)
                           ORDER BY rank DESC LIMIT ?";
        let mut statement = self.connection.prepare(query).expect("Prepare to work");
        let command_iter = statement.query_map(
            &[&like_query, &num.unwrap_or(10)],
            |row| {
                Command {
                    id: row.get_checked(0).expect("id to be readable"),
                    cmd: row.get_checked(1).expect("cmd to be readable"),
                    cmd_tpl: row.get_checked(2).expect("cmd_tpl to be readable"),
                    session_id: row.get_checked(3).expect("session_id to be readable"),
                    when_run: row.get_checked(4).expect("when_run to be readable"),
                    exit_code: row.get_checked(5).expect("exit_code to be readable"),
                    selected: row.get_checked(6).expect("selected to be readable"),
                    dir: row.get_checked(7).expect("dir to be readable"),
                    rank: row.get_checked(8).expect("rank to be readable"),
                    age_factor: row.get_checked(9).expect("age_factor to be readable"),
                    length_factor: row.get_checked(10).expect("length_factor to be readable"),
                    exit_factor: row.get_checked(11).expect("exit_factor to be readable"),
                    recent_failure_factor: row.get_checked(12).expect("recent_failure_factor to be readable"),
                    selected_dir_factor: row.get_checked(13).expect("selected_dir_factor to be readable"),
                    dir_factor: row.get_checked(14).expect("dir_factor to be readable"),
                    overlap_factor: row.get_checked(15).expect("overlap_factor to be readable"),
                    immediate_overlap_factor: row.get_checked(16).expect("immediate_overlap_factor to be readable"),
                    selected_occurrences_factor: row.get_checked(17).expect("selected_occurrences_factor to be readable"),
                    occurrences_factor: row.get_checked(18).expect("occurrences_factor to be readable"),
                }
            }).expect("Query Map to work");

        let mut names = Vec::new();
        for command in command_iter {
            names.push(command.expect("Unable to load command from DB"));
        }

        names
    }

    pub fn build_cache_table(&self, dir: &String, session_id: &Option<String>, start_time: Option<i64>, end_time: Option<i64>, now: Option<i64>) {
        let lookback: u16 = 3;
//        let now = Instant::now();

        let mut last_commands = self.last_command_templates(session_id, lookback as i16, 0);
        if last_commands.len() < lookback as usize {
            last_commands = self.last_command_templates(&None, lookback as i16, 0);
            while last_commands.len() < lookback as usize {
                last_commands.push(String::from(""));
            }
        }

        self.connection.execute("DROP TABLE IF EXISTS temp.contextual_commands;", &[])
            .expect("Removal of temp table to work");

        let (mut when_run_min, when_run_max): (f64, f64) = self.connection
            .query_row("SELECT MIN(when_run), MAX(when_run) FROM commands", &[],
                       |row| (row.get(0), row.get(1))).expect("Query to work");

        if when_run_min == when_run_max {
            when_run_min -= 60.0 * 60.0;
        }

        let max_occurrences: f64 = self.connection
            .query_row("SELECT COUNT(*) AS c FROM commands GROUP BY cmd ORDER BY c DESC LIMIT 1", &[],
                       |row| row.get(0)).unwrap_or(1.0);

        let max_selected_occurrences: f64 = self.connection
            .query_row("SELECT COUNT(*) AS c FROM commands WHERE selected = 1 GROUP BY cmd ORDER BY c DESC LIMIT 1", &[],
                       |row| row.get(0)).unwrap_or(1.0);

        let max_length: f64 = self.connection
            .query_row("SELECT MAX(LENGTH(cmd)) FROM commands", &[],
                       |row| row.get(0)).unwrap_or(100.0);

        // What I want:
        // # of times this command has been seen / max # of times any single command has been seen
        // # of times this command has been selected / # of times this command has been run
        // # of times this command has been selected / max # of times any command has been selected
        // # of times run in this directory / # of times run anywhere
        // # of times selected in this dir / # of times selected anywhere
        // # of times selected anywhere / # of times run anywhere
        // sum of the overlap scores for this command in the history relative to the last three commands / # times in the history
        // # of times the command before this one shows up before this one in the past / # of times this command has been run
        // Could choose to let the network normalize these, but I think that's hard for it?

        // What I have:
        self.connection.execute_named(
            "CREATE TEMP TABLE contextual_commands AS SELECT
                  id, cmd, cmd_tpl, session_id, when_run, exit_code, selected, dir,

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
                  COUNT(*) / :max_occurrences AS occurrences_factor,

                  /* linear function with weights */
                  :offset +
                  LENGTH(c.cmd) / :max_length * :length_weight +
                  MIN((:when_run_max - when_run) / :history_duration) * :age_weight +
                  SUM(CASE WHEN exit_code = 0 THEN 1.0 ELSE 0.0 END) / COUNT(*) * :exit_weight +
                  MAX(CASE WHEN exit_code != 0 AND :now - when_run < 120 THEN 1.0 ELSE 0.0 END) * :recent_failure_weight +
                  SUM(CASE WHEN dir = :directory THEN 1.0 ELSE 0.0 END) / COUNT(*) * :dir_weight +
                  SUM(CASE WHEN dir = :directory AND selected = 1 THEN 1.0 ELSE 0.0 END) / (SUM(CASE WHEN selected = 1 THEN 1.0 ELSE 0.0 END) + 1) * :selected_dir_weight +
                  SUM((
                    SELECT COUNT(DISTINCT c2.cmd_tpl) FROM commands c2
                    WHERE c2.id >= c.id - :lookback AND c2.id < c.id AND c2.cmd_tpl IN (:last_commands0, :last_commands1, :last_commands2)
                  ) / :lookback_f64) / COUNT(*) * :overlap_weight +
                  SUM((SELECT COUNT(*) FROM commands c2 WHERE c2.id = c.id - 1 AND c2.cmd_tpl = :last_commands0)) / COUNT(*) * :immediate_overlap_weight +
                  SUM(CASE WHEN selected = 1 THEN 1.0 ELSE 0.0 END) / :max_selected_occurrences * :selected_occurrences_weight +
                  COUNT(*) / :max_occurrences * :occurrences_weight
                  AS rank

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
                (":offset", &self.weights.offset),
                (":overlap_weight", &self.weights.overlap),
                (":immediate_overlap_weight", &self.weights.immediate_overlap),
                (":age_weight", &self.weights.age),
                (":length_weight", &self.weights.length),
                (":exit_weight", &self.weights.exit),
                (":occurrences_weight", &self.weights.occurrences),
                (":selected_occurrences_weight", &self.weights.selected_occurrences),
                (":recent_failure_weight", &self.weights.recent_failure),
                (":dir_weight", &self.weights.dir),
                (":selected_dir_weight", &self.weights.selected_dir),
                (":start_time", &start_time.unwrap_or(0).to_owned()),
                (":end_time", &end_time.unwrap_or(SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as i64).to_owned()),
                (":now", &now.unwrap_or(SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as i64).to_owned())
            ]).expect("Creation of temp table to work");

        self.connection.execute("CREATE INDEX temp.MyIndex ON contextual_commands(id);", &[])
            .expect("Creation of index on temp table to work");

//        let elapsed = now.elapsed();
//        let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
//        println!("Seconds: {}", sec);
    }

    pub fn commands(&self, session_id: &Option<String>, num: i16, offset: u16, random: bool) -> Vec<Command> {
        let order = if random { "RANDOM()" } else { "id" };
        let query = if session_id.is_none() {
            format!("SELECT id, cmd, cmd_tpl, session_id, when_run, exit_code, selected, dir FROM commands ORDER BY {} DESC LIMIT ? OFFSET ?", order)
        } else {
            format!("SELECT id, cmd, cmd_tpl, session_id, when_run, exit_code, selected, dir FROM commands WHERE session_id = ? ORDER BY {} DESC LIMIT ? OFFSET ?", order)
        };

        let mut statement = self.connection.prepare(&query).unwrap();

        let closure: fn(&Row) -> Command = |row| {
            Command {
                id: row.get(0),
                cmd: row.get(1),
                cmd_tpl: row.get(2),
                session_id: row.get(3),
                when_run: row.get(4),
                exit_code: row.get(5),
                selected: row.get(6),
                dir: row.get(7),
                ..Command::default()
            }
        };

        let command_iter: MappedRows<_> = if session_id.is_none() {
            statement.query_map(&[&num, &offset], closure).expect("Query Map to work")
        } else {
            statement.query_map(&[&session_id.to_owned().unwrap(), &num, &offset], closure).expect("Query Map to work")
        };

        let mut vec = Vec::new();
        for result in command_iter {
            if let Ok(command) = result {
                vec.push(command);
            }
        }

        vec
    }

    pub fn last_command(&self, session_id: &Option<String>) -> Option<Command> {
        self.commands(session_id, 1, 0, false).get(0).map(|cmd| cmd.clone())
    }

    pub fn last_command_templates(&self, session_id: &Option<String>, num: i16, offset: u16) -> Vec<String> {
        self.commands(session_id, num, offset, false).iter().map(|command| command.cmd_tpl.to_owned()).collect()
    }

    pub fn delete_command(&self, command: &str) {
        self.connection
            .execute_named("DELETE FROM selected_commands WHERE cmd = :command",
                           &[(":command", &command)])
            .expect("DELETE from selected_commands to work");

        self.connection
            .execute_named("DELETE FROM commands WHERE cmd = :command",
                           &[(":command", &command)])
            .expect("DELETE from commands to work");
    }

    fn from_bash_history() -> History {
        print!("McFly: Importing Bash history for the first time. One moment...");
        io::stdout().flush().expect("STDOUT flush should work");

        // Load this first to make sure it works before we create the DB.
        let bash_history = bash_history::full_history(&bash_history::bash_history_file_path());

        // Make ~/.mcfly
        fs::create_dir_all(History::storage_dir_path())
            .expect(format!("Unable to create {:?}", History::storage_dir_path()).as_str());

        // Make ~/.mcfly/history.db
        let connection = Connection::open(History::mcfly_db_path())
            .expect(format!("Unable to create history DB at {:?}", History::mcfly_db_path()).as_str());

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
                .prepare("INSERT INTO commands (cmd, cmd_tpl, session_id, when_run, exit_code, selected) VALUES (?, ?, ?, ?, ?, ?)")
                .expect("Unable to prepare insert");
            let epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as i64;
            for command in &bash_history {
                if !IGNORED_COMMANDS.contains(&command.as_str()) {
                    let simplified_command = SimplifiedCommand::new(command.as_str(), true);
                    statement.execute(&[command, &simplified_command.result.to_owned(), &"IMPORTED", &epoch, &0, &0]).expect("Insert to work");
                }
            }
        }

        schema::first_time_setup(&connection);

        println!("done.");

        History { connection, weights: Weights::default() }
    }

    fn from_db_path(path: PathBuf) -> History {
        let connection = Connection::open(path)
            .expect("Unable to open history database");
        History { connection, weights: Weights::default() }
    }

    fn storage_dir_path() -> PathBuf {
        home_dir()
            .expect("Unable to access home directory")
            .join(PathBuf::from(".mcfly"))
    }

    fn mcfly_db_path() -> PathBuf {
        History::storage_dir_path()
            .join(PathBuf::from("history.db"))
    }
}
