//! MacKeys Settings GUI Library.
//!
//! Provides the visual keyboard models, localization, views, and application shell.

pub mod app;
pub mod i18n;
pub mod installer;
pub mod ipc;
pub mod keyboard_model;
pub mod views;

pub use app::{MacKeysApp, SettingsTab};
pub use i18n::I18n;
pub use installer::{
    InstallOptions, InstallerApp, UninstallerApp, is_installed_location, perform_install,
    perform_uninstall, run_installer_gui, run_uninstaller_gui,
};
pub use ipc::SettingsIpc;
pub use keyboard_model::{A1843Model, KeyCluster, PhysicalKey};
pub use views::qa_view::HardwareQaViewState;
pub use views::wizard_view::{OnboardingWizardState, WizardAction};

/// Configures native Windows fallback fonts (Segoe UI Symbol, Segoe UI Emoji)
/// so arrows, symbols, and icons render crisp and complete without tofu (□) squares.
pub fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    #[cfg(target_os = "windows")]
    {
        // 1. Segoe UI Symbol: provides arrows (←, →, ↑, ↓), math symbols, UI glyphs
        if let Ok(data) = std::fs::read(r"C:\Windows\Fonts\seguisym.ttf") {
            fonts.font_data.insert(
                "segoe_ui_symbol".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(data)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("segoe_ui_symbol".to_owned());
        }

        // 2. Segoe UI Emoji: provides full Unicode emoji coverage
        if let Ok(data) = std::fs::read(r"C:\Windows\Fonts\seguiemj.ttf") {
            fonts.font_data.insert(
                "segoe_ui_emoji".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(data)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("segoe_ui_emoji".to_owned());
        }
    }

    ctx.set_fonts(fonts);
}

/// Helper to map a Windows Language ID (LANGID) to a `KeyboardLayout`.
#[must_use]
pub fn lang_id_to_keyboard_layout(
    lang_id: u16,
    name_fallback: &str,
) -> magicbridge_core::KeyboardLayout {
    match lang_id {
        0x0410 => magicbridge_core::KeyboardLayout::IsoItalian,
        0x0409 => magicbridge_core::KeyboardLayout::AnsiUs,
        0x0809 => magicbridge_core::KeyboardLayout::IsoUk,
        0x0407 => magicbridge_core::KeyboardLayout::IsoGerman,
        0x040C => magicbridge_core::KeyboardLayout::IsoFrench,
        0x040A => magicbridge_core::KeyboardLayout::IsoSpanish,
        0x0816 | 0x0416 => magicbridge_core::KeyboardLayout::IsoPortuguese,
        0x0807 | 0x100C => magicbridge_core::KeyboardLayout::IsoSwiss,
        0x041D | 0x040B | 0x0406 | 0x0414 => magicbridge_core::KeyboardLayout::IsoNordic,
        0x0411 => magicbridge_core::KeyboardLayout::JapaneseJis,
        0x0413 => magicbridge_core::KeyboardLayout::Dutch,
        0x0415 => magicbridge_core::KeyboardLayout::Polish,
        0x0419 => magicbridge_core::KeyboardLayout::Russian,
        0x0422 => magicbridge_core::KeyboardLayout::Ukrainian,
        0x041F => magicbridge_core::KeyboardLayout::Turkish,
        0x0405 => magicbridge_core::KeyboardLayout::Czech,
        0x040E => magicbridge_core::KeyboardLayout::Hungarian,
        0x0408 => magicbridge_core::KeyboardLayout::Greek,
        0x0401 => magicbridge_core::KeyboardLayout::Arabic,
        0x040D => magicbridge_core::KeyboardLayout::Hebrew,
        0x0412 => magicbridge_core::KeyboardLayout::Korean,
        0x0804 | 0x0404 => magicbridge_core::KeyboardLayout::Chinese,
        _ => {
            if name_fallback.is_empty() {
                magicbridge_core::KeyboardLayout::AnsiUs
            } else {
                magicbridge_core::KeyboardLayout::Custom(name_fallback.to_string())
            }
        }
    }
}

/// Detects the current active Windows keyboard input layout.
#[must_use]
pub fn detect_windows_keyboard_layout() -> magicbridge_core::KeyboardLayout {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardLayout;
        // SAFETY: GetKeyboardLayout(0) retrieves the layout handle for the current thread.
        let hkl = unsafe { GetKeyboardLayout(0) };
        let lang_id = (hkl.0 as usize & 0xFFFF) as u16;
        lang_id_to_keyboard_layout(lang_id, "")
    }
    #[cfg(not(target_os = "windows"))]
    {
        magicbridge_core::KeyboardLayout::IsoItalian
    }
}

