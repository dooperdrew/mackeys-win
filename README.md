<div align="center">

# MacKeys ⌨️ 🌉

**A tiny, invisible, native Windows 10/11 utility that makes Apple Keyboards (Full-Size & Compact) feel right at home on Windows.**

[![Latest Release](https://img.shields.io/github/v/release/DooperDrew/mackeys-win?logo=github&label=release%20v0.4.1)](https://github.com/DooperDrew/mackeys-win/releases/latest)
[![Rust Stable](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform: Windows 10/11 x64](https://img.shields.io/badge/platform-Windows%2010%2F11%20x64-informational.svg)](https://microsoft.com/windows)
[![0% Idle CPU](https://img.shields.io/badge/idle%20CPU-0%25-success.svg)](#performance--safety-budgets)
[![Anti-Cheat Safe](https://img.shields.io/badge/anti--cheat-safe%20(no%20drivers)-success.svg)](#core-philosophy)

<br/>

| 📖 [User Guide](docs/user-guide.md) | 🪄 [Setup Installer](#-download--installation) | 🧪 [Hardware QA](docs/06-qa-and-validation.md) | 📦 [WinGet & Releases](docs/release-methodology.md) | 🤝 [Community Plan](docs/08-community-and-growth-strategy.md) | 🤖 [Architecture](docs/multi-agent-methodology.md) |
|:---:|:---:|:---:|:---:|:---:|:---:|

</div>

---

## Supported Apple Keyboards

MacKeys natively supports the full desktop Apple keyboard family:
- **Full-Size with Numeric Keypad (109 Keys):** Magic Keyboard with Numeric Keypad (`A1843`, `A2520`, `A3123`) & Apple Keyboard Wired (`A1243`).
- **Compact Tenkeyless (78/79 Keys):** Magic Keyboard Compact (`A1644`, `A2449`, `A2450`, `A3122`) & Apple Wireless Keyboard (`A1314`).
- **Live Hardware Auto-Detection:** Automatically identifies connected models via USB/Bluetooth Product ID (PID) and adapts the visual keyboard geometry.

---

## The Problem

Apple Keyboards are gorgeous pieces of hardware, but when paired with Windows 10 or 11 over Bluetooth or Lightning/USB:
- **Missing or locked Fn key:** Windows' inbox `kbdhid.sys` locks the main keyboard collection exclusively, leaving the Fn layer inaccessible.
- **Modifier chaos:** `Cmd` and `Option` sit physically where Windows users expect `Alt` and `Windows`.
- **Dumb top row:** F1–F12 media keys (volume, play/pause, rewind) don't behave reliably or switchable with standard function keys.
- **Missing standard keys:** No `Print Screen`, `Scroll Lock`, `Pause`, or dedicated `Insert` / Forward `Delete`.
- **No battery indicator:** Windows does not surface battery percentage on the taskbar or tray.
- **Heavy or invasive workarounds:** Existing third-party solutions often require paid licenses, obsolete Apple Boot Camp drivers, kernel-mode filters (which trigger game anti-cheat bans), or bulky Electron apps.

**MacKeys solves this with zero kernel drivers, zero DLL injection, and zero background idle CPU.**

---

## ⚡ How MacKeys Compares to Alternatives

| Feature / Invariant | **MacKeys** | *Magic Utilities* | *Apple Boot Camp Drivers* | *WinAppleKey* | *AutoHotkey Scripts (AHK)* |
|---|:---:|:---:|:---:|:---:|:---:|
| **Architecture** | **Driverless (User Mode)** | Kernel Driver (`.sys`) | Kernel Driver (`.sys`) | Kernel LowerFilter | Script Interpreter |
| **Game Anti-Cheat Safe** | **✅ 100% Safe** | ⚠️ Risk (Kernel hook) | ⚠️ Risk (Kernel driver) | ⚠️ Risk (Test-signed) | ⚠️ Flagged by some anticheats |
| **Idle CPU Overhead** | **0.0% (Event-driven)** | Low (<1%) | High (Polling loops) | Very Low | Variable (Script polling) |
| **Memory Footprint** | **≤ 5 MB** | ~20–40 MB | ~10 MB | ~2 MB | ~15–30 MB |
| **Direct Battery Gauge** | **✅ Yes (HID Report 0x90)** | ✅ Yes | ❌ No | ❌ No | ❌ No |
| **Fn Key & Combos** | **✅ User-mode Fn layer** | ✅ Yes | ⚠️ Rigid | ⚠️ Limited | ⚠️ Flaky with modifiers |
| **ISO / Italian AltGr Fix** | **✅ Automatic (`0x21D`)** | ❌ No | ❌ No | ❌ No | ⚠️ Complex script required |
| **Price & License** | **Free & Open Source (MIT)** | Paid Subscription ($15+/yr) | Proprietary (Extract) | Free (Abandoned C++) | Free (GPL/MIT scripts) |
| **Tech Stack** | **Rust + Win32 + egui** | C++ / .NET | C / Driver Framework | C++ Driver | AutoHotkey v1/v2 |

---

## Core Philosophy

1. **Invisible & Lightweight (0% Idle CPU):** Purely event-driven Win32 messaging and hooks. No polling loops. Working set memory target is under 5 MB.
2. **Anti-Cheat Safe:** No kernel-mode driver (`.sys`), no driver signing issues, no DLL injection. Mappings run purely in user mode, with an instant global toggle to pause whenever you game.
3. **Resilient Hardware Lifecycle:** Gracefully handles sleep/resume, Bluetooth reconnects, and seamless switching between Lightning USB cable and Bluetooth Classic.
4. **Instant & Independent GUI:** Settings app (`magicbridge-settings`) is built with `egui` and only runs when opened. When closed, it takes zero memory.
5. **Privacy First:** 100% offline. No analytics, no telemetry, no network calls.

---

## Key Features

- **Customizable Fn Layer:** Use a comfortable modifier (default: Right Option / `VK_RMENU`) to access intuitive combinations:
  - `Fn + ← / →` &rarr; `Home / End`
  - `Fn + ↑ / ↓` &rarr; `Page Up / Page Down`
  - `Fn + Backspace` &rarr; `Forward Delete (⌦)`
  - `Fn + Enter` &rarr; `Insert`
  - `Fn + F1…F12` &rarr; Alternate media / function layer
- **Top Row Mode Switcher:** Toggle between **Media Keys first** and **Standard F-keys first** directly from the system tray menu or with a global hotkey.
- **One-Click Modifier Presets:** Instantly switch between **Windows Layout** (`Cmd` &rarr; `Alt`, `Option` &rarr; `Win`) and **Mac Layout** (native mapping), or customize modifier positions independently.
- **ISO / Italian Layout AltGr-Safe:** Automatically catches and suppresses the synthetic `LCtrl` (`scancode 0x21D`) generated by Windows when `AltGr` is remapped.
- **Missing Windows Keys:** Out-of-the-box defaults mapping `F13` &rarr; `Print Screen`, `F14` &rarr; `Scroll Lock`, and `F15` &rarr; `Pause`.
- **Custom Key & Macro Actions:** Map `F13`–`F19` and `Clear` (`VK_NUMLOCK`) to custom key combos, media actions, or program launches.
- **Accurate Battery Reporting:** Reads direct battery telemetry from Apple Vendor collection `0xFF00:0x0014` report `0x90` over both USB and Bluetooth, with live tray tooltip percentages and low-battery toast alerts.
- **On-Screen Display (OSD) HUD:** Press `Fn + Esc` to seamlessly flip between Media Keys and standard F-keys with a sleek, translucent floating HUD.
- **Per-Application Profile Auto-Switching:** Bind distinct keymaps and modifier layouts to specific games or software (e.g. IDEs vs. games); MacKeys automatically activates them when foreground focus changes.
- **Game Mode Auto-Pause:** Remappings temporarily pause whenever a fullscreen DirectX or Vulkan game is in the foreground, ensuring native anti-cheat and competitive gaming compatibility.
- **Named Profiles & Multi-Language Layouts:** Create and swap between named profiles with visual layout previews for `ANSI (US)`, `ISO (Italian)`, `ISO (UK)`, `ISO (German)`, `ISO (French)`, and custom layouts.

---

## 📥 Download & Installation

MacKeys requires **no administrator privileges**, installs in seconds, and provides multiple distribution channels:

### Option 1: Standalone Setup Installer (Recommended)
1. Download **[`MacKeys-Setup.exe`](https://github.com/DooperDrew/mackeys-win/releases/latest)** from GitHub Releases.
2. Run the single-file setup wizard. The 5-step installer lets you:
   - Select your installation directory (defaults to `%LOCALAPPDATA%\Programs\MacKeys`).
   - Create Start Menu and Desktop shortcuts.
   - Configure automatic launch on Windows sign-in.
   - Automatically probes connected Apple hardware and pre-configures your layout.
   - Registers cleanly in Windows **Installed Apps** (Control Panel / Settings) with zero-leftover uninstallation.

*For headless / silent installations:*
```powershell
.\MacKeys-Setup.exe --silent
```

### Option 2: Portable ZIP (Zero Installation)
If you prefer not to install anything:
1. Download **`MacKeys-v0.4.1-portable.zip`** from [GitHub Releases](https://github.com/DooperDrew/mackeys-win/releases/latest).
2. Extract and run **`mackeys.exe`**.
3. Settings and daemon start immediately. Zero registry keys touched; perfect for running from a USB drive.

### Option 3: Microsoft WinGet
Install directly via the official Windows Package Manager:
```powershell
winget install DooperDrew.MacKeys
```

### Option 4: Scoop Package Manager
Install via Scoop:
```powershell
scoop install https://raw.githubusercontent.com/DooperDrew/mackeys-win/main/packaging/scoop/mackeys.json
```

---

## Architecture Overview

MacKeys is engineered as a clean Rust Cargo workspace separating pure business logic from operating system hooks and UI:

```mermaid
flowchart LR
    KB["Apple Magic Keyboard A1843<br/>(Lightning USB & Bluetooth)"] --> WIN["Windows HID Stack"]
    WIN -->|"Raw Input / HID Reports<br/>(Battery & Telemetry)"| AGENT
    WIN -->|"WH_KEYBOARD_LL Hook<br/>(Precomputed Lookup)"| AGENT
    
    subgraph AGENT["magicbridge-agent.exe (Background Tray)"]
        HOOK["Input Hook<br/>(< 0.1ms latency)"] --> ENGINE["Mapping Engine<br/>(crates/core)"]
        HID["HID Reader<br/>(Battery 0x90)"] --> ENGINE
        ENGINE --> OUT["SendInput<br/>(Tagged dwExtraInfo)"]
        TRAY["System Tray & Menu"]
        IPC["Named Pipe Server"]
    end
    
    GUI["magicbridge-settings.exe<br/>(egui UI — only when open)"] -->|"Atomic Write"| CFG[("config.toml")]
    GUI <-->|"Named Pipe IPC"| IPC
    CFG --> IPC
```

### Workspace Structure

| Crate | Package Name | Responsibility |
|---|---|---|
| [`crates/core`](crates/core) | `magicbridge-core` | Pure business logic, key models, layout compilation, config schema. `#![forbid(unsafe_code)]`, zero Win32 dependencies, 100% unit tested. |
| [`crates/agent`](crates/agent) | `magicbridge-agent` | Minimal background daemon: low-level keyboard hook, HID battery reader, tray icon, IPC server, autostart. |
| [`crates/settings`](crates/settings) | `magicbridge-settings` | Fast, lightweight UI built with `eframe`/`egui`. Interactive visual keyboard, profile manager, and i18n support. |
| [`crates/hid-probe`](crates/hid-probe) | `hid-probe` | Milestone 0 diagnostic utility for inspecting HID descriptors, timing Raw Input vs. hooks, and probing Apple battery reports. |

---

## Performance & Safety Budgets

Every release is tested against strict non-functional constraints:

| Metric | Target | Design Guarantee |
|---|---|---|
| **Idle CPU** | **0.0%** | Fully event-driven Win32 event pump. No polling loops. |
| **Agent Memory** | **≤ 5 MB** | Zero unnecessary runtime dependencies or heavyweight frameworks. |
| **Hook Latency** | **< 0.1 ms** | Hook runs precomputed array lookups. No memory allocations, disk I/O, or thread locks on the hook thread. |
| **Game & Anti-Cheat Safety** | **100% Safe** | No kernel driver (`.sys`) and no DLL injection into other processes. Pause hotkey available. |
| **Settings Footprint** | **0 MB (idle)** | Settings window is an independent executable that terminates completely upon close. |

---

## Building & Running

### Prerequisites

- **OS:** Windows 10 or 11 (64-bit)
- **Rust:** Stable toolchain (`x86_64-pc-windows-msvc`) via [rustup](https://rustup.rs/)
- **Build Tools:** Visual Studio C++ Build Tools (MSVC linker)

### Developer Commands

Run from the repository root:

```powershell
# Check workspace formatting
cargo fmt --all -- --check

# Run linter
cargo clippy --workspace --all-targets -- -D warnings

# Execute test suite
cargo test --workspace

# Build optimized release binaries
cargo build --workspace --release

# Run the agent in debug mode (console enabled)
cargo run -p magicbridge-agent

# Run the settings GUI
cargo run -p magicbridge-settings

# Package portable release archive (.zip)
.\scripts\package-release.ps1
```

Compiled binaries will be located in `target/release/` (`magicbridge-agent.exe` and `magicbridge-settings.exe`).

---

## Roadmap & Milestones

Progress is tracked via GitHub Milestones and Issues:

- [x] **[M0: Hardware Spike 🔬](https://github.com/DooperDrew/mackeys-win/issues/1)** — Verified USB/BT descriptors, battery report `0x90`, timing benchmarks, and the `AltGr` scancode `0x21D` quirk. See [docs/hardware-findings.md](docs/hardware-findings.md).
- [x] **[M1: Agent Skeleton](https://github.com/DooperDrew/mackeys-win/issues/2)** — Single-instance tray app, config schema, autostart, and device lifecycle handling.
- [x] **[M2: Remapping Engine](https://github.com/DooperDrew/mackeys-win/issues/3)** — Zero-allocation `WH_KEYBOARD_LL` hook, atomic table swapping, modifier presets, and self-injection prevention.
- [x] **[M3: Fn Layer & Top Row](https://github.com/DooperDrew/mackeys-win/issues/4)** — Configurable Fn trigger key, Fn combos, and Media vs. F-keys toggle.
- [x] **[M4: Custom Actions & Battery](https://github.com/DooperDrew/mackeys-win/issues/5)** — Off-thread macro execution and background battery monitoring.
- [x] **[M5: Settings GUI](https://github.com/DooperDrew/mackeys-win/issues/8)** — Visual A1843 layout editor, multi-language presets, and profile manager.
- [x] **[M6: Hardening & Performance](https://github.com/DooperDrew/mackeys-win/issues/6)** — Soak testing, sleep/resume validation, and strict latency measurement.
- [x] **[M7: Packaging & Release](https://github.com/DooperDrew/mackeys-win/issues/7)** — Standalone portable zip and installer.
- [x] **[M8: Multi-Agent Methodology & Architecture](https://github.com/DooperDrew/mackeys-win/issues/9)** — Didactic multi-agent parallelization documentation, Settings architecture tab, and GitHub Release v0.1.0 parity. See [docs/multi-agent-methodology.md](docs/multi-agent-methodology.md).
- [ ] **M9: Quality Assurance & Empirical Validation** — Physical A1843 hardware matrix execution, sleep/resume, game mode auto-pause, and security verification. See [docs/06-qa-and-validation.md](docs/06-qa-and-validation.md).
- [ ] **M10: User Experience & Onboarding Improvements** — Guided installer wizard, first-run onboarding configuration wizard, responsive Settings layout, and live battery tray indicator. See [docs/07-ux-and-improvements.md](docs/07-ux-and-improvements.md).
- [x] **[M11: GitHub Copilot Agent Integration](https://github.com/DooperDrew/mackeys-win/issues/16)** — Cloud multi-agent collaborative workflows, permanent repo instructions (`.github/copilot-instructions.md`), and automated agent task issue templates. See [docs/multi-agent-methodology.md](docs/multi-agent-methodology.md).

---

## 🔍 Frequently Asked Questions (FAQ)

<details>
<summary><b>Why doesn't the physical Fn key work on Windows out-of-the-box?</b></summary>
Windows' inbox kernel driver (<code>kbdhid.sys</code>) intercepts the physical Fn key and treats it as an internal hardware modifier; it is never forwarded to user-mode software. MacKeys solves this elegantly without risky kernel drivers by providing a user-configurable Fn trigger key (Right Option / AltGr by default, or Right Control, or F19), giving you full access to Home, End, PageUp, PageDown, Forward Delete, and media controls.
</details>

<details>
<summary><b>Is MacKeys safe with anti-cheat software in online multiplayer games?</b></summary>
Yes, 100%. Unlike Boot Camp driver hacks or commercial utilities that install unsigned or third-party kernel filter drivers (<code>.sys</code>) that can trigger anti-cheat bans (Vanguard, Easy Anti-Cheat, BattlEye), MacKeys operates purely in standard Windows user mode. Furthermore, Game Mode automatically pauses all remappings whenever a fullscreen DirectX or Vulkan game is launched.
</details>

<details>
<summary><b>How does MacKeys read the battery level without Apple drivers?</b></summary>
MacKeys reads battery telemetry directly from the Apple Vendor HID Collection (Usage Page <code>0xFF00</code>, Usage <code>0x0014</code>, Report ID <code>0x90</code>). Both USB Lightning connection and Bluetooth Classic report exact real-time percentages and charging state.
</details>

<details>
<summary><b>Can I swap Command and Alt to match standard Windows keyboards?</b></summary>
Yes. In the Settings UI or System Tray, choose the <b>Windows Layout</b> preset. This maps physical Command to Control/Alt and physical Option to Windows Key, giving you the natural physical feel of a standard PC keyboard.
</details>

<details>
<summary><b>Which Apple keyboard models are supported?</b></summary>
MacKeys supports the entire desktop family:
<ul>
  <li><b>A1843</b>: Magic Keyboard with Numeric Keypad (Lightning/Bluetooth)</li>
  <li><b>A2520 / A3123</b>: Magic Keyboard with Touch ID and Numeric Keypad</li>
  <li><b>A1644</b>: Magic Keyboard Compact (TKL)</li>
  <li><b>A2449 / A2450 / A3122</b>: Magic Keyboard Compact with Touch ID</li>
  <li><b>A1314</b>: Apple Wireless Keyboard (AA battery Bluetooth)</li>
  <li><b>A1243</b>: Apple Keyboard with Numeric Keypad (Wired USB)</li>
</ul>
</details>

---

## Contributing & AI Agent Development

We welcome community feedback, bug reports, and pull requests! Please check out [CONTRIBUTING.md](CONTRIBUTING.md) for code conventions, safety requirements, and testing guidelines.

MacKeys is built using an **AI-Agent native architecture**. GitHub Copilot and autonomous coding agents follow strict performance and safety guardrails documented in [`.github/copilot-instructions.md`](.github/copilot-instructions.md). New features and bounded tasks can be directly prepared for agents using the [🤖 Agent Task Issue Template](.github/ISSUE_TEMPLATE/agent_task.md).

## License

This project is licensed under the [MIT License](LICENSE).
