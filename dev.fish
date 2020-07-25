#!/bin/bash
# Build mcfly and run a dev environment fish for local mcfly testing

this_dir=$(cd `dirname "$0"`; pwd)

# Setup for local testing.
mkdir -p $this_dir/.fish

rm -f target/debug/mcfly
rm -rf target/debug/deps/mcfly-*
cargo build
# For some reason, to get line numbers in backtraces, we have to run the binary directly.
XDG_DATA_HOME=$this_dir/.fish \
  MCFLY_PATH=$(find target/debug/deps/mcfly-* -maxdepth 1 -type f | grep -v '\.d') \
  RUST_BACKTRACE=full \
  MCFLY_DEBUG=1 \
  exec /usr/bin/env fish -i --init-command "source $this_dir/mcfly.fish; and mcfly_key_bindings"
