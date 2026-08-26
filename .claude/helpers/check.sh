#!/usr/bin/env bash
# Run exactly what CI runs, in the same order, and stop at the first failure.
#
# CI is three workflows:
#   .github/workflows/clippy.yml   -> cargo clippy -- -D warnings
#   .github/workflows/rustfmt.yml  -> cargo fmt --check
#   .github/workflows/mean_bean_ci.yml -> build + test across targets
#
# Note clippy.yml uses `cargo clippy -- -D warnings`, which does NOT lint test
# code. Use --all-targets locally to also lint #[cfg(test)] modules; a warning
# there will not fail CI but is still worth fixing.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"

fail=0
run() {
  echo "=== $* ==="
  if "$@"; then echo "  PASS"; else echo "  FAIL"; fail=1; fi
}

run cargo fmt --all -- --check
run cargo clippy --all-targets -- -D warnings
run cargo test

echo
[ "$fail" -eq 0 ] && echo "All CI checks passed." || echo "Some checks FAILED (see above)."
exit "$fail"
