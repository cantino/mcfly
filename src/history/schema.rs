use history::History;
use simplified_command::SimplifiedCommand;
use rusqlite::Connection;
use std::io;
use std::io::Write;

pub const CURRENT_SCHEMA_VERSION: u16 = 1;

pub fn first_time_setup(connection: &Connection) {
    make_schema_versions_table(connection);
    write_current_schema_version(connection);
}

pub fn make_schema_versions_table(connection: &Connection) {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_versions( \
                id INTEGER PRIMARY KEY AUTOINCREMENT, \
                version INTEGER NOT NULL, \
                when_run INTEGER NOT NULL);

             CREATE UNIQUE INDEX IF NOT EXISTS schema_versions_index ON schema_versions (version);"
    ).expect("Unable to create schema_versions db table");
}

pub fn write_current_schema_version(connection: &Connection) {
    let insert = format!("INSERT INTO schema_versions (version, when_run) VALUES ({}, strftime('%s','now'))", CURRENT_SCHEMA_VERSION);
    connection.execute_batch(&insert).expect("Unable to update schema_versions");
}

pub fn migrate(history: &History) {
    make_schema_versions_table(&history.connection);

    let current_version: u16 = history.connection
        .query_row::<Option<u16>, _>("select max(version) FROM schema_versions ORDER BY version DESC LIMIT 1", &[],
                   |row| row.get(0)).expect("Query to work").unwrap_or(0);

    if current_version < CURRENT_SCHEMA_VERSION {
        if current_version == 0 {
            print!("McFly: Upgrading McFly DB to version {}, please wait...", CURRENT_SCHEMA_VERSION);
            io::stdout().flush().expect("STDOUT flush should work");

            history.connection.execute_batch("ALTER TABLE commands ADD COLUMN cmd_tpl TEXT; UPDATE commands SET cmd_tpl = '';")
                .expect("Unable to add cmd_tpl to commands");

            let mut statement = history.connection
                .prepare("UPDATE commands SET cmd_tpl = ? WHERE id = ?")
                .expect("Unable to prepare update");

            for command in &history.commands(-1, 0) {
                let simplified_command = SimplifiedCommand::new(command.cmd.as_str(), true);
                statement.execute(&[&simplified_command.result, &command.id]).expect("Insert to work");
            }

            println!("done.");
        }

        write_current_schema_version(&history.connection);
    }
}
