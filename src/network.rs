extern crate rand;

use node::Node;
use training_sample_generator::TrainingSampleGenerator;
use history::Features;
use rand::Rng;

#[derive(Debug, Copy, Clone)]
pub struct Network {
    pub final_bias: f64,
    pub final_weights: [f64; 2],
    pub final_sum: f64,
    pub final_output: f64,
    pub hidden_nodes: [Node; 2],
    pub hidden_node_sums: [f64; 2],
    pub hidden_node_outputs: [f64; 2],
}

impl Default for Network {
    fn default() -> Network {
        Network {
            final_bias: -0.5809821008535968,
            final_weights: [
                2.2272498855369425,
                1.4793473882504211
            ],
            final_sum: 0.0,
            final_output: 0.0,
            hidden_nodes: [
                Node {
                    offset: -0.05792540247020638,
                    age: -0.404038222084296,
                    length: -0.784515923905419,
                    exit: 0.21690398489096557,
                    recent_failure: 0.8268005805715911,
                    selected_dir: 0.9304078322382491,
                    dir: -0.19777776872031264,
                    overlap: -0.037932785929368475,
                    immediate_overlap: -0.3173993964361948,
                    selected_occurrences: -0.17053559633230875,
                    occurrences: 2.168352483924481
                },
                Node {
                    offset: 0.4115659521415093,
                    age: -0.12684960037350226,
                    length: 0.7869780886334196,
                    exit: -0.3080243125660992,
                    recent_failure: 0.5190456125954599,
                    selected_dir: 0.23097424007110195,
                    dir: 2.0619093054411963,
                    overlap: 0.8830808640252389,
                    immediate_overlap: -0.9448544859217984,
                    selected_occurrences: -0.05760593990281172,
                    occurrences: 1.0789305008664094
                }
            ],
            hidden_node_sums: [
                0.0,
                0.0
            ],
            hidden_node_outputs: [
                0.0,
                0.0
            ]
        }
    }
}

impl Network {
    pub fn random() -> Network {
        Network {
            final_bias: rand::thread_rng().gen_range(-1.0, 1.0),
            final_weights: [rand::thread_rng().gen_range(-1.0, 1.0), rand::thread_rng().gen_range(-1.0, 1.0)],
            hidden_nodes: [Node::random(), Node::random()],
            hidden_node_sums: [0.0, 0.0],
            hidden_node_outputs: [0.0, 0.0],
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
