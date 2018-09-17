use history::History;
use settings::Settings;

use rand::{thread_rng, Rng};

#[derive(Debug)]
pub struct Trainer<'a> {
    settings: &'a Settings,
    history: &'a mut History
}

impl<'a> Trainer<'a> {
    pub fn new(settings: &'a Settings, history: &'a mut History) -> Trainer<'a> {
        Trainer { settings, history }
    }

    pub fn train(&mut self) {
        println!("Initial weights: {:#?}", self.history.weights);

        let depth = 10000;
        let lr = 0.001;

        for _ in 1..100 {
            let mut data_set = self.history.commands(&None, depth, 0);
            thread_rng().shuffle(&mut data_set);

            for command in data_set.iter() {
                if command.dir.is_none() || command.exit_code.is_none() || command.when_run.is_none() { continue; }
                if command.exit_code.unwrap() != 0 { continue; }
                if command.cmd.is_empty() { continue; }

                // What rank would this command have had at the time recorded?
                self.history.build_cache_table(&command.dir, &Some(command.session_id.to_owned()), None, command.when_run);

                if let Some(winner) = self.history.find_matches(&String::new(), Some(1)).get(0) {
                    let matches = self.history.find_matches(&command.cmd.to_owned(), Some(20));
                    if let Some(my_match) = matches.iter().filter(|found_command| found_command.cmd.eq(&command.cmd)).nth(0) {
                        println!("Command: {:#?}", command);
                        println!("Best guess: {:#?}", winner);
                        println!("My position: {:#?}", my_match);

                        // Update the weights such that our rank is closer to 1.0.
                        self.history.weights = self.history.weights.online_update(lr * 2.0, 1.0, my_match.age_factor, my_match.exit_factor, my_match.recent_failure_factor, my_match.dir_factor, my_match.overlap_factor, my_match.immediate_overlap_factor, my_match.occurrences_factor);
                        if !winner.cmd.eq(&my_match.cmd) {
                            // Update the weights such that the winner's rank is closer to 0.0.
                            self.history.weights = self.history.weights.online_update(lr, 0.0, winner.age_factor, winner.exit_factor, winner.recent_failure_factor, winner.dir_factor, winner.overlap_factor, winner.immediate_overlap_factor, winner.occurrences_factor);
                        }

                        println!("New weights: {:#?}", self.history.weights);
                    }
                }
            }
        }

        println!("Final weights: {:#?}", self.history.weights);
    }
}
