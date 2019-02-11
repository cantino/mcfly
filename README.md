[![Build Status](https://travis-ci.org/cantino/mcfly.svg?branch=master)](https://travis-ci.org/cantino/mcfly)
[![](https://img.shields.io/crates/v/mcfly.svg)](https://crates.io/crates/mcfly)

# McFly - fly through your shell history

<img src="/docs/screenshot.png" alt="screenshot" width="400">

McFly replaces your default `ctrl-r` Bash history search with an intelligent search engine that takes into account
your working directory and the context of recently executed commands. McFly's suggestions are prioritized
in real time with a small neural network.
 
TL;DR: an upgraded `ctrl-r` for Bash whose history results make sense for what you're working on right now.

## Features

* Rebinds `ctrl-r` to bring up a full-screen reverse history search prioritized with a small neural network.
* Augments your shell history to track command exit status, timestamp, and execution directory in a SQLite database.
* Maintains your normal Bash history file as well so that you can stop using McFly whenever you want.
* Unicode support throughout.
* Includes a simple action to scrub any history item from the McFly database and your shell history files.
* Designed to be extensible for other shells in the future.
* Written in Rust, so it's fast and safe.

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
    brew tap cantino/mcfly https://github.com/cantino/mcfly
    ```
1. Install `mcfly`:
    ```bash
    brew install mcfly
    ```
1. Add the following to the end of your `~/.bashrc` file:
    ```bash
    if [[ -r "$(brew --prefix)/opt/mcfly/mcfly.bash" ]]; then
      source "$(brew --prefix)/opt/mcfly/mcfly.bash"
    fi
    ```
1. Run `. ~/.bashrc` or restart your terminal emulator.

#### Uninstalling with Homebrew

1. Remove `mcfly`:
    ```bash
    brew uninstall mcfly
    ```
1. Remove the tap:
    ```bash
    brew untap cantino/mcfly
    ```
1. Remove the lines you added to `~/.bashrc`.

### Installing manually from GitHub

1. Download the [latest release from GitHub](https://github.com/cantino/mcfly/releases).
1. Install to a location in your `$PATH`. (For example, you could create a directory at `~/bin`, copy `mcfly` to this location, and add `export PATH="$PATH:$HOME/bin"` to your `.bashrc`.)
1. Copy `mcfly.bash` to a known location.
1. Add the following to the end of your `~/.bashrc` file:
    ```bash
    if [[ -r /path/to/mcfly.bash ]]; then
      source /path/to/mcfly.bash
    fi
    ```
1. Run `. ~/.bashrc` or restart your terminal emulator.

### Install manually from source

1. [Install Rust 1.29 or later](https://www.rust-lang.org/tools/install)
1. Run `git clone https://github.com/cantino/mcfly` and `cd mcfly`
1. Run `cargo install --path .`
1. Ensure `~/.cargo/bin` is in your `$PATH`.
1. Add the following to the end of your `~/.bashrc` file:
    ```bash
    if [[ -r /path/to/mcfly.bash ]]; then
      source /path/to/mcfly.bash
    fi
    ```
1. Run `. ~/.bashrc` or restart your terminal emulator.

## iTerm2

To avoid McFly's UI messing up your scrollback history in iTerm2, make sure this option is unchecked:

<img src="/docs/iterm2.jpeg" alt="iterm2 UI instructions">

## Light Mode

To swap the color scheme for use in a light terminal, set the environment variable `MCFLY_LIGHT`.

For example, add the following to your `~/.bash_profile`:

```bash
export MCFLY_LIGHT=TRUE
```

## Possible Future Features

* Add a screencast to README.
* Learn common command options and autocomplete them in the suggestion UI?
* Sort command line args when coming up with the template matching string.
* Possible prioritization improvements:
  * Cross validation & explicit training set selection.
  * Learn command embeddings

## Development

### Running tests

`cargo test`

### Releasing

1. Edit `Cargo.toml` and bump the version.
1. Recompile.
1. `git ci -m 'Bumping to vx.x.x'`
1. `git tag vx.x.x`
1. `git push origin head --tags`
1. Let the build finish.
1. Edit the new Release on Github.
1. Edit `pkg/brew/mcfly.rb` and update the version and SHAs. (`shasum -a 256 ...`)
