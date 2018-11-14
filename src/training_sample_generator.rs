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

    pub fn generate<F>(&self, records: i16, mut handler: F) where F: FnMut(&Command, bool) {
        let data_set = self.history.commands(&None, records, 0, true);

        let mut positive_examples = 0;
        let mut negative_examples = 0;

        for command in data_set.iter() {
            if command.dir.is_none() || command.exit_code.is_none() || command.when_run.is_none() { continue; }
            if command.cmd.is_empty() { continue; }

            // Setup the cache for the time this command was recorded.
            // Unwrap is safe here because we check command.dir.is_none() above.
            self.history.build_cache_table(&command.dir.to_owned().unwrap(), &Some(command.session_id.clone()), None, command.when_run, command.when_run);

            // Get the factors for this command at the time it was logged.
            if positive_examples <= negative_examples {
                let results = self.history.find_matches(&String::new(), Some(2000));
                if let Some(our_command_index) = results.iter().position(|ref c| c.cmd.eq(&command.cmd)) {
                    let what_should_have_been_first = results.get(our_command_index).unwrap();
                    handler(what_should_have_been_first, true);
                    positive_examples += 1;
                }
            }

            if negative_examples <= positive_examples {
                // Get the factors for another command that isn't the correct one.
                let results = self.history.find_matches(&String::new(), Some(500));
                if let Some(random_command) = rand::thread_rng().choose(&results.iter().filter(|c| !c.cmd.eq(&command.cmd)).collect::<Vec<&Command>>()) {
                    handler(random_command, false);
                    negative_examples += 1;
                }
            }
        }
    }
}
