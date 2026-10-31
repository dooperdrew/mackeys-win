# Development & Knowledge Log (`CHANGELOG.md`)

All development steps, hardware discoveries, architectural decisions, and commits are logged here in chronological order for future reference.

---

## 2026-11-01 — Milestones M15, M16, M17 & M18: OSD Overlay HUD, Per-App Profiles, CI/CD Packaging & Peripheral Ecosystem

### Objective & Architecture
- **Milestone M15 — On-Screen Display (OSD) Overlay HUD & Native Windows Toast Notifications**:
  - Implemented `OsdOverlay` in `crates/agent/src/osd.rs` using a translucent acrylic Win32 window (`WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE | WS_EX_TOPMOST | WS_POPUP`) with dark acrylic fill (`#14181E`), electric cyan accent border (`#00D2FF`), Segoe UI typography, and auto-dismiss timer.
  - Intercepted `Fn + Esc` in `WH_KEYBOARD_LL` low-level hook (`crates/agent/src/hook.rs`) to asynchronously post `IDM_TOGGLE_TOPROW_MODE` to the daemon window without blocking the hot path, instantly toggling between Media Keys and Function Keys mode with floating OSD HUD feedback.
  - Implemented 2-tier low battery notifications (warning at configurable threshold, critical alert at 10%) and native Windows Toast Notifications on cable connect/disconnect (`crates/agent/src/window.rs`).
- **Milestone M16 — Per-Application Profile Auto-Switching**:
  - Leveraged `EVENT_SYSTEM_FOREGROUND` WinEvent hook in `crates/agent/src/game_mode.rs` to detect active window changes and query the foreground executable image name.
  - Added `app_profiles: BTreeMap<String, String>` and `Config::profile_for_process(&self, exe: &str) -> Option<&str>` with case-insensitive and `.exe` suffix tolerance.
  - Dynamically swaps active profiles, recompiles `CompiledMappingTable`, updates the system tray, and triggers profile OSD notification upon application switch.
- **Milestone M17 — GitHub Actions CI/CD Matrix, Automated Releases & WinGet PR Automation**:
  - Updated `.github/workflows/release.yml` with Windows x64 runners, full verification gate (`cargo fmt`, `cargo clippy`, `cargo test`), release asset packaging, and automated SHA-256 generation.
  - Enhanced `scripts/package-release.ps1` and `scripts/generate-manifests.ps1` to produce single-executable installer (`MagicBridge-Setup.exe`), versioned installer (`MagicBridge-v0.4.0-Setup.exe`), and portable zip (`MagicBridge-v0.4.0-portable.zip`), auto-generating WinGet and Scoop package manifests.
- **Milestone M18 — Apple Peripheral Ecosystem: Magic Mouse Smooth Inertial Scrolling & Trackpad Support**:
  - Added Apple peripheral Product IDs (`MAGIC_MOUSE_2_PID`, `MAGIC_MOUSE_USBC_PID`, `MAGIC_TRACKPAD_2_PID`, `MAGIC_TRACKPAD_USBC_PID`) and hardware recognition helpers in `crates/core/src/device.rs`.
  - Implemented `InertialScroller` in `crates/core/src/inertial_scroll.rs`: pure physics momentum decay (`friction = 0.92`), sub-pixel delta accumulator (`WHEEL_DELTA = 120`), macOS-style natural scrolling inversion, and horizontal scroll emulation.
  - Added Win32 mouse wheel injection (`send_mouse_wheel`) in `crates/agent/src/injector.rs`.
  - Added Apple Peripheral Ecosystem settings (smooth scrolling, natural scrolling, scroll speed percentage) in General Settings.
- **Responsive 2-Column Layout & Autonomous UI QA Hardening (0 Defects)**:
  - Redesigned General Settings into a responsive 2-column layout ("Hardware & Appearance" vs "System & Automation") wrapped inside `egui::ScrollArea::vertical()`.
  - Resolved visual layout clipping and control overlap identified by `ui-qa-tester`.
  - Verification Gate passed 100%: format check clean, clippy clean (`-D warnings`), all 153 tests passed, release build succeeded, and autonomous QA crawl completed 40 steps with **0 findings / 0 defects**.
  - Closes #26 (M15 OSD HUD & Toasts), closes #27 (M16 Per-App Profiles), closes #28 (M17 CI/CD Releases & WinGet PRs), and closes #29 (M18 Magic Mouse Inertial Scrolling).

---

## 2026-10-31 — Milestone M14: P1 Ergonomics — Live Keystroke Glow Telemetry & Corner Lock/Touch ID Workflow Actions

### Objective & Architecture
- **Real-Time Visual Keystroke Glow (Input Telemetry)**: Implemented instantaneous physical keypress detection in `KeyboardViewState` using ultra-low-latency `GetAsyncKeyState` polling, highlighting active physical keys with radiant electric cyan (`rgb(0, 205, 255)`) and dark navy high-contrast legends, paired with an interactive live telemetry badge (`● Input: [Key] (VK 0x..) [Count]`).
- **Apple Corner Key Workflow Actions**: Added `CornerKeyAction` (`LockWorkstation` [Win+L], `TaskView` [Win+Tab], `ShowDesktop` [Win+D], `Sleep`, `PassThrough`) to `Profile` in `crates/core/src/config.rs`, allowing users to configure the top-right corner key across all Apple keyboard models (Touch ID on A2449/A3122, Lock on A2450/A1644, or PrintScreen/Eject).
- **Settings GUI Integration & i18n**: Added Corner / Lock Key action dropdown selector to General Settings (`general_view.rs`) with full localization in English (`en.toml`) and Italian (`it.toml`).
- **Visual Feedback & Autonomous QA Hardening**: Enhanced battery tray badge toggle with an explicit visual status badge (`✓ Active` / `○ Disabled`), ensuring immediate perceptual change. Autonomous QA crawler (`ui-qa-tester`) passed across 40 steps with **0 findings / 0 defects**.
- **Roadmap Planning**: Mapped future milestones M15 (OSD Overlay & Low-Battery Toasts), M16 (Per-App Profile Auto-Switching), M17 (GitHub Actions CI/CD Releases & WinGet PRs), and M18 (Apple Peripheral Ecosystem: Magic Mouse Smooth Scrolling & Trackpad Gestures).

---

## 2026-10-31 — MagicBridge v0.4.0: Universal Apple Keyboard Family Support (Full-Size & Compact)

### Objective & Architecture
- Extended first-class Windows integration across all desktop generations of Apple Keyboards:
  - **Full-Size with Numeric Keypad (109 Keys):** Magic Keyboard with Numeric Keypad (`A1843`, `A2520`, `A3123`) & Apple Keyboard Wired (`A1243`).
  - **Compact Tenkeyless (78/79 Keys):** Magic Keyboard Compact (`A1644`, `A2449`, `A2450`, `A3122`) & Apple Wireless Keyboard (`A1314`).
- Implemented core hardware model classification (`AppleModelId` & `KeyboardModelType`), USB/Bluetooth Product ID (PID) lookup table, and automatic hardware detection (`detect_connected_apple_model()`).
- Designed a dedicated 78/79-key visual geometry model (`A1843Model::new_compact`) in `crates/settings/src/keyboard_model.rs` featuring sub-millimeter row alignment (exact 14.5u width per row), compact inverted-T arrow cluster with vertical offsets (`y_offset`), model-specific corner keys (Lock / Touch ID / Eject), and responsive unit scaling.
- Integrated Apple Keyboard Model selection with one-click hardware auto-detection into General Settings, Setup Wizard Step 0, and Native Progressive Setup Wizard Step 3.
- Established the Community Hardware QA Framework with Section 6 in `docs/06-qa-and-validation.md` (`QA-MODEL-01`..`06`) and `.github/ISSUE_TEMPLATE/hardware_qa_report.md` for empirical hardware verification.

