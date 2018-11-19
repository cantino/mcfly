extern crate rand;

use node::Node;
use training_sample_generator::TrainingSampleGenerator;
use history::Features;
use rand::Rng;

#[derive(Debug, Copy, Clone)]
pub struct Network {
    pub output_bias: f64,
    pub output_weights: [f64; 1],
    pub hidden_nodes: [Node; 1],
}

impl Default for Network {
    fn default() -> Network {
        Network {
            hidden_nodes: [Node::default()],
            output_bias: 0.0,
            output_weights: [1.0],
        }
    }
}

impl Network {
    pub fn random() -> Network {
        Network {
            hidden_nodes: [Node::random()],
            output_bias: rand::thread_rng().gen_range(-1.0, 1.0),
            output_weights: [rand::thread_rng().gen_range(-1.0, 1.0)],
        }
    }

    pub fn forward(&self, features: &Features) -> f64 {
        let mut result = 0.0;
        for (node, output_weight) in self.hidden_nodes.iter().zip(self.output_weights.iter()) {
            result += node.forward(features) * output_weight;
        }
        // tanh
        result
    }

    pub fn error(&self, generator: &TrainingSampleGenerator, records: usize) -> f64 {
        let mut error = 0.0;
        let mut samples = 0.0;
        generator.generate(Some(records), |features: &Features, correct: bool| {
            let goal = if correct { 1.0 } else { 0.0 };
            let prediction = self.forward(features);
            error += (prediction - goal).powi(2); // multiply by 0.5?
            samples += 1.0;
        });

        error / samples
    }
}
