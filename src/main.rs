extern crate mcfly;

use mcfly::exporter::Exporter;
use mcfly::fake_typer;
use mcfly::history::History;
use mcfly::interface::Interface;
use mcfly::settings::Mode;
use mcfly::settings::Settings;
use mcfly::trainer::Trainer;

fn handle_addition(settings: &Settings, history: &mut History) {
    if !settings.command.starts_with('#') {
        // Ignore commented lines
        history.add(
            &settings.command,
            &settings.session_id,
            &settings.dir,
            &settings.when_run,
            &settings.exit_code,
            &settings.old_dir,
        );
    }
}

fn handle_search(settings: &Settings, history: &History) {
    history.build_cache_table(&settings.dir.to_owned(), &Some(settings.session_id.to_owned()), None, None);
    let (command, run) = Interface::new(&settings.command, history, settings.debug).select();
    if command.len() > 0 && !command.is_empty() {
        fake_typer::use_tiocsti(&command);

        if run {
            fake_typer::use_tiocsti(&"\n".to_string());
        }
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
        }
        Mode::Search => {
            handle_search(&settings, &history);
        }
        Mode::Train => {
            handle_train(&settings, &mut history);
        }
        Mode::Export => {
            handle_export(&settings, &mut history);
        }
    }
}
