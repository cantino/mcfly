#!/bin/bash

# Avoid loading this file more than once
if [[ "$__mcfly_loaded" == "loaded" ]]; then
  return 0
fi
__mcfly_loaded="loaded"

mcfly_cmd() {
  if [[ "$MCFLY_DEBUG" = "1" ]]; then
    echo "RUST_BACKTRACE=1 mcfly"
  else
    echo "mcfly"
  fi
}

# Ignore duplicate commands or those with a leading space
# ("ignoreboth" is the same as "ignorespace:ignoredups")
export HISTCONTROL="ignoreboth" 

# Append new history items to .bash_history
shopt -s histappend

# Set $PROMPT_COMMAND to do the following:
#   1. record exit status of previous command
#   2. write history to ~/.bash_history
#   3. run mcfly, telling it the exit status (it will find the last command in the history)
#   4. clear the in-memory history and reload it from disk
#   5. run whatever was already in the $PROMPT_COMMAND
export PROMPT_COMMAND="__last_exit=\$?;history -a;$(mcfly_cmd) add --exit \$__last_exit;history -c;history -r;${PROMPT_COMMAND}"

# If this is an interactive shell, take ownership of ctrl-r.
# The logic here is:
#   1. Jump to the beginning of the edit buffer and comment out the current line. We comment out the line to ensure that
#      all possible special characters, including backticks, are ignored. This commented out line will end up as the most
#      recent entry in the user's .bash_history file.
#   2. Type "mcfly search" and then run the command. McFly will pull the last line from the .bash_history file,
#      which should be the commented-out search from step #1. It will then remove that line from the history file and
#      render the search UI pre-filled.
if [[ $- =~ .*i.* ]]; then
  bind "'\C-r': '\C-a\e# $(mcfly_cmd) search\C-j'"
fi

