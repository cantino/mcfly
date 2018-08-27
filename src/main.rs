extern crate bash_wizard;

use bash_wizard::interface::Interface;
use bash_wizard::history::History;
use bash_wizard::settings::Settings;
use bash_wizard::settings::Mode;
use bash_wizard::trainer::Trainer;
use bash_wizard::exporter::Exporter;
use bash_wizard::fake_typer;

fn handle_addition(settings: &Settings, history: &mut History) {
    if !settings.command.starts_with('#') { // Ignore commented lines
        history.add(&settings.command, &settings.when_run, &settings.exit_code, &settings.dir, &settings.old_dir);
    }
}

fn handle_search(settings: &Settings, history: &History) {
    history.build_cache_table(None, None, None);
    let command = Interface::new(settings, history).select();
    if command.len() > 0 && !command.is_empty() {
        fake_typer::use_tiocsti(&command);
        fake_typer::use_tiocsti(&"\n".to_string());
    }
}

fn handle_train(settings: &Settings, history: &mut History) {
    Trainer::new(settings, history).train();
}

fn handle_export(settings: &Settings, history: &mut History) {
    Exporter::new(settings, history).export();
}

fn main() {
    let settings = Settings::parse_args();

    let mut history = History::load();

    match settings.mode {
        Mode::Add => {
            handle_addition(&settings, &mut history);
        },
        Mode::Search => {
            handle_search(&settings, &history);
        },
        Mode::Train => {
            handle_train(&settings, &mut history);
        },
        Mode::Export => {
            handle_export(&settings, &mut history);
        }
    }
}
