use super::settings::InitMode;
use std::env;

pub struct Init {}

impl Init {
    pub fn new(init_mode: &InitMode) -> Self {
        match init_mode {
            InitMode::Bash => {
                Init::init_bash();
            }
            InitMode::Zsh => {
                Init::init_zsh();
            }
            InitMode::Fish => {
                Init::init_fish();
            }
            InitMode::Powershell => {
                Init::init_pwsh();
            }
        }
        Self {}
    }
    pub fn init_bash() {
        let script = include_str!("../mcfly.bash");
        print!("{script}");
    }
    pub fn init_zsh() {
        let script = include_str!("../mcfly.zsh");
        print!("{script}");
    }
    pub fn init_fish() {
        let script = include_str!("../mcfly.fish");
        print!("{script}");
    }
    pub fn init_pwsh() {
        let script = include_str!("../mcfly.ps1")
            .replace("::MCFLY::", env::current_exe().unwrap().to_str().unwrap());
        print!("{script}");
    }
}
