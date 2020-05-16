#!/bin/bash
# Build mcfly and run a dev environment bash for local mcfly testing

this_dir=$(cd `dirname "$0"`; pwd)

rm -r target/debug/deps/mcfly-*
cargo build
# For some reason, to get line numbers in backtraces, we have to run the binary directly.
MCFLY_PATH=$(find target/debug/deps/mcfly-* -maxdepth 1 -type f | grep -v '\.d') \
  RUST_BACKTRACE=full \
  MCFLY_DEBUG=1 \
  exec /bin/bash --init-file "$this_dir/mcfly.bash" -i
