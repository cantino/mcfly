use settings::Settings;
use history::History;

#[derive(Debug)]
pub struct Interface<'a> {
    settings: &'a Settings,
    history: &'a History
}

impl <'a> Interface<'a> {
    pub fn new(settings: &'a Settings, history: &'a History) -> Interface<'a> {
        Interface { settings, history }
    }

    pub fn show(&'a self) {
        for history_match in self.history.find_matches(&self.settings.command) {
            println!("{:?}", history_match);
        }
    }
}
