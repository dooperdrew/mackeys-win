# Lessons Learned

Living log of things that went wrong or surprisingly right, and the rule they produced.
**[KIT ✓]** = promoted to the `project-kickstart` plugin. See `docs/retrospective.md` for evidence.

| Date | Context | What happened | Rule / fix adopted | Scope |
|---|---|---|---|---|
| 2026-10-25 | M0 | Hardware probe showed Fn stripped by `kbdhid.sys`, BT VID `0x004C` | Spike unknowns with a probe tool before designing | [KIT ✓] |
| 2026-10-25 | M1–M10 workers | `cargo` not on PATH in nearly every worker shell | Environment preamble in every worker prompt / AGENTS.md | [KIT ✓] |
| 2026-10-25 | M0 | Tool installed mid-session not found | Refresh PATH from the registry after installs | [KIT ✓] |
| 2026-10-25 | M2 integration | `IpcServer::start` signature changed by a worker, caller broken | Interface changes run alone/first; lead re-runs the gate on the merged tree | [KIT ✓] |
| 2026-10-25 | M1–M4 | Missing `windows` crate features / undeclared deps | Declare deps/features in the same change; `cargo add --package` | [KIT ✓] |
| 2026-10-25 | M2, M10, repo health | Named-pipe and global-atomic test races, test hang | Unique OS object names, readiness channels, timeouts | [KIT ✓] |
| 2026-10-25 | M9 | Unbounded search over `~/.cargo/registry` | Scope searches to the workspace | [KIT ✓] |
| 2026-10-25 | v0.1.0/v0.2.0 | Releases and docs claimed quality while hardware QA was Pending | Claims follow evidence; release notes list Pending QA | [KIT ✓] |
| 2026-10-25 | docs | PowerShell corrupted UTF-8 characters | Force `-Encoding utf8`; prefer file tools | [KIT ✓] |
| 2026-10-25 | packaging | `dist/*.zip` committed | Artifacts go to GitHub Releases only | [KIT ✓] |
| 2026-10-25 | orchestration | Worker agent defined only in a chat | Persist agent definitions as files | [KIT ✓] |
| 2026-10-25 | Italian ISO | Right Option synthesizes `LCONTROL` `sc=0x21D` | Swallow `0x21D` when RAlt is remapped | project |
| 2026-10-25 | M11 | GitHub Copilot without permanent instructions could violate hook budgets and Sole Committer rule | Author `.github/copilot-instructions.md` and `agent_task.yml` | [KIT ✓] |
| 2026-10-25 | distribution | Download & install options were built but absent from homepage/README | Document all distribution channels (WinGet, Scoop, Wizard, Portable) prominently in README | [KIT ✓] |
| 2026-10-25 | PowerShell | External CLI stdout capture (`gh`, `git`) on Windows corrupted non-ASCII characters | Set `[Console]::OutputEncoding = UTF8` and force UTF-8 without BOM | [KIT ✓] |
| 2026-10-31 | packaging | Releasing multiple executables in the distribution bundle confused users | Single standalone setup executable (`MagicBridge-Setup.exe`) and single-binary portable zip | [KIT ✓] |
| 2026-10-31 | UX & Wizard | Flat options list felt unpolished and running the binary intercepted direct Settings access | 5-step progressive Windows Setup Wizard with masked advanced options, uninstaller, and direct settings access | project |

| 2026-11-01 | field test | Milestones M15–M18 marked complete although the Magic Mouse engine was never wired to input and toasts/WinGet claims were overstated | A roadmap item is "done" only when an end-to-end call path from real input/OS event exists — unit-tested library code alone is "engine ready", not "integrated" | generic |
| 2026-11-01 | Hardware QA tab | QA tool identified keys from post-remap egui/text events and filled scancodes from defaults → unverifiable remapped keys, false positives, fabricated columns | Hardware verification tools must capture raw pre-remap input (scancode + extended + device) and never synthesise measured fields | generic |
| 2026-11-01 | UI QA | `ui-qa-tester` reported 0 defects while tofu glyphs (□) and right-edge clipping were visible to the owner | Autonomous UI QA must check missing glyphs and run at the real default window size; owner screenshots outrank 0-findings reports | generic |
| 2026-11-01 | PowerShell 5 | Script with em-dash saved as UTF-8 without BOM failed to parse | `.ps1` files containing non-ASCII must be saved as UTF-8 **with** BOM for Windows PowerShell 5 | generic |
| 2026-11-01 | versioning | Five milestones shipped without a version bump or release | Every completed milestone batch bumps the version and publishes a release (or explicitly records why not) | project |
| 2026-11-01 | GUI Hooks & System Stability | Installing a global `WH_KEYBOARD_LL` hook inside a GUI process (`egui`/`winit`) caused Windows hook timeout contention, system-wide keyboard lockups, and PC freezes | Never install `WH_KEYBOARD_LL` inside a GUI process. Global low-level hooks belong exclusively in headless background daemons. In GUI QA tools, pause daemon remapping via IPC (`set_paused(true)`) and read keys in-window | [KIT ✓] |
| 2026-11-01 | IPC Shared State Desync | `MessageWindow` mutated its internal `WindowData` state on `WM_DEVICECHANGE` and battery polls but never synced to `Arc<Mutex<DaemonState>>`, leaving IPC queries reporting disconnected | Any mutations to hardware connection state in message loops must immediately sync to the shared IPC state | project |
| 2026-11-01 | Release Distribution Ergonomics | Testing local builds was frustrating because users could not locate individual release binaries | Every release must output to an isolated per-version folder (`dist\v{Version}\`) containing an unzipped, ready-to-run `portable\` directory alongside installers and archives | [KIT ✓] |
