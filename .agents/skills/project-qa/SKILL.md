---
name: project-qa
description: Autonomous UI, accessibility, and hardware-in-the-loop QA testing for this project using ui-qa-tester. Use when verifying UI screens, finding dead buttons, auditing contrast and keyboard navigation, or generating empirical verification evidence for docs/06-qa-and-validation.md.
---

# Project QA & Autonomous Verification Skill

This workspace-scoped skill guides the agent in conducting non-intrusive UI testing, accessibility audits, and empirical verification using **`ui-qa-tester`**.

---

## 1. When to Use This Skill
- Before marking a UI or integration milestone as complete in [`docs/05-roadmap.md`](../../../docs/05-roadmap.md).
- To detect dead buttons, broken click targets, or unhandled exceptions without manual clicking.
- To produce empirical evidence for [`docs/06-qa-and-validation.md`](../../../docs/06-qa-and-validation.md) or submit a [`.github/ISSUE_TEMPLATE/qa_verification.yml`](../../../.github/ISSUE_TEMPLATE/qa_verification.yml) report.

---

## 2. Execution Workflow

### Step 1: Ensure the Project is Built
Before launching the QA crawler, compile the application binary or ensure the local web server is running:
```powershell
# Desktop App (Rust):
cargo build --workspace --release

# Web App (Next.js / Vite):
npm run build
```

### Step 2: Configure `qa-session.toml`
Verify `qa-session.toml` in the repository root (copied from `qa-session.template.toml`):
- `target_exe`: Points to the compiled binary (`target\release\<app>.exe`).
- `target_window_title`: Matches the visible window title.
- `destructive_control_regex`: Ensures dangerous actions (e.g., delete, format) are ignored.
- `focus_mode = "background"`: Ensures the crawl runs in the background without stealing mouse or keyboard focus.

### Step 3: Run the QA Actuator
Execute the test run via the repository script:
```powershell
powershell -ExecutionPolicy Bypass -File scripts/run-qa.ps1
```

Or directly via CLI if installed in PATH:
```powershell
ui-qa-cli crawl --config qa-session.toml
```

---

## 3. Interpreting Findings in `qa-reports/`

The crawler outputs structured results into `qa-reports/`:
- `report.md`: Executive summary of traversed controls, state transitions, and detected issues.
- `report.json`: Machine-readable telemetry and hardware/log correlations.
- `step_*.png`: Annotated screenshots showing bounding boxes, accessibility names, and click markers.

### Common Findings & Remediations:
- **`UX-DEAD-BTN` (Dead Button):** A button was clicked but yielded zero UI state change or visual diff. Ensure the button performs an action or reflects a selected/disabled state when already active.
- **`A11Y-LOW-CONTRAST`:** Text element fails WCAG AA minimum 4.5:1 contrast ratio against its background.
- **`A11Y-NO-LABEL`:** An interactive control (ComboBox, TextEdit, Button) lacks an accessible name or ARIA label.
- **`ERR-CRASH` / `ERR-LOG`:** An unhandled panic or error string was detected in stderr/stdout.

---

## 4. Updating Documentation
Following the **Claims Follow Evidence** rule:
1. Review the generated `qa-reports/report.md`.
2. Update [`docs/06-qa-and-validation.md`](../../../docs/06-qa-and-validation.md) with date, build commit, and empirical status (`PASS` or `Pending`).
3. If regressions or bugs are found, open a GitHub Issue using [`.github/ISSUE_TEMPLATE/qa_verification.yml`](../../../.github/ISSUE_TEMPLATE/qa_verification.yml) and attach the report.
