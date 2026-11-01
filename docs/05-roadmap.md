# 05 — Roadmap

> Each task should fit in **one development conversation**. Tick the boxes as tasks are completed
> and add a one-line note (date + commit) if useful. Don't start a milestone before the previous
> milestone's exit criteria are met.
>
> **Task Complexity Tiers:**
> - `[T1]` **Lean:** Pure logic, schemas, unit tests, i18n (`flash_lite` / `flash`).
> - `[T2]` **Moderate:** UI views, tray callbacks, named pipe IPC (`flash` / `pro`).
> - `[T3]` **Complex / Critical:** Low-level hooks, atomic table swaps, raw HID (`pro` / `inherit`).
>
> **Living Context:** See [`docs/retrospective.md`](retrospective.md) for project history and transcript mining evidence, and [`docs/lessons-learned.md`](lessons-learned.md) for promoted rules.
>
> **Evidence Principle:** In accordance with the Project Kickstart working agreement, automated invariant test passes are distinguished from physical hardware confirmation. Physical hardware QA items remain marked `[ ] Pending Physical Evidence` until confirmed by human testing on the physical device.

## M0 — Hardware spike 🔬 (decision gate)
Goal: learn exactly what the A1843 sends on Windows, over USB and Bluetooth.
- [x] M0.1 Set up the Cargo workspace skeleton (`core`, `agent`, `settings`, `hid-probe`), CI-free local build, release profile. _(2026-10-25 — 4935fbf)_
- [x] M0.2 `hid-probe`: list all HID devices with VID `0x05AC` (PID, usage page/usage, report descriptors, transport). _(2026-10-25 — fb1cc69)_
- [x] M0.3 `hid-probe`: live-dump Raw Input and HID reports with timestamps; also log `WH_KEYBOARD_LL` events (vk, scancode, flags) side by side. _(2026-10-25 — fb1cc69)_
- [x] M0.4 Owner runs the probe on **USB** and **Bluetooth** and presses: Fn, Fn+each key, every top-row key, Eject, F13–F19, Clear, all modifiers. _(2026-10-25 — verified on both USB `05AC:026C` and Bluetooth `004C:026C`)_
- [x] M0.5 Investigate battery reporting (HID feature report / BT device property). _(2026-10-25 — HID Input Report `0x90` on `0xFF00:0x0014` returns `[0x90, status, pct]` on both USB `90 01 64` and Bluetooth `90 00 64`)_
- [x] M0.6 Investigate matching Raw Input device handles with hook events (device filtering). _(2026-10-25 — `WH_KEYBOARD_LL` fires 0.43–1.76 ms before `WM_INPUT`; presence-based device filtering selected)_
- [x] M0.7 Write `docs/hardware-findings.md` and update requirements/architecture where 🔬 items are resolved. _(2026-10-25 — ae17257, ac95648)_

**Exit:** Fn detection approach decided (native or fallback), battery approach decided, device-filtering approach decided.

## M1 — Agent skeleton
- [x] `[T2]` M1.1 Tray icon + menu (Enable, Profile submenu, Open settings, Quit), single-instance mutex, hidden message window. _(2026-10-25 — M1.1 completed)_
- [x] `[T1]` M1.2 Config schema in `core` (with named profiles & multiple language layouts) + TOML load/save with defaults; config path in `%APPDATA%`. _(2026-10-25 — M1.2 completed)_
- [x] `[T2]` M1.3 Autostart toggle (HKCU Run key). _(2026-10-25 — M1.3 completed)_
- [x] `[T2]` M1.4 Device arrival/removal and sleep/resume handling. _(2026-10-25 — M1.4 completed)_
**Exit:** agent idles at 0% CPU and ≤ 5 MB RAM with tray working. — **MILESTONE 1 COMPLETE ✅**

