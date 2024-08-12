#!/bin/bash

function mcfly_initialize {
  # Note: We avoid using [[ ... ]] to check the Bash version because we are
  # even unsure whether it is available before confirming the Bash version.
  if [ -z "${BASH_VERSINFO-}" ] || [ "${BASH_VERSINFO-}" -lt 3 ]; then
    printf 'mcfly.bash: This setup requires Bash >= 3.0.' >&2
    return 1
  fi

  unset -f "${FUNCNAME[0]}"

  # Ensure stdin is a tty
  [[ -t 0 ]] || return 0

  # Ensure an interactive shell
  [[ $- =~ .*i.* ]] || return 0

  # Avoid loading this file more than once
  [[ ${__MCFLY_LOADED-} != "loaded" ]] || return 0
  __MCFLY_LOADED="loaded"

  # Setup MCFLY_HISTFILE and make sure it exists.
  export MCFLY_HISTFILE="${HISTFILE:-$HOME/.bash_history}"
  MCFLY_BASH_SEARCH_KEYBINDING=${MCFLY_BASH_SEARCH_KEYBINDING:-"\C-x1"}
  MCFLY_BASH_ACCEPT_LINE_KEYBINDING=${MCFLY_BASH_ACCEPT_LINE_KEYBINDING:-"\C-x2"}
  if [[ ! -r ${MCFLY_HISTFILE} ]]; then
    echo "McFly: ${MCFLY_HISTFILE} does not exist or is not readable. Please fix this or set HISTFILE to something else before using McFly."
    return 1
  fi

  # MCFLY_SESSION_ID is used by McFly internally to keep track of the commands from a particular terminal session.
  MCFLY_SESSION_ID="$(command dd if=/dev/urandom bs=256 count=1 2> /dev/null | LC_ALL=C command tr -dc 'a-zA-Z0-9' | command head -c 24)"
  export MCFLY_SESSION_ID

  # Find the binary
  MCFLY_PATH=${MCFLY_PATH:-$(builtin type -P mcfly)}
  if [[ $MCFLY_PATH != /* ]]; then
    # When the user include a relative path in PATH, "builtin type -P" may
    # produce a relative path.  We convert relative paths to the absolute ones.
    MCFLY_PATH=$PWD/$MCFLY_PATH
  fi
  if [[ ! -x $MCFLY_PATH ]]; then
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
    if [[ ! -f ${MCFLY_HISTORY-} ]]; then
      MCFLY_HISTORY=$(command mktemp "${TMPDIR:-/tmp}"/mcfly.XXXXXXXX)
      export MCFLY_HISTORY
      command tail -n100 "${MCFLY_HISTFILE}" >| "${MCFLY_HISTORY}"
    fi

    history -a "${MCFLY_HISTORY}" # Append history to $MCFLY_HISTORY.
    # Run mcfly with the saved code. It will:
    # * append commands to $HISTFILE, (~/.bash_history by default)
    #   for backwards compatibility and to load in new terminal sessions;
    # * find the text of the last command in $MCFLY_HISTORY and save it to the database.
    "$MCFLY_PATH" add --exit "${exit_code}" --append-to-histfile "${MCFLY_HISTFILE}"
    # Clear the in-memory history and reload it from $MCFLY_HISTORY
    # (to remove instances of '#mcfly: ' from the local session history).
    history -cr "${MCFLY_HISTORY}"
    return "${exit_code}" # Restore the original exit code by returning it.
  }

  function mcfly_add_prompt_command {
    local command=$1 IFS=$' \t\n'
    if ((BASH_VERSINFO[0] > 5 || BASH_VERSINFO[0] == 5 && BASH_VERSINFO[1] >= 1)); then
      # Bash 5.1 supports array PROMPT_COMMAND, where we register our prompt
      # command to a new element PROMPT_COMMAND[i] (with i >= 1) to avoid
      # conflicts with other frameworks.
      if [[ " ${PROMPT_COMMAND[*]-} " != *" $command "* ]]; then
        PROMPT_COMMAND[0]=${PROMPT_COMMAND[0]:-}
        # Note: We here use eval to avoid syntax error in Bash < 3.1.  We drop
        # the support for Bash < 3.0, but this is still needed to avoid parse
        # error before the Bash version check is performed.
        eval 'PROMPT_COMMAND+=("$command")'
      fi
    elif [[ -z ${PROMPT_COMMAND-} ]]; then
      PROMPT_COMMAND="$command"
    elif [[ $PROMPT_COMMAND != *"mcfly_prompt_command"* ]]; then
      PROMPT_COMMAND="$command;${PROMPT_COMMAND#;}"
    fi
  }
  # Set $PROMPT_COMMAND run mcfly_prompt_command, preserving any existing $PROMPT_COMMAND.
  mcfly_add_prompt_command "mcfly_prompt_command"

  function mcfly_search_with_tiocsti {
    local LAST_EXIT_CODE=$? IFS=$' \t\n'
    echo "#mcfly: ${READLINE_LINE[*]}" >> "$MCFLY_HISTORY"
    READLINE_LINE=
    "$MCFLY_PATH" search
    return "$LAST_EXIT_CODE"
  }

  # Runs mcfly search with output to file, reads the output, and sets READLINE_LINE to the command.
  # If the command is to be run, binds the MCFLY_KEYSTROKE2 to accept-line, otherwise binds it to nothing.
  function mcfly_search {
    local LAST_EXIT_CODE=$? IFS=$' \t\n'
    # Get a temp file name but don't create the file - mcfly will create the file for us.
    local MCFLY_OUTPUT
    MCFLY_OUTPUT=$(command mktemp --dry-run "${TMPDIR:-/tmp}"/mcfly.output.XXXXXXXX)
    echo "#mcfly: ${READLINE_LINE[*]}" >> "$MCFLY_HISTORY"
    "$MCFLY_PATH" search -o "$MCFLY_OUTPUT"
    # If the file doesn't exist, nothing was selected from mcfly, exit without binding accept-line
    if [[ ! -f $MCFLY_OUTPUT ]]; then
      bind "\"$MCFLY_BASH_ACCEPT_LINE_KEYBINDING\":\"\""
      return
    fi
    # Get the command and set the bash text to it, and move the cursor to the end of the line.
    local MCFLY_COMMAND
    MCFLY_COMMAND=$(command awk 'NR==2{$1=a; print substr($0, 2)}' "$MCFLY_OUTPUT")
    READLINE_LINE=$MCFLY_COMMAND
    READLINE_POINT=${#READLINE_LINE}

    # Get the mode and bind the accept-line key if the mode is run.
    local MCFLY_MODE
    MCFLY_MODE=$(command awk 'NR==1{$1=a; print substr($0, 2)}' "$MCFLY_OUTPUT")
    if [[ $MCFLY_MODE == "run" ]]; then
      bind "\"$MCFLY_BASH_ACCEPT_LINE_KEYBINDING\":accept-line"
    else
      bind "\"$MCFLY_BASH_ACCEPT_LINE_KEYBINDING\":\"\""
    fi

    command rm -f "$MCFLY_OUTPUT"
    return "$LAST_EXIT_CODE"
  }

  # Take ownership of ctrl-r.
  if ((BASH_VERSINFO[0] >= 4)); then
    # shellcheck disable=SC2016
    if [[ ${MCFLY_BASH_USE_TIOCSTI-} == 1 ]]; then
      bind -m emacs     -x '"\C-r": "mcfly_search_with_tiocsti"'
      bind -m vi-insert -x '"\C-r": "mcfly_search_with_tiocsti"'
    else
      # Bind ctrl+r to 2 keystrokes, the first one is used to search in McFly, the second one is used to run the command (if mcfly_search binds it to accept-line).
      bind -m emacs     -x "\"$MCFLY_BASH_SEARCH_KEYBINDING\":\"mcfly_search\""
      bind -m vi-insert -x "\"$MCFLY_BASH_SEARCH_KEYBINDING\":\"mcfly_search\""
      bind -m emacs     "\"\C-r\":\"$MCFLY_BASH_SEARCH_KEYBINDING$MCFLY_BASH_ACCEPT_LINE_KEYBINDING\""
      bind -m vi-insert "\"\C-r\":\"$MCFLY_BASH_SEARCH_KEYBINDING$MCFLY_BASH_ACCEPT_LINE_KEYBINDING\""
    fi
  else
    # The logic here is:
    #   1. Jump to the beginning of the edit buffer, add 'mcfly: ', and comment out the current line. We comment out the line
    #      to ensure that all possible special characters, including backticks, are ignored. This commented out line will
    #      end up as the most recent entry in the $MCFLY_HISTORY file.
    #   2. Type "mcfly search" and then run the command. McFly will pull the last line from the $MCFLY_HISTORY file,
    #      which should be the commented-out search from step #1. It will then remove that line from the history file and
    #      render the search UI pre-filled with it.
    bind -m emacs     '"\C-r": "\C-amcfly: \e# mcfly search\C-m"'
    bind -m vi-insert '"\C-r": "\e0i#mcfly: \e\C-m mcfly search\C-m"'
  fi
}
mcfly_initialize
