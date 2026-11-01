<#
.SYNOPSIS
    Manages Antigravity AI skills with Local (Workspace) vs Global scope selection.

.DESCRIPTION
    Provides built-in lifecycle management for AI skills:
    - Scaffolds new skills either locally in .agents/skills/<name> (default, 0 global tokens)
      or globally in ~/.gemini/config/skills/<name> (available across all projects).
    - Promotes a proven local skill to global scope when tagged for kit promotion ([KIT]).
    - Demotes a global skill to local workspace scope to free up global customization tokens.
    - Lists all local and global skills with token impact warnings.

.PARAMETER Action
    Operation to perform: "New", "List", "Promote", "Demote", "Delete".

.PARAMETER Name
    Unique lowercase identifier for the skill (e.g. "db-migration", "hardware-probe").

.PARAMETER Scope
    "Local" (workspace-scoped, stored in .agents/skills/) or
    "Global" (machine-scoped, stored in ~/.gemini/config/skills/).
    Defaults to "Local".

.PARAMETER Description
    A concise description explaining what the skill does and when the agent should trigger it.

.PARAMETER Force
    Overwrites destination if it already exists during Promote or Demote.

.PARAMETER Move
    During Promote, moves the skill rather than copying it, removing the local copy.

.EXAMPLE
    .\scripts\manage-skill.ps1 -Action New -Name "api-tester" -Scope Local -Description "Test REST endpoints with curl"
    .\scripts\manage-skill.ps1 -Action List
    .\scripts\manage-skill.ps1 -Action Promote -Name "api-tester"
    .\scripts\manage-skill.ps1 -Action Demote -Name "api-tester" -Force
#>

[CmdletBinding()]
param(
    [ValidateSet("New", "List", "Promote", "Demote", "Delete")]
    [string]$Action = "List",

    [string]$Name = "",

    [ValidateSet("Local", "Global")]
    [string]$Scope = "Local",

    [string]$Description = "",

    [switch]$Force,
    [switch]$Move
)

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

$LocalSkillsDir = Join-Path $RepoRoot ".agents\skills"
$GlobalConfigDir = Join-Path $env:USERPROFILE ".gemini\config"
$GlobalSkillsDir = Join-Path $GlobalConfigDir "skills"

function Ensure-Dir([string]$Path) {
    if (-not (Test-Path $Path)) { New-Item -ItemType Directory -Path $Path -Force | Out-Null }
}

function Format-SanitizedPath([string]$Path) {
    if (-not [string]::IsNullOrEmpty($env:USERPROFILE)) {
        return ($Path -replace [regex]::Escape($env:USERPROFILE), "~")
    }
    return $Path
}

# -----------------------------------------------------------------------------
# ACTION: List
# -----------------------------------------------------------------------------
if ($Action -eq "List") {
    Write-Host "==================================================" -ForegroundColor Cyan
    Write-Host "         Antigravity Skill Inventory & Scope       " -ForegroundColor Cyan
    Write-Host "==================================================" -ForegroundColor Cyan

    Write-Host "`n[LOCAL] Local Workspace Skills (.agents/skills/ - 0 Global Tokens):" -ForegroundColor Green
    if (Test-Path $LocalSkillsDir) {
        $LocalSkills = Get-ChildItem -Directory $LocalSkillsDir -ErrorAction SilentlyContinue
        if ($LocalSkills) {
            foreach ($s in $LocalSkills) {
                $SkillMd = Join-Path $s.FullName "SKILL.md"
                $DescText = ""
                if (Test-Path $SkillMd) {
                    $Content = Get-Content $SkillMd -Raw
                    if ($Content -match 'description:\s*([^\r\n]+)') { $DescText = $Matches[1].Trim() }
                }
                Write-Host "  * $($s.Name)" -ForegroundColor Green -NoNewline
                if ($DescText) { Write-Host " : $DescText" -ForegroundColor Gray } else { Write-Host "" }
            }
        } else {
            Write-Host "  (No local skills found)" -ForegroundColor Gray
        }
    } else {
        Write-Host "  (Local .agents/skills directory not initialized)" -ForegroundColor Gray
    }

    Write-Host "`n[GLOBAL] Global Machine Skills (~/.gemini/config/skills/ - Active in ALL Projects):" -ForegroundColor Yellow
    if (Test-Path $GlobalSkillsDir) {
        $GlobalSkills = Get-ChildItem -Directory $GlobalSkillsDir -ErrorAction SilentlyContinue
        if ($GlobalSkills) {
            foreach ($g in $GlobalSkills) {
                Write-Host "  * $($g.Name) [Consumes global token budget]" -ForegroundColor Yellow
            }
        } else {
            Write-Host "  (No standalone global skills found in ~/.gemini/config/skills/)" -ForegroundColor Gray
        }
    } else {
        Write-Host "  (Global skills directory not created yet)" -ForegroundColor Gray
    }

    Write-Host "`n[TIP] Keep skills 'Local' to prevent exceeding the 20,000 token customization budget." -ForegroundColor Cyan
    exit 0
}

# Validation for Name
if ([string]::IsNullOrWhiteSpace($Name)) {
    throw "Parameter -Name is required for Action '$Action'."
}
$Name = $Name.ToLowerInvariant().Trim()

$TargetLocalPath = Join-Path $LocalSkillsDir $Name
$TargetGlobalPath = Join-Path $GlobalSkillsDir $Name

