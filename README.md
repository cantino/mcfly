> **Seeking co-maintainers**:
> I don't have much time to maintain this project these days. If someone would like to jump in and become a co-maintainer, it would be appreciated!

![Build Status](https://github.com/cantino/mcfly/actions/workflows/mean_bean_ci.yml/badge.svg)
[![](https://img.shields.io/crates/v/mcfly.svg)](https://crates.io/crates/mcfly)

# McFly - fly through your shell history

<img src="/docs/screenshot.png" alt="screenshot" width="400">

McFly replaces your default `ctrl-r` shell history search with an intelligent search engine that takes into account
your working directory and the context of recently executed commands. McFly's suggestions are prioritized
in real time with a small neural network.

TL;DR: an upgraded `ctrl-r` where history results make sense for what you're working on right now.

## Features

* Rebinds `ctrl-r` to bring up a full-screen reverse history search prioritized with a small neural network.
* Augments your shell history to track command exit status, timestamp, and execution directory in a SQLite database.
* Maintains your normal shell history file as well so that you can stop using McFly whenever you want.
* Unicode support throughout.
* Includes a simple action to scrub any history item from the McFly database and your shell history files.
* Designed to be extensible for other shells in the future.
* Written in Rust, so it's fast and safe.
* You can type `%` to match any number of characters when searching.

## Prioritization

The key feature of McFly is smart command prioritization powered by a small neural network that runs
in real time. The goal is for the command you want to run to always be one of the top suggestions.

When suggesting a command, McFly takes into consideration:

* The directory where you ran the command. You're likely to run that command in the same directory in the future.
* What commands you typed before the command (e.g., the command's execution context).
* How often you run the command.
* When you last ran the command.
* If you've selected the command in McFly before.
* The command's historical exit status. You probably don't want to run old failed commands.

## Installation

### Install with Homebrew (on OS X or Linux)

1. Install the tap:
    ```bash
    brew tap cantino/mcfly
    ```
1. Install `mcfly`:
    ```bash
    brew install cantino/mcfly/mcfly
    ```
1. Add the following to the end of your `~/.bashrc`, `~/.zshrc`, or `~/.config/fish/config.fish` file:

    Bash:
    ```bash
    eval "$(mcfly init bash)"
    ```

    Zsh:
    ```bash
    eval "$(mcfly init zsh)"
    ```

    Fish:
    ```bash
    mcfly init fish | source
    ```
1. Run `. ~/.bashrc` / `. ~/.zshrc` / `source ~/.config/fish/config.fish` or restart your terminal emulator.

#### Uninstalling with Homebrew

1. Remove `mcfly`:
    ```bash
    brew uninstall mcfly
    ```
1. Remove the tap:
    ```bash
    brew untap cantino/mcfly
    ```
1. Remove the lines you added to `~/.bashrc` / `~/.zshrc` / `~/.config/fish/config.fish`.

### Install with MacPorts (on OS X)

1. Update the ports tree
    ```bash
    sudo port selfupdate
    ```
1. Install `mcfly`:
    ```bash
    sudo port install mcfly
    ```
1. Add the following to the end of your `~/.bashrc`, `~/.zshrc`, or `~/.config/fish/config.fish` file, as appropriate:

    Bash:
    ```bash
    eval "$(mcfly init bash)"
    ```

    Zsh:
    ```bash
    eval "$(mcfly init zsh)"
    ```

    Fish:
    ```bash
    mcfly init fish | source
    ```
1. Run `. ~/.bashrc` / `. ~/.zshrc` / `source ~/.config/fish/config.fish` or restart your terminal emulator.

#### Uninstalling with MacPorts

1. Remove `mcfly`:
    ```bash
    sudo port uninstall mcfly
    ```
1. Remove the lines you added to `~/.bashrc` / `~/.zshrc` / `~/.config/fish/config.fish`.

### Installing using our install script

1. `curl -LSfs https://raw.githubusercontent.com/cantino/mcfly/master/ci/install.sh | sh -s -- --git cantino/mcfly`

2. Add the following to the end of your `~/.bashrc`, `~/.zshrc`, or `~/.config/fish/config.fish` file, respectively:

   Bash:

   ```bash
   eval "$(mcfly init bash)"
   ```

   Zsh:

   ```bash
   eval "$(mcfly init zsh)"
   ```

   Fish:

   ```bash
   mcfly init fish | source
   ```

3. Run `. ~/.bashrc` / `. ~/.zshrc` / `source ~/.config/fish/config.fish` or restart your terminal emulator.

### Installing manually from GitHub

1. Download the [latest release from GitHub](https://github.com/cantino/mcfly/releases).
1. Install to a location in your `$PATH`. (For example, you could create a directory at `~/bin`, copy `mcfly` to this location, and add `export PATH="$PATH:$HOME/bin"` to your `.bashrc` / `.zshrc`, or run `set -Ua fish_user_paths "$HOME/bin"` for fish.)
1. Add the following to the end of your `~/.bashrc`, `~/.zshrc`, or `~/.config/fish/config.fish` file, respectively:

    Bash:
    ```bash
    eval "$(mcfly init bash)"
    ```

    Zsh:
    ```bash
    eval "$(mcfly init zsh)"
    ```

    Fish:
    ```bash
    mcfly init fish | source
    ```
1. Run `. ~/.bashrc` / `. ~/.zshrc` / `source ~/.config/fish/config.fish` or restart your terminal emulator.

### Install manually from source

1. [Install Rust 1.40 or later](https://www.rust-lang.org/tools/install)
1. Run `git clone https://github.com/cantino/mcfly` and `cd mcfly`
1. Run `cargo install --path .`
1. Ensure `~/.cargo/bin` is in your `$PATH`.
1. Add the following to the end of your `~/.bashrc`, `~/.zshrc`, or `~/.config/fish/config.fish` file, respectively:

    Bash:
    ```bash
    eval "$(mcfly init bash)"
    ```

    Zsh:
    ```bash
    eval "$(mcfly init zsh)"
    ```

    Fish:
    ```bash
    mcfly init fish | source
    ```
1. Run `. ~/.bashrc` / `. ~/.zshrc` / `source ~/.config/fish/config.fish` or restart your terminal emulator.

### Install by [Zinit](https://github.com/zdharma-continuum/zinit)

* Add below code to your zshrc.

    ```zsh
    zinit ice lucid wait"0a" from"gh-r" as"program" atload'eval "$(mcfly init zsh)"' 
    zinit light cantino/mcfly 
    ```
* It will download mcfly and install for you.
* `$(mcfly init zsh)` will be executed after prompt

## iTerm2

To avoid McFly's UI messing up your scrollback history in iTerm2, make sure this option is unchecked:

<img src="/docs/iterm2.jpeg" alt="iterm2 UI instructions">

## Settings
A number of settings can be set via environment variables. To set a setting you should add the following snippets to your `~/.bashrc` / `~/.zshrc` / `~/.config/fish/config.fish`.

### Light Mode
To swap the color scheme for use in a light terminal, set the environment variable `MCFLY_LIGHT`.

bash / zsh:
```bash
export MCFLY_LIGHT=TRUE
```

fish:
```bash
set -gx MCFLY_LIGHT TRUE
```

Tip: on macOS you can use the following snippet for color scheme to be configured based on system-wide settings:

bash / zsh:
```bash
if [[ "$(defaults read -g AppleInterfaceStyle 2&>/dev/null)" != "Dark" ]]; then
    export MCFLY_LIGHT=TRUE
fi
```


### VIM Key Scheme
By default Mcfly uses an `emacs` inspired key scheme. If you would like to switch to the `vim` inspired key scheme, set the environment variable `MCFLY_KEY_SCHEME`.

bash / zsh:
```bash
export MCFLY_KEY_SCHEME=vim
```

fish:
```bash
set -gx MCFLY_KEY_SCHEME vim
```

### Fuzzy Searching
To enable fuzzy searching, set `MCFLY_FUZZY` to an integer. 0 is off; higher numbers weight toward shorter matches. Values in the 2-5 range get good results so far; try a few and [report what works best for you](https://github.com/cantino/mcfly/issues/183)!

bash / zsh:
```bash
export MCFLY_FUZZY=2
```

fish:
```bash
set -gx MCFLY_FUZZY 2
```

### Results Count
To change the maximum number of results shown, set `MCFLY_RESULTS` (default: 10).

bash / zsh:
```bash
export MCFLY_RESULTS=50
```

fish:
```bash
set -gx MCFLY_RESULTS 50
```

### Interface view
To change interface view, set `MCFLY_INTERFACE_VIEW` (default: `TOP`).
Available options: `TOP` and `BOTTOM`

bash / zsh:
```bash
export MCFLY_INTERFACE_VIEW=BOTTOM
```

fish:
```bash
set -gx MCFLY_INTERFACE_VIEW BOTTOM
```

### Disable menu interface
To disable the menu interface, set the environment variable `MCFLY_DISABLE_MENU`.

bash / zsh:
```bash
export MCFLY_DISABLE_MENU=TRUE
```

fish:
```bash
set -gx MCFLY_DISABLE_MENU TRUE
```

### Results sorting
To change the sorting of results shown, set `MCFLY_RESULTS_SORT` (default: RANK).
Possible values `RANK` and `LAST_RUN`

bash / zsh:
```bash
export MCFLY_RESULTS_SORT=LAST_RUN
```

fish:
```bash
set -gx MCFLY_RESULTS_SORT LAST_RUN
```

### Database Location

McFly stores its SQLite database in the standard location for the OS. On OS X, this is in `~/Library/Application Support/McFly` and on Linux it is in `$XDG_DATA_DIR/mcfly/history.db` (default would be `~/.local/share/mcfly/history.db`). For legacy support, if `~/.mcfly/` exists, it is used instead.

### Slow startup

If you have a very large history database and you notice that McFly launches slowly, you can set `MCFLY_HISTORY_LIMIT` to something like 10000 to limit how many records are considered when searching. In this example, McFly would search only the latest 10,000 entries.

## Possible Future Features

* Add a screencast to README.
* Learn common command options and autocomplete them in the suggestion UI?
* Sort command line args when coming up with the template matching string.
* Possible prioritization improvements:
  * Cross validation & explicit training set selection.
  * Learn command embeddings

## Development

### Contributing

Contributions and bug fixes are encouraged! However, we may not merge PRs that increase complexity significantly beyond what is already required to maintain the project. If you're in doubt, feel free to open an issue and ask.

### Running tests

`cargo test`

### Releasing (notes for @cantino)

1. Edit `Cargo.toml` and bump the version.
1. Edit CHANGELOG.txt
1. Run `cargo clippy` and `cargo fmt`.
1. Recompile (`cargo build`).
1. `git add -p`
1. `git ci -m 'Bumping to vx.x.x'`
1. `git tag vx.x.x`
1. `git push origin head --tags`
1. Let the build finish.
1. Edit the new Release on Github.
1. Edit `pkg/brew/mcfly.rb` and update the version and SHAs. (`shasum -a 256 ...`)
1. Edit `../homebrew-mcfly/pkg/brew/mcfly.rb` too.
  1. `cp pkg/brew/mcfly.rb ../homebrew-mcfly/pkg/brew/mcfly.rb`
  1. Compare with `diff ../homebrew-mcfly/pkg/brew/mcfly.rb ../mcfly/pkg/brew/mcfly.rb ; diff ../homebrew-mcfly/HomebrewFormula/mcfly.rb ../mcfly/HomebrewFormula/mcfly.rb`
1. `git add -p && git ci -m 'Update homebrew' && git push`
1. `cd ../homebrew-mcfly && git add -p && git ci -m 'Update homebrew' && git push && cd ../mcfly`
1. `cargo publish`
