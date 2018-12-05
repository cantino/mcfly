extern crate mcfly;

use mcfly::bash_history;
use mcfly::fake_typer;
use mcfly::history::History;
use mcfly::interface::Interface;
use mcfly::settings::Mode;
use mcfly::settings::Settings;
use mcfly::trainer::Trainer;
use std::path::PathBuf;
use std::env;

fn handle_addition(settings: &Settings, history: &mut History) {
    if history.should_add(&settings.command) {
        history.add(
            &settings.command,
            &settings.session_id,
            &settings.dir,
            &settings.when_run,
            &settings.exit_code,
            &settings.old_dir,
        );

        if settings.append_to_histfile {
            let histfile = PathBuf::from(env::var("HISTFILE")
                .expect("Please ensure that HISTFILE is set."));
            bash_history::append_history_entry(&settings.command, &histfile)
        }
    }
}

fn handle_search(settings: &Settings, history: &History) {
    let result = Interface::new(settings, history).display();
    if let Some(cmd) = result.selection {
        fake_typer::use_tiocsti(&cmd);

        if result.run {
            fake_typer::use_tiocsti(&"\n".to_string());
        }
    }
}

fn handle_train(settings: &Settings, history: &mut History) {
    Trainer::new(settings, history).train();
}

fn handle_move(settings: &Settings, history: &mut History) {
    history.update_paths(&settings.old_dir.clone().unwrap(), &settings.dir, true);
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
        Mode::Move => {
            handle_move(&settings, &mut history);
        }
    }
}
