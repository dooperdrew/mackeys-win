<#
.SYNOPSIS
    Runs autonomous UI and accessibility QA testing on the project using ui-qa-tester.

.DESCRIPTION
    Checks for the ui-qa-cli binary (in PATH, in %LOCALAPPDATA%\Programs\ui-qa-tester,
    or in adjacent development checkouts), initializes qa-session.toml if missing,
    and executes the non-intrusive UI crawl.

.PARAMETER Config
    Path to the session configuration file (default: "qa-session.toml").

.PARAMETER Headless
    Runs in background focus mode without user interference.
#>

param(
    [string]$Config = "qa-session.toml",
    [switch]$Headless = $true
)

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

# 1. Ensure configuration exists
if (-not (Test-Path (Join-Path $RepoRoot $Config))) {
    $TemplateConfig = Join-Path $RepoRoot "qa-session.template.toml"
    if (Test-Path $TemplateConfig) {
        Write-Host "Creating $Config from template..." -ForegroundColor Yellow
        Copy-Item $TemplateConfig (Join-Path $RepoRoot $Config)
    } else {
        throw "Configuration $Config not found and qa-session.template.toml is missing."
    }
}

# 2. Locate ui-qa-cli executable
$CandidatePaths = @(
    (Get-Command ui-qa-cli.exe -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source -ErrorAction SilentlyContinue),
    (Get-Command ui-qa-cli -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source -ErrorAction SilentlyContinue),
    (Join-Path $env:LOCALAPPDATA "Programs\ui-qa-tester\ui-qa-cli.exe"),
    (Join-Path $RepoRoot "..\ui_qa_tester\target\release\ui-qa-cli.exe"),
    (Join-Path $RepoRoot "..\ui-qa-tester\target\release\ui-qa-cli.exe")
)

$CliExe = $CandidatePaths | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1

if (-not $CliExe) {
    Write-Warning "ui-qa-cli executable was not found."
    Write-Host "`nTo install or build ui-qa-tester:" -ForegroundColor Cyan
    Write-Host "  1. Open C:\Dev\AgenticAI\ui_qa_tester"
    Write-Host "  2. Run setup.cmd (installs to %LOCALAPPDATA%\Programs\ui-qa-tester)"
    Write-Host "  3. Or run 'cargo build --workspace --release'"
    exit 1
}

Write-Host "Found QA Actuator: $CliExe" -ForegroundColor Green
Write-Host "Executing autonomous UI QA crawl using $Config..." -ForegroundColor Cyan

# 3. Execute crawl
& $CliExe crawl --config (Join-Path $RepoRoot $Config)

if ($LASTEXITCODE -eq 0) {
    Write-Host "`nQA crawl completed successfully! Reports generated in qa-reports/" -ForegroundColor Green
} else {
    Write-Warning "QA crawl finished with exit code $LASTEXITCODE. Review qa-reports/ for findings."
}
