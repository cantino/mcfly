#![cfg(unix)]

use rusqlite::Connection;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

struct TestDirectory(PathBuf);

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

// Cross-compiled test binaries run under an emulator configured through
// `CARGO_TARGET_<TRIPLE>_RUNNER`. Cargo only applies it to the test binary,
// so spawn the mcfly binary through the same runner when one is set.
fn mcfly_command() -> Command {
    let bin = env!("CARGO_BIN_EXE_mcfly");
    let runner = env::vars().find_map(|(key, value)| {
        (key.starts_with("CARGO_TARGET_") && key.ends_with("_RUNNER") && !value.trim().is_empty())
            .then_some(value)
    });
    match runner {
        Some(runner) => {
            let mut parts = runner.split_whitespace();
            let mut command = Command::new(parts.next().unwrap());
            command.args(parts).arg(bin);
            command
        }
        None => Command::new(bin),
    }
}

#[test]
fn add_records_history_with_non_utf8_pwd() {
    let command = "echo hello";
    let root = env::temp_dir().join(format!("mcfly-add-{:016x}", rand::random::<u64>()));
    fs::create_dir(&root).unwrap();
    let root = TestDirectory(root);

    let pwd = root.0.join(OsStr::from_bytes(b"working-dir-\xff"));
    let data_dir = root.0.join(".mcfly");
    fs::create_dir(&data_dir).unwrap();
    let history = root.0.join("bash_history");
    fs::write(&history, "").unwrap();

    let output = mcfly_command()
        .current_dir(&root.0)
        .env_clear()
        .env("HOME", &root.0)
        .env("PWD", pwd)
        .env("HISTFILE", &history)
        .env("MCFLY_HISTORY", &history)
        .env("MCFLY_SESSION_ID", "test")
        .args(["add", "--exit", "0", command])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let db = Connection::open(data_dir.join("history.db")).unwrap();
    let recorded_dir: String = db
        .query_row(
            "SELECT dir FROM commands WHERE cmd = ?1",
            [command],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        recorded_dir,
        root.0.join("working-dir-\u{fffd}").to_string_lossy()
    );
}
