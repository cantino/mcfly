use history::History;
use settings::Settings;

use rand::{thread_rng, Rng};

#[derive(Debug)]
pub struct Trainer<'a> {
    settings: &'a Settings,
    history: &'a History
}

impl <'a> Trainer<'a> {
    pub fn new(settings: &'a Settings, history: &'a History) -> Trainer<'a> {
        Trainer { settings, history }
    }

    pub fn train(&self) {
        println!("Initial weights: {:#?}", self.history.weights);

        let depth = 1000;

        let mut data_set = self.history.commands(depth);
        thread_rng().shuffle(&mut data_set);

        for command in data_set.iter() {
            if command.dir.is_none() || command.exit_code.is_none() || command.when_run.is_none() { continue }
            if command.exit_code.unwrap() != 0 { continue }
            if command.cmd.is_empty() { continue }

            // What rank would this command have had at the time recorded?
            self.history.build_cache_table(command.dir.to_owned(), None, command.when_run);

            if let Some(winner) = self.history.find_matches(&String::new(), Some(1)).get(0) {
                let matches = self.history.find_matches(&command.cmd.to_owned(), Some(20));
                if let Some(my_match) = matches.iter().filter(|found_command| found_command.cmd.eq(&command.cmd)).nth(0) {

                    println!("Command: {:#?}", command);
                    println!("Best guess: {:#?}", winner);
                    println!("My position: {:#?}", my_match);

                    if winner.rank > my_match.rank {
                        // Update the weights such that our rank is a bit closer to the winner's rank.
                        println!("Should update!");
                    }
                }

            }

            break;
//            println!("{:#?}", command)
        }
    }
}
