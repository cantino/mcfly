#!/usr/bin/env bash
# Run the mcfly binary against a throwaway history DB.
#
# Why this exists: `mcfly add` panics unless several env vars are set, and it
# will happily write to your REAL history DB if they are not isolated. Getting
# this wrong costs a few rounds of confusing stack traces:
#
#   * the flag is --dir, NOT --directory
#   * MCFLY_HISTORY must point at a file, or settings.rs panics
#   * HISTFILE must be set and readable, or the first-run shell-history import
#     panics inside from_shell_history()
#   * HOME/XDG_DATA_HOME must be redirected, or you pollute the real
#     ~/.local/share/mcfly/history.db
#
# Usage:
#   .claude/helpers/mcfly-sandbox.sh reset                 # start clean
#   .claude/helpers/mcfly-sandbox.sh add 'git status'      # record a command
#   .claude/helpers/mcfly-sandbox.sh dump                  # list recorded cmds
#   .claude/helpers/mcfly-sandbox.sh run <raw mcfly args>  # anything else
#
# Env passthrough: any MCFLY_* var you set is honored, e.g.
#   MCFLY_SESSION_ID=other .claude/helpers/mcfly-sandbox.sh add 'git status'
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
SANDBOX="${MCFLY_SANDBOX_DIR:-${TMPDIR:-/tmp}/mcfly-sandbox}"
BIN="${MCFLY_BIN:-$ROOT/target/debug/mcfly}"

cmd="${1:-}"; shift || true

if [ "$cmd" = "reset" ]; then
  rm -rf "$SANDBOX"; echo "sandbox reset: $SANDBOX"; exit 0
fi

mkdir -p "$SANDBOX/home/.local/share"
touch "$SANDBOX/mcfly_history" "$SANDBOX/shell_history"

export HOME="$SANDBOX/home"
export XDG_DATA_HOME="$SANDBOX/home/.local/share"
export MCFLY_HISTORY="$SANDBOX/mcfly_history"
export HISTFILE="$SANDBOX/shell_history"
export MCFLY_SESSION_ID="${MCFLY_SESSION_ID:-sandbox}"

DB="$XDG_DATA_HOME/mcfly/history.db"

case "$cmd" in
  add)
    [ -x "$BIN" ] || { echo "missing $BIN -- run: cargo build" >&2; exit 1; }
    "$BIN" add --exit 0 --dir /tmp "$@" 2>&1 | grep -v '^McFly: Importing' || true
    ;;
  dump)
    [ -f "$DB" ] || { echo "(no database yet at $DB)"; exit 0; }
    python3 - "$DB" <<'PY'
import sqlite3, sys
con = sqlite3.connect(sys.argv[1])
rows = list(con.execute("select cmd from commands order by id"))
print(f"{len(rows)} command(s) recorded:")
for (cmd,) in rows:
    print("  ", repr(cmd))
PY
    ;;
  run)
    [ -x "$BIN" ] || { echo "missing $BIN -- run: cargo build" >&2; exit 1; }
    "$BIN" "$@"
    ;;
  *)
    awk 'NR==1{next} /^#/{sub(/^# ?/,""); print; next} {exit}' "$0"
    exit 1
    ;;
esac
