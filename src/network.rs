#![allow(clippy::unreadable_literal)]
use crate::history::Features;
use crate::node::Node;
use crate::training_sample_generator::TrainingSampleGenerator;
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
            final_bias: -0.3829333755179377,
            final_weights: [0.44656858145177714, -1.9550439349609872, -2.963322601316632],
            final_sum: 0.0,
            final_output: 0.0,
            hidden_nodes: [
                Node {
                    offset: -0.878184962836099,
                    age: -0.9045522440219468,
                    length: 0.5406937685800283,
                    exit: -0.3472765681766297,
                    recent_failure: -0.05291342121445077,
                    selected_dir: -0.35027519196134,
                    dir: -0.2466069217936986,
                    overlap: 0.4791784213482642,
                    immediate_overlap: 0.5565797758340211,
                    selected_occurrences: -0.3600203296209723,
                    occurrences: 0.15694312742881805,
                },
                Node {
                    offset: -0.04362945902379799,
                    age: -0.25381913331319716,
                    length: 0.4238780143901607,
                    exit: 0.21906785628210726,
                    recent_failure: -0.9510136025685453,
                    selected_dir: -0.04654084670567356,
                    dir: -2.2858050301068693,
                    overlap: -0.562274365705918,
                    immediate_overlap: -0.47252489212451904,
                    selected_occurrences: 0.2446391951417497,
                    occurrences: -1.4846489581676605,
                },
                Node {
                    offset: -0.11992725490486622,
                    age: 0.3759013420273308,
                    length: 1.674601413922965,
                    exit: -0.15529596916772864,
                    recent_failure: -0.7819181782432957,
                    selected_dir: -1.1890532332896768,
                    dir: 0.34723729558743677,
                    overlap: 0.09372412920642742,
                    immediate_overlap: 0.393989158881144,
                    selected_occurrences: -0.2383372126951215,
                    occurrences: -2.196219880265691,
                },
            ],
            hidden_node_sums: [0.0, 0.0, 0.0],
            hidden_node_outputs: [0.0, 0.0, 0.0],
        }
    }
}

impl Network {
    pub fn random() -> Network {
        let mut rng = rand::thread_rng();

        Network {
            final_bias: rng.gen_range(-1.0..1.0),
            final_weights: [
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0),
            ],
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
