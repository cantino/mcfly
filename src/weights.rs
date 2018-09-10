#[derive(Debug, Copy, Clone)]
pub struct Weights {
    pub offset: f64,
    pub age: f64,
    pub exit: f64,
    pub recent_failure: f64,
    pub dir: f64,
    pub overlap: f64,
    pub occurrences: f64
}

impl Default for Weights {
    fn default() -> Weights {
        Weights { offset: 0.0, age: -0.1, exit: -5.0, recent_failure: 5.0, dir: 2.0, overlap: 4.0, occurrences: 0.5 }
    }
}

impl Weights {
    /// Return a new `Weights` struct that represents an online stochastic gradient update of `self`.
    ///
    /// See https://en.wikipedia.org/wiki/Stochastic_gradient_descent#Example
    ///
    /// Our target is to minimize:
    ///   ((self.offset + age * self.age + exit * self.exit + recent_failure * self.recent_failure + dir * self.dir + overlap * self.overlap + occurrences * self.occurrences - 1.0)^2
    pub fn online_update(&self, lr: f64, target: f64, age: f64, exit: f64, recent_failure: f64, dir: f64, overlap: f64, occurrences: f64) -> Weights {
        let rank = self.offset + age * self.age + exit * self.exit + recent_failure * self.recent_failure + dir * self.dir + overlap * self.overlap + occurrences * self.occurrences;
        Weights {
            offset: self.offset - lr * 2.0 * (rank - target),
            age: self.age - lr * 2.0 * age * (rank - target),
            exit: self.exit - lr * 2.0 * exit * (rank - target),
            recent_failure: self.recent_failure - lr * 2.0 * recent_failure * (rank - target),
            dir: self.dir - lr * 2.0 * dir * (rank - target),
            overlap: self.overlap - lr * 2.0 * overlap * (rank - target),
            occurrences: self.occurrences - lr * 2.0 * occurrences * (rank - target)
        }
    }
}
