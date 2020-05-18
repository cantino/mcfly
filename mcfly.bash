#!/bin/bash

# Ensure stdin is a tty
[[ -t 0 ]] || return

# Avoid loading this file more than once
if [[ "$__MCFLY_LOADED" == "loaded" ]]; then
  return 0
fi
__MCFLY_LOADED="loaded"

# Ensure HISTFILE exists.
export HISTFILE="${HISTFILE:-$HOME/.bash_history}"
if [[ ! -r "${HISTFILE}" ]]; then
  echo "McFly: ${HISTFILE} does not exist or is not readable. Please fix this or set HISTFILE to something else before using McFly."
  return 1
fi

# MCFLY_SESSION_ID is used by McFly internally to keep track of the commands from a particular terminal session.
export MCFLY_SESSION_ID=$(dd if=/dev/urandom bs=256 count=1 2> /dev/null | env LC_ALL=C tr -dc 'a-zA-Z0-9' | head -c 24)

# Ignore commands with a leading space
export HISTCONTROL="ignorespace"

# Append new history items to .bash_history
shopt -s histappend

# Setup a function to be used by $PROMPT_COMMAND.
function mcfly_prompt_command {
  local exit_code=$? # Record exit status of previous command.

  # Populate McFly's temporary, per-session history file from recent commands in the shell's primary HISTFILE.
  if [[ ! -f "${MCFLY_HISTORY}" ]]; then
    export MCFLY_HISTORY=$(mktemp -t mcfly.XXXXXXXX)
    tail -n100 "${HISTFILE}" >| ${MCFLY_HISTORY}
  fi

  history -a ${MCFLY_HISTORY} # Append history to $MCFLY_HISTORY.
  # Run mcfly with the saved code. It will:
  # * append commands to $HISTFILE, (~/.bash_history by default)
  #   for backwards compatibility and to load in new terminal sessions;
  # * find the text of the last command in $MCFLY_HISTORY and save it to the database.
  mcfly add --exit ${exit_code} --append-to-histfile
  # Clear the in-memory history and reload it from $MCFLY_HISTORY
  # (to remove instances of '#mcfly: ' from the local session history).
  history -cr ${MCFLY_HISTORY}
  return ${exit_code} # Restore the original exit code by returning it.
}

# Set $PROMPT_COMMAND run mcfly_prompt_command and then any existing $PROMPT_COMMAND.
PROMPT_COMMAND="mcfly_prompt_command;$PROMPT_COMMAND"

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
    bind "'\C-r': '\e0i#mcfly: \e\C-j mcfly search\C-j'"
  else
    bind "'\C-r': '\C-amcfly: \e# mcfly search\C-j'"
  fi
fi
