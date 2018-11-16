extern crate rand;

use history::Features;
use rand::Rng;
use std::f64;

#[derive(Debug, Copy, Clone)]
pub struct Node {
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
    pub occurrences: f64,
}

impl Default for Node {
    fn default() -> Node {
        Node {
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
            occurrences: 0.4423121516974435,
        }
    }
}

impl Node {
    pub fn forward(&self, features: &Features) -> f64 {
        let result = self.offset
            + features.age_factor * self.age
            + features.length_factor * self.length
            + features.exit_factor * self.exit
            + features.recent_failure_factor * self.recent_failure
            + features.selected_dir_factor * self.selected_dir
            + features.dir_factor * self.dir
            + features.overlap_factor * self.overlap
            + features.immediate_overlap_factor * self.immediate_overlap
            + features.selected_occurrences_factor * self.selected_occurrences
            + features.occurrences_factor * self.occurrences;

        //        result.tanh()
        result
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
