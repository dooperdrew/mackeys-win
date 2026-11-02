//! Interactive step-by-step Hardware QA Key Verification Tool for Apple Magic Keyboard (A1843).
//!
//! Complies with Issue #10 Prep, F7.2, F7.4, and AGENTS.md:
//! - Visual matrix of all 109 physical keys on A1843.
//! - Captures window key events and Win32 modifier states.
//! - Turns verified keys bright green.
//! - Progress bar and category breakdowns (Modifiers, Nav, Number row, Function row, Numpad, Alphanumeric).
//! - Reset test and Export QA report to Desktop or AppData.

use std::collections::{HashSet, VecDeque};
use std::fs;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use egui::{
    Color32, CornerRadius, ProgressBar, Rect, RichText, ScrollArea, Sense, Stroke, Ui, Vec2,
};
use magicbridge_core::key::*;

use crate::i18n::I18n;
use crate::keyboard_model::{A1843Model, KeyCluster, PhysicalKey};

/// Category classification for A1843 hardware keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QaCategory {
    /// Modifiers: Command, Option, Control, Shift (Left and Right).
    Modifiers,
    /// Function Row: Escape, F1..F12, F13..F15, F16..F19.
    FunctionRow,
    /// Number Row: Grave (~), 1..0, Minus, Equals, Backspace.
    NumberRow,
    /// Navigation Cluster: Fn, Home, End, PageUp, PageDown, Delete, Arrows.
    Navigation,
    /// Numeric Keypad: Clear, =, /, *, 0..9, +, -, Enter, Decimal.
    Numpad,
    /// Alphanumeric Typing Block: Letters A-Z, punctuation, Tab, CapsLock, Enter, Space.
    Alphanumeric,
}

impl QaCategory {
    /// Returns all QA categories in logical display order.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Modifiers,
            Self::FunctionRow,
            Self::NumberRow,
            Self::Navigation,
            Self::Numpad,
            Self::Alphanumeric,
        ]
    }

    /// Translation key for the category name.
    #[must_use]
    pub const fn label_key(&self) -> &'static str {
        match self {
            Self::Modifiers => "qa.cat_modifiers",
            Self::FunctionRow => "qa.cat_function_row",
            Self::NumberRow => "qa.cat_number_row",
            Self::Navigation => "qa.cat_navigation",
            Self::Numpad => "qa.cat_numpad",
            Self::Alphanumeric => "qa.cat_alphanumeric",
        }
    }
}

/// Categorizes a physical key on the A1843 keyboard.
#[must_use]
pub fn categorize_key(key: &PhysicalKey) -> QaCategory {
    match key.id {
        // Modifiers (Cmd, Opt, Ctrl, Shift)
        "LeftShift" | "RightShift" | "LeftControl" | "RightControl" | "LeftOption"
        | "RightOption" | "LeftCommand" | "RightCommand" => QaCategory::Modifiers,

        // Function Row: Esc + F1..F19
        "Escape" | "F1" | "F2" | "F3" | "F4" | "F5" | "F6" | "F7" | "F8" | "F9" | "F10" | "F11"
        | "F12" | "F13" | "F14" | "F15" | "F16" | "F17" | "F18" | "F19" => QaCategory::FunctionRow,

        // Number Row: Grave, 1..0, Minus, Equals, Backspace
        "Grave" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "0" | "Minus"
        | "Equals" | "Backspace" => QaCategory::NumberRow,

        // Navigation cluster: Fn, Home/End, PgUp/PgDn, Delete, Arrows
        "Fn" | "Home" | "PageUp" | "Delete" | "End" | "PageDown" | "Up" | "Left" | "Down"
        | "Right" => QaCategory::Navigation,

        // Numpad: Clear, arithmetic, digits, Enter, Decimal
        "Clear" | "NumEquals" | "NumDivide" | "NumMultiply" | "Num7" | "Num8" | "Num9"
        | "NumSubtract" | "Num4" | "Num5" | "Num6" | "NumAdd" | "Num1" | "Num2" | "Num3"
        | "NumEnter" | "Num0" | "NumDecimal" => QaCategory::Numpad,

        // Main alphanumeric block
        _ => QaCategory::Alphanumeric,
    }
}

/// A recorded keypress event during QA verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QaKeyEvent {
    /// Unique key ID string from `A1843Model`.
    pub key_id: String,
    /// Canonical key name.
    pub key_name: String,
    /// Windows Virtual Key code.
    pub vk: u16,
    /// Hardware scancode.
    pub scancode: u16,
    /// Extended key flag.
    pub extended: bool,
    /// Milliseconds since the start of this QA test session.
    pub elapsed_ms: u128,
    /// User-friendly relative time string (e.g. "+1.42s").
    pub formatted_time: String,
    /// Category classification.
    pub category: QaCategory,
}

/// State for the interactive Hardware QA Key Verification view.
#[derive(Debug, Clone)]
pub struct HardwareQaViewState {
    /// Set of physical key IDs that have been verified during this session.
    pub tested_keys: HashSet<String>,
    /// Total key count on the physical keyboard (109 keys on A1843).
    pub total_keys: usize,
    /// Timestamp when this QA test session started.
    pub test_started: Instant,
    /// Event log of recently pressed keys in reverse chronological order (newest first).
    pub event_log: VecDeque<QaKeyEvent>,
    /// Whether the key listener is currently active.
    pub is_listening: bool,
    /// Keyboard geometry and key definitions.
    pub model: A1843Model,
    /// Status of the last export attempt: `(is_error, message)`.
    pub export_status: Option<(bool, String)>,
    /// Active category filter for highlighting (None = show all).
    pub selected_category: Option<QaCategory>,
    /// Last verified key ID and timestamp for deduplicating fast hardware auto-repeat.
    pub last_key: Option<(String, Instant)>,
    /// Whether daemon remapping was paused during this active QA session.
    pub was_paused_by_qa: bool,
}

