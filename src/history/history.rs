use std::env;
use std::path::PathBuf;

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

#[derive(Debug, Clone, Default)]
pub struct Command {
    pub id: i64,
    pub cmd: String,
    pub cmd_tpl: String,
    pub rank: f64,
    pub when_run: Option<i64>,
    pub exit_code: Option<i32>,
    pub dir: Option<String>,
    pub age_factor: f64,
    pub exit_factor: f64,
    pub recent_failure_factor: f64,
    pub dir_factor: f64,
    pub overlap_factor: f64,
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
        let mut history = if db_path.exists() {
            History::from_db_path(db_path)
        } else {
            History::from_bash_history()
        };
        schema::migrate(&mut history);
        history
    }

    pub fn add(&self,
               command: &String,
               when_run: &Option<i64>,
               exit_code: &Option<i32>,
               dir: &Option<String>,
               old_dir: &Option<String>) {
        if match self.last_command() {
            None => true,
            Some(ref last_command) if !command.eq(&last_command.cmd) => true,
            Some(_) => false
        } {
            if !IGNORED_COMMANDS.contains(&command.as_str()) {
                let simplified_command = SimplifiedCommand::new(command.as_str(), true);
                self.connection.execute("INSERT INTO commands (cmd, cmd_tpl, when_run, exit_code, dir, old_dir) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                                        &[
                                            &command.to_owned(),
                                            &simplified_command.result.to_owned(),
                                            &when_run.to_owned(),
                                            &exit_code.to_owned(),
                                            &dir.to_owned(),
                                            &old_dir.to_owned()
                                        ]).expect("Insert to work");
            }
        }
    }

    pub fn find_matches(&self, cmd: &String, num: Option<u16>) -> Vec<Command> {
        let mut like_query = "%".to_string();
        like_query.push_str(cmd);
        like_query.push_str("%");

        let query = "SELECT id, cmd, cmd_tpl, when_run, exit_code, dir, rank,
                                  age_factor, exit_factor, recent_failure_factor, dir_factor, overlap_factor, occurrences_factor
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
                    when_run: row.get_checked(3).expect("when_run to be readable"),
                    exit_code: row.get_checked(4).expect("exit_code to be readable"),
                    dir: row.get_checked(5).expect("dir to be readable"),
                    age_factor: row.get_checked(7).expect("age_factor to be readable"),
                    exit_factor: row.get_checked(8).expect("exit_factor to be readable"),
                    recent_failure_factor: row.get_checked(9).expect("recent_failure_factor to be readable"),
                    dir_factor: row.get_checked(10).expect("dir_factor to be readable"),
                    overlap_factor: row.get_checked(11).expect("overlap_factor to be readable"),
                    occurrences_factor: row.get_checked(12).expect("occurrences_factor to be readable"),
                    rank: row.get_checked(6).expect("rank to be readable"),
                }
            }).expect("Query Map to work");

        let mut names = Vec::new();
        for command in command_iter {
            names.push(command.expect("Unable to load command from DB"));
        }

        names
    }

    pub fn build_cache_table(&self, dir: Option<String>, start_time: Option<i64>, end_time: Option<i64>) {
        let lookback: u16 = 3;
//        let now = Instant::now();

        let mut last_commands = self.last_command_templates(lookback as i16, 0);
        while last_commands.len() < lookback as usize {
            last_commands.push(String::from(""));
        }

        let directory = dir.unwrap_or(
            env::current_dir()
                .expect("Unable to determine current directory")
                .to_string_lossy()
                .into_owned()
            );

        self.connection.execute("DROP TABLE IF EXISTS temp.contextual_commands;", &[])
            .expect("Removal of temp table to work");

        let (mut when_run_min, when_run_max): (f64, f64) = self.connection
            .query_row("SELECT MIN(when_run), MAX(when_run) FROM commands", &[],
                       |row| (row.get(0), row.get(1))).expect("Query to work");

        if when_run_min == when_run_max {
            when_run_min -= 60.0 * 60.0;
        }

        let max_occurrences: f64 = self.connection
            .query_row("select count(*) as c FROM commands GROUP BY cmd order by c desc limit 1", &[],
                       |row| row.get(0)).expect("Query to work");

        // For every unique command in the history, insert a single row into the temporary
        // contextual_commands table.
        // What we really want is "how often does a command that looks like this get run in this directory or in this context?"
        // What we have now is "how often does this exact command get run in this directory or in this context?"

        ///         "INSERT INTO test (name) VALUES (:name)",
        ///         &[(":name", &"one")],


        self.connection.execute_named(
            "CREATE TEMP TABLE contextual_commands AS SELECT
                  id, cmd, cmd_tpl, when_run, exit_code, dir,

                  MIN((:when_run_max - when_run) / :when_run_spread) AS age_factor,

                  SUM(CASE WHEN exit_code = 0 THEN 1.0 ELSE 0.0 END) / COUNT(*) as exit_factor,

                  MAX(CASE WHEN exit_code = 1 AND strftime('%s','now') - when_run < 120 THEN 1.0 ELSE 0.0 END) AS recent_failure_factor,

                  SUM(CASE WHEN dir = :directory THEN 1.0 ELSE 0.0 END) / :max_occurrences as dir_factor,

                  SUM((
                    SELECT count(DISTINCT c2.cmd_tpl) FROM commands c2
                    WHERE c2.id >= c.id - :lookback AND c2.id < c.id AND c2.cmd_tpl IN (:last_commands0, :last_commands1, :last_commands2)
                  ) / :lookback_f64) / :max_occurrences AS overlap_factor,

                  COUNT(*) / :max_occurrences AS occurrences_factor,

                  :offset +
                  MIN((:when_run_max - when_run) / :when_run_spread) * :age_weight +
                  SUM(CASE WHEN exit_code = 0 THEN 1.0 ELSE 0.0 END) / COUNT(*) * :exit_weight +
                  MAX(CASE WHEN exit_code = 1 AND strftime('%s','now') - when_run < 120 THEN 1.0 ELSE 0.0 END) * :recent_failure_weight +
                  SUM(CASE WHEN dir = :directory THEN 1.0 ELSE 0.0 END) / :max_occurrences * :dir_weight +
                  SUM((
                    SELECT count(DISTINCT c2.cmd_tpl) FROM commands c2
                    WHERE c2.id >= c.id - :lookback AND c2.id < c.id AND c2.cmd_tpl IN (:last_commands0, :last_commands1, :last_commands2)
                  ) / :lookback_f64) / :max_occurrences * :overlap_weight +
                  COUNT(*) / :max_occurrences * :occurrences_weight
                  AS rank

                  FROM commands c WHERE when_run > :start_time AND when_run < :end_time GROUP BY cmd ORDER BY id DESC LIMIT -1 OFFSET 1;",
            &[
                (":when_run_max", &when_run_max),
                (":when_run_spread", &(when_run_max - when_run_min)),
                (":directory", &directory),
                (":max_occurrences", &max_occurrences),
                (":lookback", &lookback),
                (":lookback_f64", &(lookback as f64)),
                (":last_commands0", &last_commands[0].to_owned()),
                (":last_commands1", &last_commands[1].to_owned()),
                (":last_commands2", &last_commands[2].to_owned()),
                (":offset", &self.weights.offset),
                (":overlap_weight", &self.weights.overlap),
                (":age_weight", &self.weights.age),
                (":exit_weight", &self.weights.exit),
                (":occurrences_weight", &self.weights.occurrences),
                (":recent_failure_weight", &self.weights.recent_failure),
                (":dir_weight", &self.weights.dir),
                (":start_time", &start_time.unwrap_or(0).to_owned()),
                (":end_time", &end_time.unwrap_or(SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as i64).to_owned())
            ]).expect("Creation of temp table to work");

        self.connection.execute("CREATE INDEX temp.MyIndex ON contextual_commands(id);", &[])
            .expect("Creation of index on temp table to work");

//        let elapsed = now.elapsed();
//        let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
//        println!("Seconds: {}", sec);
    }

    pub fn commands(&self, num: i16, offset: u16) -> Vec<Command> {
        let query = "SELECT id, cmd, cmd_tpl, when_run, exit_code, dir, 0 FROM commands ORDER BY id DESC LIMIT ? OFFSET ?";
        let mut statement = self.connection.prepare(query).unwrap();
        let command_iter = statement.query_map(&[&num, &offset], |row| {
            Command {
                id: row.get(0),
                cmd: row.get(1),
                cmd_tpl: row.get(2),
                when_run: row.get(3),
                exit_code: row.get(4),
                dir: row.get(5),
                rank: row.get(6),
                .. Command::default()
            }
        }).expect("Query Map to work");

        let mut vec = Vec::new();
        for result in command_iter {
            if let Ok(command) = result {
                vec.push(command);
            }
        }

        vec
    }

    pub fn last_command(&self) -> Option<Command> {
        self.commands(1, 0).get(0).map(|cmd| cmd.clone())
    }

    pub fn last_command_templates(&self, num: i16, offset: u16) -> Vec<String> {
        self.commands(num, offset).iter().map(|command| command.cmd_tpl.to_owned()).collect()
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
                      when_run INTEGER NOT NULL, \
                      exit_code INTEGER NOT NULL, \
                      dir TEXT, \
                      old_dir TEXT \
                  ); \
                  CREATE INDEX command_cmds ON commands (cmd); \
                  CREATE INDEX command_dirs ON commands (dir);"
        ).expect("Unable to initialize history db");

        {
            let mut statement = connection
                .prepare("INSERT INTO commands (cmd, cmd_tpl, when_run, exit_code) VALUES (?, ?, ?, ?)")
                .expect("Unable to prepare insert");
            let epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as i64;
            for command in &bash_history {
                if !IGNORED_COMMANDS.contains(&command.as_str()) {
                    let simplified_command = SimplifiedCommand::new(command.as_str(), true);
                    statement.execute(&[command, &simplified_command.result.to_owned(), &epoch, &0]).expect("Insert to work");
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
        env::home_dir()
            .expect("Unable to access home directory")
            .join(PathBuf::from(".mcfly"))
    }

    fn mcfly_db_path() -> PathBuf {
        History::storage_dir_path()
            .join(PathBuf::from("history.db"))
    }
}
