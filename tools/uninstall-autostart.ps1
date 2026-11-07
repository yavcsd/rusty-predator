#Requires -RunAsAdministrator
<#
.SYNOPSIS
Undo install-autostart.ps1: Rusty Predator no longer starts at logon.

.DESCRIPTION
Removes the "Rusty Predator" task. The copy in "C:\Program Files\Rusty Predator" is deleted once
the PredatorSense button task doesn't use it either.
#>
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'common.ps1')

Remove-OldTask $OldAutostartTaskName
if (Get-ScheduledTask -TaskName $AutostartTaskName -ErrorAction SilentlyContinue) {
    Unregister-ScheduledTask -TaskName $AutostartTaskName -Confirm:$false
    Write-Host "task:      '$AutostartTaskName' removed"
} else {
    Write-Host "task:      '$AutostartTaskName' was not installed"
}
Remove-InstalledCopy
