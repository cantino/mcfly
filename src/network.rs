use node::Node;
use settings::Settings;
use history::History;
use training_sample_generator::TrainingSampleGenerator;
use history::Features;

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
    pub fn randomize(&mut self) {
        self.hidden_nodes[0].randomize();
    }

    pub fn forward(&self, features: &Features) -> f64 {
        let mut result = 0.0;
        for (node, output_weight) in self.hidden_nodes.iter().zip(self.output_weights.iter()) {
            result += node.forward(features) * output_weight;
        }
        // tanh
        result
    }

    pub fn error(&self, settings: &Settings, history: &History, records: i16) -> f64 {
        let generator = TrainingSampleGenerator::new(settings, history);
        let mut error = 0.0;
        let mut samples = 0.0;
        generator.generate(records, |features: &Features, correct: bool| {
            let goal = if correct { 1.0 } else { 0.0 };
            let prediction = self.forward(features);
            error += (prediction - goal).powi(2); // multiply by 0.5?
            samples += 1.0;
        });

        error / samples
    }
}
