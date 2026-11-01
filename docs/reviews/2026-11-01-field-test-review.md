# Field-Test Review — 2026-11-01

> Source evidence: owner screenshots (Setup Wizard step 4, Settings › Keyboard tab), owner hardware QA export
> `qa-hardware-report-20261101-165016.txt` (A1843, 250 s, 103/109), live `%APPDATA%\MagicBridge\config.toml`,
> running daemon (`magicbridge.exe --daemon`, PID 79640), source audit at commit `dccdf84`.
>
> Legend: ✅ verified from evidence · 🔬 suspected, needs a probe/owner confirmation before fixing.
> Severity: **S1** broken/misleading core behaviour · **S2** significant UX/integrity defect · **S3** polish / docs.

## A. Functional defects

| ID | Sev | Finding | Evidence |
|---|---|---|---|
| A1 | S1 🔬 | **Top-row Media mode not applied at runtime.** Profile has `top_row_mode = "media_keys_first"` and the daemon was running, yet F7–F12 reached the app as plain `F7..F12`. | QA log entries 39–50 (`F7 0x76` … `F12 0x7B`) while daemon PID 79640 was alive (started 17:44, QA at 17:46–17:50). Engine unit test passes → suspect hook/table not active, not the engine. |
| A2 | S1 ✅ | **`Clear` toggles Num Lock, silently turning the numpad into navigation keys.** After pressing Clear, numpad 4/5/6/1/2/3 produced `Left`, `5`, `Right`, `End`, `Down`, `PgDn`. Mac users never expect this. | QA log entries 221–239 (+154 s…+158 s) right after `NumLock/Clear` at +130 s. `hardware-findings.md` confirms Clear = `VK_NUMLOCK`. |
| A3 | S1 ✅ | **M18 Magic Mouse smooth scrolling is not wired.** `InertialScroller` and `send_mouse_wheel()` are only referenced by unit tests; no mouse hook / Raw Input feeds them. The three Magic Mouse settings are dead controls. | `Select-String InertialScroll|send_mouse_wheel` → only `injector.rs` + tests. |
| A4 | S2 🔬 | **Setup Wizard reports "Keyboard not connected yet"** although the owner was using the A1843. | Screenshot 1. `detect_connected_apple_model()` only probes HID paths with a parsable PID; BT-LE / different collection paths may fail. Need: transport used at that moment. |
| A5 | S2 ✅ | **Corner-key action offered on a model with no corner key.** A1843 has no Lock/Touch ID/Eject key, but `corner_key_action = "lock_workstation"` is set and editable. | `config.toml` line 26; M14.2 design. |
| A6 | S3 ✅ | Wizard copy promises "Media controls (Play/Pause, Volume, **Brightness**)" but only F3/F4/F7–F12 are mapped; brightness is not implemented. | Screenshot 1; `engine.rs` top-row block. |

## B. Hardware QA tool integrity (Settings › Hardware QA)

The QA tab identifies keys from **egui key/text events + `GetAsyncKeyState` polling**, i.e. *after* remapping and *after* layout translation. It does not see physical keys.

