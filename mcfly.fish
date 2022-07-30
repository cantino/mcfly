#!/usr/bin/env fish

# Avoid loading this file more than once
if test "$__MCFLY_LOADED" != "loaded"
  set -g __MCFLY_LOADED "loaded"

  # Note: we only use the history file for the session when this file was sourced.
  # Would have to reset this before calling mcfly if you want commands from another session later.
  if not set -q MCFLY_HISTFILE
    set -gx MCFLY_HISTFILE (set -q XDG_DATA_HOME; and echo $XDG_DATA_HOME; or echo $HOME/.local/share)/fish/(set -q fish_history; and echo $fish_history; or echo fish)_history
  end
  if not test -r "$MCFLY_HISTFILE"
    echo "McFly: $MCFLY_HISTFILE does not exist or is not readable. Please fix this or set MCFLY_HISTFILE to something else before using McFly." >&2
    exit 1
  end

  # MCFLY_SESSION_ID is used by McFly internally to keep track of the commands from a particular terminal session.
  set -gx MCFLY_SESSION_ID (dd if=/dev/urandom bs=256 count=1 2>/dev/null | env LC_ALL=C tr -dc 'a-zA-Z0-9' | head -c 24)

  # Find the binary
  set -q MCFLY_PATH; or set -l MCFLY_PATH (command -v mcfly)
  if test -z "$MCFLY_PATH"; or test "$MCFLY_PATH" = "mcfly not found"
    echo "Cannot find the mcfly binary, please make sure that mcfly is in your path before sourcing mcfly.fish"
    exit 1
  end
  # We don't need a MCFLY_HISTORY file because we can get the last command in fish_postexec.
  set -g __MCFLY_CMD $MCFLY_PATH --mcfly_history /dev/null --history_format fish

  function __mcfly_save_old_pwd -d 'Save PWD before running command' -e fish_preexec
    set -g __MCFLY_OLD_PWD "$PWD"
  end

  function __mcfly_add_command -d 'Add run commands to McFly database' -e fish_postexec
    # First, retain return code of last command before we lose it
    set -l last_status $status
    # Handle first call of this function after sourcing mcfly.fish, when the old PWD won't be set
    set -q __MCFLY_OLD_PWD; or set -g __MCFLY_OLD_PWD "$PWD"

    test -n "$MCFLY_DEBUG"; and echo mcfly.fish: Run eval $__MCFLY_CMD add --exit '$last_status' --old-dir '$__MCFLY_OLD_PWD' -- '$argv[1]'
    eval $__MCFLY_CMD add --exit '$last_status' --old-dir '$__MCFLY_OLD_PWD' -- '$argv[1]'
  end

  # If this is an interactive shell, set up key binding functions.
  if status is-interactive
    function __mcfly-history-widget -d "Search command history with McFly"
      set -l mcfly_output (mktemp -t mcfly.output.XXXXXXXX)
      eval $__MCFLY_CMD search -o '$mcfly_output' -- (commandline | string escape)

      # Interpret commandline/run requests from McFly
      set -l mode; set -l commandline
      while read key val
        test "$key" = "mode"; and set mode "$val"
        test "$key" = "commandline"; and set commandline "$val"
        test "$key" = "delete"; and history delete --exact --case-sensitive "$val"
      end < "$mcfly_output"
      rm -f $mcfly_output

      if test -n "$commandline"
        commandline "$commandline"
      end
      if test "$mode" = "run"
        commandline -f execute
      end
      commandline -f repaint
    end

    function mcfly_key_bindings -d "Default key bindings for McFly"
       bind \cr __mcfly-history-widget
       if bind -M insert >/dev/null 2>&1
         bind -M insert \cr __mcfly-history-widget
       end
    end

    mcfly_key_bindings
  end
end
