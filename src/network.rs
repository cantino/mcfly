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
            hidden_nodes: [Node::default()],
            hidden_node_sums: [0.0],
            hidden_node_outputs: [0.0],
            final_bias: 0.0,
            final_weights: [1.0],
            final_sum: 0.0,
            final_output: 0.0,
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
