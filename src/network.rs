extern crate rand;

use node::Node;
use training_sample_generator::TrainingSampleGenerator;
use history::Features;
use rand::Rng;

#[derive(Debug, Copy, Clone)]
pub struct Network {
    pub final_bias: f64,
    pub final_weights: [f64; 3],
    pub final_sum: f64,
    pub final_output: f64,
    pub hidden_nodes: [Node; 3],
    pub hidden_node_sums: [f64; 3],
    pub hidden_node_outputs: [f64; 3],
}

impl Default for Network {
    fn default() -> Network {
        Network {
            final_bias: -0.2800466377853508,
            final_weights: [
                -1.8497386287878366,
                -2.137142910866708,
                -2.918950396271575
            ],
            final_sum: 0.0,
            final_output: 0.0,
            hidden_nodes: [
                Node {
                    offset: 0.5652055853199885,
                    age: -0.3706738595011909,
                    length: 0.9540740712311119,
                    exit: -0.1872885195186705,
                    recent_failure: -0.29838575658582084,
                    selected_dir: -0.23819629317146643,
                    dir: 0.3442373652577654,
                    overlap: 0.12825090109168805,
                    immediate_overlap: 0.5761975536319032,
                    selected_occurrences: -1.2077907855374392,
                    occurrences: -1.381053270657452
                },
                Node {
                    offset: 0.23585746046522635,
                    age: -0.12266956747875156,
                    length: 0.5509065668910484,
                    exit: -0.028834620080782757,
                    recent_failure: 0.39385265623001414,
                    selected_dir: 0.09480796868224452,
                    dir: -2.7341056602149942,
                    overlap: -0.16108554638003306,
                    immediate_overlap: -0.2952654013383927,
                    selected_occurrences: -0.682954359271296,
                    occurrences: -1.2057805976144782
                },
                Node {
                    offset: -0.443855990092483,
                    age: 0.5549673180113682,
                    length: 1.3927575840389037,
                    exit: 0.049541099285673594,
                    recent_failure: -1.159173467850507,
                    selected_dir: -0.4834226821225341,
                    dir: 0.3666721668757306,
                    overlap: -0.09551749106127021,
                    immediate_overlap: -0.019347236539586025,
                    selected_occurrences: 0.48123914331260764,
                    occurrences: -2.2271429831125347
                }
            ],
            hidden_node_sums: [
                0.0,
                0.0,
                0.0
            ],
            hidden_node_outputs: [
                0.0,
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
            final_weights: [rand::thread_rng().gen_range(-1.0, 1.0), rand::thread_rng().gen_range(-1.0, 1.0), rand::thread_rng().gen_range(-1.0, 1.0)],
            hidden_nodes: [Node::random(), Node::random(), Node::random()],
            hidden_node_sums: [0.0, 0.0, 0.0],
            hidden_node_outputs: [0.0, 0.0, 0.0],
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
