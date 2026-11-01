<#
.SYNOPSIS
    Automates isolated Git worktrees for parallel agent tasks and Draft PR verification.

.DESCRIPTION
    Enables parallel development without branch-switching friction or git collision:
    - Spawns isolated sibling worktrees in .worktrees/<task-id> on branch task/<task-id>.
    - Allows subagents and developers to build, test, and run verification gates concurrently.
    - Integrates with scripts/verify-gate.ps1 to enforce quality before merge.
    - Cleans up worktrees and branch references upon completion.

.PARAMETER Action
    "New", "List", "Gate", "Merge", "Remove". Defaults to "List".

.PARAMETER Task
    Task or issue identifier (e.g., "M1.1", "issue-42", "auth-token-cache").

.PARAMETER Base
    Base branch to branch off from. Defaults to "main".

.PARAMETER Mode
    Gate mode when running "Gate" or "Merge" ("Fast" or "Full"). Defaults to "Fast".

.PARAMETER Force
    Forces removal or overwrite if target worktree already exists.

.EXAMPLE
    .\scripts\manage-worktree.ps1 -Action New -Task "M1.1"
    .\scripts\manage-worktree.ps1 -Action List
    .\scripts\manage-worktree.ps1 -Action Gate -Task "M1.1" -Mode Fast
    .\scripts\manage-worktree.ps1 -Action Merge -Task "M1.1"
    .\scripts\manage-worktree.ps1 -Action Remove -Task "M1.1"
#>

[CmdletBinding()]
param(
    [ValidateSet("New", "List", "Gate", "Merge", "Remove")]
    [string]$Action = "List",

    [string]$Task = "",

    [string]$Base = "main",

    [ValidateSet("Fast", "Full")]
    [string]$Mode = "Fast",

    [switch]$Force
)

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

$WorktreesRoot = Join-Path $RepoRoot ".worktrees"

function Ensure-Dir([string]$Path) {
    if (-not (Test-Path $Path)) { New-Item -ItemType Directory -Path $Path -Force | Out-Null }
}

# -----------------------------------------------------------------------------
# ACTION: List
# -----------------------------------------------------------------------------
if ($Action -eq "List") {
    Write-Host "==================================================" -ForegroundColor Cyan
    Write-Host "            Active Git Worktrees                  " -ForegroundColor Cyan
    Write-Host "==================================================" -ForegroundColor Cyan
    git worktree list
    exit 0
}

# Validation for Task parameter
if ([string]::IsNullOrWhiteSpace($Task)) {
    throw "Parameter -Task is required for Action '$Action'."
}
$Task = $Task.ToLowerInvariant().Trim() -replace "[^a-z0-9._-]", "-"
$BranchName = "task/$Task"
$TargetDir = Join-Path $WorktreesRoot $Task

# -----------------------------------------------------------------------------
# ACTION: New
# -----------------------------------------------------------------------------
if ($Action -eq "New") {
    Ensure-Dir $WorktreesRoot

    if (Test-Path $TargetDir) {
        if ($Force) {
            Write-Host "Target $TargetDir exists. Removing existing worktree..." -ForegroundColor Yellow
            git worktree remove $TargetDir --force
        } else {
            throw "Worktree already exists at $TargetDir. Use -Force to overwrite or choose another task name."
        }
    }

    Write-Host ">> Creating worktree for '$Task' on branch '$BranchName' from '$Base'..." -ForegroundColor Yellow
    
    # Check if branch exists
    $BranchExists = git branch --list $BranchName
    if ($BranchExists) {
        git worktree add $TargetDir $BranchName
    } else {
        git worktree add -b $BranchName $TargetDir $Base
    }

    if ($LASTEXITCODE -ne 0) { throw "git worktree add failed." }

    Write-Host "[OK] Worktree initialized successfully!" -ForegroundColor Green
    Write-Host "Path:   $TargetDir" -ForegroundColor Cyan
    Write-Host "Branch: $BranchName" -ForegroundColor Cyan
    Write-Host "`nTo switch to the worktree in terminal:" -ForegroundColor Gray
    Write-Host "  cd `"$TargetDir`"" -ForegroundColor Gray
    exit 0
}

# -----------------------------------------------------------------------------
# ACTION: Gate
# -----------------------------------------------------------------------------
if ($Action -eq "Gate") {
    if (-not (Test-Path $TargetDir)) {
        throw "Worktree for '$Task' not found at $TargetDir."
    }

    Write-Host ">> Running Verification Gate ($Mode Mode) inside $TargetDir..." -ForegroundColor Yellow
    $GateScript = Join-Path $TargetDir "scripts\verify-gate.ps1"
    if (-not (Test-Path $GateScript)) { $GateScript = Join-Path $RepoRoot "scripts\verify-gate.ps1" }

    Push-Location $TargetDir
    try {
        powershell -ExecutionPolicy Bypass -File $GateScript -Mode $Mode
        if ($LASTEXITCODE -ne 0) { throw "Verification gate failed in worktree $TargetDir" }
        Write-Host "[OK] Worktree '$Task' passed the verification gate!" -ForegroundColor Green
    } finally {
        Pop-Location
    }
    exit 0
}

# -----------------------------------------------------------------------------
# ACTION: Merge (Sole Committer Protocol)
# -----------------------------------------------------------------------------
if ($Action -eq "Merge") {
    if (-not (Test-Path $TargetDir)) {
        throw "Worktree for '$Task' not found at $TargetDir."
    }

    Write-Host ">> [Sole Committer Protocol] Validating Full Gate before merge..." -ForegroundColor Yellow
    $GateScript = Join-Path $TargetDir "scripts\verify-gate.ps1"
    if (-not (Test-Path $GateScript)) { $GateScript = Join-Path $RepoRoot "scripts\verify-gate.ps1" }

    Push-Location $TargetDir
    try {
        powershell -ExecutionPolicy Bypass -File $GateScript -Mode Full
        if ($LASTEXITCODE -ne 0) { throw "Cannot merge: Full verification gate failed." }
    } finally {
        Pop-Location
    }

    Write-Host ">> Merging branch '$BranchName' into '$Base'..." -ForegroundColor Yellow
    git checkout $Base
    git merge --no-ff $BranchName -m "merge($Task): integrate verified worktree $BranchName"
    if ($LASTEXITCODE -ne 0) { throw "git merge failed. Resolve conflicts on $Base." }

    Write-Host "[OK] Successfully merged '$BranchName' into '$Base'!" -ForegroundColor Green
    Write-Host "You can now push to remote and remove the worktree via:" -ForegroundColor Gray
    Write-Host "  .\scripts\manage-worktree.ps1 -Action Remove -Task `"$Task`"" -ForegroundColor Gray
    exit 0
}

# -----------------------------------------------------------------------------
# ACTION: Remove
# -----------------------------------------------------------------------------
if ($Action -eq "Remove") {
    if (Test-Path $TargetDir) {
        Write-Host ">> Removing worktree at $TargetDir..." -ForegroundColor Yellow
        git worktree remove $TargetDir --force
    } else {
        Write-Warning "Worktree path $TargetDir was not found on disk."
    }

    git worktree prune

    if ($Force) {
        Write-Host ">> Deleting local branch '$BranchName'..." -ForegroundColor Yellow
        git branch -D $BranchName 2>$null
    }

    Write-Host "[OK] Worktree '$Task' removed and cleaned up." -ForegroundColor Green
    exit 0
}
