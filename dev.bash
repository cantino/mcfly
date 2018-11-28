#!/bin/bash
# Build mcfly and run a dev environment bash for local mcfly testing

set -e
this_dir=$(cd `dirname "$0"`; pwd)

cargo build
PATH="$PATH:$this_dir/target/debug" \
MCFLY_DEBUG=1 \
    exec /bin/bash --init-file "$this_dir/mcfly.bash" -i
