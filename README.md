# McFly - fly through your shell history

> NOTE: This open source project is pre-alpha. It works-- I'm using it every day-- but I haven't trained the prioritization linear perceptron yet because I'm still gathering training data, so the ordering is suboptimal.

<img src="/docs/screenshot.png" alt="screenshot" width="400">

## Features

* Rebinds `CTRL-R` to bring up a full-screen reverse history search with very smart prioritization.
* Augments your shell history to track return status, timestamp, and execution directory.
* Unicode support throughout.
* Written in Rust, so it's super fast.

## Prioritization

The key feature of McFly is smart command prioritization. The goal is for the command you want
to run to be as close to the first suggestion as possible.

When suggesting a command, McFly takes into consideration:

* The directory where you ran the command. You're more likely to run the same command in the same directory in the future.
* What commands you typed  before the command (e.g., the command's context).
* How often you run the command.
* When you ran the command.
* The command's exit status. You probably don't want to run old failed commands.

## Installation

### Compile it yourself

1. [Install Rust](https://www.rust-lang.org/en-US/install.html)
1. Compile with optimizations
    ```bash
    cargo build --release
    ```
1. Copy `./target/release/mcfly` into a location in your `$PATH`.

### Enable in your shell

#### Bash

Add `. /path/to/this/repository/mcfly-bash.sh` to your `~/.bashrc`.

## TODO

* Make score be dependent on position in the top N suggestions, maybe scalled by index?
* Weird history issues between windows
* Make context look at first N letters instead of full commands, or maybe ignore stuff in quotes?
* Add screencast.

