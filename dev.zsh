#!/bin/bash
# Build mcfly and run a dev environment zsh for local mcfly testing

this_dir=$(cd `dirname "$0"`; pwd)

cargo build
PATH="$PATH:$this_dir/target/debug" MCFLY_DEBUG=1 ZDOTDIR="$this_dir" /bin/zsh -i