# -----------------------------------------------------------------------------
# ACTION: New
# -----------------------------------------------------------------------------
if ($Action -eq "New") {
    if ([string]::IsNullOrWhiteSpace($Description)) {
        $Description = "Use this skill when performing tasks related to $Name."
    }

    $TargetDir = if ($Scope -eq "Local") { $TargetLocalPath } else { $TargetGlobalPath }

    if ((Test-Path $TargetDir) -and (-not $Force)) {
        throw "Skill '$Name' already exists at $TargetDir. Use -Force to overwrite."
    }

    Ensure-Dir $TargetDir
    Ensure-Dir (Join-Path $TargetDir "scripts")
    Ensure-Dir (Join-Path $TargetDir "examples")

    $Lines = @(
        "---",
        "name: $Name",
        "description: $Description",
        "---",
        "",
        "# $Name Skill Runbook",
        "",
        "This runbook guides the agent when executing workflows for **$Name**.",
        "",
        "---",
        "",
        "## 1. When to Use This Skill",
        "- $Description",
        "- Trigger condition: When the user asks to work with $Name.",
        "",
        "---",
        "",
        "## 2. Step-by-Step Execution Procedure",
        "",
        "1. **Pre-flight Check:**",
        "   Verify required tools and environment.",
        "",
        "2. **Core Workflow:**",
        "   Execute required operational scripts or commands.",
        "",
        "3. **Verification & Audit:**",
        "   Confirm that the operation completed cleanly without errors.",
        "",
        "---",
        "",
        "## 3. Reference Files & Scripts",
        "- scripts/: Helper scripts for automated execution.",
        "- examples/: Example configuration or test payloads."
    )
    $SkillContent = $Lines -join "`r`n"

    Set-Content -Path (Join-Path $TargetDir "SKILL.md") -Value $SkillContent -Encoding utf8

    Write-Host "[OK] Created new $Scope skill: $Name" -ForegroundColor Green
    Write-Host "Location: $(Format-SanitizedPath $TargetDir)\SKILL.md" -ForegroundColor Cyan
    if ($Scope -eq "Global") {
        Write-Warning "This skill is Global and will consume customization tokens in EVERY project."
    } else {
        Write-Host "Scope is Local: Consumes 0 tokens when other projects are open." -ForegroundColor Gray
    }
    exit 0
}

# -----------------------------------------------------------------------------
# ACTION: Promote (Local -> Global)
# -----------------------------------------------------------------------------
if ($Action -eq "Promote") {
    if (-not (Test-Path $TargetLocalPath)) {
        throw "Local skill '$Name' not found at $TargetLocalPath."
    }

    Ensure-Dir $GlobalSkillsDir
    if ((Test-Path $TargetGlobalPath) -and (-not $Force)) {
        throw "Global skill '$Name' already exists at $TargetGlobalPath. Use -Force to overwrite."
    }

    if ($Move) {
        Move-Item -Path $TargetLocalPath -Destination $TargetGlobalPath -Force
        Write-Host "[OK] Moved '$Name' from Local to Global!" -ForegroundColor Green
    } else {
        Copy-Item -Path $TargetLocalPath -Destination $TargetGlobalPath -Recurse -Force
        Write-Host "[OK] Copied '$Name' from Local to Global!" -ForegroundColor Green
    }

    Write-Host "Source: $(Format-SanitizedPath $TargetLocalPath)" -ForegroundColor Gray
    Write-Host "Destination: $(Format-SanitizedPath $TargetGlobalPath)" -ForegroundColor Cyan
    Write-Warning "Skill '$Name' is now globally active in all projects and will consume global customization tokens."
    exit 0
}

# -----------------------------------------------------------------------------
# ACTION: Demote (Global -> Local)
# -----------------------------------------------------------------------------
if ($Action -eq "Demote") {
    if (-not (Test-Path $TargetGlobalPath)) {
        throw "Global skill '$Name' not found at $TargetGlobalPath."
    }

    Ensure-Dir $LocalSkillsDir
    if ((Test-Path $TargetLocalPath) -and (-not $Force)) {
        throw "Local skill '$Name' already exists in this project at $TargetLocalPath. Use -Force to overwrite."
    }

    if (Test-Path $TargetLocalPath) {
        Remove-Item -Path $TargetLocalPath -Recurse -Force
    }

    Move-Item -Path $TargetGlobalPath -Destination $TargetLocalPath -Force
    Write-Host "[OK] Demoted '$Name' from Global to Local!" -ForegroundColor Green
    Write-Host "Moved to: $(Format-SanitizedPath $TargetLocalPath)" -ForegroundColor Cyan
    Write-Host "Freed global customization tokens! '$Name' is now active only in this project." -ForegroundColor Green
    exit 0
}

# -----------------------------------------------------------------------------
# ACTION: Delete
# -----------------------------------------------------------------------------
if ($Action -eq "Delete") {
    $Deleted = $false
    if (($Scope -eq "Local") -and (Test-Path $TargetLocalPath)) {
        Remove-Item -Path $TargetLocalPath -Recurse -Force
        Write-Host "[OK] Deleted local skill: $(Format-SanitizedPath $TargetLocalPath)" -ForegroundColor Green
        $Deleted = $true
    }
    if (($Scope -eq "Global") -and (Test-Path $TargetGlobalPath)) {
        Remove-Item -Path $TargetGlobalPath -Recurse -Force
        Write-Host "[OK] Deleted global skill: $(Format-SanitizedPath $TargetGlobalPath)" -ForegroundColor Green
        $Deleted = $true
    }

    if (-not $Deleted) {
        Write-Warning "Skill '$Name' not found in $Scope scope."
    }
    exit 0
}
