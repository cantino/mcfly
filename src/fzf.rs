use std::io::Write;

use crate::history::History;
use crate::settings::{self, ResultSort};

pub fn run(history: &History, sort_by: &ResultSort, zero_separated: bool) {
    let order_by_column: &str = match sort_by {
        settings::ResultSort::LastRun => "last_run DESC",
        _ => "rank DESC",
    };

    let query: &str = &format!(
        "{} {} {}",
        "SELECT cmd, last_run 
        FROM contextual_commands",
        "ORDER BY",
        order_by_column
    )[..];

    let mut statement = history
        .connection
        .prepare(query)
        .unwrap_or_else(|err| panic!("McFly error: Prepare to work ({})", err));
    let mut rows = statement
        .query([])
        .unwrap_or_else(|err| panic!("McFly error: Query Map to work ({})", err));

    let mut stdout = std::io::stdout();

    while let Ok(Some(row)) = rows.next() {
        let cmd: String = row.get(0).unwrap_or_else(|err| {
            panic!(
                "Mcfly error: unable to read database result column 'cmd': {}",
                err
            )
        });
        let last_run: i64 = row.get(1).unwrap_or_else(|err| {
            panic!(
                "Mcfly error: unable to read database result column 'last_run': {}",
                err
            )
        });

        let duration = crate::interface::format_time_since(last_run);
        let res = if zero_separated {
            stdout.write_fmt(format_args!("{}\t{}\0", duration, cmd))
        } else {
            stdout.write_fmt(format_args!("{}\t{}\n", duration, cmd))
        };
        if res.is_err() {
            break;
        }
    }
}
