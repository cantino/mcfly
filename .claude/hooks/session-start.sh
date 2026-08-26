#!/bin/bash
# SessionStart hook for mcfly.
#
# The Rust toolchain (cargo, clippy, rustfmt) is already present in the web
# image, so this hook is not about installing it. It fixes the two things that
# actually cost time when working on this repo in a fresh remote session:
#
#   1. The repo is cloned shallow. `git log -S`, `git blame` and `git log -L`
#      silently return WRONG answers against a shallow clone -- they attribute
#      changes to the graft-boundary commit instead of the real one. Deepening
#      the clone up front avoids drawing false conclusions from history.
#   2. A cold `cargo build` recompiles every dependency (rusqlite is bundled and
#      builds SQLite from C source). Warming the cache here means `cargo test`
#      and `cargo clippy` are fast for the rest of the session.
set -euo pipefail

# Only do this work in Claude Code on the web; local checkouts are already set up.
if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
cd "$PROJECT_DIR"

# --- 1. Deepen the shallow clone so git history tools are trustworthy ---------
# Non-fatal: if the network is restricted we still want the session to start.
if [ -f .git/shallow ]; then
  echo "session-start: deepening shallow clone so git log/blame are accurate..."
  git fetch --unshallow --quiet 2>/dev/null \
    || git fetch --deepen=1000 --quiet 2>/dev/null \
    || echo "session-start: WARNING: could not unshallow; git log -S and git blame may report the graft commit instead of the real one."
fi

# --- 2. Warm the build cache -------------------------------------------------
# --tests builds the unit-test binaries too, so `cargo test` afterwards is
# near-instant. Non-fatal so a transient registry failure can't block startup.
echo "session-start: warming cargo build cache (this is cached in the container image)..."
cargo build --bins --tests --quiet 2>&1 | tail -5 || echo "session-start: WARNING: cargo build --bins --tests failed; run it manually to see the error."

echo "session-start: ready. CI runs exactly three checks -- see .claude/helpers/check.sh"
