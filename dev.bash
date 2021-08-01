#!/bin/bash
# Build mcfly and run a dev environment bash for local mcfly testing

if ! this_dir=$(cd "$(dirname "$0")" && pwd); then
    exit $?
fi

rm -f target/debug/mcfly
cargo build
# For some reason, to get line numbers in backtraces, we have to run the binary directly.
HISTFILE=$HOME/.bash_history \
  MCFLY_PATH=target/debug/mcfly \
  RUST_BACKTRACE=full \
  MCFLY_DEBUG=1 \
  PATH=target/debug/:$PATH \
  exec /bin/bash --init-file "$this_dir/mcfly.bash" -i