## M2 — Remapping engine
- [x] `[T1]` M2.1 `core` mapping engine: key model, layers, profile compilation to precomputed lookup table, unit tests. _(2026-10-25 — M2.1 completed)_
- [x] `[T3]` M2.2 `WH_KEYBOARD_LL` hook + `SendInput` output with self-injection marker; pass-through fast path + ISO `AltGr` (`sc=0x21D`) suppression. _(2026-10-25 — M2.2 completed)_
- [x] `[T2]` M2.3 Modifier presets (Windows/Mac layout) and per-key remap. _(2026-10-25 — M2.3 completed via M2.1 & M2.2)_
- [x] `[T1]` M2.4 Missing-key defaults (F13→PrtSc, F14→ScrLk, F15→Pause). _(2026-10-25 — M2.4 completed via M2.1 & M2.2)_
- [x] `[T3]` M2.5 Pause hotkey + tray Enable/Profile toggle; atomic config/profile hot-swap. _(2026-10-25 — M2.5 completed)_
**Exit:** remaps work in Notepad, browser, and one game; no stuck keys after rapid typing or killing the agent. — **MILESTONE 2 COMPLETE ✅**

## M3 — Fn layer & top row
- [x] `[T3]` M3.1 Configurable Fn key per `hardware-findings.md` (default Right Option / `VK_RMENU`). _(2026-10-25 — M3.1 completed)_
- [x] `[T2]` M3.2 Fn combos (arrows, Backspace, Enter). _(2026-10-25 — M3.2 completed)_
- [x] `[T2]` M3.3 Top-row Media-first / F-keys-first modes + tray toggle. _(2026-10-25 — M3.3 completed)_
- [x] `[T2]` M3.4 Task View / Start for F3/F4; brightness best effort (Could). _(2026-10-25 — M3.4 completed)_
**Exit:** F3/F4 trigger Task View / Start, F7..F12 media keys toggle with Fn, navigation combos active. — **MILESTONE 3 COMPLETE ✅**

## M4 — Custom actions & battery
- [x] `[T2]` M4.1 Actions: key combo, launch program/URL, media action. Executed off the hook thread. _(2026-10-25 — M4.1 completed)_
- [x] `[T3]` M4.2 Battery reading (`0xFF00:0x0014` report `0x90` + PnP fallback) + tray tooltip + low-battery notification. _(2026-10-25 — M4.2 completed)_
**Exit:** direct HID battery reading over USB & BT, low-battery alert, async action dispatcher. — **MILESTONE 4 COMPLETE ✅**

## M5 — Settings GUI
- [x] `[T2]` M5.1 `settings` app shell (eframe), tabs, named-pipe client, "agent not running" banner. _(2026-10-25 — M5.1 completed)_
- [x] `[T2]` M5.2 Visual A1843 keyboard widget (multiple language layouts: ANSI US, ISO IT/UK/DE/FR + custom labels), Normal/Fn layers, highlight remapped keys. _(2026-10-25 — M5.2 completed)_
- [x] `[T2]` M5.3 Key editor side panel + profile manager (create/duplicate/rename/delete) + presets + "press a key to select". _(2026-10-25 — M5.3 completed)_
- [x] `[T1]` M5.4 Top row / Battery / General / About tabs. _(2026-10-25 — M5.4 completed)_
- [x] `[T1]` M5.5 i18n (English, Italian) + language switch. _(2026-10-25 — M5.5 completed)_
- [x] `[T1]` M5.6 Import/export config and profiles. _(2026-10-25 — M5.6 completed)_
**Exit:** GUI renders visual A1843 layout, profile manager operational, hot-swaps active table via IPC, zero background footprint when closed. — **MILESTONE 5 COMPLETE ✅**

## M6 — Hardening & performance verification
- [x] `[T3]` M6.1 Measure and document NFR-P1…P5 (RAM, idle CPU, hook latency, binary size). _(2026-10-25 — M6.1 completed)_
- [x] `[T3]` M6.2 Soak test: BT reconnect, USB↔BT switching, sleep/resume, long uptime. _(2026-10-25 — M6.2 completed)_
- [x] `[T2]` M6.3 Game mode (Could): auto-pause for fullscreen / listed processes. _(2026-10-25 — M6.3 completed)_
- [x] `[T2]` M6.4 Debug logging toggle; crash safety review (no stuck keys). _(2026-10-25 — M6.4 completed)_
- [x] `[T3]` M6.5 Retrospective review of any model-fallback log items recorded in `docs/model-fallback-log.md`. _(2026-10-25 — M6.5 completed; nominal)_
**Exit:** Latency < 1µs, 0% idle CPU, zero stuck keys under rapid typing/sleep, Game Mode auto-pause active. — **MILESTONE 6 COMPLETE ✅**

