#!/bin/zsh

# Ensure stdin is a tty
if [[ -o interactive ]]; then

  # Setup MCFLY_HISTFILE and make sure it exists.
  export MCFLY_HISTFILE="${HISTFILE:-$HOME/.zsh_history}"
  if [[ ! -r "${MCFLY_HISTFILE}" ]]; then
    echo "McFly: ${MCFLY_HISTFILE} does not exist or is not readable. Please fix this or set HISTFILE to something else before using McFly."
    return 1
  fi

  # MCFLY_SESSION_ID is used by McFly internally to keep track of the commands from a particular terminal session.
  export MCFLY_SESSION_ID=$(command dd if=/dev/urandom bs=256 count=1 2> /dev/null | LC_ALL=C command tr -dc 'a-zA-Z0-9' | command head -c 24)

  # Find the binary
  MCFLY_PATH=${MCFLY_PATH:-$(command which mcfly)}
  if [[ -z "$MCFLY_PATH" || "$MCFLY_PATH" == "mcfly not found" ]]; then
    echo "Cannot find the mcfly binary, please make sure that mcfly is in your path before sourcing mcfly.zsh."
    return 1
  fi

  # Required for commented out mcfly search commands to work.
  setopt interactive_comments   # allow comments in interactive shells (like Bash does)

  # McFly's temporary, per-session history file.
  export MCFLY_HISTORY=$(command mktemp ${TMPDIR:-/tmp}/mcfly.XXXXXXXX)

  # Check if we need to use extended history
  if [[ -o extendedhistory ]]; then
    export MCFLY_HISTORY_FORMAT="zsh-extended"
  else
    export MCFLY_HISTORY_FORMAT="zsh"
  fi

  # Setup a function to be used by $PROMPT_COMMAND.
  function mcfly_prompt_command {
    local exit_code=$? # Record exit status of previous command.

    # Populate McFly's temporary, per-session history file from recent commands in the shell's primary HISTFILE.
    if [[ ! -f "${MCFLY_HISTORY}" ]]; then
      export MCFLY_HISTORY=$(command mktemp ${TMPDIR:-/tmp}/mcfly.XXXXXXXX)
      command tail -n100 "${MCFLY_HISTFILE}" >| ${MCFLY_HISTORY}
    fi

    # Write history to $MCFLY_HISTORY.
    fc -W "${MCFLY_HISTORY}"

    # Run mcfly with the saved code. It find the text of the last command in $MCFLY_HISTORY and save it to the database.
    [ -n "$MCFLY_DEBUG" ] && echo "mcfly.zsh: Run mcfly add --exit ${exit_code}"
    $MCFLY_PATH --history_format $MCFLY_HISTORY_FORMAT add --exit ${exit_code}
    return ${exit_code} # Restore the original exit code by returning it.
  }

  if [[ -z $precmd_functions ]] || [[ "${precmd_functions[(ie)mcfly_prompt_command]}" -gt ${#precmd_functions} ]]; then
    precmd_functions+=(mcfly_prompt_command)
  else
    [ -n "$MCFLY_DEBUG" ] && echo "mcfly_prompt_command already in precmd_functions, skipping"
  fi

  # Cleanup $MCFLY_HISTORY tmp files on exit.
  mcfly_exit_logger() {
    [ -n "$MCFLY_DEBUG" ] && echo "mcfly.zsh: Exiting and removing $MCFLY_HISTORY"
    command rm -f $MCFLY_HISTORY
  }

  if [[ -z $zshexit_functions ]] || [[ "${zshexit_functions[(ie)mcfly_exit_logger]}" -gt ${#zshexit_functions} ]]; then
    zshexit_functions+=(mcfly_exit_logger)
  else
    [ -n "$MCFLY_DEBUG" ] && echo "mcfly_exit_logger already in zshexit_functions, skipping"
  fi

  # If this is an interactive shell, take ownership of ctrl-r.
  if [[ $- =~ .*i.* ]]; then
    mcfly-history-widget() {
      () {
        echoti rmkx
        exec </dev/tty
        local mcfly_output=$(mktemp ${TMPDIR:-/tmp}/mcfly.output.XXXXXXXX)
        $MCFLY_PATH --history_format $MCFLY_HISTORY_FORMAT search -o "${mcfly_output}" "${LBUFFER}"
        echoti smkx

        # Interpret commandline/run requests from McFly
        while read -r key val; do
          if [[ "$key" = "mode" ]]; then local mode="$val"; fi
          if [[ "$key" = "commandline" ]]; then local commandline="$val"; fi
        done < "${mcfly_output}"
        command rm -f $mcfly_output

        if [[ -n $commandline ]]; then
          RBUFFER=""
          LBUFFER="${commandline}"
        fi
        if [[ "${mode}" == "run" ]]; then
          zle accept-line
        fi
        zle redisplay
      }
    }
    zle -N mcfly-history-widget
    bindkey '^R' mcfly-history-widget
  fi

fi
