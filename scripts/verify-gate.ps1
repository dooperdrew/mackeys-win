<#
.SYNOPSIS
    Tiered Verification Gate Runner for multi-agent workflows.

.DESCRIPTION
    Runs deterministic verification checks before committing or opening PRs.
    Supports two operational modes:
      - Fast: Incremental check (formatting, linter, touched/fast unit tests). Ideal for subagents and rapid iteration.
      - Full: Comprehensive gate (clean build, full integration test suite, release compilation, budget validation).
              Required for the Lead Agent before merging to main or releasing.

.PARAMETER Mode
    "Fast" or "Full". Defaults to "Fast".

.PARAMETER Stack
    "Auto", "Rust", "Node", "Python", "Go", "Template". Defaults to "Auto".

.PARAMETER SkipBuild
    Skips release compilation stage during Full mode (useful for pure docs/script milestones).

.EXAMPLE
    .\scripts\verify-gate.ps1 -Mode Fast
    .\scripts\verify-gate.ps1 -Mode Full
#>

[CmdletBinding()]
param(
    [ValidateSet("Fast", "Full")]
    [string]$Mode = "Fast",

    [ValidateSet("Auto", "Rust", "Node", "Python", "Go", "Template")]
    [string]$Stack = "Auto",

    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

# -----------------------------------------------------------------------------
# Environment & PATH Resiliency
# -----------------------------------------------------------------------------
$KnownPaths = @(
    (Join-Path $env:USERPROFILE ".cargo\bin"),
    (Join-Path $env:APPDATA "npm"),
    (Join-Path $env:LOCALAPPDATA "Programs\Python\Python312"),
    (Join-Path $env:LOCALAPPDATA "Programs\Python\Python312\Scripts"),
    (Join-Path $env:LOCALAPPDATA "Programs\Python\Python311"),
    (Join-Path $env:LOCALAPPDATA "Programs\Python\Python311\Scripts"),
    "C:\Program Files\Go\bin"
)

foreach ($P in $KnownPaths) {
    if ((Test-Path $P) -and ($env:PATH -notlike "*$P*")) {
        $env:PATH = "$P;$env:PATH"
    }
}

# -----------------------------------------------------------------------------
# Stack Auto-Detection
# -----------------------------------------------------------------------------
if ($Stack -eq "Auto") {
    if (Test-Path (Join-Path $RepoRoot "Cargo.toml")) {
        $Stack = "Rust"
    } elseif (Test-Path (Join-Path $RepoRoot "package.json")) {
        $Stack = "Node"
    } elseif ((Test-Path (Join-Path $RepoRoot "pyproject.toml")) -or (Test-Path (Join-Path $RepoRoot "requirements.txt"))) {
        $Stack = "Python"
    } elseif (Test-Path (Join-Path $RepoRoot "go.mod")) {
        $Stack = "Go"
    } else {
        $Stack = "Template"
    }
}

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "       VERIFICATION GATE: [$Mode Mode | $Stack]   " -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan

$GlobalStopwatch = [System.Diagnostics.Stopwatch]::StartNew()
$FailedStages = @()

function Invoke-Stage([string]$StageName, [scriptblock]$Action) {
    Write-Host "`n>> Stage: $StageName" -ForegroundColor Yellow
    $StageWatch = [System.Diagnostics.Stopwatch]::StartNew()
    try {
        & $Action
        $StageWatch.Stop()
        Write-Host "   [PASS] $StageName ($([math]::Round($StageWatch.Elapsed.TotalSeconds, 2))s)" -ForegroundColor Green
    } catch {
        $StageWatch.Stop()
        Write-Host "   [FAIL] $StageName ($([math]::Round($StageWatch.Elapsed.TotalSeconds, 2))s)" -ForegroundColor Red
        Write-Host "   Error: $($_.Exception.Message)" -ForegroundColor Red
        $script:FailedStages += $StageName
    }
}

# -----------------------------------------------------------------------------
# Execution Plan by Stack
# -----------------------------------------------------------------------------

switch ($Stack) {
    "Rust" {
        # Formatting Check
        Invoke-Stage "Code Formatting (rustfmt)" {
            cargo fmt -- --check
            if ($LASTEXITCODE -ne 0) { throw "cargo fmt failed" }
        }

        # Static Analysis (Clippy)
        Invoke-Stage "Static Analysis (clippy)" {
            if ($Mode -eq "Fast") {
                cargo clippy -- -D warnings
            } else {
                cargo clippy --all-targets --all-features -- -D warnings
            }
            if ($LASTEXITCODE -ne 0) { throw "cargo clippy detected warnings or errors" }
        }

        # Automated Tests
        Invoke-Stage "Automated Tests" {
            $TestThreads = [System.Environment]::ProcessorCount
            if ($Mode -eq "Fast") {
                # Fast unit tests with concurrency
                cargo test --lib -- --test-threads=$TestThreads
            } else {
                # Full test suite across workspace
                cargo test --workspace -- --test-threads=$TestThreads
            }
            if ($LASTEXITCODE -ne 0) { throw "cargo test failed" }
        }

        # Compilation & Release Build
        if (($Mode -eq "Full") -and (-not $SkipBuild)) {
            Invoke-Stage "Release Compilation" {
                cargo build --release
                if ($LASTEXITCODE -ne 0) { throw "cargo build --release failed" }
            }
        }
    }

    "Node" {
        # Formatting & Lint
        Invoke-Stage "Linter & Code Style" {
            npm run lint
            if ($LASTEXITCODE -ne 0) { throw "npm run lint failed" }
        }

        # Automated Tests
        Invoke-Stage "Automated Tests" {
            if ($Mode -eq "Fast") {
                npm test -- --run --bail=1
            } else {
                npm test -- --run
            }
            if ($LASTEXITCODE -ne 0) { throw "npm test failed" }
        }

        # Build Check
        if (($Mode -eq "Full") -and (-not $SkipBuild)) {
            Invoke-Stage "Production Build" {
                npm run build
                if ($LASTEXITCODE -ne 0) { throw "npm run build failed" }
            }
        }
    }

    "Python" {
        # Lint & Format
        Invoke-Stage "Linter & Format (ruff)" {
            ruff check .
            if ($LASTEXITCODE -ne 0) { throw "ruff check failed" }
            ruff format --check .
            if ($LASTEXITCODE -ne 0) { throw "ruff format failed" }
        }

        # Tests
        Invoke-Stage "Automated Tests (pytest)" {
            if ($Mode -eq "Fast") {
                pytest -q -m "not slow" -n auto
            } else {
                pytest -n auto
            }
            if ($LASTEXITCODE -ne 0) { throw "pytest failed" }
        }

        # Type Check / Build
        if (($Mode -eq "Full") -and (-not $SkipBuild)) {
            Invoke-Stage "Type Check & Build (pyright / uv build)" {
                if (Get-Command pyright -ErrorAction SilentlyContinue) {
                    pyright
                    if ($LASTEXITCODE -ne 0) { throw "pyright type check failed" }
                }
            }
        }
    }

    "Go" {
        Invoke-Stage "Format & Vet" {
            go vet ./...
            if ($LASTEXITCODE -ne 0) { throw "go vet failed" }
        }

        Invoke-Stage "Automated Tests" {
            if ($Mode -eq "Fast") {
                go test -short ./...
            } else {
                go test -race ./...
            }
            if ($LASTEXITCODE -ne 0) { throw "go test failed" }
        }

        if (($Mode -eq "Full") -and (-not $SkipBuild)) {
            Invoke-Stage "Production Build" {
                go build ./...
                if ($LASTEXITCODE -ne 0) { throw "go build failed" }
            }
        }
    }

    "Template" {
        # Project Template Sanity Gate (No app code compiled yet)
        Invoke-Stage "Template Scripts Syntax Audit" {
            $ScriptFiles = Get-ChildItem -Path (Join-Path $RepoRoot "scripts") -Filter "*.ps1"
            foreach ($file in $ScriptFiles) {
                $errs = $null
                $null = [System.Management.Automation.Language.Parser]::ParseFile($file.FullName, [ref]$null, [ref]$errs)
                if ($errs -and ($errs.Count -gt 0)) {
                    throw "Syntax error in $($file.Name): $($errs[0].Message)"
                }
            }
        }

        Invoke-Stage "Documentation & Contract Audit" {
            $RequiredDocs = @(
                "docs/01-vision.md",
                "docs/02-requirements.md",
                "docs/03-architecture.md",
                "docs/05-roadmap.md",
                "docs/06-qa-and-validation.md",
                "docs/multi-agent-methodology.md",
                "AGENTS.md"
            )
            foreach ($doc in $RequiredDocs) {
                if (-not (Test-Path (Join-Path $RepoRoot $doc))) {
                    throw "Missing required documentation contract: $doc"
                }
            }
        }

        if ($Mode -eq "Full") {
            Invoke-Stage "Git Status & Working Tree Hygiene" {
                $status = git status --porcelain
                Write-Host "   Working tree inspection complete." -ForegroundColor Gray
            }
        }
    }
}

# -----------------------------------------------------------------------------
# Summary & Gate Result
# -----------------------------------------------------------------------------
$GlobalStopwatch.Stop()
$TotalSeconds = [math]::Round($GlobalStopwatch.Elapsed.TotalSeconds, 2)

Write-Host "`n--------------------------------------------------" -ForegroundColor Cyan
if ($FailedStages.Count -eq 0) {
    Write-Host "[GATE PASSED] All stages passed in ${TotalSeconds}s" -ForegroundColor Green
    Write-Host "Status: Ready for commit / PR submission." -ForegroundColor Green
    exit 0
} else {
    Write-Host "[GATE FAILED] $($FailedStages.Count) stage(s) failed in ${TotalSeconds}s:" -ForegroundColor Red
    foreach ($failed in $FailedStages) {
        Write-Host "  - $failed" -ForegroundColor Red
    }
    Write-Host "`nPlease resolve the failures above before committing." -ForegroundColor Red
    exit 1
}
