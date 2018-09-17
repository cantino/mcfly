use simplified_command::SimplifiedCommand;
use rusqlite::Connection;
use std::io;
use std::io::Write;

pub const CURRENT_SCHEMA_VERSION: u16 = 2;

pub fn first_time_setup(connection: &Connection) {
    make_schema_versions_table(connection);
    write_current_schema_version(connection);
}

pub fn migrate(connection: &Connection) {
    make_schema_versions_table(connection);

    let current_version: u16 = connection
        .query_row::<Option<u16>, _>("select max(version) FROM schema_versions ORDER BY version DESC LIMIT 1", &[],
                   |row| row.get(0)).expect("Query to work").unwrap_or(0);

    if current_version < CURRENT_SCHEMA_VERSION {
        print!("McFly: Upgrading McFly DB to version {}, please wait...", CURRENT_SCHEMA_VERSION);
        io::stdout().flush().expect("STDOUT flush should work");
    }

    if current_version < 1 {
        connection.execute_batch("ALTER TABLE commands ADD COLUMN cmd_tpl TEXT; UPDATE commands SET cmd_tpl = '';")
            .expect("Unable to add cmd_tpl to commands");

        let mut statement = connection
            .prepare("UPDATE commands SET cmd_tpl = ? WHERE id = ?")
            .expect("Unable to prepare update");

        for (id, cmd) in cmd_strings(connection) {
            let simplified_command = SimplifiedCommand::new(cmd.as_str(), true);
            statement.execute(&[&simplified_command.result, &id]).expect("Insert to work");
        }

    }

    if current_version < 2 {
        connection.execute_batch(
            "ALTER TABLE commands ADD COLUMN session_id TEXT; \
            UPDATE commands SET session_id = 'UNKNOWN'; \
            CREATE INDEX command_session_id ON commands (session_id);")
            .expect("Unable to add session_id to commands");
    }

    if current_version < CURRENT_SCHEMA_VERSION {
        println!("done.");
        write_current_schema_version(connection);
    }
}

fn make_schema_versions_table(connection: &Connection) {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_versions( \
                id INTEGER PRIMARY KEY AUTOINCREMENT, \
                version INTEGER NOT NULL, \
                when_run INTEGER NOT NULL);

             CREATE UNIQUE INDEX IF NOT EXISTS schema_versions_index ON schema_versions (version);"
    ).expect("Unable to create schema_versions db table");
}

fn write_current_schema_version(connection: &Connection) {
    let insert = format!("INSERT INTO schema_versions (version, when_run) VALUES ({}, strftime('%s','now'))", CURRENT_SCHEMA_VERSION);
    connection.execute_batch(&insert).expect("Unable to update schema_versions");
}

fn cmd_strings(connection: &Connection) -> Vec<(i64, String)> {
    let query = "SELECT id, cmd FROM commands ORDER BY id DESC";
    let mut statement = connection.prepare(query).unwrap();
    let command_iter = statement.query_map(&[], |row| {
        (row.get(0), row.get(1))
    }).expect("Query Map to work");

    let mut vec = Vec::new();
    for result in command_iter {
        if let Ok(command) = result {
            vec.push(command);
        }
    }

    vec
}
