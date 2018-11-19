extern crate rand;

use node::Node;
use training_sample_generator::TrainingSampleGenerator;
use history::Features;
use rand::Rng;

#[derive(Debug, Copy, Clone)]
pub struct Network {
    pub final_bias: f64,
    pub final_weights: [f64; 1],
    pub final_sum: f64,
    pub final_output: f64,
    pub hidden_nodes: [Node; 1],
    pub hidden_node_sums: [f64; 1],
    pub hidden_node_outputs: [f64; 1],
}

impl Default for Network {
    fn default() -> Network {
        Network {
            final_bias: -0.24635202721130312,
            final_weights: [
                1.7594807105987207
            ],
            final_sum: 0.0,
            final_output: 0.0,
            hidden_nodes: [
                Node {
                    offset: 0.14860413625826777,
                    age: -0.7986909450585928,
                    length: -0.2549746094410215,
                    exit: 0.16226005476246494,
                    recent_failure: 0.6729021877538784,
                    selected_dir: 0.8248136661473066,
                    dir: 0.4976741959678194,
                    overlap: 0.5207984980557485,
                    immediate_overlap: -0.7025151688745555,
                    selected_occurrences: -0.10318908435959996,
                    occurrences: 1.381179142147253
                }
            ],
            hidden_node_sums: [
                0.0
            ],
            hidden_node_outputs: [
                0.0
            ]
        }
    }
}

impl Network {
    pub fn random() -> Network {
        Network {
            final_bias: rand::thread_rng().gen_range(-1.0, 1.0),
            final_weights: [rand::thread_rng().gen_range(-1.0, 1.0)],
            hidden_nodes: [Node::random()],
            hidden_node_sums: [0.0],
            hidden_node_outputs: [0.0],
            final_sum: 0.0,
            final_output: 0.0,
        }
    }

    pub fn compute(&mut self, features: &Features) {
        self.final_sum = self.final_bias;
        for i in 0..self.hidden_nodes.len() {
            self.hidden_node_sums[i] = self.hidden_nodes[i].dot(features);
            self.hidden_node_outputs[i] = self.hidden_node_sums[i].tanh();
            self.final_sum += self.hidden_node_outputs[i] * self.final_weights[i];
        }
        self.final_output = self.final_sum.tanh();
    }

    pub fn dot(&self, features: &Features) -> f64 {
        let mut network_output = self.final_bias;
        for (node, output_weight) in self.hidden_nodes.iter().zip(self.final_weights.iter()) {
            let node_output = node.output(features);
            network_output += node_output * output_weight;
        }
        network_output
    }

    pub fn output(&self, features: &Features) -> f64 {
        self.dot(features).tanh()
    }

    pub fn average_error(&self, generator: &TrainingSampleGenerator, records: usize) -> f64 {
        let mut error = 0.0;
        let mut samples = 0.0;
        generator.generate(Some(records), |features: &Features, correct: bool| {
            let target = if correct { 1.0 } else { -1.0 };
            let output = self.output(features);
            error += 0.5 * (target - output).powi(2);
            samples += 1.0;
        });

        error / samples
    }
}