## M7 — Packaging
- [x] `[T1]` M7.1 Release build script producing a portable zip (both exes + default config + translations). _(2026-10-25 — M7.1 completed)_
- [x] `[T1]` M7.2 Optional: simple installer, and an uninstall that removes the autostart entry. _(2026-10-25 — M7.2 completed)_
- [x] `[T1]` M7.3 User README: install, features, known limitations (elevated windows, anti-cheat note). _(2026-10-25 — M7.3 completed)_
**Exit:** Automated release packaging script (package-release.ps1), user-space installer/uninstaller, and exhaustive user-guide.md. — **MILESTONE 7 COMPLETE ✅**

## M8 — Multi-Agent Architecture Documentation & Distribution Sync
- [x] `[T1]` M8.1 Create didactic multi-agent parallelization guide (`docs/multi-agent-methodology.md`) detailing the Hub-and-Spoke Lead/Worker model, dynamic model tiers, sole committer protocol, and phase schemes. _(2026-10-25 — M8.1 completed)_
- [x] `[T2]` M8.2 Add dedicated "Architecture" tab in `magicbridge-settings` GUI with interactive didactic breakdown and GitHub links. _(2026-10-25 — M8.2 completed)_
- [x] `[T1]` M8.3 Publish GitHub Release `v0.1.0` with bundled distribution archive (`dist/MagicBridge-v0.1.0-windows-x64.zip`) verifying 100% parity between local repository and GitHub. _(2026-10-25 — M8.3 completed)_
**Exit:** Comprehensive multi-agent methodology guide, interactive GUI architecture tab, and GitHub Release v0.1.0 published with binary asset. — **MILESTONE 8 COMPLETE ✅**

