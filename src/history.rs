use std::fs::File;
use std::io::Read;
use std::env;
use std::path::PathBuf;

use rusqlite::Connection;
use std::fs;

#[derive(Debug)]
pub struct Command {
    pub id: i64,
    pub cmd: String,
    pub rank: f64
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
            History::from_bash_history_path(History::bash_history_file_path())
        }
    }

    pub fn from_bash_history_path(path: PathBuf) -> History {
        // Load this first to make sure it works before we create the DB.
        let bash_history = History::bash_history(path);

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
                      time INTEGER, \
                      exit_code INTEGER, \
                      pwd TEXT, \
                      old_pwd TEXT \
                  ); \
                  CREATE INDEX command_cmds ON commands (cmd); \
                  CREATE INDEX command_pwds ON commands (pwd);"
        ).expect("Unable to initialize history db");

        {
            let mut statement = connection
                .prepare("INSERT INTO commands (cmd) VALUES (?)")
                .expect("Unable to prepare insert");
            for command in &bash_history {
                statement.execute(&[command]);
            }
        }

        History { connection }
    }

    pub fn from_db_path(path: PathBuf) -> History {
        let connection = Connection::open(path)
            .expect("Unable to open history database");
        History { connection }
    }

    pub fn find_matches(&self, cmd: &String) -> Vec<Command> {
        let mut like_query = "%".to_string();
        like_query.push_str(cmd);
        like_query.push_str("%");

        let query = "SELECT \
                             id, \
                             cmd, \
                             (strftime('%s','now') - COALESCE(time, 0)) * 0.001 AS rank \
                           FROM commands \
                           WHERE cmd \
                           LIKE (?) \
                           ORDER BY rank ASC";
        let mut statement = self.connection.prepare(query).unwrap();
        let command_iter = statement.query_map(&[&like_query], |row| {
            Command {
                id: row.get(0),
                cmd: row.get(1),
                rank: row.get(2)
            }
        }).unwrap();

        let mut names = Vec::new();
        for command in command_iter {
            names.push(command.expect("Unable to load command from DB"));
        }

        names
    }

    fn bash_history_file_path() -> PathBuf {
        env::home_dir()
            .expect("Unable to access home directory")
            .join(PathBuf::from(".bash_history"))
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


    fn bash_history(path: PathBuf) -> Vec<String> {
        let mut f: File = File::open(&path)
            .expect(format!("{:?} file not found", &path).as_str());

        let mut bash_history_contents = String::new();
        f.read_to_string(&mut bash_history_contents)
            .expect(format!("Unable to read {:?}", &path).as_str());

        bash_history_contents
            .split("\n")
            .filter(|line| !line.starts_with('#'))
            .map(String::from)
            .collect::<Vec<String>>()
    }
}