### Key Deliverables & Enhancements
1. **Core Hardware Classification & PID Detection Table (`crates/core`):**
   - Added `AppleModelId` (`A1843`, `A1644`, `A2449`, `A2450`, `A2520`, `A3122`, `A3123`, `A1314`, `A1243`, `GenericApple`) and `KeyboardModelType` (`FullSize`, `Compact`) enums.
   - Mapped USB/Bluetooth Product IDs: `0x026C` (A1843), `0x0267` (A1644), `0x029A` (A2449), `0x029B` (A2450), `0x029C` (A2520), `0x02C0` (A3122), `0x02C1` (A3123), `0x0255` (A1314), `0x0220` (A1243).
   - Added `detect_connected_apple_model()` in `crates/agent` parsing connected HID paths to automatically detect plugged Apple hardware.
   - Added `keyboard_model` configuration property with default `AppleModelId::A1843` and backward-compatible serde serialization.
2. **Compact Visual Keyboard Geometry & Inverted-T Arrows (`crates/settings/src/keyboard_model.rs`):**
   - Added `pub y_offset: f32` to `PhysicalKey` to support half-height key positioning (Up arrow at row 5, y_offset 0.0, height 0.5; Down arrow at row 5, y_offset 0.5, height 0.5).
   - Implemented `A1843Model::new_compact` producing an exact 78-key (ANSI) / 79-key (ISO) layout with zero dead numpad space.
   - Added `calculate_unit_size_for_model` auto-scaling compact keyboards up to 50px key width to fill the screen comfortably.
   - Re-clicking or selecting any key works identically across both compact and full-size models.
3. **UI/UX Model Selector & Auto-Detection Integration:**
   - Added Apple Keyboard Model combobox and `[Auto-Detect Model]` button to General Settings (`general_view.rs`).
   - Integrated model selection and auto-detection into Onboarding Setup Wizard Step 0 and Native Progressive Setup Wizard Step 3.
   - Dynamic top-rail device status pill (`⚡ {ShortName} Connected`) with full model name in tooltip, eliminating visual tab overlap.
4. **Autonomous UI QA Hardening (0 Findings):**
   - Fixed `VIS-OVERLAP-000-06-07`: switched status pill to use concise model short names (`A1843`, `A1644`, `A2449`, `A1314`), leaving ample breathing room and eliminating overlap with navigation tabs.
   - Fixed `A11Y-NAME-007-28`: assigned explicit AccessKit accessible name to the Setup Wizard Model combobox.
   - Fixed `UX-DEAD-BTN-032`: forced immediate UI repaint (`request_repaint()`) on tray badge toggle to eliminate frame lag.
5. **Community Hardware QA Framework (`docs/06-qa-and-validation.md`, `.github`):**
   - Added Section 6 *Multi-Model Apple Keyboard Hardware Matrix* with structured test procedures for models A1644, A2449, A2450, A2520, A1314, and A1243.
   - Updated GitHub Issue template for community hardware probe submissions.
   - All 141 workspace unit tests pass.

---

## 2026-10-31 — Extensive Autonomous UI/UX Test, Visual Ergonomics Hardening & Theme Selection (39 Steps, 0 Defects)

### Objective & Architecture
- Executed an extensive 39-step autonomous UI/UX crawl and deep verification session using `ui-qa-tester` against the unified `target\release\magicbridge.exe` application alone.
- Audited all views, tab transitions, modal dialogs, and interactive widgets for accessibility (AccessKit / Windows UI Automation), responsive layout clipping, responsiveness (dHash visual delta and stdout/stderr log correlation), and process health.
- Implemented core user-requested UI/UX ergonomics: Color Theme Selection (System Default / Dark / Light), Keyboard Canvas Zoom Controls (Fit / 90% / 100% / 115%), Real-Time Key Remapping Search Filter, 1-Click Quick Presets for missing Windows keys, and a persistent top-rail battery and device status pill.
- Authored Section 5 *UI/UX & Ergonomics Verification Matrix* (`QA-UI-01` to `QA-UI-07`) in `docs/06-qa-and-validation.md` for human testing.

### Key Deliverables & Enhancements
1. **Color Theme Mode Selection (`crates/core/src/config.rs`, `general_view.rs`):**
   - Added `ThemeMode` enum (`System`, `Dark`, `Light`) to `core::Config` with serde serialization and localization support.
   - Added Color Theme selector combobox in General Settings with immediate live `egui::Context::set_visuals` application (`egui::Visuals::dark()` / `egui::Visuals::light()`).
2. **Visual Keyboard Canvas Zoom & Scale Controls (`crates/settings/src/views/keyboard_view.rs`):**
   - Added zoom toolbar controls (`Fit`, `90%`, `100%`, `115%`) to `KeyboardViewState`.
   - Scaled key pitch and canvas dimension dynamically with horizontal scrolling support in `CentralPanel`'s `ScrollArea`, ensuring the visual keyboard can be zoomed comfortably on any screen size.
3. **Key Remapping Real-Time Search Filter (`crates/settings/src/views/keyboard_view.rs`):**
   - Added instant search text filter box in the Key Remapping Editor side panel (`target_filter`).
   - Filters candidate target keys dynamically across standard, numpad, navigation, and media categories.
4. **Missing Windows Keys Quick Presets (`crates/settings/src/views/keyboard_view.rs`):**
   - Added 1-click Quick Preset buttons (`PrintScreen`, `ScrollLock`, `Pause`, `Insert`, `Delete`) in the Key Editor parameter section of `keyboard_view.rs`, allowing users to assign missing Windows keys with a single click instead of scrolling through the full dropdown.
5. **AccessKit & Windows UI Automation Compliance:**
   - Resolved `A11Y-NAME-015-08` by assigning explicit accessible labels to all container groups and modal dialogs (`ui.group`) in `keyboard_view.rs` (`ProfileDialog::New`, `Duplicate`, `Rename`, `DeleteConfirm`, and the Key Remapping Editor).
   - Set semantic accessible names on all major view containers: `battery_view.rs` ("Battery & Power"), `top_row_view.rs` ("Top Row & Fn Keys"), `general_view.rs` ("General Settings"), and `about_view.rs` ("About & System Status").
   - Labeled `CentralPanel` ("Main Settings View") and `ScrollArea` inner/outer IDs ("Settings Content Area") in `crates/settings/src/app.rs`.
6. **Dialog Usability & Keyboard Focus Ergonomics:**
   - Added `dialog_focus_needed` flag in `KeyboardViewState` to automatically focus the name text edit field (`res.request_focus()`) whenever a Profile Dialog (New, Duplicate, Rename) opens, allowing users to type immediately without an extra click.
7. **Persistent Top-Bar Connection & Battery Status Pill:**
   - Added a right-aligned live status pill in the top navigation rail (`🔋 {pct}%` or `⚡ A1843 Connected` / `⚪ Disconnected`), enabling users to monitor battery health from any tab.
   - Wired the status pill to switch to the Battery tab on click, auto-disabled when already on the Battery tab, and refreshed daemon status with responsive stdout logging.
8. **Config Mutation Feedback & Responsiveness:**
   - Updated `MagicBridgeApp::persist_and_notify` to log successful configuration writes and IPC reload notifications to stdout with immediate flushing (`[CONFIG] Saved and applied config successfully: OK`), ensuring instant responsiveness acknowledgment across all checkboxes and settings toggles.
