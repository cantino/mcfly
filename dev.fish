#!/bin/bash
# Build mcfly and run a dev environment fish for local mcfly testing

this_dir=$(cd `dirname "$0"`; pwd)

# Setup for local testing.
mkdir -p $this_dir/.fish

rm -f target/debug/mcfly
cargo build
# For some reason, to get line numbers in backtraces, we have to run the binary directly.
XDG_DATA_HOME=$this_dir/.fish \
  MCFLY_PATH=target/debug/mcfly \
  RUST_BACKTRACE=full \
  MCFLY_DEBUG=1 \
  PATH=target/debug/:$PATH \
  exec /usr/bin/env fish -i --init-command "source $this_dir/mcfly.fish; and mcfly_key_bindings"
