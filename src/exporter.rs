use history::History;
use settings::Settings;
use unicode_segmentation::UnicodeSegmentation;

use csv::Writer;
use std::fs::File;
use history::Command;
use rand::random;

#[derive(Debug)]
pub struct Exporter<'a> {
    settings: &'a Settings,
    history: &'a mut History,
    writer: Box<Writer<File>>
}

impl <'a> Exporter<'a> {
    pub fn new(settings: &'a Settings, history: &'a mut History) -> Exporter<'a> {
        let path = settings.file.clone().unwrap();
        let writer = Box::new(Writer::from_path(path)
            .expect("Expected to be able to write a CSV"));

        Exporter { settings, history, writer }
    }

    fn output_header(&mut self) {
        self.writer.write_record(&[
            "age_factor",
            "exit_factor",
            "dir_factor",
            "overlap_factor",
            "immediate_overlap_factor",
            "occurrences_factor",
            "correct"
        ]).expect("Expected to write to CSV");
        self.writer.flush().expect("Expected to flush CSV");
    }

    fn output_row(&mut self, winner: &Command, correct: bool) {
        self.writer.write_record(&[
            format!("{}", winner.age_factor),
            format!("{}", winner.exit_factor),
            format!("{}", winner.dir_factor),
            format!("{}", winner.overlap_factor),
            format!("{}", winner.immediate_overlap_factor),
            format!("{}", winner.occurrences_factor),
            if correct { String::from("1.0") } else { String::from("0.0") }
        ]).expect("Expected to write to CSV");
        self.writer.flush().expect("Expected to flush CSV");
    }

    pub fn export(&mut self) {
        self.output_header();

        let data_set = self.history.commands(-1, 0);

        for command in data_set.iter() {
            if command.dir.is_none() || command.exit_code.is_none() || command.when_run.is_none() { continue }
            if command.cmd.is_empty() { continue }

            // Setup the cache for the time this command was recorded.
            self.history.build_cache_table(command.dir.to_owned(), None, command.when_run);

            // Record how it would do by default.
            if let Some(winner) = self.history.find_matches(&String::new(), Some(1)).get(0) {
                if winner.cmd.eq(&command.cmd) {
                    // Good guess!
                    self.output_row(&winner, true);
                } else {
                    self.output_row(&winner, false);
                }
            }

            // Do a random substring.
            let graphemes= command.cmd.graphemes(true);
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

            if let Some(winner) = self.history.find_matches(&substring, Some(1)).get(0) {
                if winner.cmd.eq(&command.cmd) {
                    // Good guess!
                    self.output_row(&winner, true);
                } else {
                    self.output_row(&winner, false);
                }
            }
        }
    }
}
