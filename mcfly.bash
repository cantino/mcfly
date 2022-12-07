#!/bin/bash

# Ensure stdin is a tty
# Avoid loading this file more than once
if [[ -t 0 ]] && [[ "$__MCFLY_LOADED" != "loaded" ]]; then
  __MCFLY_LOADED="loaded"

  # Ensure HISTFILE exists.
  export HISTFILE="${HISTFILE:-$HOME/.bash_history}"
  if [[ ! -r "${HISTFILE}" ]]; then
    echo "McFly: ${HISTFILE} does not exist or is not readable. Please fix this or set HISTFILE to something else before using McFly."
    return 1
  fi

  # MCFLY_SESSION_ID is used by McFly internally to keep track of the commands from a particular terminal session.
  MCFLY_SESSION_ID="$(command dd if=/dev/urandom bs=256 count=1 2> /dev/null | LC_ALL=C command tr -dc 'a-zA-Z0-9' | command head -c 24)"
  export MCFLY_SESSION_ID

  # Find the binary
  MCFLY_PATH=${MCFLY_PATH:-$(command -v mcfly)}
  if [ -z "$MCFLY_PATH" ]; then
    echo "Cannot find the mcfly binary, please make sure that mcfly is in your path before sourcing mcfly.bash."
    return 1
  fi

  # Ignore commands with a leading space
  export HISTCONTROL="${HISTCONTROL:-ignorespace}"

  # Append new history items to .bash_history
  shopt -s histappend

  # Setup a function to be used by $PROMPT_COMMAND.
  function mcfly_prompt_command {
    local exit_code=$? # Record exit status of previous command.

    # Populate McFly's temporary, per-session history file from recent commands in the shell's primary HISTFILE.
    if [[ ! -f "${MCFLY_HISTORY}" ]]; then
      MCFLY_HISTORY=$(mktemp ${TMPDIR:-/tmp}/mcfly.XXXXXXXX)
      export MCFLY_HISTORY
      command tail -n100 "${HISTFILE}" >| "${MCFLY_HISTORY}"
    fi

    history -a "${MCFLY_HISTORY}" # Append history to $MCFLY_HISTORY.
    # Run mcfly with the saved code. It will:
    # * append commands to $HISTFILE, (~/.bash_history by default)
    #   for backwards compatibility and to load in new terminal sessions;
    # * find the text of the last command in $MCFLY_HISTORY and save it to the database.
    $MCFLY_PATH add --exit ${exit_code} --append-to-histfile
    # Clear the in-memory history and reload it from $MCFLY_HISTORY
    # (to remove instances of '#mcfly: ' from the local session history).
    history -cr "${MCFLY_HISTORY}"
    return ${exit_code} # Restore the original exit code by returning it.
  }

  # Set $PROMPT_COMMAND run mcfly_prompt_command, preserving any existing $PROMPT_COMMAND.
  if [ -z "$PROMPT_COMMAND" ]
  then
    PROMPT_COMMAND="mcfly_prompt_command"
  elif [[ ! "$PROMPT_COMMAND" =~ "mcfly_prompt_command" ]]
  then
    PROMPT_COMMAND="mcfly_prompt_command;${PROMPT_COMMAND#;}"
  fi

  # If this is an interactive shell, take ownership of ctrl-r.
  if [[ $- =~ .*i.* ]]; then

    # Function to perform ctrl-r binding for fzf. Adapted from junegunn/fzf shell/key-bindings.bash
    __mcfly_fzf_history__() {
      local output opts script header
      opts="--height ${FZF_TMUX_HEIGHT:-40%} --bind=ctrl-z:ignore $FZF_DEFAULT_OPTS 
          --nth=2.. --delimiter='\t' --no-hscroll --tiebreak=index --read0 --layout reverse 
          --header='F1 Sort Rank | F2 Sort Time | Ctrl-R Toggle Sort'
          --bind=ctrl-r:toggle-sort 
          --bind='f1:reload(mcfly fzf -0 --sort RANK)' 
          --bind='f2:reload(mcfly fzf -0 --sort LAST_RUN)' 
          $FZF_CTRL_R_OPTS +m"
      output=$(
        mcfly fzf -0 |
          FZF_DEFAULT_OPTS="$opts" fzf --query "$READLINE_LINE"
      ) || return
      READLINE_LINE=${output#*$'\t'}
      if [[ -z "$READLINE_POINT" ]]; then
        echo "$READLINE_LINE"
      else
        READLINE_POINT=0x7fffffff
      fi
    }

    if [ -z "$MCFLY_FZF" ]; then
      # Perform standard ctrl-r bindings
      if [[ ${BASH_VERSINFO[0]} -ge 4 ]]; then
        # shellcheck disable=SC2016
        bind -x '"\C-r": "echo \#mcfly: ${READLINE_LINE[@]} >> $MCFLY_HISTORY ; READLINE_LINE= ; mcfly search"'
      else
        # The logic here is:
        #   1. Jump to the beginning of the edit buffer, add 'mcfly: ', and comment out the current line. We comment out the line
        #      to ensure that all possible special characters, including backticks, are ignored. This commented out line will
        #      end up as the most recent entry in the $MCFLY_HISTORY file.
        #   2. Type "mcfly search" and then run the command. McFly will pull the last line from the $MCFLY_HISTORY file,
        #      which should be the commented-out search from step #1. It will then remove that line from the history file and
        #      render the search UI pre-filled with it.
        if set -o | grep "vi " | grep -q on; then
          bind "'\C-r': '\e0i#mcfly: \e\C-m mcfly search\C-m'"
        else
          bind "'\C-r': '\C-amcfly: \e# mcfly search\C-m'"
        fi
      fi
    else 
      bind -x '"\C-r": __mcfly_fzf_history__'
    fi
  fi

fi
