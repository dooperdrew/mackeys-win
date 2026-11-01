# Workspace-Scoped Rules (`.agents/rules/`)

This directory allows you to define **project-specific operating rules** for AI assistants (Antigravity and other agents).

---

## 💡 Why Workspace-Scoped Rules?

Unlike global rules configured in `~/.gemini/config/`, rules placed in this directory:
1. **Zero Global Token Cost:** They load **only** when this specific project workspace is open in Antigravity.
2. **Version-Controlled:** They are checked into Git and automatically shared with any human developer or AI agent working on this repository.
3. **Domain-Specific:** Perfect for repository-specific architecture invariants, database migration protocols, or API conventions that don't belong in other projects.

---

## 🛠️ How to Add a Rule

1. Create a markdown file here: `.agents/rules/<rule-name>.md`.
2. Add YAML frontmatter specifying its trigger conditions (or leave blank to load with this workspace):
   ```markdown
   ---
   description: Enforcement of pure logic boundaries in the core module
   trigger: model_decision
   ---

   # Rule Title
   - Rule 1: All financial calculations must live in `src/lib/calculations.ts`.
   - Rule 2: Never import database models or ORM clients into calculation modules.
   ```
3. The Antigravity agent will automatically discover and follow these rules during its sessions.
