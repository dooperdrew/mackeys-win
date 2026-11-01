---
name: skill-manager
description: Manage and scope Antigravity skills between Local (Workspace: .agents/skills/) and Global (Machine: ~/.gemini/config/skills/) to optimize the 20,000 customization token budget. Use when creating new skills, promoting project skills to global, or demoting global skills to local.
---

# Skill Manager Skill

Provides operational procedures for creating, auditing, promoting, and demoting Antigravity AI skills with explicit scope management to prevent customization token exhaustion.

---

## 1. When to Use This Skill
- When the user asks to create a new skill and you need to decide or configure whether it should be **Local** to this project or **Global** across the machine.
- When the user is hitting the 20,000 token limit warning and needs to audit or demote skills from global to local.
- When a local skill has proven reusable across multiple projects and is tagged for global kit promotion (`[KIT]`).

---

## 2. Scope Architecture & Principles

| Scope | Filesystem Location | Token Impact | Recommended For |
|---|---|---|---|
| **Local (Workspace)** | `<repo-root>/.agents/skills/<name>/SKILL.md` | **0 tokens** in other projects. Loaded only when this repo is opened. | Project-specific tools, domain logic, project QA, custom build recipes. **(Default choice)** |
| **Global (Machine)** | `~/.gemini/config/skills/<name>/SKILL.md` | **Consumes global budget** across every single project and chat session. | Cross-cutting developer utilities used daily in every repo (e.g. general git helpers, global linters). |

---

## 3. Tooling & Automation

The repository includes a PowerShell automation utility: `scripts/manage-skill.ps1`.

### A. List Inventory & Audit Token Impact
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manage-skill.ps1 -Action List
```

### B. Create a New Local Skill (Recommended Default)
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manage-skill.ps1 -Action New -Name "<skill-name>" -Scope Local -Description "<Short description of what the skill does>"
```

### C. Create a New Global Skill
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manage-skill.ps1 -Action New -Name "<skill-name>" -Scope Global -Description "<Short description of what the skill does>"
```

### D. Promote Local Skill to Global
```powershell
# Copy local skill to global (leaves local intact):
powershell -ExecutionPolicy Bypass -File .\scripts\manage-skill.ps1 -Action Promote -Name "<skill-name>"

# Move local skill to global (deletes local copy):
powershell -ExecutionPolicy Bypass -File .\scripts\manage-skill.ps1 -Action Promote -Name "<skill-name>" -Move
```

### E. Demote Global Skill to Local (Free Global Customization Tokens)
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manage-skill.ps1 -Action Demote -Name "<skill-name>" -Force
```

### F. Delete a Skill
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manage-skill.ps1 -Action Delete -Name "<skill-name>" -Scope Local
```

---

## 4. Agent Guidelines
1. **Default to Local:** Always scaffold skills in `.agents/skills/<name>/` unless the user explicitly requests machine-wide availability.
2. **Promote only on empirical evidence:** When a project-specific workflow demonstrates cross-repo utility and passes verification, propose promoting it via `manage-skill.ps1 -Action Promote`.
3. **Preserve Token Budget:** If token limits are approached, use `manage-skill.ps1 -Action List` to identify global skills that can be demoted to local workspace skills.
