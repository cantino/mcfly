extern crate rand;

use history::History;
use settings::Settings;
use history::Command;
use rand::Rng;

#[derive(Debug)]
pub struct TrainingSampleGenerator<'a> {
    settings: &'a Settings,
    history: &'a History,
}

impl<'a> TrainingSampleGenerator<'a> {
    pub fn new(settings: &'a Settings, history: &'a History) -> TrainingSampleGenerator<'a> {
        TrainingSampleGenerator { settings, history }
    }

    pub fn generate<F>(&self, records: i16, mut handler: F) where F: FnMut(&Command, f64, bool) {
        let data_set = self.history.commands(&None, records, 0, true);

        let mut positive_examples = 0;
        let mut negative_examples = 0;

        for command in data_set.iter() {
            if command.dir.is_none() || command.exit_code.is_none() || command.when_run.is_none() { continue; }
            if command.cmd.is_empty() { continue; }

            // Setup the cache for the time this command was recorded.
            // Unwrap is safe here because we check command.dir.is_none() above.
            self.history.build_cache_table(&command.dir.to_owned().unwrap(), &Some(command.session_id.clone()), None, command.when_run, command.when_run);

            // max_occurrences at time command was recorded.
            let max_occurrences: f64 = self.history.connection
                .query_row("SELECT COUNT(*) AS c FROM commands WHERE when_run < ? GROUP BY cmd ORDER BY c DESC LIMIT 1", &[&command.when_run],
                           |row| row.get(0)).unwrap_or(0.0);
            if max_occurrences == 0.0 { continue; }

            // Get the factors for this command at the time it was logged.
            if positive_examples <= negative_examples {
                let results = self.history.find_matches(&command.cmd, Some(10));
                if let Some(our_command_index) = results.iter().position(|ref c| c.cmd.eq(&command.cmd)) {
                    let what_should_have_been_first = results.get(our_command_index).unwrap();
                    handler(what_should_have_been_first, max_occurrences, true);
                    positive_examples += 1;
                }
            }

            if negative_examples <= positive_examples {
                if rand::thread_rng().gen::<f64>() > 0.5 {
                    // Get the factors for a top-5 other command that isn't the correct one.
                    let results = self.history.find_matches(&String::new(), Some(5));
                    if let Some(random_command) = rand::thread_rng().choose(&results.iter().filter(|c| !c.cmd.eq(&command.cmd)).collect::<Vec<&Command>>()) {
                        handler(random_command, max_occurrences, false);
                        negative_examples += 1;
                    }
                } else {
                    // Get the factors for some other random command that could have been suggested at that time.
                    if let Some(random_command) = rand::thread_rng().choose(&data_set) {
                        let results = self.history.find_matches(&random_command.cmd, Some(1));
                        if let Some(found_random_command) = results.get(0) {
                            if !found_random_command.cmd.eq(&command.cmd) {
                                handler(found_random_command, max_occurrences, false);
                                negative_examples += 1;
                            }
                        }
                    }
                }
            }
        }
    }
}
