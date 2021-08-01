#!/bin/bash
# Build mcfly and run a dev environment zsh for local mcfly testing

this_dir=$(cd `dirname "$0"`; pwd)

# Setup for local testing.
touch $this_dir/.zsh_history

# Needed so that the test instance of zsh sources the local mcfly.zsh file on startup.
echo "source ./mcfly.zsh" > $this_dir/.zshrc

rm -f target/debug/mcfly
cargo build
# For some reason, to get line numbers in backtraces, we have to run the binary directly.
HISTFILE=$HOME/.zsh_history \
  MCFLY_PATH=target/debug/mcfly \
  RUST_BACKTRACE=full \
  MCFLY_DEBUG=1 \
  ZDOTDIR="$this_dir" \
  PATH=target/debug/:$PATH \
  /bin/zsh -i