| ID | Sev | Finding | Evidence |
|---|---|---|---|
| B1 | S1 ✅ | **Remapped keys can never be verified**: F13/F14/F15 (→ PrtSc/ScrLk/Pause), Right ⌘ (→ Right Alt under Windows preset — owner pressed it 34× and got `RightOption`). | "Unverified keys" list; log entries 1–34. |
| B2 | S1 ✅ | **False positives inflate the score**: numpad `3` also marks main-row `3`; numpad `.` marks `Period`; numpad `/` marks `Slash`; `*` marks `NumAdd`; numpad-as-navigation (A2) marks the Navigation cluster. The 94.5 % is not trustworthy. | Paired entries at identical timestamps (e.g. #72/#71, #66/#65, #286/#285, #277/#276). |
| B3 | S1 ✅ | **Scancode / Extended columns are fabricated** from `default_scancode_and_extended(vk)`, not measured. Violates *claims follow evidence*. | `qa_view.rs` ~L393, L427, L846. |
| B4 | S2 ✅ | Numpad Enter collapses into main Enter (no extended flag); Numpad `=` collides with main `=` (both VK `0xBB`) → both permanently unverifiable/ambiguous. | Unverified list; entries 52–54. |
| B5 | S2 ✅ | Log flooded by poll/auto-repeat duplicates (Num7 ×9, RightOption ×34); "Key Name" = `Unknown` for all numpad & punctuation keys. | Log body. |
| B6 | S3 ✅ | Category label "Function Row (F1..F19)" counts 20 keys (includes Esc). Fn logged as `VK 0x00` although Fn is invisible to Windows — origin unclear 🔬. | Report header; entry 226. |

**Proposed direction:** QA tab consumes *raw pre-remap* events — either from the daemon hook over IPC (`vk`, `scanCode`, `LLKHF_EXTENDED`, before table lookup) or via Raw Input (`RIDEV_INPUTSINK`) in the settings process, which also yields the device handle so we can prove the event came from the Apple keyboard. Key identity = (scancode, extended), not VK/text.

## C. UI / UX (screenshots)

| ID | Sev | Finding |
|---|---|---|
| C1 | S1 ✅ | **Tofu glyphs (□)**: all tab-bar icons except ⓘ, and the four arrow keys, render as empty squares. No font fallback is installed (no `FontDefinitions` in `crates/settings`). Navigation is unreadable — only the active tab shows a text label. |
| C2 | S2 ✅ | **Keyboard canvas clipped on the right** at the default ~860 px width (numpad column/Enter cut) even with *Fit* selected. |
| C3 | S2 ✅ | Toolbar legend wraps: "Selected" is orphaned on the next line, separated from its swatch; toolbar has 2 crowded rows (profile/layout + layer/presets/scale/glow/legend). |
| C4 | S2 ✅ | Key Editor takes the lower half with a permanent "How Customization Works" tutorial; right half empty. Should be a side panel or collapsible / dismissible help. |
| C5 | S2 ✅ | **No application icon**: title bar/taskbar show the default eframe "e" (both Setup Wizard and Settings); no `.ico` resource in the exe. |
| C6 | S2 ✅ | Setup Wizard step 4: no **Back** button, large empty area, three radio options crammed per line with long wrapping labels, Auto-Detect buttons inline with dropdowns, yellow "not connected" banner (see A4). |
| C7 | S3 ✅ | ISO-Italian triple legends crammed (`èé[`, `òç@`, `à°#`); `pg up`/`pg dn`/`clear` tiny. Needs 2-line legends (shift on top, base bottom, AltGr corner). |
| C8 | S3 ✅ | "⚡ Live Ready" looks disabled; meaning of *Live Glow* vs *Live Ready* unclear. |
| C9 | S2 ✅ | **Autonomous `ui-qa-tester` reported 0 defects** while C1–C3 were visible → QA blind spots: no tofu/missing-glyph check, no check at the real default window size. |

## D. Performance / NFR

| ID | Sev | Finding |
|---|---|---|
| D1 | S2 🔬 | Daemon working set **15.1 MB** idle vs NFR-P1 ≤ 10 MB (target ≤ 5 MB). Single sample; the unified exe now links egui/wgpu into the daemon process. |
| D2 | S3 ✅ | Unified `magicbridge.exe` is 3.8 MB vs NFR-P4 "agent binary ≤ 3 MB" — requirement predates the single-binary decision and must be restated. |

## E. Docs, versioning & governance

| ID | Sev | Finding |
|---|---|---|
| E1 | S1 ✅ | **Versioning not respected**: M14–M18 shipped under `0.4.0`; no tag/release after v0.4.0; installed binary (00:19) predates the final M15–M18 commit (00:25). |
| E2 | S2 ✅ | **Roadmap/CHANGELOG over-claims**: M15.2 "native Windows 10/11 Toast" is a `Shell_NotifyIcon` balloon; "20 % warning" vs default threshold 15 %; M17.3 "automated WinGet submission" — `release.yml` has no WinGet step; M18.3 "integration" not done (A3). Milestones M15–M18 were closed on GitHub. |
| E3 | S2 ✅ | README stale: install steps reference `MagicBridge-v0.2.1-windows-x64.zip` and the `0.2.1` WinGet manifest path; actual asset is `MagicBridge-v0.4.0-portable.zip`; no mention of OSD, Fn+Esc, per-app profiles, Magic Mouse. |
| E4 | S3 ✅ | `scripts/verify-gate.ps1` gained npm steps (commits `67d021e`, `dccdf84`) in a Rust-only repo — harmless (guarded by `package.json` absence) but noise. |
| E5 | S3 ✅ | `06-qa-and-validation.md` has no rows for M15–M18 features (OSD, Fn+Esc, per-app switching, battery toasts). |

## Proposed milestones (for owner triage)

| Milestone | Version | Scope |
|---|---|---|
| **M19 — Field-Test Fixes & Honest QA** | v0.4.1 | A1, A2, A4, A5, A6, B1–B6, E1, E2 |
| **M20 — Visual Polish & Readability** | v0.5.0 | C1–C9, D1, D2 |
| **M21 — Docs & Release Sync** | with v0.4.1 | E3, E4, E5 |
| **M22 — Magic Mouse Integration (real)** | v0.6.0 | A3 (needs a Magic Mouse owner for 🔬 hardware QA) |