9. **UI/UX & Ergonomics Verification Matrix (`docs/06-qa-and-validation.md`):**
   - Added structured test cases (`QA-UI-01` to `QA-UI-07`) covering theme switching, canvas zoom, key filter search, top-bar status pill, quick presets, modal focus, and uninstaller clean removal.
10. **Verification & Quality Gate:**
    - Executed 39-step deep autonomous crawl via `ui-qa-cli.exe run --config qa-session.toml`:
      - **0 Critical | 0 High | 0 Medium | 0 UX Suggestions**
      - **158 interactive elements discovered, 0 dead controls, 0 accessibility gaps, 0 UI clipping defects, 0 hung processes**.
    - Verified cargo formatting (`cargo fmt --all -- --check`), strict clippy linter (`cargo clippy --workspace --all-targets -- -D warnings`), 139 unit tests (`cargo test --workspace -- --test-threads=1`), and release packaging.

---

## 2026-10-31 — MagicBridge v0.3.0: Unified Binary, Native Installer & Visual Ergonomics Overhaul (Release ✅, closes #19, #20, #21, #22, #23)

### Objective & Architecture
- Consolidated the dual-executable architecture (`magicbridge-agent.exe` + `magicbridge-settings.exe`) into a single, unified Windows binary (`magicbridge.exe`) that dispatches between background hook daemon (`--daemon`), native GUI installer wizard (`--install` or outside install dir), native uninstaller (`--uninstall`), and settings GUI.
- Implemented a zero-elevation native GUI installer/uninstaller wizard that manages `%LOCALAPPDATA%\Programs\MagicBridge`, creates Start Menu and Desktop shortcuts, registers in Windows Settings "Installed apps", and guarantees clean uninstallation.
- Re-architected first-run user onboarding: when launched fresh without existing configuration, the application forces the Setup Wizard (hardware detection, model, layout, modifier preset, Fn key, autostart) and automatically transitions to the interactive Hardware Key Testing Tool before unlocking the full keyboard editor.
- Fixed key label overlap bug on the visual A1843 keyboard canvas by replacing verbose struct identifiers (`NumMultiply`, `NumSubtract`, `LeftControl`, `RightCommand`) with clean localized typography (`*`, `-`, `ctrl`, `cmd`), implementing dynamic font scaling with inner keycap width boundary clamping, enabling re-click key deselection, and adding a dedicated `[Deselect Key]` button.
- Extended keyboard layout support to all major Windows languages: added Windows layout auto-detection via `GetKeyboardLayout(0)` and implemented visual keyboard models for ISO Spanish, ISO Portuguese, ISO Swiss, ISO Nordic, and Japanese JIS.