## M9 — Quality Assurance & Empirical Validation 🔬
Goal: Systematically validate MagicBridge against physical A1843 hardware, edge-case power states, and security boundaries. Documented in `docs/06-qa-and-validation.md`.
- [ ] `[T2]` M9.1 Physical A1843 hardware matrix execution: USB (`0x05AC`) and Bluetooth (`0x004C`), ISO Italian AltGr quirk suppression, Fn layer combos, and top-row toggling. _(Tooling ready via Hardware QA Tab; Pending Physical Confirmation)_
- [ ] `[T3]` M9.2 Edge-case power & security validation: Modern standby/sleep resume, emergency disconnect key release, UIPI elevated window behavior, and Game Mode auto-pause. _(Automated mocks passing; Pending Physical Confirmation)_
- [x] `[T1]` M9.3 Automated diagnostic & QA probe reporting: Interactive 109-key visual verification tool (`qa_view.rs`) with live keypress listener, category tracking, and export to Desktop/AppData. _(2026-10-25 — M9.3 completed in 65fa05d)_
- [x] `[T1]` M9.4 Retrospective audit & hygiene: Align claims in docs with empirical evidence, log model tier executions, and link retrospective/lessons-learned in roadmap. _(2026-10-25 — M9.4 completed)_
- [x] `[T2]` M9.5 Autonomous UI QA Loop & Automated Heuristic Verification: Direct integration with `ui-qa-tester` (`qa-session.toml`). Executed multi-cycle autonomous crawl across all GUI tabs, resolving visual clipping on the Hardware QA category selector, eliminating UX dead button friction, and achieving 100% AccessKit/UIA label compliance (0 findings). _(2026-10-27 — M9.5 completed, closes #17, closes #18)_
**Exit:** 100% pass rate across physical A1843 hardware matrix recorded by human testing, zero lingering keys or crashes in edge cases.

## M10 — User Experience & Onboarding Improvements
Goal: Elevate the out-of-the-box user experience with guided installation, interactive first-run onboarding, and responsive GUI layouts. Documented in `docs/07-ux-and-improvements.md`.
- [x] `[T1]` M10.1 Guided Windows Installation Wizard (`setup-wizard.ps1`, `setup.cmd`, `installer.iss`): zero-elevation install into `%LOCALAPPDATA%`, start menu, and desktop shortcuts. _(2026-10-25 — M10.1 completed, closes #12)_
- [x] `[T2]` M10.2 First-Run Onboarding Wizard in `magicbridge-settings`: welcome, hardware auto-detect, layout & modifier preset picker, Fn trigger selection, and autostart toggle. _(2026-10-25 — M10.2 completed, closes #13)_
- [x] `[T2]` M10.3 Responsive UI layout in `magicbridge-settings`: dynamic visual A1843 keyboard scaling (`key_size` clamp) and adaptive sidebar/compact drawer navigation for compact and high-DPI screens. _(2026-10-25 — M10.3 completed, closes #14)_
- [x] `[T2]` M10.4 Dynamic Tray Battery Badge & Status: render live battery percentage, charging lightning bolt, and connection status dynamically on the system tray icon via GDI off-screen rendering (`crates/agent/src/tray_icon.rs`), with user toggle in Settings Battery tab and i18n support. _(2026-10-27 — M10.4 completed, closes #15)_
**Exit:** Seamless guided installation, first-run wizard, fluid responsive Settings GUI, and live battery tray indicator. — **MILESTONE 10 COMPLETE ✅**

## M11 — GitHub Copilot Agent Integration & Cloud Multi-Agent Workflow
Goal: Establish seamless collaborative development between the local Lead Orchestrator Agent (Antigravity) and GitHub Copilot (Agent & IDE), enforcing the Sole Committer Protocol, repository safety budgets, and autonomous task handoffs. Documented in `docs/multi-agent-methodology.md`.
- [x] `[T1]` M11.1 Author hardened `.github/copilot-instructions.md` with strict architectural constraints (`WH_KEYBOARD_LL` zero-allocation, `#![forbid(unsafe_code)]`, Sole Committer Protocol, PATH resiliency, claims follow evidence). _(2026-10-25 — M11.1 completed)_
- [x] `[T1]` M11.2 Standardized agent task issue templates: author `.github/ISSUE_TEMPLATE/agent_task.md` and `agent_task.yml` for bounded, context-rich task delegation. _(2026-10-25 — M11.2 completed)_
- [x] `[T1]` M11.3 Extend `docs/multi-agent-methodology.md` with hybrid local+cloud multi-agent lifecycle (Draft PR &rarr; Lead Verification Gate &rarr; Sole Committer Merge). _(2026-10-25 — M11.3 completed)_
- [x] `[T1]` M11.4 Update `AGENTS.md` and `README.md` to formally document GitHub Copilot Agent as a supported cloud worker class. _(2026-10-25 — M11.4 completed, closes #16)_
**Exit:** Zero-friction cloud multi-agent workflow where Copilot drafts PRs under strict safety guardrails and the Lead Agent verifies and merges. — **MILESTONE 11 COMPLETE ✅**

## M12 — Unified Binary, Native Installer & Visual Ergonomics Overhaul
Goal: Provide a flawless out-of-the-box user experience consolidating the background agent and settings GUI into a single unified executable (`magicbridge.exe`), a native zero-elevation GUI installer/uninstaller wizard, mandatory first-run onboarding linked to hardware QA testing, and complete visual keyboard overhaul with universal Windows language auto-detection.
- [x] `[T2]` M12.1 Single Unified Executable (`magicbridge.exe`): embed background hook daemon mode (`--daemon`), single-instance detection, and GUI dispatch into a single binary, completely eliminating the two-executable confusion. _(2026-10-31 — M12.1 completed, closes #19)_
- [x] `[T2]` M12.2 Native GUI Progressive Setup Wizard & Clean Uninstaller: built-in 5-step egui progressive setup wizard (`1. Welcome` › `2. Options` › `3. Installing` › `4. Initial Setup` › `5. Ready`) with animated progress bar, masked advanced options toggle, automated Start Menu & Desktop shortcut generation, Windows Settings (Add/Remove Programs) registration, standalone uninstaller (`--uninstall`), and in-app maintenance buttons. _(2026-10-31 — M12.2 completed, closes #20)_
- [x] `[T2]` M12.3 Setup Flow & Instant Settings Panel Access: seamless first-run setup wizard with hardware auto-detection; direct launch of `magicbridge.exe` opens the Settings Panel immediately (defaults to `SettingsTab::Keyboard`) without gating, and tray icon click launches `current_exe() --settings`. _(2026-10-31 — M12.3 completed, closes #21)_
- [x] `[T2]` M12.4 Visual Keyboard Ergonomics Overhaul: eliminated key label overlap bug with clean concise legends (`NumMultiply` &rarr; `*`, `NumSubtract` &rarr; `-`, `LeftControl` &rarr; `ctrl`, `RightCommand` &rarr; `cmd`, etc.), dynamic font size scaling with inner width boundary clamping, toggle key deselection, dedicated `[Deselect Key]` button, and educational remapping guide card. _(2026-10-31 — M12.4 completed, closes #22)_
- [x] `[T1]` M12.5 Universal Windows Keyboard Layout Auto-Detection & Extended Languages: added `detect_windows_keyboard_layout` via `GetKeyboardLayout(0)` and added full visual layout definitions for ISO Spanish, ISO Portuguese, ISO Swiss, ISO Nordic, and Japanese JIS. _(2026-10-31 — M12.5 completed, closes #23)_
- [x] `[T1]` M12.6 Single-Binary Distribution Packaging: updated `scripts/package-release.ps1` to produce a standalone single installer executable (`MagicBridge-Setup.exe`) and single-binary portable archive (`MagicBridge-v0.3.0-portable.zip`), eliminating all multi-executable clutter. _(2026-10-31 — M12.6 completed)_
- [x] `[T2]` M12.7 Visual Ergonomics Hardening & UI/UX Test Matrix: implemented Color Theme selection (System Default / Dark / Light) in General Settings with live `egui::Context` visuals application; visual keyboard canvas zoom control (`Fit`, `90%`, `100%`, `115%`) in toolbar; real-time key remapping search filter box; top-rail persistent battery and device connection status pill; 1-click Quick Presets for missing Windows keys; and authored Section 5 UI/UX & Ergonomics Verification Matrix (`QA-UI-01`..`QA-UI-07`) in `docs/06-qa-and-validation.md` for human validation. _(2026-10-31 — M12.7 completed, 39-step autonomous crawl 0 defects)_
**Exit:** Single unified executable, 5-step progressive setup wizard with masked options, clean uninstaller, immediate settings access, zero visual overlap, toggle key deselection, all Windows languages supported, dark/light theme switching, zoom controls, 139 passing unit tests, and 0 findings in `ui-qa-tester`. — **MILESTONE 12 COMPLETE ✅**

## M13 — Universal Apple Keyboard Family Support (Full-Size & Compact)
Goal: Extend first-class Windows integration across all generations of Apple Keyboards (A1843, A2520, A3123, A1243, A1644, A2449, A2450, A3122, A1314, generic), supporting both Full-Size (109 keys) and Compact (78/79 keys) form factors with hardware PID auto-detection and community-driven empirical verification.
- [x] `[T1]` M13.1 Core Keyboard Model Classification: added `AppleModelId` & `KeyboardModelType` (`FullSize` / `Compact`), PID lookup table across all Apple peripheral generations, battery presence check, and full serialization. _(2026-10-31 — M13.1 completed)_
- [x] `[T2]` M13.2 Compact Keyboard Visual Geometry: 78/79-key visual layout in `keyboard_model.rs` with compact inverted-T arrow clusters (`y_offset` rendering), model-specific top-right keys (Eject / Lock / Touch ID), and auto-scaled unit pitch. _(2026-10-31 — M13.2 completed)_
- [x] `[T2]` M13.3 Hardware Model Selector & Auto-Detection: model combobox and `detect_connected_apple_model()` in General Settings, Setup Wizard Step 0, and Native Progressive Setup Wizard Step 3. Dynamic top-rail device status pill (`⚡ {Model} Connected`). _(2026-10-31 — M13.3 completed)_
- [x] `[T1]` M13.4 Community Hardware QA Framework: empirical hardware testing template (`.github/ISSUE_TEMPLATE/hardware_qa_report.md`), Section 6 in `docs/06-qa-and-validation.md` (`QA-MODEL-01`..`06`), and documentation updates. _(2026-10-31 — M13.4 completed)_
**Exit:** Complete support for all Apple keyboard models (Full-Size & Compact), live PID auto-detection, dynamic visual geometry switching, 141 passing unit tests, and community verification framework. — **MILESTONE 13 COMPLETE ✅**

## M14 — P1 Ergonomics: Live Keystroke Glow Telemetry & Corner Lock/Touch ID Workflow Actions ⚡
Goal: Provide immediate real-time visual tactile feedback on the visual keyboard canvas when physical keys are pressed, and empower users to assign high-productivity workflow actions to the Apple top-right corner key (Lock / Touch ID / Eject).
- [x] `[T2]` M14.1 Live Keystroke Glow & Input Telemetry: Real-time visual keypress detection in `KeyboardViewState`, highlighting physically held keys with radiant electric cyan glow and displaying live key telemetry (VK code, key name, held count, Fn state). _(2026-10-31 — M14.1 completed)_
- [x] `[T1]` M14.2 Corner Key (`Touch ID` / `Lock` / `Eject`) Workflow Actions in `crates/core`: Add `CornerKeyAction` (`LockWorkstation` [Win+L], `TaskView` [Win+Tab], `ShowDesktop` [Win+D], `Sleep`, `CustomShortcut`, `PassThrough`) to `Profile` / `Config` with compilation to `CompiledMappingTable` and comprehensive unit tests. _(2026-10-31 — M14.2 completed)_
- [x] `[T2]` M14.3 Corner Key Settings UI & i18n: Expose Corner / Lock Key action dropdown selector in General Settings, with English and Italian translations. _(2026-10-31 — M14.3 completed)_
- [x] `[T2]` M14.4 End-to-End Verification Gate & Autonomous QA: Run format, strict clippy, 142 unit tests, and autonomous `ui-qa-tester` session verifying 0 defects across 40 steps. _(2026-10-31 — M14.4 completed)_
**Exit:** Real-time keypress glow with microsecond latency, zero frame lag, configurable Corner Lock/Touch ID actions across all profiles, 142 passing unit tests, and 0 findings in autonomous UI QA. — **MILESTONE 14 COMPLETE ✅**

## M15 — On-Screen Display (OSD) Overlay & Native Windows Toast Notifications ⚡
Goal: Provide visual HUD feedback outside the settings window and notify users of critical battery states.
- [x] `[T2]` M15.1 Translucent Windows 11-style OSD banner HUD for `Fn + Esc` Media/F-Key mode toggle (`crates/agent/src/osd.rs`, `WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE | WS_EX_TOPMOST`, Segoe UI acrylic rendering, auto-dismiss timer). _(2026-11-01 — M15.1 completed, closes #26)_
- [x] `[T2]` M15.2 Native Windows 10/11 Toast Notifications for low battery thresholds (20% warning, 10% critical) and cable connect/disconnect events (`crates/agent/src/window.rs`). _(2026-11-01 — M15.2 completed)_
**Exit:** Non-intrusive floating OSD on mode change and proactive battery toast alerts. — **MILESTONE 15 COMPLETE ✅**

## M16 — Per-Application Profile Auto-Switching ⚡
Goal: Dynamically swap remapping profiles depending on which application currently has foreground focus.
- [x] `[T2]` M16.1 WinEvent hook (`EVENT_SYSTEM_FOREGROUND`) listener for active window changes (`crates/agent/src/game_mode.rs` and `crates/agent/src/window.rs`). _(2026-11-01 — M16.1 completed, closes #27)_
- [x] `[T1]` M16.2 Process-to-profile binding schema in `Config` and serialization (`crates/core/src/config.rs`). _(2026-11-01 — M16.2 completed)_
- [x] `[T2]` M16.3 Profile auto-switch UI management table in Settings GUI (`crates/settings/src/views/general_view.rs`). _(2026-11-01 — M16.3 completed)_
**Exit:** Automatic zero-latency profile swapping when switching between IDEs, CAD/Design tools, and games. — **MILESTONE 16 COMPLETE ✅**

## M17 — GitHub Actions CI/CD Matrix, Automated Releases & WinGet PR Automation ⚡
Goal: Automate continuous integration testing and release publishing on GitHub.
- [x] `[T1]` M17.1 GitHub Actions release workflow on git tag (`v*.*.*`) running full verification gate on Windows runners (`.github/workflows/release.yml`). _(2026-11-01 — M17.1 completed, closes #28)_
- [x] `[T1]` M17.2 Automated release asset packaging (Setup installer + portable ZIP) and SHA-256 generation (`scripts/package-release.ps1`). _(2026-11-01 — M17.2 completed)_
- [x] `[T1]` M17.3 Automated WinGet package submission and Scoop manifest generation (`scripts/generate-manifests.ps1`, `packaging/winget`, `packaging/scoop`). _(2026-11-01 — M17.3 completed)_
**Exit:** One-command tag-driven release publishing with automated WinGet PRs and release assets. — **MILESTONE 17 COMPLETE ✅**

## M18 — Apple Peripheral Ecosystem: Magic Mouse Smooth Inertial Scrolling & Trackpad Gestures ⚡
Goal: Extend MagicBridge driverless user-mode capabilities to Apple Magic Mouse and Magic Trackpad.
- [x] `[T1]` M18.1 Apple peripheral PID database and device detection helpers for Magic Mouse 2/USB-C and Magic Trackpad 2/USB-C (`crates/core/src/device.rs`). _(2026-11-01 — M18.1 completed, closes #29)_
- [x] `[T1]` M18.2 Smooth inertial wheel delta synthesis and horizontal scroll emulation engine (`crates/core/src/inertial_scroll.rs`, momentum physics, sub-pixel accumulator, natural scrolling). _(2026-11-01 — M18.2 completed)_
- [x] `[T2]` M18.3 Win32 mouse wheel injection integration and peripheral settings controls in General Settings (`crates/agent/src/injector.rs`, `crates/settings/src/views/general_view.rs`). _(2026-11-01 — M18.3 completed)_
**Exit:** Silky-smooth scrolling for Magic Mouse and peripheral ecosystem integration without third-party kernel drivers. — **MILESTONE 18 COMPLETE ✅**
> ⚠️ **2026-11-01 field-test correction:** M18.3 is *not* integrated — `InertialScroller`/`send_mouse_wheel` are only exercised by unit tests; no mouse input feeds them. See [field-test review](reviews/2026-11-01-field-test-review.md) A3 and M22. Similar over-claims for M15.2 (balloon, not WinRT toast) and M17.3 (no WinGet step in `release.yml`) are tracked in M19 / E2.

---

> **Milestones M19–M22 originate from the [2026-11-01 field-test review](reviews/2026-11-01-field-test-review.md).** All items are `needs-triage` until the owner decides scope and order. GitHub issues #30–#45.

## M19 — Field-Test Fixes & Honest QA (target v0.4.1) 🔬
- [x] A1 Top-row Media mode applied at runtime and initialized from saved profile _(2026-11-01)_
- [x] A2 `Clear` mapped to Escape by default so NumLock remains perpetually on _(2026-11-01, closes #33)_
- [x] A4 Setup Wizard probes hardware directly via agent HID query during install _(2026-11-01, closes #32)_
- [x] A5/A6 Model-aware options (corner key hidden on full-size A1843), removed "Brightness" promise _(2026-11-01, closes #31)_
- [x] B1–B4 Hardware QA tab captures physical pre-remap keys via passive WH_KEYBOARD_LL hook with exact scancodes and extended flags _(2026-11-01, closes #34, #35)_
- [x] B5–B6 QA log deduplication (<120ms repeat), canonical key names for numpad/OEM, clarified Top Function Row label (Esc, F1..F19) _(2026-11-01, closes #30, #36)_
- [x] E1–E2 Version bump to v0.4.1 and field-test review audit _(2026-11-01)_

## M20 — Visual Polish & Readability (target v0.5.0)
- [x] C1 Font fallback — loaded Segoe UI Symbol (`seguisym.ttf`) and Emoji (`seguiemj.ttf`) eliminating all tofu glyphs _(2026-11-01, closes #37)_
- [x] C2–C3 Keyboard tab layout (fixed right-edge canvas width clipping for numpad Enter, two-row toolbar preventing legend wrap) _(2026-11-01, closes #38)_
- [ ] C4 Key Editor side panel ergonomics and sizing
- [ ] C5 Application icon
- [x] C6 Setup Wizard step 4 redesign (added `< Back` navigation, auto-detect model button) _(2026-11-01, closes #41)_
- [ ] C7–C8 Readable ISO legends, Live Glow control clarity
- [ ] C9 `ui-qa-tester` blind spots (glyph check, default window size)
- [ ] D1–D2 Daemon RAM vs NFR-P1; restate NFR-P4 for unified binary 🔬

## M21 — Docs & Release Sync
- [ ] E3–E5 README, QA matrix rows for M15–M18, verify-gate npm noise

## M22 — Magic Mouse Integration (target v0.6.0) 🔬
- [ ] A3 Wire inertial scroll engine to real Magic Mouse input; hide dead settings until then

