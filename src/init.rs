use super::settings::InitMode;

pub struct Init {}

// Intermediating a `source <(...)` step is important so that the actual init
// scripts don't have too much power in `.bashrc` or `.zshrc` when `eval()`'d. For
// instance eval'ing `return 0` returns from the _entire_ .bashrc/.zshrc script,
// not just the mcfly init script.
//
// Perhaps just asking users to directly `source <(mcfly init bash --print_full_init)`
// is better, but we now also need to be backward compatible for those that already
// have `eval "$(mcfly init bash)"` in their config.
//
// See: https://github.com/cantino/mcfly/issues/254
const BASH_SOURCE_STANZA: &str = "source <(mcfly init bash --print_full_init)";
const ZSH_SOURCE_STANZA: &str = "source <(mcfly init zsh --print_full_init)";

impl Init {
    pub fn new(init_mode: &InitMode, is_print_full_init: bool) -> Self {
        match init_mode {
            InitMode::Bash => {
                Init::init_bash(is_print_full_init);
            }
            InitMode::Zsh => {
                Init::init_zsh(is_print_full_init);
            }
            InitMode::Fish => {
                Init::init_fish();
            }
        }
        Self {}
    }
    pub fn init_bash(is_print_full_init: bool) {
        if is_print_full_init {
            let script = include_str!("../mcfly.bash");
            print!("{}", script);
        } else {
            print!("{}", BASH_SOURCE_STANZA);
        }
    }
    pub fn init_zsh(is_print_full_init: bool) {
        if is_print_full_init {
            let script = include_str!("../mcfly.zsh");
            print!("{}", script);
        } else {
            print!("{}", ZSH_SOURCE_STANZA);
        }
    }
    pub fn init_fish() {
        let script = include_str!("../mcfly.fish");
        print!("{}", script);
    }
}
