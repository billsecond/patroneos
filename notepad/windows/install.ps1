<#
Installs Patrone Notepad for the account that runs it, with no
administrator rights: the app in %LOCALAPPDATA%\Programs\Patrone Notepad,
and Patrone Notepad in the Start menu. It doesn't change which app opens
any kind of file.

  powershell -ExecutionPolicy Bypass -File install.ps1
#>
param([string]$Destination = "$env:LOCALAPPDATA\Programs\Patrone Notepad")
$ErrorActionPreference = 'Stop'

$source = Join-Path $PSScriptRoot 'Patrone Notepad'
if (-not (Test-Path "$source\bin\patrone-notepad.exe")) {
    throw "Run install.ps1 from the folder the zip was unpacked into: $source isn't there"
}
# Its files can't be replaced while it runs
$running = Get-Process patrone-notepad -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -and $_.Path.StartsWith($Destination, [StringComparison]::OrdinalIgnoreCase) }
if ($running) { throw 'Close Patrone Notepad first, then run install.ps1 again.' }

if (Test-Path $Destination) { Remove-Item -Recurse -Force $Destination }
New-Item -ItemType Directory -Force (Split-Path $Destination) | Out-Null
Copy-Item -Recurse $source $Destination

$programs = [Environment]::GetFolderPath('Programs')
$shell = New-Object -ComObject WScript.Shell
$link = $shell.CreateShortcut((Join-Path $programs 'Patrone Notepad.lnk'))
$link.TargetPath = Join-Path $Destination 'bin\patrone-notepad.exe'
$link.WorkingDirectory = [Environment]::GetFolderPath('MyDocuments')
$link.IconLocation = (Join-Path $Destination 'patrone-notepad.ico') + ',0'
$link.Description = 'Plain text and Markdown, in tabs'
$link.Save()

Write-Host "Patrone Notepad is installed in $Destination, and it's in the Start menu."
