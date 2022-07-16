use crate::history::Features;
use rand::Rng;
use std::f64;

#[derive(Debug, Copy, Clone, Default)]
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

impl Node {
    pub fn random() -> Node {
        let mut rng = rand::thread_rng();

        Node {
            offset: rng.gen_range(-1.0..1.0),
            age: rng.gen_range(-1.0..1.0),
            length: rng.gen_range(-1.0..1.0),
            exit: rng.gen_range(-1.0..1.0),
            recent_failure: rng.gen_range(-1.0..1.0),
            selected_dir: rng.gen_range(-1.0..1.0),
            dir: rng.gen_range(-1.0..1.0),
            overlap: rng.gen_range(-1.0..1.0),
            immediate_overlap: rng.gen_range(-1.0..1.0),
            selected_occurrences: rng.gen_range(-1.0..1.0),
            occurrences: rng.gen_range(-1.0..1.0),
        }
    }

    pub fn dot(&self, features: &Features) -> f64 {
        self.offset
            + features.age_factor * self.age
            + features.length_factor * self.length
            + features.exit_factor * self.exit
            + features.recent_failure_factor * self.recent_failure
            + features.selected_dir_factor * self.selected_dir
            + features.dir_factor * self.dir
            + features.overlap_factor * self.overlap
            + features.immediate_overlap_factor * self.immediate_overlap
            + features.selected_occurrences_factor * self.selected_occurrences
            + features.occurrences_factor * self.occurrences
    }

    pub fn output(&self, features: &Features) -> f64 {
        self.dot(features).tanh()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot() {
        let node = Node {
            offset: 0.0,
            age: 0.0,
            length: 0.0,
            exit: 0.0,
            recent_failure: 0.0,
            selected_dir: 0.0,
            dir: 0.0,
            overlap: 0.0,
            immediate_overlap: 0.0,
            selected_occurrences: 0.0,
            occurrences: 0.0,
        };

        let features = Features {
            age_factor: 1.0,
            length_factor: 1.0,
            exit_factor: 1.0,
            recent_failure_factor: 1.0,
            selected_dir_factor: 1.0,
            dir_factor: 1.0,
            overlap_factor: 1.0,
            immediate_overlap_factor: 1.0,
            selected_occurrences_factor: 1.0,
            occurrences_factor: 1.0,
        };

        assert_eq!(node.dot(&features), 0.0);
    }
}
