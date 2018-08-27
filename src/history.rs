use std::env;
use std::path::PathBuf;

use rusqlite::Connection;
use std::fs;
use bash_history;
use std::fmt;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use weights::Weights;

#[derive(Debug, Clone, Default)]
pub struct Command {
    pub id: i64,
    pub cmd: String,
    pub rank: f64,
    pub when_run: Option<i64>,
    pub exit_code: Option<i32>,
    pub dir: Option<String>,
    pub age_factor: f64,
    pub exit_factor: f64,
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

impl History {
    pub fn load() -> History {
        let db_path = History::bash_wizard_db_path();
        if db_path.exists() {
            History::from_db_path(db_path)
        } else {
            History::from_bash_history()
        }
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
            self.connection.execute(
                "INSERT INTO commands (cmd, when_run, exit_code, dir, old_dir) VALUES (?1, ?2, ?3, ?4, ?5)",
                &[
                    &command.to_owned(),
                    &when_run.to_owned(),
                    &exit_code.to_owned(),
                    &dir.to_owned(),
                    &old_dir.to_owned()
                ]).expect("Insert to work");
        }
    }

    pub fn find_matches(&self, cmd: &String, num: Option<u16>) -> Vec<Command> {
        let mut like_query = "%".to_string();
        like_query.push_str(cmd);
        like_query.push_str("%");

        let query = "SELECT id, cmd, when_run, exit_code, dir, rank,
                                  age_factor, exit_factor, dir_factor, overlap_factor, occurrences_factor
                           FROM contextual_commands
                           WHERE cmd LIKE (?)
                           ORDER BY rank DESC LIMIT ?";
        let mut statement = self.connection.prepare(query).unwrap();
        let command_iter = statement.query_map(
            &[&like_query, &num.unwrap_or(10)],
            |row| {
                Command {
                    id: row.get(0),
                    cmd: row.get(1),
                    when_run: row.get(2),
                    exit_code: row.get(3),
                    dir: row.get(4),
                    rank: row.get(5),
                    age_factor: row.get(6),
                    exit_factor: row.get(7),
                    dir_factor: row.get(8),
                    overlap_factor: row.get(9),
                    occurrences_factor: row.get(10)
                }
            }).expect("Query Map to work");

        let mut names = Vec::new();
        for command in command_iter {
            names.push(command.expect("Unable to load command from DB"));
        }

        names
    }

