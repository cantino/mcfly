use unicode_segmentation::UnicodeSegmentation;
use history::History;
use settings::Settings;
use history::Command;
use rand::random;

#[derive(Debug)]
pub struct TrainingSampleGenerator<'a> {
    settings: &'a Settings,
    history: &'a History,
}

impl<'a> TrainingSampleGenerator<'a> {
    pub fn new(settings: &'a Settings, history: &'a History) -> TrainingSampleGenerator<'a> {
        TrainingSampleGenerator { settings, history }
    }

    pub fn generate<F>(&self, records: i16, samples_per_record: u16, mut handler: F) where F: FnMut(&Command, f64, bool) {
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

            // Record how it would do by default.
            let results = self.history.find_matches(&String::new(), Some(10));
            if let Some(our_command_index) = results.iter().position(|ref c| c.cmd.eq(&command.cmd)) {
                let what_should_have_been_first = results.get(our_command_index).unwrap();
                handler(what_should_have_been_first, max_occurrences, true);
                positive_examples += 1;
                for (index, command) in results.iter().enumerate() {
                    if index != our_command_index {
                        if negative_examples < positive_examples {
                            handler(command, max_occurrences, false);
                            negative_examples += 1;
                        }
                    }
                }
            }

            // Do random substrings.
            for _ in 0..samples_per_record {
                let graphemes = command.cmd.graphemes(true);
                let mut substring = String::new();
                let start_prob = 1.0 / (command.cmd.graphemes(true).count() as f64);
                let avg_substring_length = 3.0;
                let stop_prob = 1.0 / avg_substring_length;

                while substring.len() == 0 {
                    let mut started = false;
                    for grapheme in graphemes.clone() {
                        if random::<f64>() < start_prob {
                            // Start reading a substring.
                            started = true;
                        }

                        if started {
                            substring.push_str(grapheme);

                            if random::<f64>() < stop_prob {
                                break;
                            }
                        }
                    }
                }

                let results = self.history.find_matches(&substring, Some(20));
                if let Some(our_command_index) = results.iter().position(|ref c| c.cmd.eq(&command.cmd)) {
                    let what_should_have_been_first = results.get(our_command_index).unwrap();
                    handler(what_should_have_been_first, max_occurrences, true);
                    positive_examples += 1;
                    for (index, command) in results.iter().enumerate() {
                        if index != our_command_index {
                            if negative_examples < positive_examples {
                                handler(command, max_occurrences, false);
                                negative_examples += 1;
                            }
                        }
                    }
                }
            }
        }
    }
}
