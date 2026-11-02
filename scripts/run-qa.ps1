<#
.SYNOPSIS
    Runs autonomous UI and accessibility QA testing on the project using ui-qa-tester.

.DESCRIPTION
    Checks for the ui-qa-cli binary (in PATH, in %LOCALAPPDATA%\Programs\ui-qa-tester,
    or in adjacent development checkouts), initializes qa-session.toml if missing,
    and executes the non-intrusive UI crawl or self-test.

.PARAMETER Config
    Path to the session configuration file (default: "qa-session.toml").

.PARAMETER Headless
    Runs in background focus mode without user interference (default: $true).

.PARAMETER SelfTest
    Runs the built-in self-test against the Hardware Controller mock window.

.PARAMETER Exe
    Optional override path to the compiled application binary.

.PARAMETER Title
    Optional override substring filter for attaching to a visible window.

.PARAMETER MaxSteps
    Optional override for the maximum number of autonomous UI steps.

.PARAMETER Output
    Optional override directory for reports and screenshots.

.PARAMETER SyncGithubIssues
    If specified, synchronizes high/critical findings to GitHub Issues with deduplication.
#>

param(
    [string]$Config = "qa-session.toml",
    [switch]$Headless = $true,
    [switch]$SelfTest = $false,
    [string]$Exe = "",
    [string]$Title = "",
    [int]$MaxSteps = 0,
    [string]$Output = "",
    [switch]$SyncGithubIssues = $false
)

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$ScriptDir = if ($PSScriptRoot) { $PSScriptRoot } else { Split-Path -Parent $MyInvocation.MyCommand.Path }
if (-not $ScriptDir) { $ScriptDir = (Get-Location).Path }
$RepoRoot = if (Test-Path (Join-Path $ScriptDir "qa-session.template.toml")) {
    $ScriptDir
} elseif (Test-Path (Join-Path (Split-Path -Parent $ScriptDir) "qa-session.template.toml")) {
    Split-Path -Parent $ScriptDir
} else {
    (Get-Location).Path
}
Set-Location $RepoRoot

# 1. Locate ui-qa-cli executable
$CliFromPath = (Get-Command ui-qa-cli -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source -ErrorAction SilentlyContinue)
$CliExeFromPath = (Get-Command ui-qa-cli.exe -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source -ErrorAction SilentlyContinue)

$CandidatePaths = @(
    $CliFromPath,
    $CliExeFromPath,
    (Join-Path $env:LOCALAPPDATA "Programs\ui-qa-tester\ui-qa-cli.exe"),
    (Join-Path $RepoRoot "..\ui_qa_tester\target\release\ui-qa-cli.exe"),
    (Join-Path $RepoRoot "..\ui-qa-tester\target\release\ui-qa-cli.exe")
)
$CliExe = $null
foreach ($p in $CandidatePaths) {
    if (-not [string]::IsNullOrWhiteSpace($p) -and (Test-Path $p)) {
        $CliExe = (Resolve-Path $p).Path
        break
    }
}

if (-not $CliExe) {
    Write-Warning "ui-qa-cli executable was not found."
    Write-Host "`nTo install or build ui-qa-tester:" -ForegroundColor Cyan
    Write-Host "  1. Open C:\Dev\AgenticAI\ui_qa_tester"
    Write-Host "  2. Run setup.cmd (installs to %LOCALAPPDATA%\Programs\ui-qa-tester)"
    Write-Host "  3. Or run 'cargo build --workspace --release'"
    exit 1
}

Write-Host "Found QA Actuator: $CliExe" -ForegroundColor Green

# 2. Self-test mode
if ($SelfTest) {
    $TargetOutput = if ($Output) { $Output } else { Join-Path $RepoRoot "qa-output\self-test" }
    Write-Host "Executing ui-qa-cli self-test with output to $TargetOutput..." -ForegroundColor Cyan
    & $CliExe self-test --output $TargetOutput
    if ($LASTEXITCODE -eq 0) {
        Write-Host "`nQA self-test completed successfully! Reports generated in $TargetOutput" -ForegroundColor Green
    } else {
        Write-Warning "QA self-test finished with exit code $LASTEXITCODE."
    }
    exit $LASTEXITCODE
}

# 3. Ensure configuration exists for standard run
if (-not (Test-Path (Join-Path $RepoRoot $Config))) {
    $TemplateConfig = Join-Path $RepoRoot "qa-session.template.toml"
    if (Test-Path $TemplateConfig) {
        Write-Host "Creating $Config from template..." -ForegroundColor Yellow
        Copy-Item $TemplateConfig (Join-Path $RepoRoot $Config)
    } else {
        throw "Configuration $Config not found and qa-session.template.toml is missing."
    }
}

Write-Host "Executing autonomous UI QA crawl using $Config..." -ForegroundColor Cyan

# 4. Assemble arguments
$CliArgs = @("run", "--config", (Join-Path $RepoRoot $Config))
if ($Exe) { $CliArgs += @("--exe", $Exe) }
if ($Title) { $CliArgs += @("--title", $Title) }
if ($MaxSteps -gt 0) { $CliArgs += @("--max-steps", $MaxSteps.ToString()) }
if ($Output) { $CliArgs += @("--output", $Output) }
if ($SyncGithubIssues) { $CliArgs += @("--sync-github-issues") }

& $CliExe @CliArgs

if ($LASTEXITCODE -eq 0) {
    Write-Host "`nQA crawl completed successfully! Reports generated in qa-reports/" -ForegroundColor Green
} else {
    Write-Warning "QA crawl finished with exit code $LASTEXITCODE. Review qa-reports/ for findings."
}
exit $LASTEXITCODE
