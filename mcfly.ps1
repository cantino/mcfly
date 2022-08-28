#!/usr/bin/env pwsh

# Ensure stdin is a tty
# Can't

$null = New-Module starship {
    if ($env:__MCFLY_LOADED -eq "loaded") {
        return ;
    }
    $env:__MCFLY_LOADED = "loaded";

    # We need PSReadLine for a number of capabilities
    if ($null -eq (Get-Module -Name PSReadLine)) {
        Write-Output "Installing PSReadLine for keybindings"
        Install-Module PSReadLine
    }

    # Get history file and make a dummy file for psreadline (hopefully after it has loaded the real history file to its in memory history)
    $env:HISTFILE = $null -eq $env:HISTFILE ? (Get-PSReadLineOption).HistorySavePath : $env:HISTFILE;
    $_PSREADLINEHISTORY = (Get-PSReadLineOption).HistorySavePath
    $psreadline_dummy = New-TemporaryFile
    Set-PSReadLineOption -HistorySavePath $psreadline_dummy.FullName


    $fileExists = Test-Path -path $env:HISTFILE
    if (-not $fileExists) {
        Write-Output "McFly: ${env:HISTFILE} does not exist or is not readable. Please fix this or set HISTFILE to something else before using McFly.";
        return 1;
    }

    # MCFLY_SESSION_ID is used by McFly internally to keep track of the commands from a particular terminal session.

    $MCFLY_SESSION_ID = new-guid
    $env:MCFLY_SESSION_ID = $MCFLY_SESSION_ID

    $env:MCFLY_HISTORY = New-TemporaryFile
    Get-Content $env:HISTFILE | Select-Object -Last 100 | Set-Content $env:MCFLY_HISTORY

    <#
.SYNOPSIS
Cmdlet to run McFly

.PARAMETER CommandToComplete
The command to complete

.EXAMPLE
Invoke-McFly -CommandToComplete "cargo bu"
#>
    function Invoke-McFly {
        Param([string]$CommandToComplete)
        $lastExitTmp = $LASTEXITCODE
        Start-Process -FilePath '::MCFLY::' -ArgumentList "search", "$CommandToComplete" -NoNewWindow -Wait
        $LASTEXITCODE = $lastExitTmp
    }

    <#
.SYNOPSIS
Add a command to McFly's history.

.PARAMETER Command
The string of the command to add to McFly's history

.PARAMETER ExitCode
The exit code of the command to add

.EXAMPLE
Add-CommandToMcFly -Command "cargo build"
#>
    function Add-CommandToMcFly {
        Param([string]$Command)
        Param([int]$ExitCode)
        $ExitCode = $ExitCode ?? 0;
        Write-Host "Adding Command: $command"
        Start-Process -FilePath '::MCFLY::'  -ArgumentList add, $command, --exit, $ExitCode -NoNewWindow | Out-Host
    }

    # We need to make sure we call out AddToHistoryHandler right after each command is called
    Set-PSReadLineOption -HistorySaveStyle SaveIncrementally

    Set-PSReadLineOption -PredictionSource HistoryAndPlugin

    Set-PSReadLineOption -AddToHistoryHandler {
        Param([string]$line)
        # save the exit code to restore in case the program depends on it
        $lastExitTmp = $LASTEXITCODE
        $line = $line.Trim();
        # PSReadLine executes this before the command even runs, so we don't know its exit code - assume 0
        Add-CommandToMcFly -Command $line -ExitCode 0
        # Add the command to PSReadLine's real history file
        Out-File -FilePath $_PSREADLINEHISTORY -Append -InputObject $line
        $LASTEXITCODE = $lastExitTmp
        # Tell PSReadLine to save the command to their in-memory history (and also the dummy file)
        return $true
    }

    Set-PSReadLineKeyHandler -Chord "Ctrl+r" -ScriptBlock {
        $line = $null
        $cursor = $null
        [Microsoft.PowerShell.PSConsoleReadline]::GetBufferState([ref]$line, [ref]$cursor)
        Invoke-McFly -CommandToComplete $line
    }
    Export-ModuleMember -Function @(
        "Invoke-McFly"
        "Add-LastCommandToMcFly"
    )
}