    pub fn build_cache_table(&self, dir: Option<String>, start_time: Option<i64>, end_time: Option<i64>) {
        let lookback: u16 = 5;
        let now = Instant::now();

        let mut last_commands = self.last_command_strings(lookback);
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

        let (when_run_min, when_run_max): (f64, f64) = self.connection
            .query_row("SELECT MIN(when_run), MAX(when_run) FROM commands", &[],
                       |row| (row.get(0), row.get(1))).expect("Query to work");

        let max_occurrences: f64 = self.connection
            .query_row("select count(*) as c FROM commands GROUP BY cmd order by c desc limit 1", &[],
                       |row| row.get(0)).expect("Query to work");

        self.connection.execute(
            "CREATE TEMP TABLE contextual_commands AS SELECT
                  id, cmd, when_run, exit_code, dir,

                  MIN((? - when_run) / ?) AS age_factor,
                  MIN(CASE WHEN exit_code = 0 THEN 1.0 ELSE 0.0 END) AS exit_factor,
                  MAX(CASE WHEN dir = ? THEN 1.0 ELSE 0.0 END) AS dir_factor,
                  MAX((
                    SELECT count(DISTINCT c2.cmd) FROM commands c2
                    WHERE c2.id > c.id - ? AND c2.id < c.id AND c2.cmd IN (?, ?, ?, ?, ?)
                  ) / ?) AS overlap_factor,
                  count(*) / ? AS occurrences_factor,

                      ? +
                      MIN((? - when_run) / ?) * ? +
                      MIN(CASE WHEN exit_code = 0 THEN 1.0 ELSE 0.0 END) * ? +
                      MAX(CASE WHEN dir = ? THEN 1.0 ELSE 0.0 END) * ? +
                      MAX((
                        SELECT count(DISTINCT c2.cmd) FROM commands c2
                        WHERE c2.id > c.id - ? AND c2.id < c.id AND c2.cmd IN (?, ?, ?, ?, ?)
                      ) / ?) * ? +
                      count(*) / ? * ?
                  AS rank

                  FROM commands c WHERE when_run > ? AND when_run < ? GROUP BY cmd ORDER BY id DESC;

                  CREATE INDEX temp.MyIndex ON contextual_commands(id);",
            &[
                &when_run_max,
                &(when_run_max - when_run_min),
                &directory,
                &lookback,
                &last_commands[0].to_owned(),
                &last_commands[1].to_owned(),
                &last_commands[2].to_owned(),
                &last_commands[3].to_owned(),
                &last_commands[4].to_owned(),
                &(lookback as f64),
                &max_occurrences,
                &self.weights.offset,
                &when_run_max,
                &(when_run_max - when_run_min),
                &self.weights.age,
                &self.weights.exit,
                &directory,
                &self.weights.dir,
                &lookback,
                &last_commands[0].to_owned(),
                &last_commands[1].to_owned(),
                &last_commands[2].to_owned(),
                &last_commands[3].to_owned(),
                &last_commands[4].to_owned(),
                &(lookback as f64),
                &self.weights.overlap,
                &max_occurrences,
                &self.weights.occurrences,
                &start_time.unwrap_or(0).to_owned(),
                &end_time.unwrap_or(SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as i64).to_owned()
            ]).expect("Creation of temp table to work");

        let elapsed = now.elapsed();
        let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
        println!("Seconds: {}", sec);
    }

    pub fn commands(&self, num: u16) -> Vec<Command> {
        let query = "SELECT id, cmd, when_run, exit_code, dir, 0 FROM commands ORDER BY id DESC LIMIT ? OFFSET 1";
        let mut statement = self.connection.prepare(query).unwrap();
        let command_iter = statement.query_map(&[&num], |row| {
            Command {
                id: row.get(0),
                cmd: row.get(1),
                when_run: row.get(2),
                exit_code: row.get(3),
                dir: row.get(4),
                rank: row.get(5),
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

    fn last_command(&self) -> Option<Command> {
        self.commands(1).get(0).map(|cmd| cmd.clone())
    }

    pub fn last_command_strings(&self, num: u16) -> Vec<String> {
        self.commands(num).iter().map(|command| command.cmd.to_owned()).collect()
    }

    fn from_bash_history() -> History {
        println!("Importing Bash history for the first time. One moment...");

        // Load this first to make sure it works before we create the DB.
        let bash_history = bash_history::full_history(&bash_history::bash_history_file_path());

        // Make ~/.bash_wizard
        fs::create_dir_all(History::storage_dir_path())
            .expect(format!("Unable to create {:?}", History::storage_dir_path()).as_str());

        // Make ~/.bash_wizard/history.db
        let connection = Connection::open(History::bash_wizard_db_path())
            .expect(format!("Unable to create history DB at {:?}", History::bash_wizard_db_path()).as_str());

        connection.execute_batch(
            "CREATE TABLE commands( \
                      id INTEGER PRIMARY KEY AUTOINCREMENT, \
                      cmd TEXT NOT NULL, \
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
                .prepare("INSERT INTO commands (cmd, when_run, exit_code) VALUES (?, ?, ?)")
                .expect("Unable to prepare insert");
            let epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as i64;
            for command in &bash_history {
                statement.execute(&[command, &epoch, &0]).expect("Insert to work");
            }
        }

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
            .join(PathBuf::from(".bash_wizard"))
    }

    fn bash_wizard_db_path() -> PathBuf {
        History::storage_dir_path()
            .join(PathBuf::from("history.db"))
    }
}
