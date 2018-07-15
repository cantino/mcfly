extern crate bash_wizard;

use bash_wizard::history::History;
use bash_wizard::settings::Settings;

fn main() {
    let settings = Settings::parse_args();

    let history = History::load();

    println!("{}", settings.cmd);

    for history_entry in &history.entries {
        if history_entry.matches(&settings.cmd) {
            println!("{}", history_entry.cmd);
        }
    }
}