### Key Deliverables & Bug Fixes
1. **Single Unified Executable & Standalone Installer Distribution (`magicbridge.exe` — closes #19):**
   - Exported `run_daemon() -> i32` from `crates/agent/src/lib.rs`.
   - Unified binary `magicbridge` in `crates/settings` parses command line arguments:
     - `--daemon`: Runs headless background hook daemon and tray icon without window creation.
     - `--uninstall`: Runs native egui Uninstaller Wizard.
     - `--install` or filename matching `*setup*.exe`: Runs native egui 5-step Setup Wizard.
     - Default / `--settings`: Automatically verifies/spawns background daemon if not alive, and ALWAYS opens the Settings GUI directly (never blocks or hijacks user access).
   - Cleaned distribution packaging in `scripts/package-release.ps1`: eliminated secondary executable clutter by producing a single standalone installer (`MagicBridge-Setup.exe` / `MagicBridge-v0.3.0-Setup.exe`) and a clean portable archive (`MagicBridge-v0.3.0-portable.zip`) containing strictly `magicbridge.exe` and translation resources.
2. **Progressive Windows Setup Wizard & Clean Uninstaller (`crates/settings/src/installer.rs` — closes #20):**
   - Redesigned installer as a true 5-step Windows Wizard with step breadcrumbs header (`1. Welcome` › `2. Options` › `3. Installing` › `4. Initial Setup` › `5. Ready`).
   - Clean basic options presentation by default (Desktop shortcut, autostart), with all advanced configurations (custom directory path, start menu shortcuts) masked behind a collapsible `[ ▸ Advanced Options ]` toggle.
   - Real animated installation progress bar with granular status text and background daemon startup.
   - Initial hardware detection probe (USB/Bluetooth + battery percentage) and first-run layout/modifier selection integrated into Step 4.
   - Complete uninstallation wizard (`--uninstall`), accessible via Windows Settings (Add/Remove Programs), Start Menu (`Uninstall MagicBridge.lnk`), and the Settings GUI (General and About views).
3. **Seamless Settings Panel Accessibility:**
   - Guaranteed immediate access to the full visual keyboard editor upon opening `magicbridge.exe` (defaults `current_tab` to `SettingsTab::Keyboard`).
   - Fixed Win32 tray menu click handler in `crates/agent/src/window.rs` to spawn `current_exe() --settings` instead of legacy executable name.
   - Added Installation & Maintenance section to General Settings view allowing one-click installation or uninstallation.
3. **Mandatory First-Run Setup & Testing Flow (closes #21):**
   - Detects whether `config.toml` existed on launch; starts on `SettingsTab::Wizard` for fresh installations.
   - When finishing the setup wizard, seamlessly transitions to `SettingsTab::HardwareQa` for interactive key testing.
   - Added `[Complete Testing & Open Keyboard Editor]` button to transition smoothly to the main editor.
4. **Visual Keyboard Ergonomics & Overlap Remediation (closes #22):**
   - In `crates/settings/src/keyboard_model.rs`, replaced `id.to_string()` fallback with `label_default(id)` mapping modifiers and numpad keys to concise symbols (`*`, `-`, `+`, `/`, `ctrl`, `cmd`, `opt`, `del`, `ins`).
   - Implemented inner keycap width boundary clamping in `calculate_key_font_size` ensuring labels never cross key boundaries.
   - In `keyboard_view.rs`, re-clicking the currently selected key toggles it off (`selected_key_id = None`).
   - Added `[Deselect Key]` button in key editor header and an informative instructional card explaining layers, Fn triggering, and modifier presets when no key is selected.
5. **Universal Windows Keyboard Layout Auto-Detection & Extended Languages (closes #23):**
   - Implemented `detect_windows_keyboard_layout()` calling Win32 `GetKeyboardLayout(0)`.
   - Implemented `get_installed_windows_layouts()` calling Win32 `GetKeyboardLayoutList` and `GetLocaleInfoW` (`LOCALE_SENGLISHDISPLAYNAME`) to dynamically enumerate all input languages active on the user's specific Windows PC.
   - Added full catalog of all standard Windows keyboard layouts across the globe: Dutch, Polish (Programmers), Russian, Ukrainian, Turkish (Q/F), Czech, Hungarian, Greek, Arabic, Hebrew, Korean, Chinese, ISO Spanish, ISO Portuguese, ISO Swiss, ISO Nordic, Japanese JIS, ISO Italian, ANSI US, and ISO UK, plus `Custom(String)` for any arbitrary language.
   - Integrated full Windows layout picker across Settings (Keyboard Profiles), Setup Wizard Step 1, and Native Progressive Setup Wizard Step 4.
6. **Quality Assurance & Verification:**
   - Ran `ui-qa-tester` autonomous crawler against `target\release\magicbridge.exe`: 15 steps completed with **0 findings / 0 defects**.
   - All 143 automated tests in workspace pass.
   - Packaged release bundle `MagicBridge-v0.3.0-windows-x64.zip` and standalone `MagicBridge-v0.3.0-Setup.exe` with WinGet and Scoop manifests.

### Objective & Architecture
- Implemented **IMP-04** (Dynamic Tray Battery Badge) in `crates/agent/src/tray_icon.rs` to render real-time battery status, connection state, charging indicator, and pause status directly on the Windows notification area system tray icon.
- Engineered 32-bit ARGB off-screen DIB section rendering via Windows GDI and `CreateIconIndirect`, providing true per-pixel alpha channel transparency compatible with Windows 10/11 taskbars without memory leaks.

### Key Deliverables & Changes
1. **Dynamic Battery Tray Badge Renderer (`crates/agent/src/tray_icon.rs`):**
   - Implemented `RgbaCanvas` supporting per-pixel ARGB drawing, axis-aligned rectangles, shadows, and a custom 3x5 bitmap font for crisp 2-digit percentage labels ("0".."100").
   - Responsive geometry automatically adapts to system DPI metrics (`SM_CXSMICON` / `SM_CYSMICON`, 16x16 up to 48x48).
   - Dynamic states rendered:
     - **Connected (Battery Gauge):** Visual battery chassis with fill color reflecting battery thresholds: Green (>25%), Amber (11-25%), Red (<= threshold).
     - **Charging State:** Electric yellow lightning bolt glyph with dark contrast outline overlaying the battery frame.
     - **Paused / Game Mode:** Centered amber pause bars (`||`).
     - **Disconnected:** Subtle dark slate keyboard chassis with disconnected status indicator.
   - Resource-safe GDI lifecycle: `CreateDIBSection` and `CreateBitmap` allocated temporary handles are deleted immediately with `DeleteObject` after `CreateIconIndirect`, and custom `HICON` handles are destroyed with `DestroyIcon` upon replacement or application shutdown.
2. **Tray Icon Lifecycle & Message Pump Integration (`crates/agent/src/tray.rs`, `window.rs`):**
   - Extended `TrayIcon` to track both `default_hicon` and `custom_hicon`.
   - Updated `poll_battery`, `device arrival/removal`, `WM_COMMAND`, `WM_POWERBROADCAST`, and `WM_IPC_STATE_UPDATED` to pass latest `DaemonState` (including `is_charging`).
3. **Configuration & GUI Toggle (`crates/core/src/config.rs`, `crates/settings/src/views/battery_view.rs`):**
   - Added `battery_tray_badge: bool` (default `true`) to `Config`.
   - Added interactive checkbox in Settings &rarr; Battery tab with localized descriptions in `en.toml` and `it.toml`.
4. **Verification & Quality Gate:**
   - Expanded unit test suite to 139 passing tests (including direct Win32 `HICON` generation and canvas pixel tests).
   - Autonomous UI crawler (`ui-qa-tester`) verified 0 findings across 15 steps and 9 UI states.

---

## 2026-10-27 — MagicBridge v0.2.1: Autonomous UI QA Loop & AccessKit Accessibility (Release ✅, closes #17, closes #18)

### Objective & QA Loop Execution
- Connected `ui-qa-tester` (`https://github.com/dooperdrew/ui-qa-tester`) to MagicBridge via dedicated configuration `qa-session.toml` in the repository root.
- Executed multi-cycle autonomous background crawl testing (`magicbridge-settings.exe`):
  - **Cycle 1 (Initial Crawl):** Discovered 7 findings (2 High visual clipping, 1 Medium UX dead button friction, 4 Medium accessibility gaps). Opened tracking issues #17 and #18.
  - **Cycle 2:** Fixed Hardware QA layout clipping and keyboard canvas allocation; findings reduced from 7 to 3.
  - **Cycle 3 & 4:** Added `set_accessible_name` helper leveraging AccessKit `set_label` on interactive dialog controls, sliders, and combo boxes; findings dropped to 2.
  - **Cycle 5 (Final Verification):** Implemented disabled state on already-active tab buttons and accessible labels for general settings inputs. Resulted in **0 Critical, 0 High, 0 Medium, 0 Low — 100% clean QA run**.

### Key Deliverables & Bug Fixes
1. **Visual Layout & Responsive Clamping Fixes (Closes #17):**
   - In `qa_view.rs`, converted the 7-button category selector from a rigid `ui.horizontal` to `ui.horizontal_wrapped`, preventing overflow clipping beyond the 936px window bounds.
   - Refactored `render_qa_visual_keyboard` to dynamically clamp `unit_size` against `available_width.min(ui.max_rect().width())` without causing child element inflation.
   - Changed background plate interaction from `Sense::click_and_drag()` to `Sense::hover()`, eliminating ghost custom controls in UIA.
2. **AccessKit / UIA Accessibility & UX Friction Remediation (Closes #18):**
   - Implemented `crate::set_accessible_name(ui, id, name)` helper wrapping `ui.ctx().accesskit_node_builder(...)` with `node.set_label(...)`.
   - Annotated all interactive ComboBoxes (`profile_selector`, `layout_selector`, `presets_dropdown`, `ui_language_combo`, `fn_key_combo`, `dialog_new_layout`) and Singleline/Multiline TextEdits with explicit accessible labels and tooltips.
   - Fixed UX friction on active tab buttons: tab buttons now render as disabled when already selected (`ui.add_enabled(!is_active, btn)`), providing unambiguous accessibility state feedback to assistive technologies and autonomous crawlers.
3. **Repository Tooling:**
   - Added `qa-session.toml` configured for headless background crawling with targeted navigation goals for all 8 application tabs.
   - Added `qa-reports/` to `.gitignore`.

---

## 2026-10-25 — Milestone 11: GitHub Copilot Agent Integration & Cloud Multi-Agent Workflow (Completed ✅, closes #16)

### Objective & Architecture
- Integrated GitHub Copilot (Agent & IDE) into the MagicBridge multi-agent ecosystem using a strictly defined division of labor:
  - **Lead Orchestrator (Antigravity):** High-level architect, task planner, verification gatekeeper, and **Sole Committer** to `origin main`.
  - **GitHub Copilot Agent:** Fast draft implementer working in the cloud on isolated feature branches (`task/<issue-id>-<description>`) delivering changes exclusively via **Draft Pull Requests**.

### Key Deliverables & Changes
1. **Hardened Permanent Copilot Instructions (`.github/copilot-instructions.md`):**
   - Codified non-negotiable performance budgets: `WH_KEYBOARD_LL` input hook callback must NEVER allocate heap memory, perform disk/file I/O, output log prints, or acquire locks (<0.1ms latency).
   - Enforced pure Rust in `crates/core` with `#![forbid(unsafe_code)]` and zero Win32 dependencies.
   - Enforced Sole Committer protocol: Copilot is strictly prohibited from pushing directly to `main` or closing issues.
   - Codified Apple A1843 hardware ground truth (VID `0x05AC`/`0x004C`, kernel Fn stripping, Italian AltGr `0x21D` scancode quirk, battery report ID `0x90`).
   - Documented toolchain PATH resiliency preamble (`$env:Path = ...`) and test concurrency isolation (`-- --test-threads=1`).
   - Enforced the "Claims follow evidence" principle.
2. **Automated Agent Task Issue Templates:**
   - Authored `.github/ISSUE_TEMPLATE/agent_task.md` (markdown template) and `.github/ISSUE_TEMPLATE/agent_task.yml` (interactive GitHub form) allowing the user or Lead Agent to quickly prepare bounded, context-rich implementation tasks for AI coding agents with explicit file ownership, invariants, and acceptance criteria.
3. **Multi-Agent Methodology Extension (`docs/multi-agent-methodology.md`):**
   - Added Section 7 detailing the hybrid local + cloud multi-agent lifecycle with Mermaid interaction flowchart and sequence diagrams.
4. **Governance Alignment & Documentation Centralization:**
   - Updated `AGENTS.md` and `README.md` to formally document GitHub Copilot Agent as a supported cloud worker class.
   - Hardened `.github/pull_request_template.md` with Agent task options, requirement tracking, and Sole Committer checklist.
   - Updated `CONTRIBUTING.md` with multi-agent branch conventions and task handoff lifecycle.
   - Created centralized documentation hub `docs/README.md` cataloging the specification hierarchy, reading order, and 16-document inventory.
   - Added prominent **📥 Download & Installation** guide to `README.md` and expanded `docs/user-guide.md` with detailed instructions for all 4 distribution methods: Microsoft WinGet (`winget install DooperDrew.MagicBridge`), Scoop (`scoop install ...`), Graphical Setup Wizard (`setup.cmd` / `setup-wizard.ps1`), and Portable Release ZIP.

---

## 2026-10-25 — MagicBridge v0.2.0: UX Onboarding, Responsive Settings & Hardware QA Tooling (Release ✅)

### Distribution Channels & Ecosystem Strategy
- **Frictionless Distribution:**
  - Generated official Microsoft WinGet package manifests in `packaging/winget/manifests/d/DooperDrew/magicbridge/0.2.0/` targeting `DooperDrew.MagicBridge`.
  - Created Scoop community package manifest in `packaging/scoop/magicbridge.json`.
  - Implemented automated manifest generation script `scripts/generate-manifests.ps1` and integrated into `scripts/package-release.ps1`.
- **Community Growth & Multi-Device Roadmap:**
  - Published comprehensive master plan `docs/08-community-and-growth-strategy.md` covering distribution, launch playbooks (Hacker News Show HN, Reddit showcases), crowdsourced hardware QA registry, and expansion into Compact Magic Keyboards (A1644), Magic Trackpad (A1535), and Magic Mouse (A1657).
  - Added structured GitHub Issue templates for Hardware QA Verification, Bug Reports, and Feature Requests.
- **Repository Alignment:**
  - Renamed GitHub repository to `https://github.com/DooperDrew/magicbridge`.
  - Formulated official release and versioning methodology in `docs/release-methodology.md`.

### M10.1 — Guided Windows Installer Wizard (Completed ✅, closes #12)
- Implemented rich graphical Windows setup wizard (`scripts/setup-wizard.ps1`) and Explorer double-click launcher (`scripts/setup.cmd`):
  - Zero-elevation installation directly into `%LOCALAPPDATA%\Programs\MagicBridge`.
  - Windows Forms wizard with custom install destination selection, Start Menu shortcut toggle, Desktop shortcut toggle, Windows sign-in autostart toggle, and option to launch the setup wizard immediately upon finish.
  - Automatically terminates running `magicbridge-agent.exe` prior to file copy to prevent in-use binary locking.
  - Registers Windows Settings "Installed apps" & Control Panel uninstaller in `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\MagicBridge` with complete metadata (`DisplayName`, `Publisher`, `DisplayVersion`, `DisplayIcon`, `UninstallString`, `QuietUninstallString`).
  - Added production-grade Inno Setup script (`scripts/installer.iss`) with `PrivilegesRequired=lowest`.
  - Self-cleaning uninstallation script (`scripts/uninstall.ps1`) supporting interactive and silent modes.
  - Updated `scripts/package-release.ps1` to bundle setup tools in `dist/`.

### M10.2 — First-Run Initial Configuration Onboarding Wizard (Completed ✅, closes #13)
- Created `crates/settings/src/views/wizard_view.rs`:
  - 6-step guided setup flow: Welcome & hardware auto-detection (USB/BT, battery), Layout selection (ISO Italian, ANSI US, ISO UK, ISO German, ISO French), Modifier Preset selection (Mac Layout vs Windows Standard), Fn Trigger key choice (Right Option, Right Control, Caps Lock, F19), Autostart preference, and Real-time interactive key test box.
  - Integrated into Settings GUI as dedicated **🪄 Setup Wizard** tab.
  - Complete English and Italian localization keys under `[wizard]`.
  - Comprehensive unit test suite covering transitions and config mutation.

### M10.3 — Fluid Responsive Settings Layout & Adaptive Navigation (Completed ✅, closes #14)
- Enhanced `crates/settings/src/views/keyboard_view.rs`:
  - Responsive visual keyboard grid with dynamic unit size clamping: `(available_width / 24.5).clamp(20.0, 42.0)`.
  - Proportional font scaling (`calculate_key_font_size`) and key padding avoiding label clipping on compact screens (<900px) and maintaining crispness on 4K ultra-wide displays.
  - Adaptive Key Editor side-panel: dynamic stacked layout on compact screens (<850px) vs side-by-side on wide screens.
- Enhanced `crates/settings/src/app.rs`:
  - Defined `COMPACT_MODE_BREAKPOINT = 850.0`.
  - Adaptive Navigation Tab Bar: Compact Mode (<850px) renders compact icon-only buttons with hover tooltips and scrollable horizontal rail, preventing overflow or awkward wrapping. Full Mode (>=850px) renders complete icon + label tabs.

### M9.3 — Interactive Step-by-Step Hardware QA Key Verification Tool (Completed ✅)
- Created `crates/settings/src/views/qa_view.rs`:
  - Visual matrix of all 109 physical keys on Apple Magic Keyboard (Model A1843).
  - Dual-listener engine capturing window events and Win32 `GetAsyncKeyState` modifier states (differentiating physical Left vs Right Command, Option, Control, and Shift).
  - Turns verified keys bright green (`#2ECC71`) in real-time as keys are pressed physically.
  - Progress tracking (`Verified: X / 109 keys (Y%)`) with celebratory state and category breakdowns (Modifiers, Navigation, Number Row, Function Row F1..F19, Numpad including Clear, Alphanumeric).
  - One-click QA Log Export saving formatted `.txt` and programmatic `.json` reports to Desktop/AppData.
  - Integrated as dedicated **🧪 Hardware QA** tab in Settings.

---

## 2026-10-25 — QA Framework, UX Specifications & GitHub Issue Tracking

### QA Test Suite & UX Backlog Formalization
- **Dedicated QA Specification (`docs/06-qa-and-validation.md`):**
  - Level 1-4 validation architecture spanning automated unit invariants, soak/hardening gates, empirical hardware tests, and security edge cases.
  - Complete 12-point empirical hardware test matrix for physical Apple Magic Keyboard (Model A1843) across USB (`VID 0x05AC`) and Bluetooth Classic (`VID 0x004C`).
  - Validation procedures for the Italian/ISO AltGr quirk (`sc=0x21D`), Modern Standby/sleep recovery, UIPI admin window isolation, and Game Mode auto-pause.
- **Dedicated UX Roadmap (`docs/07-ux-and-improvements.md`):**
  - Specification for Guided Windows Installer Wizard (`magicbridge-setup.exe` / Inno Setup) with zero UAC elevation requirement.
  - Architecture for First-Run Initial Configuration Onboarding Wizard in `magicbridge-settings`: device auto-detect, layout picker, modifier preset switcher, Fn trigger key choice, and autostart toggle.
  - Technical design for Fluid Responsive Layout in `magicbridge-settings` (`egui`): dynamic keyboard scale clamping (`ui.available_width() / 25.5`) and adaptive navigation rail for compact and high-DPI screens.
  - Feature backlog: dynamic battery tray badge, DDC/CI monitor brightness integration, per-app profiles, and GitHub release checker.
- **GitHub Issue Tracking & Milestone Alignment:**
  - Created GitHub Milestones **M9 — Quality Assurance & Empirical Validation** and **M10 — UX & Onboarding Improvements**.
  - Logged tracking issues: #10 (Hardware QA Matrix), #11 (Power States & Edge Cases), #12 (Installer Wizard), #13 (Onboarding Wizard), #14 (Responsive Settings Layout), and #15 (Battery Tray Badge).
  - Integrated direct links to QA Matrix and UX Roadmap in the Settings Architecture Tab and `README.md`.

---

## 2026-10-25 — Milestone 8: Multi-Agent Methodology & Release Parity (Completed ✅)

### M8.1..M8.3 — Didactic Multi-Agent Guide, GUI Architecture Tab & Release Parity
- Created comprehensive didactic documentation [`docs/multi-agent-methodology.md`](docs/multi-agent-methodology.md):
  - In-depth explanation of the Hub-and-Spoke Lead/Worker model, dynamic model selection tiers (T1/T2/T3), Sole Committer protocol, and the 4-step Lead Verification Gate.
  - Complete Mermaid sequence and dataflow architecture diagrams for all 5 parallel execution phases.
  - Subagent taxonomy, reporting contract templates, and instructions on how to extend the workflow for future tasks.
- Added dedicated **"🤖 Architecture"** tab in `magicbridge-settings` (`crates/settings/src/views/architecture_view.rs`):
  - Interactive multi-section view covering Overview, Hub-and-Spoke model, Complexity Tiers, Parallel Phases, and Extending Over Time.
  - Added direct browser links to GitHub documentation and localization keys in English (`en.toml`) and Italian (`it.toml`).
- Release & Distribution Parity (M8.3):
  - Created official **GitHub Release `v0.1.0`** attaching the distribution archive `MagicBridge-v0.1.0-windows-x64.zip` directly to the release page.
  - Re-synced and confirmed 100% parity between local repository, git remotes, and GitHub release assets.

---

## 2026-10-25 — Milestone 7: Packaging & Release Automation (Completed ✅)

### M7.1..M7.3 — Portable ZIP Script, User-Space Installer & User Guide
- Implemented `scripts/package-release.ps1`:
  - Compiles release binaries with optimizations.
  - Assembles staging bundle with `magicbridge-agent.exe`, `magicbridge-settings.exe`, `locales/` (`en.toml`, `it.toml`), documentation, and installer scripts.
  - Generates portable ZIP archive `dist/MagicBridge-v<version>-windows-x64.zip` (~2.1 MB).
- Implemented user-space installer and uninstaller scripts:
  - `scripts/install.ps1`: Deploys to `%LOCALAPPDATA%\Programs\MagicBridge`, creates Start Menu shortcuts, and registers per-user autostart in `HKCU\...\Run` (zero admin rights required).
  - `scripts/uninstall.ps1`: Gracefully terminates running processes, removes shortcuts, and cleans autostart keys.
- Comprehensive End-User Documentation:
  - Created `docs/user-guide.md` covering setup, pairing, Fn navigation, media key controls, named profiles, battery telemetry, and Windows UIPI limitations.
  - Added official `LICENSE` file (MIT).
- Subagent: `T1-Package-M7` (flash).

---

## 2026-10-25 — Milestone 6: Hardening, Performance Verification & Game Mode (Completed ✅)

### M6.1..M6.5 — Game Mode Auto-Pause, Debug Logger, Soak Tests & Latency Budget
- Implemented `crates/agent/src/game_mode.rs`:
  - Event-driven foreground tracker using `SetWinEventHook` for `EVENT_SYSTEM_FOREGROUND` (0% idle CPU per NFR-P2).
  - Auto-pauses input hook and releases synthetic keys when a game from `config.game_mode_processes` or fullscreen exclusive application is focused.
  - Restores active hook state upon return to desktop.
- Implemented `crates/agent/src/logger.rs`:
  - Zero-overhead atomic flag check when disabled.
  - Timestamped diagnostic log writing to `%APPDATA%\MagicBridge\debug.log` and in-memory ring buffer (up to 200 entries) when enabled.
- Hardening & Soak Tests (`crates/agent/tests/`):
  - Benchmarked hook latency: average latency measured < 1 µs (< 1,000 ns), outperforming NFR-P3's 100 µs (0.1 ms) budget by >100x.
  - Verified emergency key release and rapid 500-iteration typing/pause toggling with zero stuck keys (`NFR-S3`).
  - Simulated 500 consecutive sleep/wake (`WM_POWERBROADCAST`) and device arrival/removal loops.
- Model Fallback Audit (M6.5):
  - Audited `docs/model-fallback-log.md`: zero fallback occurrences recorded; all tasks were executed using required target tiers.
- Subagent: `T3-Hardening-M6` (pro).

---

## 2026-10-25 — Milestone 5: Settings GUI (Completed ✅)

### M5.1..M5.6 — Standalone `eframe`/`egui` Settings GUI with Visual Keyboard & Profiles
- Implemented `crates/settings` (`magicbridge-settings.exe`):
  - Standalone native binary opening a `~920x540` window; exits completely upon close with zero resident memory (`NFR-P5`).
  - Top banner with one-click "Start Agent" button when `magicbridge-agent.exe` is not running.
  - Named Pipe IPC client connecting to `\\.\pipe\MagicBridge_Agent_IPC` for instant live status queries and atomic table reloading (`F7.5`).
- Visual A1843 Keyboard Widget:
  - Drawn to scale with all 109 keys across 3 distinct clusters (Main, Navigation, and Numpad).
  - Physical & Language layout switching (`ISO Italian`, `ANSI US`, `ISO UK`, `ISO German`, `ISO French`, and custom labels).
  - Normal vs. Fn layer toggling with visual highlighting for remapped keys.
  - Interactive Key Editor side panel and "Press a key to select" mode.
  - Modifier presets dropdown (Windows layout, Mac layout, Reset All).
- Complete Tab Navigation:
  - **Keyboard:** Profile bar (Create, Duplicate, Rename, Delete), visual map, layout selector, and key editor.
  - **Top Row & Fn:** Mode toggle (Media-first vs. F-keys first), toggle hotkey, and Fn fallback key selector.
  - **Battery:** Live gauge/percentage, charging indicator, alert threshold, and poll interval.
  - **General:** UI language selector, autostart toggle (`HKCU\...\Run`), pause hotkey, game mode process list, and configuration import/export.
  - **About:** Diagnostic daemon status, version, open-source license, and GitHub repository link.
- Zero Hardcoded UI Text (i18n):
  - English (`locales/en.toml`) and Italian (`locales/it.toml`) translation dictionaries with dynamic runtime switching.
- Subagent: `T2-UI-M5` (Claude 3.7 Sonnet).

---

## 2026-10-25 — Milestone 4: Custom Actions & Battery (Completed ✅)

### M4.1 & M4.2 — Off-Thread Action Dispatcher & Direct Apple HID Battery Reader
- Implemented `crates/agent/src/action_worker.rs`:
  - Off-thread asynchronous action dispatcher using a bounded channel (`sync_channel(64)`).
  - Guarantees zero blocking on the `WH_KEYBOARD_LL` input hook (`NFR-P3`).
  - Supports `Launch { program, args }`, `OpenUrl { url }` via `ShellExecuteW`, `Shortcut` (multi-key sequences), and `Media` actions.
- Implemented `crates/agent/src/battery.rs`:
  - Direct Apple HID Input Report `0x90` reader on collection `0xFF00:0x0014` via `HidD_GetInputReport`.
  - Works identically on both USB (`MI_00&Col02`) and Bluetooth (`Col02`).
  - Decodes charging/discharging/AC present bitflags and battery percentage (`0..100%`).
  - Fallback support for Bluetooth PnP battery property (`PKEY_BLUETOOTH_BATTERY`).
- Low-frequency polling & toast alerts in `crates/agent/src/window.rs`:
  - Win32 periodic timer (`SetTimer` `IDT_BATTERY_TIMER`) based on `config.battery_poll_minutes` (default 10 min, zero busy loops).
  - Immediate poll on device arrival (`DBT_DEVICEARRIVAL`) and wake from sleep (`WM_POWERBROADCAST`).
  - Windows notification balloon/toast alert when battery drops below `low_battery_threshold_percent` (15%).
- Subagent: `T3-Battery-M4` (inherit/pro).

---

## 2026-10-25 — Milestone 3: Fn Layer & Top Row Modes (Completed ✅)

### M3.1..M3.4 — Top Row Special Keys, Media Switching & Navigation Combos
- Implemented top row shortcuts and mode switching:
  - F3 (Mission Control) -> Windows Task View (`Win + Tab`) via atomic multi-key shortcut synthesis.
  - F4 (Launchpad) -> Windows Start Menu (`VK_LWIN`).
  - F7..F12 -> Media keys (Rewind, Play/Pause, Fast Forward, Mute, Volume Down, Volume Up).
  - `TopRowMode::MediaKeysFirst` vs. `TopRowMode::FKeysFirst` toggles smoothly between direct press and Fn+press.
  - Built-in navigation combos: `Fn + Arrows` -> `Home/End/PgUp/PgDn`, `Fn + Backspace` -> `Forward Delete (⌦)`, `Fn + Enter` -> `Insert`.
- Subagent: `T2-TopRow-M3` (flash/pro).

---

## 2026-10-25 — Milestone 2: Remapping Engine (Completed ✅)

### M2.2 & M2.5 — Low-Level Hook, SendInput Injector, AltGr Suppression & Named Pipe IPC
- Implemented `crates/agent/src/hook.rs`:
  - `WH_KEYBOARD_LL` low-level hook procedure operating with zero heap allocation, zero disk/file I/O, and zero mutex locks (`NFR-P3`).
  - Fast-path for non-remapped keys passing directly to `CallNextHookEx`.
  - Evaluates `dwExtraInfo == MAGICBRIDGE_EXTRA_INFO` (`0x4D424745`) to prevent self-injection loops.
  - Automatically swallows fake Windows `VK_LCONTROL` with `scanCode == 0x21D` when Right Option is remapped or acts as Fn (`hardware-findings §4.1`).
  - Lock-free `[AtomicU32; 256]` active key state tracker; `release_all_active_keys` releases any active synthetic keys on pause/disable/sleep/exit to guarantee zero stuck keys (`NFR-S3`).
- Implemented `crates/agent/src/injector.rs`:
  - Thin `SendInput` wrapper tagging synthetic keystrokes with `MAGICBRIDGE_EXTRA_INFO` sentinel.
- Implemented `crates/agent/src/ipc.rs`:
  - `AtomicMappingTable`: Lock-free table swapping built on `arc_swap::ArcSwap<CompiledMappingTable>`.
  - Named Pipe IPC server listening on `\\.\pipe\MagicBridge_Agent_IPC`.
  - Request/response protocol supporting `Reload`, `SwitchProfile`, `GetStatus`, and `Pause`.
  - Lightweight `IpcClient` with end-to-end integration tests.
- Verified modifier presets (Windows layout Cmd<->Alt, Option<->Win) and missing key defaults (`F13`->`PrintScreen`, `F14`->`ScrollLock`, `F15`->`Pause`) across the hook pipeline (M2.3, M2.4).
- Subagents: `T3-Hook-M2.2` (inherit/pro) and `T2-IPC-M2.5` (flash/sonnet) running in parallel.

### M2.1 — Pure Remapping Engine in `crates/core`
- Implemented core key models, layer definitions, and zero-allocation precomputed lookup table in `crates/core`:
  - `key`: Windows Virtual Key constants (`VK_LWIN`, `VK_RMENU`, `VK_SNAPSHOT`, `VK_SCROLL`, `VK_PAUSE`, etc.), default scancodes, and `MAGICBRIDGE_EXTRA_INFO: usize = 0x4D424745` ("MBGE") self-injection prevention sentinel.
  - `engine`: `CompiledMappingTable` with flat 256-element arrays for normal and Fn layers, supporting `PassThrough`, `RemapKey`, `CustomAction`, and `Swallow`.
  - O(1) table lookup with zero heap allocations, zero locks, and zero I/O (`NFR-P3`).
  - `compile_profile`: Compiles `NamedProfile` applying modifier presets (Windows layout 4-key swap vs. Mac layout), default missing keys (`F13`->`PrintScreen`, `F14`->`ScrollLock`, `F15`->`Pause`), top-row mode, and Fn combos (`Home`/`End`/`PgUp`/`PgDn`/`Del`/`Ins`).
  - Automatic ISO Italian `AltGr` synthetic `LCtrl` (`sc=0x21D`) suppression whenever Right Option (`VK_RMENU`) is remapped or acts as the Fn key.
  - Subagent: `T1-Engine-M2.1` running `flash`.

---

## 2026-10-25 — Milestone 1: Agent Skeleton (Completed ✅)

### M1.4 — Device Arrival/Removal & Power Management
- Implemented Windows hardware event and power management lifecycle in `crates/agent`:
  - `device_notify`: RAII `DeviceNotificationHandle` registering `RegisterDeviceNotificationW` for `GUID_DEVINTERFACE_HID`.
  - Filtered arrival (`DBT_DEVICEARRIVAL`) and removal (`DBT_DEVICEREMOVECOMPLETE`) notifications using `magicbridge_core::is_apple_vendor` (`0x05AC` on USB and `0x004C` over Bluetooth Classic).
  - Tracked connected Apple keyboards with dynamic tray tooltip updates.
  - Handled `WM_POWERBROADCAST` (`PBT_APMSUSPEND`, `PBT_APMRESUMEAUTOMATIC`, `PBT_APMRESUMESUSPEND`) to re-probe connected keyboards after system sleep/resume.
  - Subagent: `T2-Device-M1.4` running `flash`.

### M1.1 — Daemon Shell, Single-Instance Mutex, Hidden Window & Tray Icon
- Implemented background daemon runtime shell in `crates/agent`:
  - `single_instance`: Named Win32 Mutex `Local\MagicBridge_SingleInstance_Mutex` to ensure single instance (F8.3), exiting cleanly with code 0 on duplicate.
  - `window`: Hidden message window (`MagicBridge_MsgWndClass`) with clean Win32 message pump (`GetMessageW` / `TranslateMessage` / `DispatchMessageW`). Handles `WM_CREATE`, `WM_COMMAND`, `WM_TRAY_CALLBACK`, and `TaskbarCreated` broadcast message to restore tray icon if Windows Explorer restarts.
  - `tray`: System tray icon (`Shell_NotifyIconW`) with tooltip showing keyboard connection status, battery, active profile, and top-row mode.
  - Context menu: Enabled/Paused toggle, Profile submenu, Top Row mode toggle, Open Settings... (launches `magicbridge-settings.exe`), Start with Windows toggle, and Quit.
  - Subagent: `T2-Agent-M1.1` running `flash`.

### M1.3 — Windows Autostart Toggle (HKCU Run Key)
- Implemented per-user autostart manager in `crates/agent/src/autostart.rs`:
  - Interacts with `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` value `"MagicBridge"` (`REG_SZ`).
  - Pure user-space, zero administrator or UAC elevation required (NFR-S1).
  - Handles command line path quoting, RAII `RegCloseKey` handles, and idempotent toggling (`enable_autostart`, `disable_autostart`, `is_autostart_enabled`).
  - Includes 8 tests verifying quoting, spacing, and isolated registry CRUD operations.
  - Subagent: `T1-Autostart-M1.3` running `flash`.

### M1.2 — Config Schema in `crates/core`
- Implemented complete configuration schema, named profiles, multi-language layout models, and TOML persistence in `magicbridge_core::config`:
  - `Config`: global settings (`active_profile`, `autostart`, `battery_poll_minutes`, `low_battery_threshold_percent`, `ui_language`, `pause_hotkey`, `debug_logging`, and `profiles`).
  - `NamedProfile`: encapsulates `name`, `layout: KeyboardLayout`, `modifier_preset: ModifierPreset`, `top_row_mode: TopRowMode`, `fn_key: FnKeyChoice`, `mappings: BTreeMap<String, KeyAction>`, and `fn_mappings: BTreeMap<String, KeyAction>`.
  - Enums: `KeyboardLayout` (`IsoItalian`, `AnsiUs`, `IsoUk`, `IsoGerman`, `IsoFrench`, `Custom`), `ModifierPreset` (`WindowsLayout`, `MacLayout`, `Custom`), `TopRowMode` (`MediaKeysFirst`, `FKeysFirst`), and `FnKeyChoice` (`RightOption`, `RightControl`, `F19`, `Disabled`, `Other`).
  - Sensible out-of-the-box defaults: generates a "Default (IT)" profile matching an Italian ISO keyboard with Windows layout modifiers (`Cmd`->`Alt`, `Option`->`Win`), Right Option as Fn trigger, Media keys first, default missing keys (`F13`->`PrintScreen`, `F14`->`ScrollLock`, `F15`->`Pause`), and standard Fn combos (`Left/Right`->`Home/End`, `Up/Down`->`PageUp/PageDown`, `Backspace`->`Delete`, `Enter`->`Insert`).
  - Safe atomic file persistence via temporary file creation, data flushing (`sync_all()`), and atomic replacement.
  - Resolved `%APPDATA%\MagicBridge\config.toml` path helpers.
- Added 10 comprehensive unit tests covering serialization, round-trip parsing, custom profiles, duplication, deletion safety guards, and atomic writes.
- Subagent: `T1-Core-M1` running token-efficient `flash` model. All local verification gates passed.

---

## 2026-10-25 — Milestone 0: Hardware Spike 🔬 (Completed)

### M0.1 — Cargo Workspace Skeleton (`4935fbf`)
- Created Cargo workspace (`resolver = "3"`, Rust 2024 edition) with 4 crates:
  - `crates/core` (`magicbridge-core`): pure business logic, `#![forbid(unsafe_code)]`, zero Win32 dependencies.
  - `crates/agent` (`magicbridge-agent`): background tray/hook/HID agent executable.
  - `crates/settings` (`magicbridge-settings`): `egui`/`eframe` settings GUI executable.
  - `crates/hid-probe` (`hid-probe`): diagnostic CLI for Milestone 0 hardware probing.
- Configured size/latency-tuned release profile (`opt-level = "z"`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = true`) and workspace lints (`unsafe_op_in_unsafe_fn = "deny"`, `clippy::undocumented_unsafe_blocks = "deny"`).

### M0.2 & M0.3 — `hid-probe` Implementation (`fb1cc69`)
- Added `magicbridge_core::device::parse_hid_path` to extract transport (`USB`, `Bluetooth`, `BluetoothLe`), `VID`, `PID`, USB interface (`MI_xx`), and top-level collection (`Colxx`) from Windows HID paths, with unit tests.
- Implemented `hid-probe list`, `hid-probe dump` (simultaneous `WH_KEYBOARD_LL`, `WM_INPUT` `RIM_TYPEKEYBOARD` / `RIM_TYPEHID`, and direct `ReadFile` threads with microsecond QPC timestamps and Hook↔Raw Input pairing), and `hid-probe battery`.

### M0.4, M0.5, M0.6, M0.7 — Physical A1843 Hardware Findings (`ae17257`, `ac95648`)
- **Dual Vendor IDs (`0x05AC` on USB, `0x004C` on Bluetooth):**
  - Over USB, the A1843 reports `VID = 0x05AC` (USB-IF Apple Inc.), `PID = 0x026C`, rev `0x0852`.
  - Over Bluetooth Classic (`{00001124-0000-1000-8000-00805f9b34fb}`), the A1843 reports **`VID = 0x004C`** (Bluetooth SIG Apple Inc., `VID&0001004c`), `PID = 0x026C`.
  - Updated `magicbridge_core::is_apple_vendor` to match both `0x05AC` and `0x004C`.
- **Why Physical `Fn` (`0x00FF:0x0003`) and `Eject` (`0x000C:0x00B8`) Are Blocked in User Mode:**
  - Both usages are declared inside Input Report `0x01` (10B) of the main `0x0001:0x0006` (Generic Desktop / Keyboard) collection (`MI_01&Col01` on USB, `Col01` on Bluetooth).
  - Windows `kbdhid.sys` opens `0x0001:0x0006` collections exclusively (`ERROR_ACCESS_DENIED`) and strips non-`0x07` usages in kernel mode before `WH_KEYBOARD_LL` or `WM_INPUT`.
  - **Decision:** Use the configurable Fn key approach (`F1.1` / `F1.4`, default Right Option / `VK_RMENU`) in user mode rather than a kernel filter driver (`NFR-S1`).
- **ISO / Italian `AltGr` Quirk (`scanCode = 0x21D`):**
  - Pressing Right Option (`VK_RMENU`, `vk=0xA5`, `sc=0x38`, `ext=1`) on an Italian/ISO layout causes Windows to synthesize a fake `VK_LCONTROL` (`vk=0xA2`, `sc=0x21D`, `ext=0`) ~0.8 ms earlier. The M2 hook must swallow `sc=0x21D` whenever `VK_RMENU` is remapped.
- **`Clear` Key:**
  - Sends `VK_NUMLOCK` (`vk=0x90`, `sc=0x45`, `ext=1`).
- **Battery Reporting (`0xFF00:0x0014`, Report `0x90`):**
  - Works identically on both USB (`MI_00&Col02`) and Bluetooth (`Col02`) via `HidD_GetInputReport(0x90)` (3 bytes: `[0x90, status_flags, percentage]`). Observed `90 01 64` (100%, charging) on USB and `90 00 64` (100%, on battery) on Bluetooth.
- **Device Filtering Timing:**
  - `WH_KEYBOARD_LL` always fires **0.28–2.41 ms before** `WM_INPUT` (`RAWKB`). Per-keystroke device filtering inside `WH_KEYBOARD_LL` is therefore impossible without blocking; presence-based activation (active while an Apple `0x05AC`/`0x004C` keyboard is connected) is used.

### Scope & Workflow Alignment — Profiles, Multi-Language Layouts & GitHub Tracking
- Added requirement **F7.7** for **named profiles** and **multiple language/physical keyboard layouts** (`ISO Italian`, `ANSI US`, `ISO UK`, `ISO German`, `ISO French`, plus custom labels), switchable from the tray menu, GUI, or hotkey.
- Made the GitHub repository (`DooperDrew/magicbridge`) public, added `CHANGELOG.md`, and created GitHub Issues for all milestones (`M0`–`M7`).
