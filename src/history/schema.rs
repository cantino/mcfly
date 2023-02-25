use crate::simplified_command::SimplifiedCommand;
use rusqlite::{named_params, Connection};
use std::io;
use std::io::Write;

pub const CURRENT_SCHEMA_VERSION: u16 = 3;

pub fn first_time_setup(connection: &Connection) {
    make_schema_versions_table(connection);
    write_current_schema_version(connection);
}

pub fn migrate(connection: &Connection) {
    make_schema_versions_table(connection);

    let current_version: u16 = connection
        .query_row::<Option<u16>, _, _>(
            "select max(version) FROM schema_versions ORDER BY version DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|err| panic!("McFly error: Query to work ({})", err))
        .unwrap_or(0);

    if current_version > CURRENT_SCHEMA_VERSION {
        panic!(
            "McFly error: Database schema version ({}) is newer than the max version supported by this binary ({}). You should update mcfly.",
            current_version,
            CURRENT_SCHEMA_VERSION,
        );
    }

    if current_version < CURRENT_SCHEMA_VERSION {
        print!(
            "McFly: Upgrading McFly DB to version {}, please wait...",
            CURRENT_SCHEMA_VERSION
        );
        io::stdout()
            .flush()
            .unwrap_or_else(|err| panic!("McFly error: STDOUT flush should work ({})", err));
    }

    if current_version < 1 {
        connection
            .execute_batch(
                "ALTER TABLE commands ADD COLUMN cmd_tpl TEXT; UPDATE commands SET cmd_tpl = '';",
            )
            .unwrap_or_else(|err| {
                panic!("McFly error: Unable to add cmd_tpl to commands ({})", err)
            });

        let mut statement = connection
            .prepare("UPDATE commands SET cmd_tpl = :cmd_tpl WHERE id = :id")
            .unwrap_or_else(|err| panic!("McFly error: Unable to prepare update ({})", err));

        for (id, cmd) in cmd_strings(connection) {
            let simplified_command = SimplifiedCommand::new(cmd.as_str(), true);
            statement
                .execute(named_params! { ":cmd_tpl": &simplified_command.result, ":id": &id })
                .unwrap_or_else(|err| panic!("McFly error: Insert to work ({})", err));
        }
    }

    if current_version < 2 {
        connection
            .execute_batch(
                "ALTER TABLE commands ADD COLUMN session_id TEXT; \
                 UPDATE commands SET session_id = 'UNKNOWN'; \
                 CREATE INDEX command_session_id ON commands (session_id);",
            )
            .unwrap_or_else(|err| {
                panic!(
                    "McFly error: Unable to add session_id to commands ({})",
                    err
                )
            });
    }

    if current_version < 3 {
        connection
            .execute_batch(
                "CREATE TABLE selected_commands( \
              id INTEGER PRIMARY KEY AUTOINCREMENT, \
              cmd TEXT NOT NULL, \
              session_id TEXT NOT NULL, \
              dir TEXT NOT NULL \
            ); \
            CREATE INDEX selected_command_session_cmds ON selected_commands (session_id, cmd); \
            \
            ALTER TABLE commands ADD COLUMN selected INTEGER; \
            UPDATE commands SET selected = 0;",
            )
            .unwrap_or_else(|err| panic!("McFly error: Unable to add selected_commands ({})", err));
    }

    if current_version < CURRENT_SCHEMA_VERSION {
        println!("done.");
        write_current_schema_version(connection);
    }
}

fn make_schema_versions_table(connection: &Connection) {
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_versions( \
                id INTEGER PRIMARY KEY AUTOINCREMENT, \
                version INTEGER NOT NULL, \
                when_run INTEGER NOT NULL);

             CREATE UNIQUE INDEX IF NOT EXISTS schema_versions_index ON schema_versions (version);",
        )
        .unwrap_or_else(|err| {
            panic!(
                "McFly error: Unable to create schema_versions db table ({})",
                err
            )
        });
}

fn write_current_schema_version(connection: &Connection) {
    let insert = format!(
        "INSERT INTO schema_versions (version, when_run) VALUES ({}, strftime('%s','now'))",
        CURRENT_SCHEMA_VERSION
    );
    connection
        .execute_batch(&insert)
        .unwrap_or_else(|err| panic!("McFly error: Unable to update schema_versions ({})", err));
}

fn cmd_strings(connection: &Connection) -> Vec<(i64, String)> {
    let query = "SELECT id, cmd FROM commands ORDER BY id DESC";
    let mut statement = connection.prepare(query).unwrap();
    let command_iter = statement
        .query_map([], |row| Ok((row.get_unwrap(0), row.get_unwrap(1))))
        .unwrap_or_else(|err| panic!("McFly error: Query Map to work ({})", err));

    let mut vec = Vec::new();
    for command in command_iter.flatten() {
        vec.push(command);
    }

    vec
}
