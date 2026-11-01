<#
.SYNOPSIS
    Collects execution metrics and token economics across AI models.

.DESCRIPTION
    Scans subagent task logs, verification gate output, and fallback records to
    aggregate latency, failure rates, and token consumption by model tier.
    Stores the resulting statistical database in .agents/config/model-stats.json.
#>

[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$RepoRoot = Split-Path -Parent $PSScriptRoot
$ConfigDir = Join-Path $RepoRoot ".agents\config"
$StatsFile = Join-Path $ConfigDir "model-stats.json"
$FallbackFile = Join-Path $RepoRoot "docs\model-fallback-log.md"
$TasksLogDir = Join-Path $RepoRoot ".system_generated\tasks"

if (-not (Test-Path $ConfigDir)) {
    New-Item -ItemType Directory -Path $ConfigDir -Force | Out-Null
}

$Stats = @{}
if (Test-Path $StatsFile) {
    try {
        $Raw = Get-Content $StatsFile -Raw -Encoding utf8
        if (-not [string]::IsNullOrWhiteSpace($Raw)) {
            $Obj = ConvertFrom-Json $Raw -ErrorAction SilentlyContinue
            if ($Obj) {
                foreach ($Prop in $Obj.psobject.Properties) {
                    $Stats[$Prop.Name] = @{
                        Runs = [int]$Prop.Value.Runs
                        Successes = [int]$Prop.Value.Successes
                        Failures = [int]$Prop.Value.Failures
                        TotalDurationSecs = [double]$Prop.Value.TotalDurationSecs
                        TotalTokens = [double]$Prop.Value.TotalTokens
                        AvgDurationSecs = [double]$Prop.Value.AvgDurationSecs
                        AvgTokens = [double]$Prop.Value.AvgTokens
                        SuccessRate = [double]$Prop.Value.SuccessRate
                        LastUpdated = [string]$Prop.Value.LastUpdated
                    }
                }
            }
        }
    } catch {
        Write-Warning "Could not parse existing $StatsFile, re-initializing..."
    }
}

function Add-ModelMeasurement([string]$ModelName, [double]$DurationSecs, [double]$TokenCount, [bool]$IsSuccess) {
    if ([string]::IsNullOrWhiteSpace($ModelName) -or ($ModelName -match '^[-—]+$') -or ($ModelName -notmatch '^[a-zA-Z]')) { return }
    $ModelName = $ModelName.Trim().ToLowerInvariant()
    
    if (-not $Stats.ContainsKey($ModelName)) {
        $Stats[$ModelName] = @{
            Runs = 0
            Successes = 0
            Failures = 0
            TotalDurationSecs = 0.0
            TotalTokens = 0.0
            AvgDurationSecs = 0.0
            AvgTokens = 0.0
            SuccessRate = 1.0
            LastUpdated = (Get-Date -Format "yyyy-MM-dd HH:mm:ss")
        }
    }

    $Entry = $Stats[$ModelName]
    $Entry.Runs = [int]$Entry.Runs + 1
    if ($IsSuccess) {
        $Entry.Successes = [int]$Entry.Successes + 1
    } else {
        $Entry.Failures = [int]$Entry.Failures + 1
    }

    $Entry.TotalDurationSecs = [double]$Entry.TotalDurationSecs + $DurationSecs
    $Entry.TotalTokens = [double]$Entry.TotalTokens + $TokenCount

    if ($Entry.Runs -gt 0) {
        $Entry.AvgDurationSecs = [math]::Round(([double]$Entry.TotalDurationSecs / [double]$Entry.Runs), 2)
        $Entry.AvgTokens = [math]::Round(([double]$Entry.TotalTokens / [double]$Entry.Runs), 0)
        $Entry.SuccessRate = [math]::Round(([double]$Entry.Successes / [double]$Entry.Runs), 4)
    }
    $Entry.LastUpdated = (Get-Date -Format "yyyy-MM-dd HH:mm:ss")
}

# 1. Parse Subagent Task Logs if available
if (Test-Path $TasksLogDir) {
    $LogFiles = Get-ChildItem -Path $TasksLogDir -Recurse -Filter "*.log" -ErrorAction SilentlyContinue
    foreach ($File in $LogFiles) {
        $Content = Get-Content $File.FullName -Raw -ErrorAction SilentlyContinue
        if ($Content) {
            $Model = "flash_lite"
            if ($Content -match 'Model\s*used:\s*([\w_-]+)') { $Model = $matches[1] }
            elseif ($Content -match 'model\s*=\s*"([\w_-]+)"') { $Model = $matches[1] }

            $Duration = 1.0
            if ($Content -match 'Duration:\s*([\d\.]+)s') { $Duration = [double]$matches[1] }
            elseif ($Content -match 'in\s*([\d\.]+)s') { $Duration = [double]$matches[1] }

            $Tokens = 0.0
            if ($Content -match 'Tokens:\s*(\d+)') { $Tokens = [double]$matches[1] }

            $IsSuccess = -not ($Content -match 'ERROR|FATAL|EXCEPTION|FAIL')
            Add-ModelMeasurement -ModelName $Model -DurationSecs $Duration -TokenCount $Tokens -IsSuccess $IsSuccess
        }
    }
}

# 2. Parse Fallback Log Entries
if (Test-Path $FallbackFile) {
    $FallbackLines = Get-Content $FallbackFile -Encoding utf8 -ErrorAction SilentlyContinue
    foreach ($Line in $FallbackLines) {
        if ($Line -match '\|\s*[^\|]+\s*\|\s*[^\|]+\s*\|\s*([\w_-]+)\s*\|\s*([\w_-]+)\s*\|\s*([^\|]+)\s*\|') {
            $TargetModel = $matches[1].Trim()
            $UsedModel = $matches[2].Trim()
            if ($TargetModel -and ($TargetModel -notmatch '^[-—]+$') -and ($TargetModel -ne "Target model") -and ($TargetModel -match '^[a-zA-Z]')) {
                Add-ModelMeasurement -ModelName $TargetModel -DurationSecs 0 -TokenCount 0 -IsSuccess $false
                Add-ModelMeasurement -ModelName $UsedModel -DurationSecs 2.0 -TokenCount 500 -IsSuccess $true
            }
        }
    }
}

# Ensure standard model tiers have baseline representation
$KnownTiers = @("flash_lite", "flash", "pro")
foreach ($Tier in $KnownTiers) {
    if (-not $Stats.ContainsKey($Tier)) {
        $Stats[$Tier] = @{
            Runs = 0
            Successes = 0
            Failures = 0
            TotalDurationSecs = 0.0
            TotalTokens = 0.0
            AvgDurationSecs = 0.0
            AvgTokens = 0.0
            SuccessRate = 1.0
            LastUpdated = (Get-Date -Format "yyyy-MM-dd HH:mm:ss")
        }
    }
}

$JsonPayload = $Stats | ConvertTo-Json -Depth 5
[System.IO.File]::WriteAllText($StatsFile, $JsonPayload, [System.Text.UTF8Encoding]::new($false))
Write-Host "[OK] Model metrics collected and saved to .agents/config/model-stats.json" -ForegroundColor Green
