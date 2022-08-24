
# Ensure stdin is a tty
# Can't

if ($env:__MCFLY_LOADED == "loaded") {
    return 0;
}
$env:__MCFLY_LOADED = "loaded";

$env:HISTFILE = $null -eq $env:HISTFILE ? "$env:APPDATA\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt" : $HISTFILE;

$fileExists = Test-Path -path $env:HISTFILE
if (-not $fileExists) {
    Write-Output "McFly: ${env:HISTFILE} does not exist or is not readable. Please fix this or set HISTFILE to something else before using McFly.";
    return 1;
}

# MCFLY_SESSION_ID is used by McFly internally to keep track of the commands from a particular terminal session.

$MCFLY_SESSION_ID = new-guid
$env:MCFLY_SESSION_ID = $MCFLY_SESSION_ID

(Get-Command "mcfly.exe" -ErrorAction SilentlyContinue) | Select-Object -ExpandProperty Path | Set-Variable -Name MCFLY_PATH
if ($null -eq $MCFLY_PATH)
{
    Write-output "Cannot find the mcfly binary, please make sure that mcfly is in your path before sourcing mcfly.bash.";
    return 1;
}

if ($null -eq (Get-Module -Name PSReadLine)){
    Write-Output "Installing PSReadLine for keybindings"
    Install-Module PSReadLine
}

if ($MCFLY_HISTORY -eq 1) {
    $env:MCFLY_HISTORY = New-TemporaryFile
    Get-Content $env:HISTFILE | Select-Object -Last 100 | Set-Content $env:MCFLY_HISTORY
}

function Add-LastCommandToMcFly {
    $lastCommand = Get-History -Count 1;
    $lastCommandStart = Get-Date -Date $lastCommand.StartExecutionTime -UFormat %s
    $MCFLY_PATH add --when $lastCommandStart
}
function global:Invoke-McFly {
    $startInfo = New-Object System.Diagnostics.ProcessStartInfo -ArgumentList $MCFLY_PATH -Property @{
        StandardOutputEncoding = [System.Text.Encoding]::UTF8;
        RedirectStandardOutput = $true;
        RedirectStandardError = $true;
        CreateNoWindow = $true;
        UseShellExecute = $false;
    };
    if ($startInfo.ArgumentList.Add) {
        # PowerShell 6+ uses .NET 5+ and supports the ArgumentList property
        # which bypasses the need for manually escaping the argument list into
        # a command string.
        $startInfo.ArgumentList.Add($search);
    }
    $process = [System.Diagnostics.Process]::Start($startInfo)
}

function global:prompt {
    Add-LastCommandToMcFly
}

Set-PSReadLineKeyHandler -Chord "Ctrl+r" -Function Invoke-McFly