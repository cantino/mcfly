use mcfly::dumper::Dumper;
use mcfly::fake_typer;
use mcfly::history::History;
use mcfly::init::Init;
use mcfly::interface::Interface;
use mcfly::settings::Mode;
use mcfly::settings::Settings;
use mcfly::shell_history;
use mcfly::trainer::Trainer;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn handle_addition(settings: &Settings) {
    let history = History::load(settings.history_format);
    if history.should_add(&settings.command) {
        history.add(
            &settings.command,
            &settings.session_id,
            &settings.dir,
            &settings.when_run,
            settings.exit_code,
            &settings.old_dir,
        );

        if settings.append_to_histfile.is_some() {
            let histfile = PathBuf::from(settings.append_to_histfile.as_ref().unwrap());
            let command = shell_history::HistoryCommand::new(
                &settings.command,
                settings.when_run.unwrap_or(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_else(|err| panic!("McFly error: Time went backwards ({})", err))
                        .as_secs() as i64,
                ),
                settings.history_format,
            );
            shell_history::append_history_entry(&command, &histfile, settings.debug)
        }
    }
}

fn handle_search(settings: &Settings) {
    let history = History::load(settings.history_format);
    let result = Interface::new(settings, &history).display();
    if let Some(cmd) = result.selection {
        if let Some(path) = &settings.output_selection {
            // Output selection results to a file.
            let mut out: String = String::new();

            // First we say the desired mode, depending on the key pressed by the user - simply
            // displaying the selected command, or running it.
            if result.run {
                out.push_str("mode run\n");
            } else {
                out.push_str("mode display\n");
            }

            // Next, the desired commandline selected by the user.
            out.push_str("commandline ");
            out.push_str(&cmd);
            out.push('\n');

            // Finally, any requests for deletion of commands from shell history, for cases where
            // shells need to handle this natively instead of through us editing HISTFILE/MCFLY_HISTFILE.
            for delete_request in result.delete_requests {
                out.push_str("delete ");
                out.push_str(&delete_request);
                out.push('\n');
            }

            fs::write(path, &out)
                .unwrap_or_else(|err| panic!("McFly error: unable to write to {}: {}", path, err));
        } else {
            fake_typer::use_tiocsti(&cmd);

            if result.run {
                fake_typer::use_tiocsti("\n");
            }
        }
    }
}

fn handle_train(settings: &Settings) {
    let mut history = History::load(settings.history_format);
    Trainer::new(settings, &mut history).train();
}

fn handle_move(settings: &Settings) {
    let history = History::load(settings.history_format);
    history.update_paths(&settings.old_dir.clone().unwrap(), &settings.dir, true);
}

fn handle_init(settings: &Settings) {
    Init::new(&settings.init_mode);
}

fn handle_dump(settings: &Settings) {
    let history = History::load(settings.history_format);
    Dumper::new(settings, &history).dump();
}

fn main() {
    let settings = Settings::parse_args();

    match settings.mode {
        Mode::Add => {
            handle_addition(&settings);
        }
        Mode::Search => {
            handle_search(&settings);
        }
        Mode::Train => {
            handle_train(&settings);
        }
        Mode::Move => {
            handle_move(&settings);
        }
        Mode::Init => {
            handle_init(&settings);
        }
        Mode::Dump => {
            handle_dump(&settings);
        }
    }
}
