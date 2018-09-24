use history::History;
use settings::Settings;

use training_sample_generator::TrainingSampleGenerator;
use history::Command;
use weights::Weights;

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

        let lr = 0.0001;
        let momentum = 0.75;

        for _ in 1..1000 {
            let mut weights = self.history.weights.clone();
            let mut error = 0.0;
            let mut samples = 0.0;

            let mut offset_increment = 0.0;
            let mut age_increment = 0.0;
            let mut exit_increment = 0.0;
            let mut recent_failure_increment = 0.0;
            let mut selected_dir_increment = 0.0;
            let mut dir_increment = 0.0;
            let mut overlap_increment = 0.0;
            let mut immediate_overlap_increment = 0.0;
            let mut selected_occurrences_increment = 0.0;
            let mut occurrences_increment = 0.0;

            {
                let generator = TrainingSampleGenerator::new(self.settings, self.history);
                generator.generate(200, 5, |command: &Command, _max_occurrences: f64, correct: bool| {
                    let goal = if correct { 1.0 } else { 0.0 };
                    let prediction = weights.rank(command.age_factor, command.exit_factor, command.recent_failure_factor, command.selected_dir_factor, command.dir_factor, command.overlap_factor, command.immediate_overlap_factor, command.selected_occurrences_factor, command.occurrences_factor);
                    let prediction_minus_goal = prediction - goal;
                    error += prediction_minus_goal.powi(2);
                    samples += 1.0;

                    offset_increment = momentum * offset_increment + lr * 2.0 * prediction_minus_goal;
                    age_increment = momentum * age_increment + lr * 2.0 * command.age_factor * prediction_minus_goal;
                    exit_increment = momentum * exit_increment + lr * 2.0 * command.exit_factor * prediction_minus_goal;
                    recent_failure_increment = momentum * recent_failure_increment + lr * 2.0 * command.recent_failure_factor * prediction_minus_goal;
                    selected_dir_increment = momentum * selected_dir_increment + lr * 2.0 * command.selected_dir_factor * prediction_minus_goal;
                    dir_increment = momentum * dir_increment + lr * 2.0 * command.dir_factor * prediction_minus_goal;
                    overlap_increment = momentum * overlap_increment + lr * 2.0 * command.overlap_factor * prediction_minus_goal;
                    immediate_overlap_increment = momentum * immediate_overlap_increment + lr * 2.0 * command.immediate_overlap_factor * prediction_minus_goal;
                    selected_occurrences_increment = momentum * selected_occurrences_increment + lr * 2.0 * command.selected_occurrences_factor * prediction_minus_goal;
                    occurrences_increment = momentum * occurrences_increment + lr * 2.0 * command.occurrences_factor * prediction_minus_goal;

                    weights = Weights {
                        offset: weights.offset - offset_increment,
                        age: weights.age - age_increment,
                        exit: weights.exit - exit_increment,
                        recent_failure: weights.recent_failure - recent_failure_increment,
                        selected_dir: weights.selected_dir - selected_dir_increment,
                        dir: weights.dir - dir_increment,
                        overlap: weights.overlap - overlap_increment,
                        immediate_overlap: weights.immediate_overlap - immediate_overlap_increment,
                        selected_occurrences: weights.selected_occurrences - selected_occurrences_increment,
                        occurrences: weights.occurrences - occurrences_increment
                    };
                });
            }
            self.history.weights = weights;
            println!("Error: {}", error / samples);
            println!("New weights: {:#?}", weights);
        }

        println!("Final weights: {:#?}", self.history.weights);
    }
}