/// Retrieves all keyboard input languages and layouts currently installed in Windows for this user.
#[must_use]
pub fn get_installed_windows_layouts() -> Vec<(String, magicbridge_core::KeyboardLayout)> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Globalization::{GetLocaleInfoW, LOCALE_SENGLISHDISPLAYNAME};
        use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyboardLayoutList, HKL};

        // SAFETY: GetKeyboardLayoutList(None) retrieves count of keyboard layout handles for caller.
        let count = unsafe { GetKeyboardLayoutList(None) };
        if count <= 0 {
            return vec![(
                "English (US)".to_string(),
                magicbridge_core::KeyboardLayout::AnsiUs,
            )];
        }

        let mut hkls = vec![HKL::default(); count as usize];
        // SAFETY: Buffer hkls has capacity equal to count.
        let actual = unsafe { GetKeyboardLayoutList(Some(&mut hkls)) };
        hkls.truncate(actual as usize);

        let mut result = Vec::new();
        for hkl in hkls {
            let lang_id = (hkl.0 as usize & 0xFFFF) as u32;
            let mut buf = [0u16; 128];
            // SAFETY: GetLocaleInfoW reads the localized display name into buf.
            let len =
                unsafe { GetLocaleInfoW(lang_id, LOCALE_SENGLISHDISPLAYNAME, Some(&mut buf)) };
            let name = if len > 1 {
                String::from_utf16_lossy(&buf[..(len as usize - 1)])
            } else {
                format!("Windows Layout 0x{:04X}", lang_id)
            };

            let layout = lang_id_to_keyboard_layout(lang_id as u16, &name);
            if !result.iter().any(|(n, _)| n == &name) {
                result.push((name, layout));
            }
        }
        result
    }
    #[cfg(not(target_os = "windows"))]
    {
        vec![(
            "Default (ISO Italian)".to_string(),
            magicbridge_core::KeyboardLayout::IsoItalian,
        )]
    }
}

/// Sets an explicit accessible name on an AccessKit node for screen-reader and UI automation compliance.
pub fn set_accessible_name(ui: &egui::Ui, id: egui::Id, name: impl Into<String>) {
    let name_str = name.into();
    ui.ctx().accesskit_node_builder(id, |node| {
        node.set_label(name_str);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use magicbridge_core::{Config, CustomAction, KeyAction, KeyboardLayout, NamedProfile};

    #[test]
    fn test_settings_initialization_defaults() {
        let i18n = I18n::default();
        assert_eq!(i18n.current_language(), "en");
        assert_eq!(i18n.t("tab.keyboard"), "Keyboard");

        let model = A1843Model::new();
        assert!(model.keys.len() > 100);
    }

    #[test]
    fn test_profile_operations_roundtrip() {
        let mut config = Config::default();
        assert_eq!(config.active_profile, "Default (IT)");

        // Add new profile
        let new_prof = NamedProfile::new("Gaming", KeyboardLayout::AnsiUs);
        config.add_profile(new_prof);
        assert!(config.profiles.contains_key("Gaming"));

        // Switch profile
        config
            .set_active_profile("Gaming")
            .expect("switch succeeds");
        assert_eq!(config.active_profile, "Gaming");

        // Duplicate profile
        config
            .duplicate_profile("Gaming", "Gaming (Custom)")
            .expect("duplicate succeeds");
        assert!(config.profiles.contains_key("Gaming (Custom)"));

        // Rename profile
        config
            .rename_profile("Gaming (Custom)", "Gaming (V2)")
            .expect("rename succeeds");
        assert!(config.profiles.contains_key("Gaming (V2)"));

        // Delete profile
        config
            .delete_profile("Gaming (V2)")
            .expect("delete succeeds");
        assert!(!config.profiles.contains_key("Gaming (V2)"));
    }

    #[test]
    fn test_key_action_serialization_in_profile() {
        let mut profile = NamedProfile::default_it();

        // Custom launch action
        profile.mappings.insert(
            "F16".to_string(),
            KeyAction::Custom(CustomAction::Launch {
                command: "calc.exe".to_string(),
            }),
        );

        // Custom URL action
        profile.mappings.insert(
            "F17".to_string(),
            KeyAction::Custom(CustomAction::OpenUrl {
                url: "https://github.com".to_string(),
            }),
        );

        // Custom media action
        profile.mappings.insert(
            "F18".to_string(),
            KeyAction::Custom(CustomAction::Media {
                command: "volume_up".to_string(),
            }),
        );

        // Custom shortcut action
        profile.mappings.insert(
            "F19".to_string(),
            KeyAction::Custom(CustomAction::Shortcut {
                keys: vec!["Win".to_string(), "Shift".to_string(), "S".to_string()],
            }),
        );

        let toml_str = toml::to_string_pretty(&profile).expect("toml serialization succeeds");
        let deserialized: NamedProfile =
            toml::from_str(&toml_str).expect("toml deserialization succeeds");

        assert_eq!(profile, deserialized);
    }
}
