#!/usr/bin/env nu

# Nushell can't source the output of a command, so save this script to the vendor autoload
# directory, which Nushell loads after config.nu. Add this to the end of config.nu:
#
#   mkdir ($nu.data-dir | path join "vendor/autoload")
#   mcfly init nushell | save --force ($nu.data-dir | path join "vendor/autoload/mcfly.nu")

# Run the mcfly binary from $MCFLY_PATH, or from $PATH.
def --wrapped __mcfly [...args] {
  ^($env.MCFLY_PATH? | default "mcfly") --history_format nu ...$args
}

# Search command history with McFly and put the selection on the command line.
def __mcfly_search [] {
  let mcfly_output = (mktemp --tmpdir mcfly.output.XXXXXXXX)
  __mcfly search -o $mcfly_output -- (commandline)

  # Interpret commandline/run requests from McFly
  let results = (open --raw $mcfly_output | lines | parse "{key} {val}")
  rm --force $mcfly_output

  let mode = ($results | where key == mode | get 0?.val)
  let selection = ($results | where key == commandline | get 0?.val)
  if $selection != null {
    if $mode == "run" {
      commandline edit --replace --accept $selection
    } else {
      commandline edit --replace $selection
    }
  }
}

def --env __mcfly_init [] {
  # Only set up interactive shells, and only once
  if not $nu.is-interactive { return }
  if ($env.config.keybindings | any {|binding| $binding.name == "mcfly" }) { return }

  if (which ($env.MCFLY_PATH? | default "mcfly") | is-empty) {
    print --stderr "Cannot find the mcfly binary, please make sure that mcfly is in your path before loading mcfly.nu."
    return
  }

  # MCFLY_SESSION_ID is used by McFly internally to keep track of the commands from a particular terminal session.
  $env.MCFLY_SESSION_ID = (random chars --length 24)

  # We don't need a MCFLY_HISTORY file because the hooks below pass each command to McFly directly.
  $env.MCFLY_HISTORY = if $nu.os-info.family == "windows" { "NUL" } else { "/dev/null" }

  # Remember the command and the directory it starts in, so it can be recorded once it has finished.
  $env.config.hooks.pre_execution = ($env.config.hooks.pre_execution? | default [] | append {||
    let command = (commandline)
    # The ctrl-r binding runs as a host command, which triggers this hook too
    if $command == "__mcfly_search" { return }
    $env.__MCFLY_COMMAND = $command
    $env.__MCFLY_OLD_PWD = $env.PWD
  })

  # Add the last command to the McFly database, along with its exit code.
  $env.config.hooks.pre_prompt = ($env.config.hooks.pre_prompt? | default [] | append {||
    let exit_code = $env.LAST_EXIT_CODE

    # McFly imports Nushell's own history (plaintext or SQLite) the first time it runs. This is
    # set here because during startup $nu.history-path doesn't reflect config.nu yet.
    $env.MCFLY_HISTFILE = $nu.history-path

    if "__MCFLY_COMMAND" not-in $env { return }
    let command = $env.__MCFLY_COMMAND
    hide-env __MCFLY_COMMAND

    if ($command | str trim | is-empty) or not $nu.history-enabled { return }
    # Respect Nushell's setting for keeping commands with a leading space out of history
    if ($env.config.history.ignore_space_prefixed? | default false) and ($command =~ '^[ \t]') { return }

    if ($env.MCFLY_DEBUG? | is-not-empty) {
      print $"mcfly.nu: Run mcfly add --exit ($exit_code) --old-dir ($env.__MCFLY_OLD_PWD) -- ($command)"
    }
    __mcfly add --exit $exit_code --old-dir $env.__MCFLY_OLD_PWD -- $command
  })

  # Take ownership of ctrl-r.
  $env.config.keybindings ++= [{
    name: mcfly
    modifier: control
    keycode: char_r
    mode: [emacs, vi_insert, vi_normal]
    event: { send: executehostcommand, cmd: "__mcfly_search" }
  }]
}

__mcfly_init
