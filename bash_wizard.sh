#!/bin/bash

# Avoid duplicate inclusion
if [[ "$__bw_loaded" == "loaded" ]]; then
  return 0
fi
__bw_loaded="loaded"

export HISTCONTROL="ignoreboth" # leading space hides commands from history
shopt -s histappend             # append new history items to .bash_history
export PROMPT_COMMAND="__last_exit=\$?;history -a;bash_wizard add --exit \$__last_exit;${PROMPT_COMMAND}"

# If interactive shell, bind to ctrl-r.
if [[ $- =~ .*i.* ]]; then
  bind '"\C-r": "\C-a bash_wizard search -- \C-j"'
fi

