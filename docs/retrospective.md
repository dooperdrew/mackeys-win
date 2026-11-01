# MagicBridge — Retrospective (M0 → v0.4.1)

> Date: 2026-11-01 · Scope: 45+ commits, milestones M0–M19, releases v0.1.0 through v0.4.1,
> multi-session interactive lead orchestrations + worker subagent parallelizations.
> Method: transcript mining + git history + field-test reviews + post-mortem analysis.
> Generic lessons promoted to the `project-kickstart` plugin.

---

## 1. Timeline

| Phase | Commits | Milestones & Focus | Key Deliverables & Conversations |
|---|---|---|---|
| **Phase 0: Definition & Spike** | `8cbeb8b` → `ac95648` | M0 Hardware Spike | [Project definition](conversation://8b5c8372-6864-48d7-b03e-af49fbb1944a), [M0 session](conversation://74a20b38-6821-4721-a963-ab06ab006a26). Ground-truth hardware discoveries (`hid-probe`, VID `0x004C` over BT, report `0x90`, ISO AltGr `0x21D`). |
| **Phase 1: Core Engine & GUI** | `5369267` → `892674e` | M1–M10 Skeleton, Engine, GUI, Packaging | v0.1.0 and v0.2.0 releases. Sole Committer Protocol, complexity tiers, 2-process architecture (`magicbridge-agent` + `magicbridge-settings`), basic QA tool. |
| **Phase 2: Power Features & Ecosystem** | `b9f3900` → `5928cb5` | M11–M18 Game Mode, Profiles, OSD HUD, Multi-Model, CI/CD | v0.3.0 and v0.4.0 releases. Acrylic OSD HUD (`Fn+Esc`), foreground process auto-switching, multi-model Apple database (A1843, A1644, A2449, A2520, A1314, A1243), 5-step installer wizard. |
| **Phase 3: Field-Test Audit & Stability** | `9c9b482` → `0ee0c1a` | M19 Field-Test Remediation & Stability Hotfix | v0.4.1 release. Direct hardware probe, `Clear -> Escape` mapping, honest pre-remap QA, elimination of GUI hook lockup, IPC shared-state sync, isolated `dist\v0.4.1\` pipeline. |

---

## 2. What Worked

1. **Spec First, Followed by M0 Hardware Spike:**
   - Writing `hid-probe` before writing daemon code grounded the entire architecture in hardware realities:
     Fn stripped by `kbdhid.sys` in kernel mode, Bluetooth Classic using Apple VID `0x004C`, battery status in vendor report `0x90`, and Windows synthesizing synthetic `VK_LCONTROL` (`0x21D`) on Italian/ISO layouts.
2. **Strict Crate Boundaries for Parallelism:**
   - `magicbridge-core` (`#![forbid(unsafe_code)]`, pure Rust, zero Win32) decoupled completely from `magicbridge-agent` (Win32 hooks, tray, IPC) and `magicbridge-settings` (egui GUI).
   - This boundary allowed unit test suites to run in parallel and kept unsafe Win32 APIs isolated to audited wrappers.
3. **Mechanical Verification Gate with Fast & Full Modes:**
   - Running `scripts/verify-gate.ps1` locally before every commit caught compiler warnings, clippy regressions, and test failures early.
   - The test suite expanded to **154 automated unit & integration tests**, passing in under 1.6 seconds.
4. **Direct HID Hardware Probe Fallback:**
   - Probing USB and Bluetooth HID devices directly via Setup Wizard (`crates/settings/src/installer.rs`) provided instant model and battery feedback even when the background daemon service was not yet initialized.
5. **Zero-Hook In-Window QA Architecture:**
   - Replacing global `WH_KEYBOARD_LL` hooks in the GUI with IPC daemon pause (`ipc.set_paused(true)`) and in-window `egui::Event::Key` / `GetKeyState` input capture eliminated hook contention completely while providing clean, unmapped keystroke verification.
6. **Native Font Fallback Integration:**
   - Loading Windows system fonts `seguisym.ttf` (Segoe UI Symbol) and `seguiemj.ttf` (Segoe UI Emoji) natively in `setup_custom_fonts()` resolved all missing glyph tofu boxes (`□`) without bloating binary sizes.

---

## 3. What Went Wrong & Empirical Post-Mortem

| # | Phase | Issue | Evidence | Root Cause | Impact & Resolution |
|---|---|---|---|---|---|
| 1 | M1–M10 | `cargo` missing from PATH in worker subshells | 13+ workers (`2d845bf5`, etc.) | New PowerShell subshells do not inherit modified user environment | Codified PATH preamble in `AGENTS.md` and worker scripts. |
| 2 | M2 | IPC test hang on global OS state | Commit `13e8acb` | Named pipe tests raced on hardcoded pipe names without timeouts | Refactored tests to use unique UUID pipe names and channel readiness signals. |
| 3 | M10 | Claims ahead of evidence (v0.1.0/v0.2.0) | CHANGELOG vs QA matrix | Release notes claimed "production ready" while physical QA was untested | Codified "Claims Follow Evidence": automated tests marked PASS; physical matrix rows marked Pending. |
| 4 | M15–M18 | Overstated completion ("integrated" vs "engine ready") | Field test review [`docs/reviews/2026-11-01-field-test-review.md`](reviews/2026-11-01-field-test-review.md) | M18.3 claimed mouse scrolling done when only math library existed; M15.2 claimed WinRT toasts but used balloons | Redefined "Done": a feature is done only when an end-to-end call path from physical/OS event to output is wired and verified. |
| 5 | M19 | **Global `WH_KEYBOARD_LL` in GUI process froze Windows OS** | User report: *"QA/testing is crashing my PC and it is not working; the application looks totally unstable"* | `qa_view.rs` installed a low-level input hook on the interactive `egui`/`winit` rendering thread; hook contention triggered Windows input timeouts | **Critical Rule:** Never install `WH_KEYBOARD_LL` in GUI processes. Purged hook from GUI; pause daemon remapping via IPC instead. |
| 6 | M19 | **Wizard reported "Keyboard not connected" with keyboard plugged in** | User screenshot `media_1793555023966_83268769.png` | `MessageWindow` mutated internal `WindowData` on `WM_DEVICECHANGE` and battery polls but never synced to `Arc<Mutex<DaemonState>>` | Added `sync_to_shared_state()` called on every connection/battery change; added direct HID probe fallback in Wizard. |
| 7 | M19 | **A1843 `Clear` key broke numeric keypad typing** | Field-test report | Physical Clear sends `VK_NUMLOCK` (`0x90`), which toggled Windows NumLock off and turned numbers into navigation keys | Mapped `VK_NUMLOCK -> Escape` by default in `magicbridge-core`, keeping NumLock permanently active. |
| 8 | v0.4.0 | **Unorganized flat `dist/` directory caused testing friction** | User report: *"can't find the exe on local to execute, can you ensure any release has its own separate folder?"* | Packaging script output all archives into a single flat `dist\` directory, requiring manual extraction | Updated `package-release.ps1` to isolate every release into `dist\v{Version}\` with an unzipped, ready-to-test `portable\` folder. |

---

## 4. User Corrections → Concrete Rules

| User Correction / Feedback | Architectural Rule Adopted | Location |
|---|---|---|
| *"QA/testing is crashing my PC and it is not working; the application looks totally unstable"* | **GUI Hook Prohibition:** Never install `WH_KEYBOARD_LL` in a GUI process. Global low-level hooks belong strictly in headless background services. In GUI QA, pause daemon via IPC (`set_paused(true)`) and read keys in-window. | `AGENTS.md` §5.B, `docs/lessons-learned.md` |
| *"In the screenshot the wizard is not detecting the keyboard even if connected"* | **IPC State Synchronization:** All window message loop mutations (`WM_DEVICECHANGE`, battery polls) must immediately synchronize to `Arc<Mutex<DaemonState>>`. | `AGENTS.md` §5.B, `crates/agent/src/window.rs` |
| *"Can't find the exe on local to execute, can you ensure any release has its own separate folder with all the dist files?"* | **Per-Version Release Directories:** Every release must be staged in an isolated folder (`dist\v{Version}\`) containing an unzipped `portable\` directory alongside installers and zip archives. | `AGENTS.md` §5.C, `scripts/package-release.ps1` |
| *"The message in the wizard makes no sense with the connected keyboard"* | Remove unverified feature promises (e.g. "Brightness") and accurately display model name and battery status. | `crates/settings/src/installer.rs` |
| *"How hard would be to give support to all the others apple keyboards?"* | Multi-model architecture: parameterize keyboard models with form factor, PID database, and layout geometry. | `crates/core/src/config.rs`, `crates/settings/src/keyboard_model.rs` |

---

## 5. Metrics & Project Status

- **Git & Commits:** 45+ commits on `main`.
- **Milestones:** M0 through M19 complete; M20–M22 scoped and prioritized.
- **Releases:** 6 releases published (`v0.1.0`, `v0.2.0`, `v0.2.1`, `v0.3.0`, `v0.4.0`, `v0.4.1`).
- **Automated Tests:** **154 tests** across workspace (81 agent, 41 core, 32 settings) — **100% PASS** in 1.52s.
- **Hook Performance:** Lookup latency **`0.0006 ms`** (~600 ns), well below the 0.1 ms NFR-P3 budget; zero allocations or locks on hook path.
- **Physical Hardware QA:** 0/12 rows passed on physical A1843 (all tooling ready and marked `Pending Physical Confirmation`).

---

## 6. Open Follow-ups & Next Milestones

1. **Physical A1843 Validation:** User testing of `dist\v0.4.1\portable\magicbridge.exe` and `MagicBridge-Setup.exe` with physical A1843 keyboard over USB and Bluetooth.
2. **M20 — Visual Polish & Readability (v0.5.0):**
   - Application icon for `.exe`, title bar, and system tray (#39).
   - Key Editor side-panel ergonomics (#38).
   - Readable 2-line ISO key legends (#41).
   - Daemon memory profile optimization vs NFR-P1 (#43).
3. **M21 — Docs & Release Sync:**
   - Synchronize README, user guide, and QA matrix (#44).
4. **M22 — Magic Mouse Integration (v0.6.0):**
   - Connect the inertial scrolling engine to real raw mouse input (#45).

---

## 7. Generic Lessons Promoted to Project Kickstart

1. **Spike Unknowns Before Designing (M0 Spike):** Never guess hardware or OS behavior. Write a minimal probe tool first to establish empirical ground truth.
2. **Decouple Headless Daemons from Interactive GUIs:** Low-level OS hooks (`WH_KEYBOARD_LL`, `WH_MOUSE_LL`) belong exclusively in headless daemons on dedicated message pumps. GUI applications must remain 100% hook-free.
3. **IPC State Must Never Lag Window Events:** Internal message-window state (`WindowData`) and thread-safe IPC state (`Arc<Mutex<T>>`) must be synchronized at every mutation point.
4. **Distribute End-to-End Features, Not Just Library Logic:** A feature is only complete when an unbroken call path from hardware/OS input to user output is established. Library code without input plumbing is "engine ready", not "integrated".
5. **Claims Follow Evidence:** Distinguish automated invariant tests from physical hardware validation. Keep hardware matrix rows `Pending Physical Confirmation` until human-tested on physical devices.
6. **Ergonomic Release Packaging:** Always output releases into clean, per-version directories with pre-extracted portable targets so testers never have to hunt for or unzip binaries.
7. **Sole Committer Protocol:** Only the lead orchestrator commits to `main` and pushes; workers deliver via isolated branches and structured completion reports.
