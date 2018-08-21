use std::env;
use std::path::PathBuf;

use rusqlite::Connection;
use std::fs;
use bash_history;
use std::fmt;
use std::time::Instant;

#[derive(Debug)]
pub struct Command {
    pub id: i64,
    pub cmd: String,
    pub rank: f64,
    pub when: Option<i64>,
    pub exit_code: Option<i32>,
    pub dir: Option<String>
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
    pub connection: Connection
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
               when: &Option<i64>,
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
                    &when.to_owned(),
                    &exit_code.to_owned(),
                    &dir.to_owned(),
                    &old_dir.to_owned()
                ]).expect("Insert to work");
        }
    }

    pub fn find_matches(&self, cmd: &String) -> Vec<Command> {
        let mut like_query = "%".to_string();
        like_query.push_str(cmd);
        like_query.push_str("%");

        let query = "SELECT
                             id, cmd, when_run, exit_code, dir,
                                 age * -0.01 +
                                 exit_code * -1000.0 +
                                 dir_match * 1000.0 +
                                 overlap * 100.0 +
                                 occurrences * 1.0
                             AS rank
                           FROM contextual_commands
                           WHERE cmd
                           LIKE (?)
                           ORDER BY rank DESC LIMIT ?";
        let mut statement = self.connection.prepare(query).unwrap();
        let command_iter = statement.query_map(&[&like_query, &10], |row| {
            Command {
                id: row.get(0),
                cmd: row.get(1),
                when: row.get(2),
                exit_code: row.get(3),
                dir: row.get(4),
                rank: row.get(5)
            }
        }).expect("Query Map to work");

        let mut names = Vec::new();
        for command in command_iter {
            names.push(command.expect("Unable to load command from DB"));
        }

        names
    }

    fn last_command(&self) -> Option<Command> {
        let query = "SELECT id, cmd, when_run, exit_code, dir, 0 FROM commands ORDER BY id DESC LIMIT 1";
        let mut statement = self.connection.prepare(query).unwrap();
        let command_iter = statement.query_map(&[], |row| {
            Command {
                id: row.get(0),
                cmd: row.get(1),
                when: row.get(2),
                exit_code: row.get(3),
                dir: row.get(4),
                rank: row.get(5)
            }
        }).expect("Query Map to work");

        if let Some(Ok(last)) = command_iter.last() {
            Some(last)
        } else {
            None
        }
    }

    pub fn build_cache_table(&self) {
        let dir = env::current_dir().expect("Unable to determine current directory").to_string_lossy().into_owned();
        let lookback: u16 = 5;
        let now = Instant::now();

        let mut last_commands = self.last_command_strings(lookback);
        while last_commands.len() < lookback as usize {
            last_commands.push(String::from(""));
        }

        self.connection.execute(
            "CREATE TEMP TABLE contextual_commands AS SELECT
                  id,
                  cmd,
                  when_run,
                  count(*) as occurrences,
                  (strftime('%s', 'now') - COALESCE(when_run, 0)) AS age,
                  COALESCE(exit_code, 0) AS exit_code,
                  dir,
                  (CASE WHEN dir = ? THEN 1.0 ELSE 0.0 END) AS dir_match,
                  (SELECT count(DISTINCT c2.cmd) FROM commands c2 WHERE c2.id > c.id - ? AND c2.id < c.id AND c2.cmd IN (?, ?, ?, ?, ?)) AS overlap
                  FROM commands c GROUP BY cmd ORDER BY id DESC; CREATE INDEX temp.MyIndex ON contextual_commands(id);",
            &[
                &dir.to_owned(),
                &lookback.to_owned(),
                &last_commands[0].to_owned(),
                &last_commands[1].to_owned(),
                &last_commands[2].to_owned(),
                &last_commands[3].to_owned(),
                &last_commands[4].to_owned()
            ]).expect("Creation of temp table to work");

        let elapsed = now.elapsed();
        let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
        println!("Seconds: {}", sec);
    }

    pub fn last_command_strings(&self, num: u16) -> Vec<String> {
        let query = "SELECT cmd FROM commands GROUP BY cmd ORDER BY id DESC LIMIT ?";
        let mut statement = self.connection.prepare(query).unwrap();
        let command_iter = statement
            .query_map(&[&num], |row| row.get(0))
            .expect("Query Map to work");

        let mut vec: Vec<String> = Vec::new();
        for result in command_iter {
            if let Ok(string) = result {
                vec.push(string);
            }
        }

        vec
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
                      when_run INTEGER, \
                      exit_code INTEGER, \
                      dir TEXT, \
                      old_dir TEXT \
                  ); \
                  CREATE INDEX command_cmds ON commands (cmd); \
                  CREATE INDEX command_dirs ON commands (dir);"
        ).expect("Unable to initialize history db");

        {
            let mut statement = connection
                .prepare("INSERT INTO commands (cmd) VALUES (?)")
                .expect("Unable to prepare insert");
            for command in &bash_history {
                statement.execute(&[command]).expect("Insert to work");
            }
        }

        History { connection }
    }

    fn from_db_path(path: PathBuf) -> History {
        let connection = Connection::open(path)
            .expect("Unable to open history database");
        History { connection }
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
