#!/bin/zsh

# Ensure stdin is a tty
[[ -t 0 ]] || return

# Avoid loading this file more than once
if [[ "$__MCFLY_LOADED" == "loaded" ]]; then
  return 0
fi
__MCFLY_LOADED="loaded"

# Ensure HISTFILE exists.
export HISTFILE="${HISTFILE:-$HOME/.zsh_history}"
if [[ ! -r "${HISTFILE}" ]]; then
  echo "McFly: ${HISTFILE} does not exist or is not readable. Please fix this or set HISTFILE to something else before using McFly."
  return 1
fi

# MCFLY_SESSION_ID is used by McFly internally to keep track of the commands from a particular terminal session.
export MCFLY_SESSION_ID=$(dd if=/dev/urandom bs=256 count=1 2> /dev/null | env LC_ALL=C tr -dc 'a-zA-Z0-9' | head -c 24)

# Required for commented out mcfly search commands to work.
setopt interactive_comments   # allow comments in interactive shells (like Bash does)

# Setup a function to clear the shell history.
function erase_history { local HISTSIZE=0; }

# Setup a function to be used by $PROMPT_COMMAND.
function mcfly_prompt_command {
  local exit_code=$? # Record exit status of previous command.

  # Populate McFly's temporary, per-session history file from recent commands in the shell's primary HISTFILE.
  if [[ ! -f "${MCFLY_HISTORY}" ]]; then
    export MCFLY_HISTORY=$(mktemp -t mcfly.XXXXXXXX)
    fc -ln 2>/dev/null >| ${MCFLY_HISTORY}
  fi

  if [[ $(fc -Iln 2>/dev/null | tail -n1) = ' mcfly search'* ]] ; then
    # If the most recent history item is a mcfly search, do not update $MCFLY_HISTORY.
    [ -n "$MCFLY_DEBUG" ] && echo "mcfly.zsh: Not appending 'mcfly search' to $MCFLY_HISTORY."
  else
    # Write history to $MCFLY_HISTORY.
    fc -W "${MCFLY_HISTORY}"
  fi

  # Run mcfly with the saved code. It will:
  # * append commands to $HISTFILE, (~/.zsh_history by default)
  #   for backwards compatibility and to load in new terminal sessions;
  # * find the text of the last command in $MCFLY_HISTORY and save it to the database.
  [ -n "$MCFLY_DEBUG" ] && echo "mcfly.zsh: Run mcfly add --exit ${exit_code}"
  mcfly add --exit ${exit_code}
  # Clear the in-memory history and reload it from $MCFLY_HISTORY
  # (to remove instances of '#mcfly: ' from the local session history).
  erase_history
  fc -R "${MCFLY_HISTORY}"
  return ${exit_code} # Restore the original exit code by returning it.
}
precmd_functions+=(mcfly_prompt_command)

# Avoid logging #mcfly commands or commands starting with a space to the shell history (this emulates hist_ignore_space).
hist_filter() {
  emulate -L zsh
  local hist_cmd=${1%%$'\n'}
  if [[ $hist_cmd = '#mcfly:'* ]] || [[ $hist_cmd =~ "^ " ]] ; then
    [ -n "$MCFLY_DEBUG" ] && echo "mcfly.zsh: Not saving '$hist_cmd'"
    return 2
  else
    [ -n "$MCFLY_DEBUG" ] && echo "mcfly.zsh: Saving '$hist_cmd'"
    return 0
  fi
}
zshaddhistory_functions+=(hist_filter)

# Cleanup $MCFLY_HISTORY tmp files on exit.
exit_logger() {
  [ -n "$MCFLY_DEBUG" ] && echo "mcfly.zsh: Exiting and removing $MCFLY_HISTORY"
  rm -f $MCFLY_HISTORY
}
zshexit_functions+=(exit_logger)

# If this is an interactive shell, take ownership of ctrl-r.
# The logic here is:
#   1. Jump to the beginning of the edit buffer, add 'mcfly: ', and comment out the current line. We comment out the line
#      to ensure that all possible special characters, including backticks, are ignored. This commented out line will
#      end up as the most recent entry in the $MCFLY_HISTORY file.
#   2. Type "mcfly search" and then run the command. McFly will pull the last line from the $MCFLY_HISTORY file,
#      which should be the commented-out search from step #1. It will then remove that line from the history file and
#      render the search UI pre-filled with it.
if [[ $- =~ .*i.* ]]; then
  if set -o | grep "vi " | grep -q on; then
    bindkey -s '\C-r' '\e0i#mcfly: \e\C-j mcfly search\C-j'
  else
    bindkey -s '\C-r' '\C-a#mcfly: \C-j mcfly search\C-j'
  fi
fi
