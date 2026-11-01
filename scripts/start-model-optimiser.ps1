<#
.SYNOPSIS
    Runs continuous background optimization of model routing and token usage.

.DESCRIPTION
    Runs as an autonomous daemon loop, collecting execution stats and updating
    model routing defaults every N seconds (default: 300 seconds / 5 minutes).

.PARAMETER IntervalSeconds
    Loop interval in seconds. Defaults to 300.

.EXAMPLE
    .\scripts\start-model-optimiser.ps1 -IntervalSeconds 60
#>

[CmdletBinding()]
param(
    [int]$IntervalSeconds = 300
)

$ErrorActionPreference = "Continue"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "    Autonomous Model Optimizer Daemon Started     " -ForegroundColor Cyan
Write-Host "    Interval: $IntervalSeconds seconds | Press Ctrl+C to stop " -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan

while ($true) {
    $Now = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Host "`n[$Now] Running periodic model optimization cycle..." -ForegroundColor Yellow

    try {
        & powershell -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "collect-model-stats.ps1")
        & powershell -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "apply-model-optimisation.ps1")
    } catch {
        Write-Warning "Cycle encountered an error: $($_.Exception.Message)"
    }

    Write-Host "Sleeping for $IntervalSeconds seconds..." -ForegroundColor Gray
    Start-Sleep -Seconds $IntervalSeconds
}
