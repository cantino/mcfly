#!/usr/bin/env pwsh

$null = New-Module mcfly {
    # We want PSReadLine 2.2.0+ for full prediction support if available
    $psReadLineMinVersion = [version]'2.2.0'
    $hasNewPSReadLine = Get-Module PSReadLine -ListAvailable |
        Where-Object { $_.Version -ge $psReadLineMinVersion }
    if (-not $hasNewPSReadLine -and [Environment]::UserInteractive) {
        try {
            Write-Host "Installing PSReadLine $psReadLineMinVersion+ as McFly dependency"
            Install-Module PSReadLine -Force -Scope CurrentUser -MinimumVersion $psReadLineMinVersion -ErrorAction Stop
        }
        catch {
            Write-Warning "Could not install PSReadLine $psReadLineMinVersion+ ($($_.Exception.Message)). Continuing with the existing version."
        }
    }

    # Prefer the newest available; fall back to whatever is already loaded.
    Import-Module PSReadLine -MinimumVersion $psReadLineMinVersion -ErrorAction SilentlyContinue
    if (-not (Get-Module PSReadLine)) {
        Import-Module PSReadLine -ErrorAction SilentlyContinue
    }
    if (-not (Get-Module PSReadLine)) {
        Write-Warning "McFly requires PSReadLine, which is not available. Skipping McFly setup."
        return
    }

    # Get history file and make a dummy file for psreadline (hopefully after it has loaded the real history file to its in memory history)
    $env:HISTFILE = $null -eq $env:HISTFILE -or "" -eq $env:HISTFILE ? (Get-PSReadLineOption).HistorySavePath : $env:HISTFILE;
    $psreadline_dummy = New-TemporaryFile
    # Append history to dummy file for compatibility
    Get-Content -Path $Env:HISTFILE | Out-File -FilePath $psreadline_dummy -Force
    Set-PSReadLineOption -HistorySavePath $psreadline_dummy.FullName


    $fileExists = Test-Path -path $env:HISTFILE
    if (-not $fileExists) {
        Write-Host "McFly: ${env:HISTFILE} does not exist or is not readable. Please fix this or set HISTFILE to something else before using McFly.";
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
        $tempFile = New-TemporaryFile
        Start-Process -FilePath '::MCFLY::' -ArgumentList "search", "$CommandToComplete", -o, "$tempFile" -NoNewWindow -Wait
        foreach ($line in Get-Content $tempFile) {
            $key, $value = $line -split ' ', 2
            if ("mode" -eq $key) {
                $mode = $value
            }
            if ("commandline" -eq $key) {
                $commandline = $value
            }
        }
        if (-not ($null -eq $commandline)) {
            [Microsoft.PowerShell.PSConsoleReadLine]::DeleteLine()
            [Microsoft.PowerShell.PSConsoleReadline]::Insert($commandline)
            if ("run" -eq $mode) {
                [Microsoft.PowerShell.PSConsoleReadline]::AcceptLine()
            }
        }
        Remove-Item $tempFile
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
        Param (
            [string] $Command,
            [int] $ExitCode
        )
        $ExitCode = $ExitCode ?? 0;
        $Command | Out-File -FilePath $env:MCFLY_HISTORY -Append
        Start-Process -FilePath '::MCFLY::' -ArgumentList add, --exit, $ExitCode, --append-to-histfile, $env:HISTFILE -NoNewWindow | Write-Host
    }

    # We need to make sure we call out AddToHistoryHandler right after each command is called
    Set-PSReadLineOption -HistorySaveStyle SaveIncrementally

    #   -PredictionSource       -> added in 2.1.0
    #   'HistoryAndPlugin' value -> added in 2.2.0 (older versions only accept None/History)
    $loadedPSReadLineVersion = (Get-Module -Name PSReadLine).Version
    if ($loadedPSReadLineVersion -ge [version]'2.2.0') {
        Set-PSReadLineOption -PredictionSource HistoryAndPlugin
    }
    elseif ($loadedPSReadLineVersion -ge [version]'2.1.0') {
        Set-PSReadLineOption -PredictionSource History
    }

    Set-PSReadLineOption -AddToHistoryHandler {
        Param([string]$Command)
        $lastExitTmp = $LASTEXITCODE
        $Command = $Command.Trim();
        # PSReadLine executes this before the command even runs, so we don't know its exit code - assume 0
        Add-CommandToMcFly -Command $Command -ExitCode 0
        $LASTEXITCODE = $lastExitTmp
        # Tell PSReadLine to save the command to their in-memory history (and also the dummy file)
        return $true
    }

    Set-PSReadLineKeyHandler -Chord "Ctrl+r" -ScriptBlock {
        $line = $null
        $cursor = $null
        [Microsoft.PowerShell.PSConsoleReadline]::GetBufferState([ref]$line, [ref]$cursor)
        "#mcfly: $line" | Out-File -FilePath $env:MCFLY_HISTORY -Append
        Invoke-McFly -CommandToComplete "`"$line`""
    }

    Export-ModuleMember -Function @(
        "Invoke-McFly"
        "Add-CommandToMcFly"
    )
}
