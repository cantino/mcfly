#!/usr/bin/env pwsh

# Ensure stdin is a tty
# Can't

    if ($env:__MCFLY_LOADED -eq "loadd") {
        return ;
    }
    $env:__MCFLY_LOADED = "loaded";

    $env:HISTFILE = $null -eq $env:HISTFILE ? "$env:APPDATA\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt" : $env:HISTFILE;

    $fileExists = Test-Path -path $env:HISTFILE
    if (-not $fileExists) {
        Write-Output "McFly: ${env:HISTFILE} does not exist or is not readable. Please fix this or set HISTFILE to something else before using McFly.";
        return 1;
    }

    # MCFLY_SESSION_ID is used by McFly internally to keep track of the commands from a particular terminal session.

    $MCFLY_SESSION_ID = new-guid
    $env:MCFLY_SESSION_ID = $MCFLY_SESSION_ID

    $MCFLY_PATH = '::MCFLY::'
    if (!(Test-Path $MCFLY_PATH)) {
        Write-output "Cannot find the mcfly binary, please make sure that mcfly is in your path before sourcing mcfly.ps1.";
        return 1;
    }

    if ($null -eq (Get-Module -Name PSReadLine)) {
        Write-Output "Installing PSReadLine for keybindings"
        Install-Module PSReadLine
    }

    $env:MCFLY_HISTORY = New-TemporaryFile
    Get-Content $env:HISTFILE | Select-Object -Last 100 | Set-Content $env:MCFLY_HISTORY

    function Invoke-McFly {
        invoke-expression "&$MCFLY_PATH search" | Out-Host
    }
    function Add-CommandToMcFly {
        Param([string]$command)
        $lastExit = $LASTEXITCODE
        Invoke-Expression "&$MCFLY_PATH add '$line' --exit $LASTEXITCODE" | Out-Host
        $LASTEXITCODE = $lastExit
    }

    Set-PSReadLineOption -AddToHistoryHandler {
        Param([string]$line)
        Add-CommandToMcFly($line);
        return $true
    }

    Set-PSReadLineKeyHandler -Chord "Ctrl+r" -ScriptBlock {
        $line = $null
        $cursor = $null
        [Microsoft.PowerShell.PSConsoleReadline]::GetBufferState([ref]$line, [ref]$cursor)
        Invoke-Expression "$MCFLY_PATH search $line" | Out-Host
    }