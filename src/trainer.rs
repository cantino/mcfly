use history::History;
use settings::Settings;

use history::Features;
use node::Node;
use network::Network;
use training_sample_generator::TrainingSampleGenerator;

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

        println!("Evaluating error rate on current {:#?}", self.history.network);

        let mut best_overall_network = self.history.network.clone();
        let mut best_overall_error =
            self.history.network.error(self.settings, self.history, batch_size);

        loop {
            self.history.network.randomize();

            let mut best_restart_network = self.history.network.clone();
            let mut best_restart_error = 10000.0;
            let mut cycles_since_best_restart_error = 0;

            println!("Starting a random restart with current error rate: {}", best_overall_error);

            loop {
                let mut network = self.history.network.clone();
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
                    generator.generate(batch_size, |features: &Features, correct: bool| {
                        let goal = if correct { 1.0 } else { 0.0 };
                        let prediction = network.forward(features);
                        let prediction_minus_goal = prediction - goal;
                        error += prediction_minus_goal.powi(2);
                        samples += 1.0;

                        offset_increment = momentum * offset_increment + lr * 2.0 * prediction_minus_goal;
                        age_increment = momentum * age_increment + lr * 2.0 * features.age_factor * prediction_minus_goal;
                        length_increment = momentum * length_increment + lr * 2.0 * features.length_factor * prediction_minus_goal;
                        exit_increment = momentum * exit_increment + lr * 2.0 * features.exit_factor * prediction_minus_goal;
                        recent_failure_increment = momentum * recent_failure_increment + lr * 2.0 * features.recent_failure_factor * prediction_minus_goal;
                        selected_dir_increment = momentum * selected_dir_increment + lr * 2.0 * features.selected_dir_factor * prediction_minus_goal;
                        dir_increment = momentum * dir_increment + lr * 2.0 * features.dir_factor * prediction_minus_goal;
                        overlap_increment = momentum * overlap_increment + lr * 2.0 * features.overlap_factor * prediction_minus_goal;
                        immediate_overlap_increment = momentum * immediate_overlap_increment + lr * 2.0 * features.immediate_overlap_factor * prediction_minus_goal;
                        selected_occurrences_increment = momentum * selected_occurrences_increment + lr * 2.0 * features.selected_occurrences_factor * prediction_minus_goal;
                        occurrences_increment = momentum * occurrences_increment + lr * 2.0 * features.occurrences_factor * prediction_minus_goal;

                        let single_node = network.hidden_nodes[0];
                        network = Network {
                            hidden_nodes: [
                                Node {
                                    offset: single_node.offset - offset_increment,
                                    age: single_node.age - age_increment,
                                    length: single_node.length - length_increment,
                                    exit: single_node.exit - exit_increment,
                                    recent_failure: single_node.recent_failure - recent_failure_increment,
                                    selected_dir: single_node.selected_dir - selected_dir_increment,
                                    dir: single_node.dir - dir_increment,
                                    overlap: single_node.overlap - overlap_increment,
                                    immediate_overlap: single_node.immediate_overlap - immediate_overlap_increment,
                                    selected_occurrences: single_node.selected_occurrences - selected_occurrences_increment,
                                    occurrences: single_node.occurrences - occurrences_increment,
                                }
                            ],
                            output_bias: 0.0,
                            output_weights: [1.0],
                        };
                    });
                }
                if error / samples < best_restart_error {
                    best_restart_error = error / samples;
                    best_restart_network = network.clone();
                    cycles_since_best_restart_error = 0;
                } else {
                    cycles_since_best_restart_error += 1;
                    if cycles_since_best_restart_error > plateau_threshold {
                        println!("Plateaued.");

                        if best_restart_error < best_overall_error {
                            best_overall_error = best_restart_error;
                            best_overall_network = best_restart_network;

                            println!(
                                "New best overall error {} for {:#?}",
                                best_overall_error, best_overall_network
                            );
                        } else {
                            println!(
                                "Best overall error remains {} for {:#?}",
                                best_overall_error, best_overall_network
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
                self.history.network = network;
            }
        }
    }
}
