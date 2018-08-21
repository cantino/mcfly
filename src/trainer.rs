use history::History;
use settings::Settings;

#[derive(Debug)]
pub struct Learner<'a> {
    settings: &'a Settings,
    history: &'a History
}

impl <'a> Learner<'a> {
    pub fn new(settings: &'a Settings, history: &'a History) -> Learner<'a> {
        Learner { settings, history }
    }

    pub fn learn(&self) {
        println!("Learning!");
    }
}