impl Default for HardwareQaViewState {
    fn default() -> Self {
        Self::new()
    }
}

impl HardwareQaViewState {
    /// Maximum number of events kept in the in-memory recent event log.
    pub const MAX_LOG_EVENTS: usize = 300;

    /// Creates and initializes a new Hardware QA view state.
    #[must_use]
    pub fn new() -> Self {
        let model = A1843Model::new();
        let total_keys = model.keys.len();

        Self {
            tested_keys: HashSet::with_capacity(total_keys),
            total_keys,
            test_started: Instant::now(),
            event_log: VecDeque::with_capacity(Self::MAX_LOG_EVENTS),
            is_listening: true,
            model,
            export_status: None,
            selected_category: None,
            last_key: None,
            was_paused_by_qa: false,
        }
    }

    /// Resets the QA test session, clearing verified keys and event log.
    pub fn reset(&mut self) {
        self.tested_keys.clear();
        self.event_log.clear();
        self.test_started = Instant::now();
        self.export_status = None;
        self.selected_category = None;
        self.last_key = None;
        self.was_paused_by_qa = false;
    }

    /// Marks a physical key as verified and logs the keypress event.
    pub fn mark_key_verified(&mut self, key_id: &str, vk: u16, scancode: u16, extended: bool) {
        let key_id_owned = key_id.to_string();
        self.tested_keys.insert(key_id_owned.clone());

        let elapsed = self.test_started.elapsed();
        let elapsed_ms = elapsed.as_millis();
        let formatted_time = format!("+{:.2}s", elapsed.as_secs_f32());

        let category = self
            .model
            .find_by_id(key_id)
            .map_or(QaCategory::Alphanumeric, categorize_key);

        let key_name = if vk != 0 {
            vk_name(vk).to_string()
        } else {
            key_id.to_string()
        };

        let event = QaKeyEvent {
            key_id: key_id_owned,
            key_name,
            vk,
            scancode,
            extended,
            elapsed_ms,
            formatted_time,
            category,
        };

        self.event_log.push_front(event);
        if self.event_log.len() > Self::MAX_LOG_EVENTS {
            self.event_log.pop_back();
        }
    }

    /// Returns the verified key count and completion percentage.
    #[must_use]
    pub fn progress(&self) -> (usize, usize, f32) {
        let verified = self.tested_keys.len();
        let total = self.total_keys;
        let percentage = if total > 0 {
            (verified as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        (verified, total, percentage)
    }

    /// Returns the count of verified keys within a specific category.
    #[must_use]
    pub fn category_progress(&self, category: QaCategory) -> (usize, usize) {
        let total = self
            .model
            .keys
            .iter()
            .filter(|k| categorize_key(k) == category)
            .count();
        let verified = self
            .model
            .keys
            .iter()
            .filter(|k| categorize_key(k) == category && self.tested_keys.contains(k.id))
            .count();
        (verified, total)
    }
}

/// Matches an `egui::Key` (and optional physical key fallback) to an A1843 physical key ID.
#[must_use]
pub fn map_egui_key_to_a1843_id(
    key: egui::Key,
    physical: Option<egui::Key>,
) -> Option<&'static str> {
    use egui::Key as K;

    let id = match key {
        K::Escape => "Escape",
        K::F1 => "F1",
        K::F2 => "F2",
        K::F3 => "F3",
        K::F4 => "F4",
        K::F5 => "F5",
        K::F6 => "F6",
        K::F7 => "F7",
        K::F8 => "F8",
        K::F9 => "F9",
        K::F10 => "F10",
        K::F11 => "F11",
        K::F12 => "F12",
        K::F13 => "F13",
        K::F14 => "F14",
        K::F15 => "F15",
        K::F16 => "F16",
        K::F17 => "F17",
        K::F18 => "F18",
        K::F19 => "F19",
        K::Tab => "Tab",
        K::Backspace => "Backspace",
        K::Enter => "Enter",
        K::Space => "Space",
        K::Insert => "Fn",
        K::Delete => "Delete",
        K::Home => "Home",
        K::End => "End",
        K::PageUp => "PageUp",
        K::PageDown => "PageDown",
        K::ArrowLeft => "Left",
        K::ArrowRight => "Right",
        K::ArrowUp => "Up",
        K::ArrowDown => "Down",
        K::A => "A",
        K::B => "B",
        K::C => "C",
        K::D => "D",
        K::E => "E",
        K::F => "F",
        K::G => "G",
        K::H => "H",
        K::I => "I",
        K::J => "J",
        K::K => "K",
        K::L => "L",
        K::M => "M",
        K::N => "N",
        K::O => "O",
        K::P => "P",
        K::Q => "Q",
        K::R => "R",
        K::S => "S",
        K::T => "T",
        K::U => "U",
        K::V => "V",
        K::W => "W",
        K::X => "X",
        K::Y => "Y",
        K::Z => "Z",
        K::Num0 => "0",
        K::Num1 => "1",
        K::Num2 => "2",
        K::Num3 => "3",
        K::Num4 => "4",
        K::Num5 => "5",
        K::Num6 => "6",
        K::Num7 => "7",
        K::Num8 => "8",
        K::Num9 => "9",
        K::Backtick => "Grave",
        K::Minus => "Minus",
        K::Equals => "Equals",
        K::OpenBracket => "LBracket",
        K::CloseBracket => "RBracket",
        K::Backslash => "Backslash",
        K::Semicolon => "Semicolon",
        K::Quote => "Quote",
        K::Comma => "Comma",
        K::Period => "Period",
        K::Slash => "Slash",
        K::Plus => "NumAdd",
        _ => {
            if let Some(phys) = physical {
                return map_egui_key_to_a1843_id(phys, None);
            }
            return None;
        }
    };

    Some(id)
}

/// Matches input text characters (e.g. on ISO keyboards) to physical key IDs.
#[must_use]
pub fn map_text_to_a1843_id(text: &str) -> Option<&'static str> {
    match text {
        "<" | ">" => Some("IsoLessGreater"),
        "\\" | "|" => Some("Grave"),
        "à" | "°" | "#" => Some("Quote"),
        "è" | "é" | "[" | "{" => Some("LBracket"),
        "ò" | "ç" | "@" => Some("Semicolon"),
        "ù" | "§" => Some("RBracket"),
        "+" | "*" => Some("NumAdd"),
        "-" | "_" => Some("Minus"),
        "=" => Some("Equals"),
        _ => None,
    }
}

