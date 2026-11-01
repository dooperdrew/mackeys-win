---
name: model-optimizer
description: Autonomous monitoring and dynamic optimization of AI model routing, execution latency, and token economics.
---

# Model Optimizer Skill Runbook

This runbook guides agents and automated pipelines in monitoring, auditing, and dynamically tuning AI model selection across autonomous tasks.

---

## 1. When to Use This Skill
- **Continuous Learning Loop:** After test executions, CI jobs, or batched parallel worker subagents complete.
- **Token Economics Tuning:** When optimizing execution speed and token budgets to stay well within quotas.
- **Model Fallback Recovery:** When auditing or adapting following model rate limits, API deprecations, or quota exhausts.

---

## 2. Core Execution Procedure

1. **Collect Empirical Telemetry:**
   Run the metrics collector to scan subagent task logs, gate results, and fallback records:
   ```powershell
   powershell -ExecutionPolicy Bypass -File .\scripts\collect-model-stats.ps1
   ```
   *Outputs:* Aggregated statistical data saved in `.agents/config/model-stats.json`.

2. **Evaluate & Apply Policy Optimization:**
   Run the decision engine to evaluate performance envelopes and update repository defaults:
   ```powershell
   powershell -ExecutionPolicy Bypass -File .\scripts\apply-model-optimisation.ps1
   ```
   *Outputs:*
   - Updates `.agents/config/model-default.toml` when another model tier delivers superior token/speed efficiency with $\ge 95\%$ reliability.
   - Appends an audit check entry to `docs/model-optimization-report.md`.

3. **Autonomous Background Daemon (Optional):**
   To keep model defaults continuously fine-tuned during intensive pair programming or swarm work:
   ```powershell
   powershell -ExecutionPolicy Bypass -File .\scripts\start-model-optimiser.ps1 -IntervalSeconds 300
   ```

---

## 3. Architecture & Artifacts

- `scripts/collect-model-stats.ps1`: Empirical log parser and statistical aggregator.
- `scripts/apply-model-optimisation.ps1`: Threshold decision engine and policy updater.
- `scripts/start-model-optimiser.ps1`: Autonomous periodic daemon runner.
- `.agents/config/model-stats.json`: Quantitative runtime performance database.
- `.agents/config/model-default.toml`: Active repository-wide default model routing configuration.
- `docs/model-optimization-report.md`: Living markdown audit trail.
