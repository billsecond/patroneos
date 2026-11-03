<#
Removes Patrone Notepad for this account: its folder and its Start menu
entry. Its settings and kept tabs (%LOCALAPPDATA%\patrone-notepad) stay,
unless -RemoveSettings.

  powershell -ExecutionPolicy Bypass -File uninstall.ps1 [-RemoveSettings]
#>
param(
    [string]$Destination = "$env:LOCALAPPDATA\Programs\Patrone Notepad",
    [switch]$RemoveSettings
)
$ErrorActionPreference = 'Stop'

$running = Get-Process patrone-notepad -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -and $_.Path.StartsWith($Destination, [StringComparison]::OrdinalIgnoreCase) }
if ($running) { throw 'Close Patrone Notepad first, then run uninstall.ps1 again.' }

$link = Join-Path ([Environment]::GetFolderPath('Programs')) 'Patrone Notepad.lnk'
if (Test-Path $link) { Remove-Item -Force $link }
if (Test-Path $Destination) { Remove-Item -Recurse -Force $Destination }
if ($RemoveSettings -and (Test-Path "$env:LOCALAPPDATA\patrone-notepad")) {
    Remove-Item -Recurse -Force "$env:LOCALAPPDATA\patrone-notepad"
}
Write-Host 'Patrone Notepad is removed.'
