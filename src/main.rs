use mcfly::fake_typer;
use mcfly::history::History;
use mcfly::interface::Interface;
use mcfly::settings::Mode;
use mcfly::settings::Settings;
use mcfly::shell_history;
use mcfly::trainer::Trainer;
use std::{env, fs};
use std::path::PathBuf;

fn handle_addition(settings: &Settings, history: &mut History) {
    if history.should_add(&settings.command) {
        history.add(
            &settings.command,
            &settings.session_id,
            &settings.dir,
            &settings.when_run,
            settings.exit_code,
            &settings.old_dir,
        );

        if settings.append_to_histfile {
            let histfile = PathBuf::from(env::var("HISTFILE").unwrap_or_else(|err| {
                panic!(format!(
                    "McFly error: Please ensure that HISTFILE is set ({})",
                    err
                ))
            }));
            shell_history::append_history_entry(
                &settings.command,
                settings.when_run,
                &histfile,
                settings.zsh_extended_history,
                settings.debug,
            )
        }
    }
}

fn handle_search(settings: &Settings, history: &History) {
    let result = Interface::new(settings, history).display();
    if let Some(cmd) = result.selection {
        if let Some(path) = &settings.output_selection {
            // Output selection to a file, with the first line indicating if the user chose to run the selection or not.
            let mut out: String = String::new();

            if result.run {
                out.push_str("run\n");
            } else {
                out.push_str("display\n");
            }

            out.push_str(&cmd);

            fs::write(path, &out)
                .unwrap_or_else(|err| panic!(format!("McFly error: unable to write to {}: {}", path, err)));
        } else {
            fake_typer::use_tiocsti(&cmd);

            if result.run {
                fake_typer::use_tiocsti(&"\n".to_string());
            }
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