/// Matches a raw physical hardware event (VK, scancode, extended) to an A1843 physical key ID.
#[must_use]
pub fn map_raw_event_to_a1843_id(vk: u16, scancode: u16, extended: bool) -> Option<&'static str> {
    match vk {
        VK_ESCAPE => Some("Escape"),
        VK_F1 => Some("F1"),
        VK_F2 => Some("F2"),
        VK_F3 => Some("F3"),
        VK_F4 => Some("F4"),
        VK_F5 => Some("F5"),
        VK_F6 => Some("F6"),
        VK_F7 => Some("F7"),
        VK_F8 => Some("F8"),
        VK_F9 => Some("F9"),
        VK_F10 => Some("F10"),
        VK_F11 => Some("F11"),
        VK_F12 => Some("F12"),
        VK_F13 => Some("F13"),
        VK_F14 => Some("F14"),
        VK_F15 => Some("F15"),
        VK_F16 => Some("F16"),
        VK_F17 => Some("F17"),
        VK_F18 => Some("F18"),
        VK_F19 => Some("F19"),

        // Return / Enter: distinguish main Enter (extended=0) vs Numpad Enter (extended=1)
        VK_RETURN => {
            if extended {
                Some("NumEnter")
            } else {
                Some("Enter")
            }
        }

        // Modifiers
        VK_LSHIFT => Some("LeftShift"),
        VK_RSHIFT => Some("RightShift"),
        VK_LCONTROL => {
            // ISO Italian AltGr quirk: scancode 0x21D is synthetic Left Control injected before Right Option.
            if scancode == 0x21D {
                None
            } else {
                Some("LeftControl")
            }
        }
        VK_RCONTROL => Some("RightControl"),
        VK_LMENU => Some("LeftOption"),
        VK_RMENU => Some("RightOption"),
        VK_LWIN => Some("LeftCommand"),
        VK_RWIN => Some("RightCommand"),
        VK_CAPITAL => Some("CapsLock"),

        // Navigation cluster
        VK_DELETE => Some("Delete"),
        VK_HOME => Some("Home"),
        VK_END => Some("End"),
        VK_PRIOR => Some("PageUp"),
        VK_NEXT => Some("PageDown"),
        VK_LEFT => Some("Left"),
        VK_RIGHT => Some("Right"),
        VK_UP => Some("Up"),
        VK_DOWN => Some("Down"),

        // Editing
        VK_BACK => Some("Backspace"),
        VK_TAB => Some("Tab"),
        VK_SPACE => Some("Space"),

        // Numpad cluster
        VK_NUMLOCK => Some("Clear"),
        VK_NUMPAD0 => Some("Num0"),
        VK_NUMPAD1 => Some("Num1"),
        VK_NUMPAD2 => Some("Num2"),
        VK_NUMPAD3 => Some("Num3"),
        VK_NUMPAD4 => Some("Num4"),
        VK_NUMPAD5 => Some("Num5"),
        VK_NUMPAD6 => Some("Num6"),
        VK_NUMPAD7 => Some("Num7"),
        VK_NUMPAD8 => Some("Num8"),
        VK_NUMPAD9 => Some("Num9"),
        VK_DECIMAL => Some("NumDecimal"),
        VK_ADD => Some("NumAdd"),
        VK_SUBTRACT => Some("NumSubtract"),
        VK_MULTIPLY => Some("NumMultiply"),
        VK_DIVIDE => Some("NumDivide"),

        // Letters A..Z
        0x41 => Some("A"),
        0x42 => Some("B"),
        0x43 => Some("C"),
        0x44 => Some("D"),
        0x45 => Some("E"),
        0x46 => Some("F"),
        0x47 => Some("G"),
        0x48 => Some("H"),
        0x49 => Some("I"),
        0x4A => Some("J"),
        0x4B => Some("K"),
        0x4C => Some("L"),
        0x4D => Some("M"),
        0x4E => Some("N"),
        0x4F => Some("O"),
        0x50 => Some("P"),
        0x51 => Some("Q"),
        0x52 => Some("R"),
        0x53 => Some("S"),
        0x54 => Some("T"),
        0x55 => Some("U"),
        0x56 => Some("V"),
        0x57 => Some("W"),
        0x58 => Some("X"),
        0x59 => Some("Y"),
        0x5A => Some("Z"),

        // Digits 0..9 on main number row
        0x30 => Some("0"),
        0x31 => Some("1"),
        0x32 => Some("2"),
        0x33 => Some("3"),
        0x34 => Some("4"),
        0x35 => Some("5"),
        0x36 => Some("6"),
        0x37 => Some("7"),
        0x38 => Some("8"),
        0x39 => Some("9"),

        // Punctuation & OEM keys
        VK_OEM_1 => Some("Semicolon"),
        VK_OEM_PLUS => {
            // A1843 Numpad Equals: scancode 0x59 on Apple keyboards
            if scancode == 0x59 {
                Some("NumEquals")
            } else {
                Some("Equals")
            }
        }
        VK_OEM_COMMA => Some("Comma"),
        VK_OEM_MINUS => Some("Minus"),
        VK_OEM_PERIOD => Some("Period"),
        VK_OEM_2 => Some("Slash"),
        VK_OEM_3 => Some("Grave"),
        VK_OEM_4 => Some("LBracket"),
        VK_OEM_5 => Some("Backslash"),
        VK_OEM_6 => Some("RBracket"),
        VK_OEM_7 => Some("Quote"),
        VK_OEM_102 => Some("IsoLessGreater"),

        // Numpad equals fallback (some drivers report 0x0C)
        0x0C => Some("NumEquals"),

        _ => {
            // Fallback by scancode if VK differs by layout
            match scancode {
                0x29 => Some("Grave"),
                0x0C => Some("Minus"),
                0x0D => Some("Equals"),
                0x59 => Some("NumEquals"),
                0x1A => Some("LBracket"),
                0x1B => Some("RBracket"),
                0x2B => Some("Backslash"),
                0x27 => Some("Semicolon"),
                0x28 => Some("Quote"),
                0x33 => Some("Comma"),
                0x34 => Some("Period"),
                0x35 => {
                    if extended {
                        Some("NumDivide")
                    } else {
                        Some("Slash")
                    }
                }
                0x56 => Some("IsoLessGreater"),
                _ => None,
            }
        }
    }
}

