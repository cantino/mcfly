extern crate rand;

use std::f64;
use rand::Rng;
use history::Command;
use training_sample_generator::TrainingSampleGenerator;
use settings::Settings;
use history::History;

#[derive(Debug, Copy, Clone)]
pub struct Weights {
    pub offset: f64,
    pub age: f64,
    pub length: f64,
    pub exit: f64,
    pub recent_failure: f64,
    pub selected_dir: f64,
    pub dir: f64,
    pub overlap: f64,
    pub immediate_overlap: f64,
    pub selected_occurrences: f64,
    pub occurrences: f64
}

impl Default for Weights {
    fn default() -> Weights {
        Weights {
            offset: 0.39403019679847806,
            age: -0.16867884562772287,
            length: -1.0132719748233774,
            exit: 0.049452285909269283,
            recent_failure: 0.5096291806456213,
            selected_dir: 0.9063681241547119,
            dir: 0.20987523780246092,
            overlap: 0.030463793073770368,
            immediate_overlap: -0.5068322907651295,
            selected_occurrences: 0.12099345850355443,
            occurrences: 0.4423121516974435
        }
    }
}

impl Weights {
    pub fn rank(&self, command: &Command) -> f64 {
        self.offset + command.age_factor * self.age + command.length_factor * self.length + command.exit_factor * self.exit + command.recent_failure_factor * self.recent_failure + command.selected_dir_factor * self.selected_dir + command.dir_factor * self.dir + command.overlap_factor * self.overlap + command.immediate_overlap_factor * self.immediate_overlap + command.selected_occurrences_factor * self.selected_occurrences + command.occurrences_factor * self.occurrences
    }

    pub fn error(&self, settings: &Settings, history: &History, records: i16) -> f64 {
        let generator = TrainingSampleGenerator::new(settings, history);
        let mut error = 0.0;
        let mut samples = 0.0;
        generator.generate(records, |command: &Command, correct: bool| {
            let goal = if correct { 1.0 } else { 0.0 };
            let prediction = self.rank(command);
            error += (prediction - goal).powi(2);
            samples += 1.0;
        });

        error / samples
    }

    pub fn randomize(&mut self) {
        let min = -1.0;
        let max = 1.0;
        self.offset = rand::thread_rng().gen_range(min, max);
        self.age = rand::thread_rng().gen_range(min, max);
        self.length = rand::thread_rng().gen_range(min, max);
        self.exit = rand::thread_rng().gen_range(min, max);
        self.recent_failure = rand::thread_rng().gen_range(min, max);
        self.selected_dir = rand::thread_rng().gen_range(min, max);
        self.dir = rand::thread_rng().gen_range(min, max);
        self.overlap = rand::thread_rng().gen_range(min, max);
        self.immediate_overlap = rand::thread_rng().gen_range(min, max);
        self.selected_occurrences = rand::thread_rng().gen_range(min, max);
        self.occurrences = rand::thread_rng().gen_range(min, max);
    }
}
