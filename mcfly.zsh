#!/bin/zsh

# Ensure stdin is a tty
[[ -t 0 ]] || return

# Avoid loading this file more than once
if [[ "$__MCFLY_LOADED" == "loaded" ]]; then
  return 0
fi
__MCFLY_LOADED="loaded"

emulate -L zsh

# Ensure HISTFILE exists.
if [ -z "${HISTFILE}" ]; then
  export HISTFILE="${HOME}/.zsh_history"
fi

if [[ ! -r "${HISTFILE}" ]]; then
  echo "McFly: ${HISTFILE} does not exist or is not readable. Please fix this or set HISTFILE to something else before using McFly."
  return 1
fi

# MCFLY_SESSION_ID is used by McFly internally to keep track of the commands from a particular terminal session.
export MCFLY_SESSION_ID=$(dd if=/dev/urandom bs=256 count=1 2> /dev/null | env LC_ALL=C tr -dc 'a-zA-Z0-9' | head -c 24)

# Find the binary
MCFLY_PATH=${MCFLY_PATH:-$(which mcfly)}

# Required for commented out mcfly search commands to work.
setopt interactive_comments   # allow comments in interactive shells (like Bash does)

# McFly's temporary, per-session history file.
if [[ ! -f "${MCFLY_HISTORY}" ]]; then
  export MCFLY_HISTORY=$(mktemp -t mcfly.XXXXXXXX)
fi

# Setup a function to be used by $PROMPT_COMMAND.
function mcfly_prompt_command {
  local exit_code=$? # Record exit status of previous command.

  # Populate McFly's temporary, per-session history file from recent commands in the shell's primary HISTFILE.
  if [[ ! -f "${MCFLY_HISTORY}" ]]; then
    export MCFLY_HISTORY=$(mktemp -t mcfly.XXXXXXXX)
    tail -n100 "${HISTFILE}" >| ${MCFLY_HISTORY}
  fi

  # Write history to $MCFLY_HISTORY.
  fc -W "${MCFLY_HISTORY}"

  # Run mcfly with the saved code. It fill find the text of the last command in $MCFLY_HISTORY and save it to the database.
  [ -n "$MCFLY_DEBUG" ] && echo "mcfly.zsh: Run mcfly add --exit ${exit_code}"
  $MCFLY_PATH add --exit ${exit_code}
  return ${exit_code} # Restore the original exit code by returning it.
}
precmd_functions+=(mcfly_prompt_command)

# Cleanup $MCFLY_HISTORY tmp files on exit.
exit_logger() {
  [ -n "$MCFLY_DEBUG" ] && echo "mcfly.zsh: Exiting and removing $MCFLY_HISTORY"
  rm -f $MCFLY_HISTORY
}
zshexit_functions+=(exit_logger)

# If this is an interactive shell, take ownership of ctrl-r.
if [[ $- =~ .*i.* ]]; then
  mcfly-history-widget() {
    () {
      tput init
      exec </dev/tty
      local mcfly_output=$(mktemp -t mcfly.output.XXXXXXXX)
      $MCFLY_PATH search -o "${mcfly_output}" "${LBUFFER}"
      local mode=$(sed -n 1p $mcfly_output)
      local selected=$(sed 1d $mcfly_output)
      rm -f $mcfly_output
      if [[ -n $selected ]]; then
        RBUFFER=""
        LBUFFER="${selected}"
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
