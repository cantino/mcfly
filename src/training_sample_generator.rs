use crate::history::Command;
use crate::history::Features;
use crate::history::History;
use crate::settings::Settings;
use crate::training_cache;
use rand::Rng;

#[derive(Debug)]
pub struct TrainingSampleGenerator<'a> {
    settings: &'a Settings,
    history: &'a History,
    data_set: Vec<(Features, bool)>,
}

impl<'a> TrainingSampleGenerator<'a> {
    pub fn new(settings: &'a Settings, history: &'a History) -> TrainingSampleGenerator<'a> {
        let cache_path = Settings::mcfly_training_cache_path();
        let data_set =
            if settings.refresh_training_cache || !cache_path.exists() {
                let ds = TrainingSampleGenerator::generate_data_set(history);
                training_cache::write(&ds, &cache_path);
                ds
            } else {
                training_cache::read(&cache_path)
            };

        TrainingSampleGenerator { settings, history, data_set }
    }

    pub fn generate_data_set(history: &History) -> Vec<(Features, bool)> {
        let mut data_set: Vec<(Features, bool)> = Vec::new();
        let commands = history.commands(&None, -1, 0, true);

        let mut positive_examples = 0;
        let mut negative_examples = 0;

        println!("Generating training set for {} commands", commands.len());

        for (i, command) in commands.iter().enumerate() {
            if command.dir.is_none() || command.exit_code.is_none() || command.when_run.is_none() { continue; }
            if command.cmd.is_empty() { continue; }

            if i % 100 == 0 {
                println!("Done with {}", i);
            }

            // Setup the cache for the time this command was recorded.
            // Unwrap is safe here because we check command.dir.is_none() above.
            history.build_cache_table(
                &command.dir.to_owned().unwrap(),
                &Some(command.session_id.clone()),
                None,
                command.when_run,
                command.when_run,
            );

            // Load the entire match set.
            let results = history.find_matches(&String::new(), -1);

            // Get the features for this command at the time it was logged.
            if positive_examples <= negative_examples {
                if let Some(our_command_index) = results.iter().position(|ref c| c.cmd.eq(&command.cmd)) {
                    let what_should_have_been_first = &results[our_command_index];
                    data_set.push((what_should_have_been_first.features.clone(), true));
                    positive_examples += 1;
                }
            }

            if negative_examples <= positive_examples {
                // Get the features for another command that isn't the correct one.
                if let Some(random_command) = rand::thread_rng().choose(&results
                    .iter()
                    .filter(|c| !c.cmd.eq(&command.cmd))
                    .collect::<Vec<&Command>>())
                    {
                        data_set.push((random_command.features.clone(), false));
                        negative_examples += 1;
                    }
            }
        }

        println!("Done!");

        data_set
    }

    pub fn generate<F>(&self, records: Option<usize>, mut handler: F) where F: FnMut(&Features, bool) {
        let mut positive_examples = 0;
        let mut negative_examples = 0;
        let records = records.unwrap_or_else(|| self.data_set.len());

        loop {
            if let Some((features, correct)) = rand::thread_rng().choose(&self.data_set) {
                if *correct && positive_examples <= negative_examples {
                    handler(features, *correct);
                    positive_examples += 1;
                } else if !*correct && negative_examples <= positive_examples {
                    handler(features, *correct);
                    negative_examples += 1;
                }
            }

            if positive_examples + negative_examples >= records {
                break;
            }
        }
    }
}