/// Formats the current UTC timestamp string for reports (YYYYMMDD-HHMMSS).
#[must_use]
pub fn current_timestamp_string() -> String {
    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();

    let days = secs / 86400;
    let rem_secs = secs % 86400;
    let hours = rem_secs / 3600;
    let mins = (rem_secs % 3600) / 60;
    let s = rem_secs % 60;

    let mut year = 1970;
    let mut d = days;
    loop {
        let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_in_year = if leap { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        year += 1;
    }

    let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let days_in_months = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1;
    for &dim in &days_in_months {
        if d < dim {
            break;
        }
        d -= dim;
        month += 1;
    }
    let day = d + 1;

    format!("{year:04}{month:02}{day:02}-{hours:02}{mins:02}{s:02}")
}

/// Generates a comprehensive QA report and saves it to a specified directory.
pub fn export_qa_report_to_dir(
    state: &HardwareQaViewState,
    i18n: &I18n,
    target_dir: &std::path::Path,
) -> Result<PathBuf, String> {
    let ts = current_timestamp_string();
    let filename_txt = format!("qa-hardware-report-{ts}.txt");
    let filename_json = format!("qa-hardware-report-{ts}.json");

    if !target_dir.exists() {
        let _ = fs::create_dir_all(target_dir);
    }

    let report_path_txt = target_dir.join(filename_txt);
    let report_path_json = target_dir.join(filename_json);

    let (verified, total, pct) = state.progress();

    // 1. Generate text report
    let mut txt = String::new();
    txt.push_str("=================================================================\n");
    txt.push_str(" MAGICBRIDGE HARDWARE QA KEY VERIFICATION REPORT\n");
    txt.push_str("=================================================================\n\n");
    txt.push_str(&format!("Report Generated : {ts} (UTC)\n"));
    txt.push_str("Hardware Model   : Apple Magic Keyboard with Numeric Keypad (A1843)\n");
    txt.push_str(&format!(
        "Test Duration    : {:.2}s\n",
        state.test_started.elapsed().as_secs_f32()
    ));
    txt.push_str(&format!(
        "Verification Score: {verified} / {total} keys verified ({pct:.1}%)\n"
    ));
    txt.push_str(&format!(
        "Overall Status   : {}\n\n",
        if verified == total {
            "PASSED (100% COMPLETE)"
        } else {
            "IN PROGRESS"
        }
    ));

    txt.push_str("-----------------------------------------------------------------\n");
    txt.push_str(" CATEGORY BREAKDOWNS\n");
    txt.push_str("-----------------------------------------------------------------\n");
    for &cat in QaCategory::all() {
        let (v, t) = state.category_progress(cat);
        let cat_pct = if t > 0 {
            (v as f32 / t as f32) * 100.0
        } else {
            0.0
        };
        let status = if v == t { "[PASSED]" } else { "[PENDING]" };
        txt.push_str(&format!(
            "{:<35} : {:2} / {:2} keys ({:5.1}%) {}\n",
            i18n.t(cat.label_key()),
            v,
            t,
            cat_pct,
            status
        ));
    }
    txt.push('\n');

    // List unverified keys if any
    let unverified: Vec<_> = state
        .model
        .keys
        .iter()
        .filter(|k| !state.tested_keys.contains(k.id))
        .collect();

    if unverified.is_empty() {
        txt.push_str("All 109 keys successfully tested and verified.\n\n");
    } else {
        txt.push_str("-----------------------------------------------------------------\n");
        txt.push_str(&format!(" UNVERIFIED KEYS ({})\n", unverified.len()));
        txt.push_str("-----------------------------------------------------------------\n");
        for key in &unverified {
            let cat = categorize_key(key);
            txt.push_str(&format!(
                "- {:<16} (Default: {:<8}, VK: 0x{:02X}, Cluster: {:?}, Category: {:?})\n",
                key.id, key.default_label, key.vk, key.cluster, cat
            ));
        }
        txt.push('\n');
    }

    // Chronological event log
    txt.push_str("-----------------------------------------------------------------\n");
    txt.push_str(&format!(
        " EVENT LOG ({} recent entries)\n",
        state.event_log.len()
    ));
    txt.push_str("-----------------------------------------------------------------\n");
    txt.push_str(
        "Idx | Time    | Key Name        | ID              | VK Code | Scancode | Extended\n",
    );
    txt.push_str(
        "----+---------+-----------------+-----------------+---------+----------+---------\n",
    );
    for (i, ev) in state.event_log.iter().enumerate() {
        txt.push_str(&format!(
            "{:3} | {:<7} | {:<15} | {:<15} | 0x{:02X}    | 0x{:02X}     | {}\n",
            i + 1,
            ev.formatted_time,
            ev.key_name,
            ev.key_id,
            ev.vk,
            ev.scancode,
            ev.extended
        ));
    }

    fs::write(&report_path_txt, txt).map_err(|e| format!("Failed to write text report: {e}"))?;

    // 2. Generate JSON report for programmatic ingestion
    let json_events: Vec<serde_json::Value> = state
        .event_log
        .iter()
        .map(|ev| {
            serde_json::json!({
                "key_id": ev.key_id,
                "key_name": ev.key_name,
                "vk": ev.vk,
                "scancode": ev.scancode,
                "extended": ev.extended,
                "elapsed_ms": ev.elapsed_ms,
                "formatted_time": ev.formatted_time,
                "category": format!("{:?}", ev.category),
            })
        })
        .collect();

    let json_data = serde_json::json!({
        "timestamp": ts,
        "keyboard_model": "A1843",
        "verified_keys": verified,
        "total_keys": total,
        "completion_percentage": pct,
        "status": if verified == total { "PASSED" } else { "IN_PROGRESS" },
        "tested_keys": state.tested_keys.iter().collect::<Vec<_>>(),
        "unverified_keys": unverified.iter().map(|k| k.id).collect::<Vec<_>>(),
        "events": json_events,
    });

    if let Ok(json_str) = serde_json::to_string_pretty(&json_data) {
        let _ = fs::write(&report_path_json, json_str);
    }

    Ok(report_path_txt)
}

/// Generates a comprehensive QA report and saves it to Desktop or AppData.
pub fn export_qa_report(state: &HardwareQaViewState, i18n: &I18n) -> Result<PathBuf, String> {
    // Resolve target directory: Desktop -> AppData -> Current Dir
    let target_dir = std::env::var("USERPROFILE")
        .map(|p| PathBuf::from(p).join("Desktop"))
        .ok()
        .filter(|p| p.is_dir())
        .or_else(|| {
            std::env::var("APPDATA")
                .map(|p| PathBuf::from(p).join("MacKeys"))
                .ok()
        })
        .unwrap_or_else(|| PathBuf::from("."));

    export_qa_report_to_dir(state, i18n, &target_dir)
}

/// Renders the visual A1843 QA keyboard matrix.
fn render_qa_visual_keyboard(ui: &mut Ui, state: &mut HardwareQaViewState) {
    let available_width = ui.available_width().min(ui.max_rect().width());
    let total_units_x = match state.model.form_factor() {
        magicbridge_core::KeyboardModelType::FullSize => 23.5f32,
        magicbridge_core::KeyboardModelType::Compact => 15.0f32,
    };
    let unit_size = ((available_width - 8.0) / total_units_x).clamp(16.0, 34.0);
    let key_height = unit_size * 0.95;
    let key_spacing = 2.0;
    let total_matrix_width = (total_units_x * (unit_size + key_spacing)) + 8.0;
    let total_height = (6.0 * (key_height + key_spacing)) + 8.0;

    let (rect, _response) = ui.allocate_exact_size(
        Vec2::new(available_width.min(total_matrix_width), total_height),
        Sense::hover(),
    );

    let painter = ui.painter_at(rect);

    // Draw keyboard base plate
    painter.rect_filled(
        rect,
        CornerRadius::same(6),
        Color32::from_rgb(235, 237, 240),
    );

    let mut clicked_key: Option<PhysicalKey> = None;

    // Draw each physical key
    for key in &state.model.keys {
        let key_x = rect.left() + (key.x * (unit_size + key_spacing)) + 4.0;
        let key_y =
            rect.top() + ((key.row as f32 + key.y_offset) * (key_height + key_spacing)) + 4.0;
        let key_w = (key.width * unit_size) + ((key.width - 1.0) * key_spacing);
        let key_h = (key.height * key_height) + ((key.height - 1.0) * key_spacing);

        let key_rect = Rect::from_min_size(egui::pos2(key_x, key_y), Vec2::new(key_w, key_h));

        let id = ui.make_persistent_id(format!("qa_key_{}", key.id));
        let mut response = ui.interact(key_rect, id, Sense::click());
        let label = key.default_label;
        response
            .widget_info(move || egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
        response = response.on_hover_text(format!("Key: {} (VK: {:#04x})", label, key.vk));

        if response.clicked() {
            clicked_key = Some(key.clone());
        }

        let is_verified = state.tested_keys.contains(key.id);
        let key_cat = categorize_key(key);
        let is_in_selected_cat = state.selected_category.is_some_and(|cat| cat == key_cat);

        // Styling: Bright green for verified keys!
        let fill_color = if is_verified {
            Color32::from_rgb(46, 204, 113) // Bright verified green
        } else if is_in_selected_cat {
            Color32::from_rgb(255, 243, 205) // Highlighted category
        } else if response.hovered() {
            Color32::from_rgb(240, 244, 248)
        } else {
            Color32::WHITE
        };

        let stroke_color = if is_verified {
            Color32::from_rgb(25, 135, 65)
        } else if is_in_selected_cat {
            Color32::from_rgb(220, 140, 0)
        } else {
            Color32::from_rgb(195, 200, 205)
        };

        let stroke_width = if is_verified || is_in_selected_cat {
            1.5_f32
        } else {
            1.0_f32
        };

        painter.rect(
            key_rect,
            CornerRadius::same(3),
            fill_color,
            Stroke::new(stroke_width, stroke_color),
            egui::StrokeKind::Inside,
        );

        // Key label
        let text_color = if is_verified {
            Color32::WHITE
        } else {
            Color32::from_rgb(45, 50, 55)
        };

        let font_size = if key.width >= 1.5
            || key.cluster == KeyCluster::Main && key.default_label.len() <= 2
        {
            11.0
        } else {
            9.5
        };

        let label_text = if is_verified {
            format!("{}\n✓", key.default_label)
        } else {
            key.default_label.to_string()
        };

        painter.text(
            key_rect.center(),
            egui::Align2::CENTER_CENTER,
            label_text,
            egui::FontId::proportional(font_size),
            text_color,
        );
    }

    // Handle manual click to verify
    if let Some(key) = clicked_key {
        state.mark_key_verified(key.id, key.vk, 0, false);
    }
}

/// Renders the complete Hardware QA View tab. Returns true when user clicks "Complete Testing".
pub fn show(ui: &mut Ui, state: &mut HardwareQaViewState, i18n: &I18n) -> bool {
    // 1. Keyboard event listener mode
    if state.is_listening {
        // Pause daemon remapping while on Hardware QA tab so physical keys reach the window unmodified
        if !state.was_paused_by_qa {
            let ipc = crate::ipc::SettingsIpc::new();
            let _ = ipc.set_paused(true);
            state.was_paused_by_qa = true;
        }

        let mut pressed_ids: Vec<&'static str> = Vec::new();

        // Capture standard window keypress events (safe, in-process, zero OS hooks)
        ui.input(|i| {
            for event in &i.events {
                if let egui::Event::Key {
                    key,
                    physical_key,
                    pressed: true,
                    repeat: false,
                    ..
                } = event
                    && let Some(id) = map_egui_key_to_a1843_id(*key, *physical_key)
                {
                    pressed_ids.push(id);
                }
            }
        });

        // Query modifier and numpad states using thread-safe Win32 GetKeyState
        #[cfg(windows)]
        {
            use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
            const WIN_KEYS: &[(u16, &str)] = &[
                (VK_LSHIFT, "LeftShift"),
                (VK_RSHIFT, "RightShift"),
                (VK_LCONTROL, "LeftControl"),
                (VK_RCONTROL, "RightControl"),
                (VK_LMENU, "LeftOption"),
                (VK_RMENU, "RightOption"),
                (VK_LWIN, "LeftCommand"),
                (VK_RWIN, "RightCommand"),
                (VK_CAPITAL, "CapsLock"),
                (VK_NUMPAD0, "Num0"),
                (VK_NUMPAD1, "Num1"),
                (VK_NUMPAD2, "Num2"),
                (VK_NUMPAD3, "Num3"),
                (VK_NUMPAD4, "Num4"),
                (VK_NUMPAD5, "Num5"),
                (VK_NUMPAD6, "Num6"),
                (VK_NUMPAD7, "Num7"),
                (VK_NUMPAD8, "Num8"),
                (VK_NUMPAD9, "Num9"),
                (VK_DECIMAL, "NumDecimal"),
                (VK_ADD, "NumAdd"),
                (VK_SUBTRACT, "NumSubtract"),
                (VK_MULTIPLY, "NumMultiply"),
                (VK_DIVIDE, "NumDivide"),
                (VK_NUMLOCK, "Clear"),
            ];

            for &(vk, id) in WIN_KEYS {
                // SAFETY: GetKeyState queries current thread message queue without installing any system hooks.
                let s = unsafe { GetKeyState(vk as i32) };
                if (s as u16 & 0x8000) != 0 && !state.tested_keys.contains(id) {
                    pressed_ids.push(id);
                }
            }
        }

        for id in pressed_ids {
            if !state.tested_keys.contains(id) {
                let vk = state.model.find_by_id(id).map_or(0, |k| k.vk);
                state.mark_key_verified(id, vk, 0, false);
            }
        }
    } else if state.was_paused_by_qa {
        let ipc = crate::ipc::SettingsIpc::new();
        let _ = ipc.set_paused(false);
        state.was_paused_by_qa = false;
    }

    // 2. Header and Controls Toolbar
    ui.horizontal(|ui| {
        ui.heading(i18n.t("qa.title"));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Export QA Log button
            if ui
                .button(format!("📄 {}", i18n.t("qa.export_button")))
                .clicked()
            {
                match export_qa_report(state, i18n) {
                    Ok(path) => {
                        let path_str = path.to_string_lossy().to_string();
                        state.export_status = Some((
                            false,
                            i18n.t_format("qa.export_success", &[("path", &path_str)]),
                        ));
                    }
                    Err(err) => {
                        state.export_status =
                            Some((true, i18n.t_format("qa.export_failed", &[("error", &err)])));
                    }
                }
            }

            // Reset Test button
            if ui
                .button(format!("🔄 {}", i18n.t("qa.reset_button")))
                .clicked()
            {
                state.reset();
            }

            // Listen Mode toggle
            let listen_label = if state.is_listening {
                format!("👂 {}", i18n.t("qa.listen_active"))
            } else {
                format!("⏸ {}", i18n.t("qa.listen_paused"))
            };
            ui.toggle_value(&mut state.is_listening, listen_label);
        });
    });

    ui.label(
        RichText::new(i18n.t("qa.subtitle"))
            .color(Color32::from_rgb(110, 115, 120))
            .small(),
    );

    // Export banner feedback
    if let Some((is_error, ref msg)) = state.export_status {
        let (bg, fg) = if is_error {
            (
                Color32::from_rgb(255, 235, 235),
                Color32::from_rgb(180, 40, 30),
            )
        } else {
            (
                Color32::from_rgb(235, 250, 240),
                Color32::from_rgb(25, 135, 65),
            )
        };
        egui::Frame::NONE
            .fill(bg)
            .corner_radius(CornerRadius::same(4))
            .inner_margin(6.0)
            .show(ui, |ui| {
                ui.colored_label(fg, msg);
            });
    }

    ui.add_space(4.0);

    // 3. Progress Bar and Counter
    let (verified, total, pct) = state.progress();
    let is_all_verified = verified == total && total > 0;

    ui.horizontal(|ui| {
        let count_text = if is_all_verified {
            RichText::new(i18n.t("qa.progress_all_verified"))
                .strong()
                .color(Color32::from_rgb(25, 135, 65))
        } else {
            let v_str = verified.to_string();
            let t_str = total.to_string();
            let p_str = format!("{pct:.0}");
            RichText::new(i18n.t_format(
                "qa.progress",
                &[("verified", &v_str), ("total", &t_str), ("percent", &p_str)],
            ))
            .strong()
        };
        ui.label(count_text);
    });

    let progress_bar = ProgressBar::new(if total > 0 {
        verified as f32 / total as f32
    } else {
        0.0
    })
    .show_percentage()
    .animate(state.is_listening);
    ui.add(progress_bar);

    ui.add_space(6.0);

    // 4. Category Breakdown Cards Bar (wraps responsibly to prevent clipping)
    ui.horizontal_wrapped(|ui| {
        let is_all_selected = state.selected_category.is_none();
        if ui
            .selectable_label(is_all_selected, i18n.t("qa.category_all"))
            .clicked()
        {
            state.selected_category = None;
        }

        for &cat in QaCategory::all() {
            let (v, t) = state.category_progress(cat);
            let is_selected = state.selected_category == Some(cat);
            let badge = if v == t && t > 0 {
                format!("{} ({v}/{t} ✓)", i18n.t(cat.label_key()))
            } else {
                format!("{} ({v}/{t})", i18n.t(cat.label_key()))
            };

            let text = if is_selected {
                RichText::new(badge).strong()
            } else if v == t && t > 0 {
                RichText::new(badge).color(Color32::from_rgb(25, 135, 65))
            } else {
                RichText::new(badge)
            };

            if ui.selectable_label(is_selected, text).clicked() {
                state.selected_category = if is_selected { None } else { Some(cat) };
            }
        }
    });

    ui.add_space(4.0);

    // 5. Visual A1843 Keyboard Canvas
    render_qa_visual_keyboard(ui, state);

    // Informational note about physical Fn key stripped by kbdhid.sys
    ui.label(
        RichText::new(i18n.t("qa.fn_note"))
            .color(Color32::from_rgb(130, 135, 140))
            .small(),
    );

    ui.separator();

    // 6. Recent Keypress Events Log Table
    ui.horizontal(|ui| {
        ui.strong(i18n.t("qa.recent_events"));
        ui.label(format!("({} events)", state.event_log.len()));
    });

    if state.event_log.is_empty() {
        ui.label(
            RichText::new(i18n.t("qa.no_events"))
                .color(Color32::from_rgb(120, 125, 130))
                .italics(),
        );
    } else {
        ScrollArea::vertical()
            .max_height(140.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Grid::new("qa_event_log_grid")
                    .striped(true)
                    .min_col_width(70.0)
                    .show(ui, |ui| {
                        ui.strong(i18n.t("qa.col_time"));
                        ui.strong(i18n.t("qa.col_key"));
                        ui.strong(i18n.t("qa.col_id"));
                        ui.strong(i18n.t("qa.col_vk"));
                        ui.strong(i18n.t("qa.col_scancode"));
                        ui.strong(i18n.t("qa.col_category"));
                        ui.end_row();

                        for ev in state.event_log.iter().take(30) {
                            ui.label(&ev.formatted_time);
                            ui.colored_label(Color32::from_rgb(25, 135, 65), &ev.key_name);
                            ui.label(&ev.key_id);
                            ui.label(format!("0x{:02X}", ev.vk));
                            ui.label(format!(
                                "0x{:02X}{}",
                                ev.scancode,
                                if ev.extended { " (Ext)" } else { "" }
                            ));
                            ui.label(i18n.t(ev.category.label_key()));
                            ui.end_row();
                        }
                    });
            });
    }

    ui.add_space(8.0);
    let mut finished = false;
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let finish_btn = egui::Button::new(
                RichText::new(i18n.t("qa.btn_done"))
                    .size(13.0)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(Color32::from_rgb(0, 120, 215))
            .corner_radius(CornerRadius::same(6));

            if ui.add(finish_btn).clicked() {
                finished = true;
                if state.was_paused_by_qa {
                    let ipc = crate::ipc::SettingsIpc::new();
                    let _ = ipc.set_paused(false);
                    state.was_paused_by_qa = false;
                }
            }
        });
    });

    finished
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qa_state_defaults() {
        let state = HardwareQaViewState::new();
        assert_eq!(state.total_keys, 109);
        assert_eq!(state.tested_keys.len(), 0);
        assert!(state.is_listening);
        assert!(state.event_log.is_empty());

        let (verified, total, pct) = state.progress();
        assert_eq!(verified, 0);
        assert_eq!(total, 109);
        assert!((pct - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_all_109_keys_categorized_without_overlap() {
        let state = HardwareQaViewState::new();
        assert_eq!(state.model.keys.len(), 109);

        let mut counts = std::collections::HashMap::new();
        for key in &state.model.keys {
            let cat = categorize_key(key);
            *counts.entry(cat).or_insert(0) += 1;
        }

        // Verify counts per category
        assert_eq!(*counts.get(&QaCategory::Modifiers).unwrap_or(&0), 8);
        assert_eq!(*counts.get(&QaCategory::FunctionRow).unwrap_or(&0), 20);
        assert_eq!(*counts.get(&QaCategory::NumberRow).unwrap_or(&0), 14);
        assert_eq!(*counts.get(&QaCategory::Navigation).unwrap_or(&0), 10);
        assert_eq!(*counts.get(&QaCategory::Numpad).unwrap_or(&0), 18);
        assert_eq!(*counts.get(&QaCategory::Alphanumeric).unwrap_or(&0), 39);

        let total_sum: usize = counts.values().sum();
        assert_eq!(total_sum, 109);
    }

    #[test]
    fn test_mark_key_verified_and_progress() {
        let mut state = HardwareQaViewState::new();

        state.mark_key_verified("F13", VK_F13, 0x64, false);
        assert_eq!(state.tested_keys.len(), 1);
        assert!(state.tested_keys.contains("F13"));
        assert_eq!(state.event_log.len(), 1);

        let ev = &state.event_log[0];
        assert_eq!(ev.key_id, "F13");
        assert_eq!(ev.vk, VK_F13);
        assert_eq!(ev.category, QaCategory::FunctionRow);

        let (verified, total, pct) = state.progress();
        assert_eq!(verified, 1);
        assert_eq!(total, 109);
        assert!(pct > 0.9 && pct < 1.0);
    }

    #[test]
    fn test_qa_state_reset() {
        let mut state = HardwareQaViewState::new();
        state.mark_key_verified("Space", VK_SPACE, 0x39, false);
        state.mark_key_verified("Enter", VK_RETURN, 0x1C, false);
        assert_eq!(state.tested_keys.len(), 2);

        state.reset();
        assert_eq!(state.tested_keys.len(), 0);
        assert_eq!(state.event_log.len(), 0);
        assert!(state.export_status.is_none());
    }

    #[test]
    fn test_map_egui_key_and_text() {
        assert_eq!(
            map_egui_key_to_a1843_id(egui::Key::Escape, None),
            Some("Escape")
        );
        assert_eq!(map_egui_key_to_a1843_id(egui::Key::F19, None), Some("F19"));
        assert_eq!(
            map_egui_key_to_a1843_id(egui::Key::Space, None),
            Some("Space")
        );
        assert_eq!(map_egui_key_to_a1843_id(egui::Key::A, None), Some("A"));

        assert_eq!(map_text_to_a1843_id("<"), Some("IsoLessGreater"));
        assert_eq!(map_text_to_a1843_id(">"), Some("IsoLessGreater"));
        assert_eq!(map_text_to_a1843_id("à"), Some("Quote"));
    }

    #[test]
    fn test_map_raw_event_to_a1843_id() {
        // Return vs NumEnter
        assert_eq!(
            map_raw_event_to_a1843_id(VK_RETURN, 0x1C, false),
            Some("Enter")
        );
        assert_eq!(
            map_raw_event_to_a1843_id(VK_RETURN, 0x1C, true),
            Some("NumEnter")
        );

        // Clear (VK_NUMLOCK)
        assert_eq!(
            map_raw_event_to_a1843_id(VK_NUMLOCK, 0x45, true),
            Some("Clear")
        );

        // NumEquals (scancode 0x59)
        assert_eq!(
            map_raw_event_to_a1843_id(VK_OEM_PLUS, 0x59, false),
            Some("NumEquals")
        );
        assert_eq!(
            map_raw_event_to_a1843_id(VK_OEM_PLUS, 0x0D, false),
            Some("Equals")
        );

        // F13..F19
        assert_eq!(map_raw_event_to_a1843_id(VK_F13, 0x64, false), Some("F13"));
        assert_eq!(map_raw_event_to_a1843_id(VK_F19, 0x6A, false), Some("F19"));

        // Commands and Options
        assert_eq!(
            map_raw_event_to_a1843_id(VK_LWIN, 0x5B, true),
            Some("LeftCommand")
        );
        assert_eq!(
            map_raw_event_to_a1843_id(VK_RWIN, 0x5C, true),
            Some("RightCommand")
        );
        assert_eq!(
            map_raw_event_to_a1843_id(VK_LMENU, 0x38, false),
            Some("LeftOption")
        );
        assert_eq!(
            map_raw_event_to_a1843_id(VK_RMENU, 0x38, true),
            Some("RightOption")
        );

        // Synthetic ISO AltGr Left Control must be ignored
        assert_eq!(map_raw_event_to_a1843_id(VK_LCONTROL, 0x21D, false), None);
        assert_eq!(
            map_raw_event_to_a1843_id(VK_LCONTROL, 0x1D, false),
            Some("LeftControl")
        );
    }

    #[test]
    fn test_current_timestamp_format() {
        let ts = current_timestamp_string();
        assert_eq!(ts.len(), 15); // YYYYMMDD-HHMMSS
        assert!(ts.contains('-'));
    }

    #[test]
    fn test_export_qa_report() {
        let mut state = HardwareQaViewState::new();
        state.mark_key_verified("Escape", VK_ESCAPE, 0x01, false);
        state.mark_key_verified("F13", VK_F13, 0x64, false);

        let i18n = I18n::new("en");
        let temp_dir = std::env::temp_dir().join(format!(
            "magicbridge-qa-test-{}",
            current_timestamp_string()
        ));
        let res = export_qa_report_to_dir(&state, &i18n, &temp_dir);
        assert!(res.is_ok());

        let report_path = res.expect("report exported");
        assert!(report_path.exists());

        let txt = fs::read_to_string(&report_path).expect("read report");
        assert!(txt.contains("MAGICBRIDGE HARDWARE QA KEY VERIFICATION REPORT"));
        assert!(txt.contains("Escape"));
        assert!(txt.contains("F13"));
        assert!(txt.contains("2 / 109 keys verified"));

        // Check json file was also generated
        let json_path = report_path.with_extension("json");
        assert!(json_path.exists());
        let json_content = fs::read_to_string(&json_path).expect("read json");
        assert!(json_content.contains("\"keyboard_model\": \"A1843\""));
        assert!(json_content.contains("\"verified_keys\": 2"));

        // Clean up
        let _ = fs::remove_file(&report_path);
        let _ = fs::remove_file(&json_path);
        let _ = fs::remove_dir(&temp_dir);
    }
}
