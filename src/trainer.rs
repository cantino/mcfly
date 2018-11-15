use history::History;
use settings::Settings;

use history::Command;
use training_sample_generator::TrainingSampleGenerator;
use weights::Weights;

#[derive(Debug)]
pub struct Trainer<'a> {
    settings: &'a Settings,
    history: &'a mut History,
}

impl<'a> Trainer<'a> {
    pub fn new(settings: &'a Settings, history: &'a mut History) -> Trainer<'a> {
        Trainer { settings, history }
    }

    pub fn train(&mut self) {
        let lr = 0.0001;
        let momentum = 0.75;
        let batch_size = 250;
        let plateau_threshold = 10;

        println!(
            "Evaluating error rate on current {:#?}",
            self.history.weights
        );

        let mut best_overall_weights = self.history.weights.clone();
        let mut best_overall_error =
            self.history
                .weights
                .error(self.settings, self.history, batch_size);

        loop {
            self.history.weights.randomize();

            let mut best_restart_weights = self.history.weights.clone();
            let mut best_restart_error = 10000.0;
            let mut cycles_since_best_restart_error = 0;

            println!(
                "Starting a random restart with current error rate: {}",
                best_overall_error
            );

            loop {
                let mut weights = self.history.weights.clone();
                let mut error = 0.0;
                let mut samples = 0.0;

                let mut offset_increment = 0.0;
                let mut age_increment = 0.0;
                let mut length_increment = 0.0;
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
                    generator.generate(batch_size, |command: &Command, correct: bool| {
                        let goal = if correct { 1.0 } else { 0.0 };
                        let prediction = weights.rank(command);
                        let prediction_minus_goal = prediction - goal;
                        error += prediction_minus_goal.powi(2);
                        samples += 1.0;

                        offset_increment =
                            momentum * offset_increment + lr * 2.0 * prediction_minus_goal;
                        age_increment = momentum * age_increment
                            + lr * 2.0 * command.age_factor * prediction_minus_goal;
                        length_increment = momentum * length_increment
                            + lr * 2.0 * command.length_factor * prediction_minus_goal;
                        exit_increment = momentum * exit_increment
                            + lr * 2.0 * command.exit_factor * prediction_minus_goal;
                        recent_failure_increment = momentum * recent_failure_increment
                            + lr * 2.0 * command.recent_failure_factor * prediction_minus_goal;
                        selected_dir_increment = momentum * selected_dir_increment
                            + lr * 2.0 * command.selected_dir_factor * prediction_minus_goal;
                        dir_increment = momentum * dir_increment
                            + lr * 2.0 * command.dir_factor * prediction_minus_goal;
                        overlap_increment = momentum * overlap_increment
                            + lr * 2.0 * command.overlap_factor * prediction_minus_goal;
                        immediate_overlap_increment = momentum * immediate_overlap_increment
                            + lr * 2.0 * command.immediate_overlap_factor * prediction_minus_goal;
                        selected_occurrences_increment = momentum * selected_occurrences_increment
                            + lr
                                * 2.0
                                * command.selected_occurrences_factor
                                * prediction_minus_goal;
                        occurrences_increment = momentum * occurrences_increment
                            + lr * 2.0 * command.occurrences_factor * prediction_minus_goal;

                        weights = Weights {
                            offset: weights.offset - offset_increment,
                            age: weights.age - age_increment,
                            length: weights.length - length_increment,
                            exit: weights.exit - exit_increment,
                            recent_failure: weights.recent_failure - recent_failure_increment,
                            selected_dir: weights.selected_dir - selected_dir_increment,
                            dir: weights.dir - dir_increment,
                            overlap: weights.overlap - overlap_increment,
                            immediate_overlap: weights.immediate_overlap
                                - immediate_overlap_increment,
                            selected_occurrences: weights.selected_occurrences
                                - selected_occurrences_increment,
                            occurrences: weights.occurrences - occurrences_increment,
                        };
                    });
                }
                if error / samples < best_restart_error {
                    best_restart_error = error / samples;
                    best_restart_weights = weights.clone();
                    cycles_since_best_restart_error = 0;
                } else {
                    cycles_since_best_restart_error += 1;
                    if cycles_since_best_restart_error > plateau_threshold {
                        println!("Plateaued.");

                        if best_restart_error < best_overall_error {
                            best_overall_error = best_restart_error;
                            best_overall_weights = best_restart_weights;

                            println!(
                                "New best overall error {} for {:#?}",
                                best_overall_error, best_overall_weights
                            );
                        } else {
                            println!(
                                "Best overall error remains {} for {:#?}",
                                best_overall_error, best_overall_weights
                            );
                        }
                        break;
                    }
                }

                println!(
                    "Error of {} (vs {} {} ago)",
                    error / samples,
                    best_restart_error,
                    cycles_since_best_restart_error
                );
                self.history.weights = weights;
            }
        }
    }
}
