use std::f64;

#[derive(Debug, Copy, Clone)]
pub struct Weights {
    pub offset: f64,
    pub age: f64,
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
//        Weights {
//            offset: 0.0,
//            age: -0.05,
//            exit: 30.0,
//            recent_failure: 30.0,
//            dir: 75.0,
//            overlap: 50.0,
//            immediate_overlap: 150.0,
//            occurrences: 20.0
//        }
        Weights {
            offset: 0.29245419930668487,
            age: -0.02498043751672841,
            exit: 0.3676148896415478,
            recent_failure: 0.07832196604005508,
            selected_dir: 5.0,
            dir: 1.0,
            overlap: 0.5186653870671801,
            immediate_overlap: 0.8630829374776654,
            selected_occurrences: 0.4,
            occurrences: 0.24541731107371384
        }
    }
}

impl Weights {
    /// Return a new `Weights` struct that represents an online stochastic gradient update of `self`.
    ///
    /// See https://en.wikipedia.org/wiki/Stochastic_gradient_descent#Example
    ///
    /// Our target is to minimize:
    ///   ((self.offset + age * self.age + exit * self.exit + recent_failure * self.recent_failure + dir * self.dir + overlap * self.overlap + immediate_overlap * self.immediate_overlap + occurrences * self.occurrences - correct)^2
    pub fn online_update(&self, lr: f64, correct: f64, age: f64, exit: f64, recent_failure: f64, selected_dir: f64, dir: f64, overlap: f64, immediate_overlap: f64, selected_occurrences: f64, occurrences: f64) -> Weights {
        let rank = self.rank(age, exit, recent_failure, selected_dir, dir, overlap, immediate_overlap, selected_occurrences, occurrences);
        Weights {
            offset: self.offset - lr * 2.0 * (rank - correct),
            age: self.age - lr * 2.0 * age * (rank - correct),
            exit: self.exit - lr * 2.0 * exit * (rank - correct),
            recent_failure: self.recent_failure - lr * 2.0 * recent_failure * (rank - correct),
            selected_dir: self.selected_dir - lr * 2.0 * selected_dir * (rank - correct),
            dir: self.dir - lr * 2.0 * dir * (rank - correct),
            overlap: self.overlap - lr * 2.0 * overlap * (rank - correct),
            immediate_overlap: self.immediate_overlap - lr * 2.0 * immediate_overlap * (rank - correct),
            selected_occurrences: self.selected_occurrences - lr * 2.0 * selected_occurrences * (rank - correct),
            occurrences: self.occurrences - lr * 2.0 * occurrences * (rank - correct)
        }
    }

    pub fn rank(&self, age: f64, exit: f64, recent_failure: f64, selected_dir: f64, dir: f64, overlap: f64, immediate_overlap: f64, selected_occurrences: f64, occurrences: f64) -> f64 {
        self.offset + age * self.age + exit * self.exit + recent_failure * self.recent_failure + selected_dir * self.selected_dir + dir * self.dir + overlap * self.overlap + immediate_overlap * self.immediate_overlap + selected_occurrences * self.selected_occurrences + occurrences * self.occurrences
    }
}
