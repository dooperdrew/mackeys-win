# Skill Scope & Token Economics Guide

This guide explains how to control the visibility of Antigravity AI skills and maximize the token efficiency of your development sessions.

---

## 1. Local vs. Global Skill Architecture

Antigravity supports two discovery scopes for AI skills:

- **Local Skills (`.agents/skills/<name>/`):**
  - **Scope:** Workspace-only. Loaded strictly when this project is active.
  - **Token Impact:** **0 Global Tokens**. Does not consume any portion of your global 20,000-token customization budget.
  - **Best For:** Project-specific runbooks, domain schemas, specialized hardware QA scripts, or internal tools.

- **Global Skills (`~/.gemini/config/skills/<name>/`):**
  - **Scope:** Machine-wide. Loaded automatically into **every Antigravity chat session** across all repositories.
  - **Token Impact:** Consumes global tokens proportional to documentation length on every prompt.
  - **Best For:** Universal development tools used across all coding projects (e.g., git helpers, universal linter runners).

---

## 2. Managing Skill Scopes (`scripts/manage-skill.ps1`)

The repository includes a dedicated CLI manager for skill lifecycle operations:

### Creating a New Local Skill
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manage-skill.ps1 -Action New -Name "api-tester" -Scope Local -Description "Test API endpoints"
```
*Creates `.agents/skills/api-tester/SKILL.md` with zero global token overhead.*

### Promoting a Proven Local Skill to Global
When a skill proves useful across multiple independent projects:
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manage-skill.ps1 -Action Promote -Name "api-tester"
```

### Demoting a Global Skill back to Local
To reclaim global tokens for your session budget:
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manage-skill.ps1 -Action Demote -Name "api-tester"
```

### Auditing Active Skill Token Economics
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manage-skill.ps1 -Action List
```

---

## 3. Synergy with Autonomous Model Optimization

Skill scoping directly integrates with the **Model Optimizer** (`.agents/skills/model-optimizer/`):
- Local skills ensure that subagent workers (`flash_lite` and `flash`) operate with maximum context window availability.
- The `collect-model-stats.ps1` script logs token consumption per task, ensuring your team identifies skills that can be compressed or demoted to local scope.
- See [`docs/model-optimization-report.md`](model-optimization-report.md) for live telemetry and model tier recommendations.
