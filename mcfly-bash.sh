#!/bin/bash

# Avoid loading this file more than once
if [[ "$__mcfly_loaded" == "loaded" ]]; then
  return 0
fi
export __mcfly_loaded="loaded"
export MCFLY_SESSION_ID=$(cat /dev/urandom | env LC_CTYPE=C tr -dc 'a-zA-Z0-9' | fold -w 24 | head -n 1)
export MCFLY_HISTORY=$(mktemp -t mcfly)
export HISTFILE="${HISTFILE:-$HOME/.bash_history}"

if [ -f "$HISTFILE" ];
then
  tail -n100 $HISTFILE > $MCFLY_HISTORY
else
  printf "Welcome to McFly\n" > $MCFLY_HISTORY
fi

mcfly_cmd() {
  if [[ "$MCFLY_DEBUG" = "1" ]]; then
    echo "RUST_BACKTRACE=1 mcfly"
  else
    echo "mcfly"
  fi
}

# Ignore commands with a leading space
export HISTCONTROL="ignorespace"

# Append new history items to .bash_history
shopt -s histappend

# Set $PROMPT_COMMAND to do the following:
#   1. record exit status of previous command
#   2. append history to $MCFLY_HISTORY
#   3. run mcfly
#      a. tell mcfly the exit status
#      b. tell mcfly to append commands to $HISTFILE (~/.bash_history by default) for backwards compatibility and
#         to load in new terminal sessions
#      c. mcfly will find the text of the last command in $MCFLY_HISTORY and save it to the database
#   4. clear the in-memory history and reload it from $MCFLY_HISTORY (to remove instances of '#mcfly: ' from the
#      local session history)
#   5. run whatever was already in $PROMPT_COMMAND
export PROMPT_COMMAND="__last_exit=\$?;history -a \$MCFLY_HISTORY;$(mcfly_cmd) add --exit \$__last_exit --append-to \$HISTFILE;history -cr \$MCFLY_HISTORY;${PROMPT_COMMAND}"

# If this is an interactive shell, take ownership of ctrl-r.
# The logic here is:
#   1. Jump to the beginning of the edit buffer, add 'mcfly: ', and comment out the current line. We comment out the line
#      to ensure that all possible special characters, including backticks, are ignored. This commented out line will
#      end up as the most recent entry in the $MCFLY_HISTORY file.
#   2. Type "mcfly search" and then run the command. McFly will pull the last line from the $MCFLY_HISTORY file,
#      which should be the commented-out search from step #1. It will then remove that line from the history file and
#      render the search UI pre-filled with it.
if [[ $- =~ .*i.* ]]; then
  bind "'\C-r': '\C-amcfly: \e# $(mcfly_cmd) search\C-j'"
fi

