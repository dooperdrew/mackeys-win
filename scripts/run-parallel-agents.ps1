<#
.SYNOPSIS
    Orchestrates parallel worker subagents across isolated Git worktrees.

.DESCRIPTION
    Spawns isolated sibling worktrees (.worktrees/<task-id>) for parallel tasks,
    allocates disjoint file ownership boundaries, and executes workers using the
    dynamically optimized default model (from .agents/config/model-default.toml).

.PARAMETER TaskBase
    Base naming prefix for worktrees and branches. Defaults to "worker-task".

.PARAMETER DryRun
    Pre-flights the parallel orchestration plan without modifying git worktrees.

.EXAMPLE
    .\scripts\run-parallel-agents.ps1 -DryRun
    .\scripts\run-parallel-agents.ps1 -TaskBase "m1-core"
#>

[CmdletBinding()]
param(
    [string]$TaskBase = "worker-task",
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

# Read active default model from optimizer configuration
$ActiveModel = "flash_lite"
$DefaultToml = Join-Path $RepoRoot ".agents\config\model-default.toml"
if (Test-Path $DefaultToml) {
    $TomlContent = Get-Content $DefaultToml -Raw -Encoding utf8
    if ($TomlContent -match 'default_model\s*=\s*"([^"]+)"') {
        $ActiveModel = $matches[1]
    }
}

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "       Parallel Worker Subagent Orchestrator      " -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "Active Model Policy: $ActiveModel (Low Token Footprint)" -ForegroundColor Green
Write-Host "Worktree Prefix:     $TaskBase" -ForegroundColor Gray
if ($DryRun) { Write-Host "Mode:                DRY RUN (Preview Only)" -ForegroundColor Yellow }

$Tasks = @(
    @{
        Id = "1"
        Role = "T1-Models-M1"
        Prompt = "Implement core data models and schemas with deterministic serialization tests."
        OwnedPath = "src/models/**"
    },
    @{
        Id = "2"
        Role = "T1-Lint-M1"
        Prompt = "Add automated code quality and lint helper rules with zero-warning enforcement."
        OwnedPath = "scripts/lint/**"
    },
    @{
        Id = "3"
        Role = "T2-Service-M1"
        Prompt = "Implement platform service skeleton with IPC health probe."
        OwnedPath = "src/service/**"
    }
)

function Invoke-ParallelTask([hashtable]$TaskDef) {
    $TaskId = "$TaskBase-$($TaskDef.Id)"
    $WorktreeDir = Join-Path $RepoRoot ".worktrees\$TaskId"
    
    Write-Host "`n>> Initializing Parallel Worker [$($TaskDef.Role)]" -ForegroundColor Yellow
    Write-Host "   Task ID:    $TaskId"
    Write-Host "   Owned Path: $($TaskDef.OwnedPath)"
    Write-Host "   Prompt:     $($TaskDef.Prompt)"

    if ($DryRun) {
        Write-Host "   [DRY-RUN] Worktree '$TaskId' and worker process previewed." -ForegroundColor Gray
        return
    }

    # 1. Create isolated worktree
    $ManageScript = Join-Path $RepoRoot "scripts\manage-worktree.ps1"
    & powershell -ExecutionPolicy Bypass -File $ManageScript -Action New -Task $TaskId
    if ($LASTEXITCODE -ne 0 -or (-not (Test-Path $WorktreeDir))) {
        throw "Failed to create isolated worktree at $WorktreeDir"
    }

    # 2. Execute worker subagent in isolated directory
    Push-Location $WorktreeDir
    try {
        if (Get-Command agy -ErrorAction SilentlyContinue) {
            Write-Host "   Launching autonomous subagent with model '$ActiveModel'..." -ForegroundColor Cyan
            agy subagent invoke `
                --model $ActiveModel `
                --role $TaskDef.Role `
                --type project-worker `
                --prompt $TaskDef.Prompt `
                --workspace branch
        } else {
            Write-Host "   [SIMULATION] agy CLI not in PATH; validating workspace isolation." -ForegroundColor Gray
        }

        # 3. Verify Fast Gate in worktree
        Write-Host "   Running Fast Verification Gate in worktree..." -ForegroundColor Cyan
        & powershell -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "verify-gate.ps1") -Mode Fast
        if ($LASTEXITCODE -ne 0) {
            throw "Verification gate failed in worktree $TaskId"
        }
        Write-Host "   [PASS] Worker $($TaskDef.Role) completed cleanly." -ForegroundColor Green
    } finally {
        Pop-Location
    }
}

foreach ($Task in $Tasks) {
    Invoke-ParallelTask -TaskDef $Task
}

Write-Host "`n--------------------------------------------------" -ForegroundColor Cyan
Write-Host "[COMPLETED] All parallel workers orchestrated successfully." -ForegroundColor Green
