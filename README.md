# McFly - fly through your shell history

## TODO

* Make score be dependent on position in the top N suggestions, maybe scalled by index?
* Weird history issues between windows
* Make context look at first N letters instead of full commands, or maybe ignore stuff in quotes?
* Write README.
* Write blog post and make video.

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
