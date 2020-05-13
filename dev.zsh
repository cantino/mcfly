#!/bin/bash
# Build mcfly and run a dev environment zsh for local mcfly testing

this_dir=$(cd `dirname "$0"`; pwd)

# Setup for local testing.
touch $this_dir/.zsh_history

# Needed so that the test instance of zsh sources the local mcfly.zsh file on startup.
echo "source ./mcfly.zsh" > $this_dir/.zshrc

cargo build
PATH="$PATH:$this_dir/target/debug" MCFLY_DEBUG=1 ZDOTDIR="$this_dir" /bin/zsh -i
