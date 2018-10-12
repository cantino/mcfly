[![Build Status](https://travis-ci.org/cantino/mcfly.svg?branch=master)](https://travis-ci.org/cantino/mcfly)

# McFly - fly through your shell history

> NOTE: This open source project is pre-alpha. It works-- I'm using it every day-- but I haven't trained the prioritization linear perceptron yet because I'm still gathering training data, so the ordering is suboptimal.

<img src="/docs/screenshot.png" alt="screenshot" width="400">

## Features

* Rebinds `CTRL-R` to bring up a full-screen reverse history search with very smart prioritization.
* Augments your shell history to track return status, timestamp, and execution directory.
* Unicode support throughout.
* Also writes to your existing shell history file so you can stop using McFly whenever you want.
* Simple command to scrub a history item from the database and shell history.
* Designed to be extensible in the future for other shells.
* Written in Rust, so it's fast and reliable.

## Prioritization

The key feature of McFly is smart command prioritization. The goal is for the command you want
to run to always be one of the suggestions.

When suggesting a command, McFly takes into consideration:

* The directory where you ran the command. You're more likely to run the same command in the same directory in the future.
* What commands you typed before the command (e.g., the command's context).
* How often you run the command.
* When you last ran the command.
* The command's historical exit status. You probably don't want to run old failed commands.

## Installation

### Install with Homebrew

1. Install the tap:
    ```bash
    brew tap cantino/mcfly https://github.com/cantino/mcfly
    ```
1. Install `mcfly`:
    ```bash
    brew install mcfly
    ```
1. Add the following to your `~/.bash_profile`:
    ```bash
    if [ -f $(brew --prefix mcfly)/mcfly-bash.sh ]; then
      . $(brew --prefix mcfly)/mcfly-bash.sh
    fi
    ```

#### Uninstalling with Homebrew

1. Remove `mcfly`:
    ```bash
    brew uninstall mcfly
    ```
1. Remove the tap:
    ```bash
    brew untap cantino/mcfly
    ```
1. Remove the lines you added to `~/.bash_profile`.

### Installing manually from GitHub

1. Download the [latest release from GitHub](https://github.com/cantino/mcfly/releases).
1. Install to a location in your `$PATH`. (For example, you could create a directory at `~/bin`, copy `mcfly` to this location, and add `export PATH="$PATH:$HOME/bin"` to your `.bash_profile`.)
1. Copy `mcfly-bash.sh` to a known location.
1. Add the following to your `~/.bash_profile`:
    ```bash
    if [ -f /path/to/mcfly-bash.sh ]; then
      . /path/to/mcfly-bash.sh
    fi
    ```

### Install manually from source

1. [Install Rust 1.29 or later](https://www.rust-lang.org/en-US/install.html)
1. Compile with optimizations
    ```bash
    cargo build --release
    ```
1. Copy `./target/release/mcfly` into a location in your `$PATH`. (For example, you could create a directory at `~/bin`
and add `export PATH="$PATH:$HOME/bin"` to your `.bash_profile`.)
1. Add the following to your `~/.bash_profile`:
    ```bash
    if [ -f /path/to/mcfly-bash.sh ]; then
      . /path/to/mcfly-bash.sh
    fi
    ```

## iTerm2

To avoid McFly's UI messing up your scrollback history in iTerm2, make sure this option is unchecked:

<img src="/docs/iterm2.jpeg" alt="iterm2 UI instructions">

## Future / Upcoming Features

* Add screencast to README.
* Add keyboard shortcut to delete something from the history / make it not get suggested.
* Better prioritization:
  * Finish training the linear perceptron after gathering more selection data.
  * Learn embeddings per template and use to predict the next embedding, then do approximate nearest neighbor lookup?
    * Could train by predicting whether or not one command should follow another and doing gradient descent.
* Learn common command options and autocomplete them in the suggestion UI?
* Sort command line args when coming up with the template matching string.
* Allow suggesting of the last command seen if it's been a while.
* Detect folder renames?
