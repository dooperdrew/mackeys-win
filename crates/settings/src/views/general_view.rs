//! General application settings, autostart, and configuration import/export view.
//!
//! Complies with F7.3, F7.6, F8.1, F9.1, F9.2, M15, M16, M18, and docs/04-ux.md.

use std::fs;
use std::path::PathBuf;

use egui::Ui;
use magicbridge_agent::autostart;
use magicbridge_core::{AppleModelId, Config, CornerKeyAction, NamedProfile, ThemeMode};

use crate::i18n::I18n;
use crate::ipc::SettingsIpc;

/// State for the General tab.
#[derive(Debug, Clone, Default)]
pub struct GeneralViewState {
    pub pause_hotkey_text: String,
    pub game_processes_text: String,
    pub new_app_process: String,
    pub new_app_profile: String,
    pub status_message: Option<String>,
}

impl GeneralViewState {
    /// Creates a new view state initialized from the current configuration.
    #[must_use]
    pub fn new_from_config(config: &Config) -> Self {
        let first_profile = config.profiles.keys().next().cloned().unwrap_or_default();
        Self {
            pause_hotkey_text: config.pause_hotkey.clone().unwrap_or_default(),
            game_processes_text: config.game_mode_processes.join(", "),
            new_app_process: String::new(),
            new_app_profile: first_profile,
            status_message: None,
        }
    }
}

/// Renders the General settings view in a responsive 2-column layout.
pub fn show(
    ui: &mut Ui,
    state: &mut GeneralViewState,
    config: &mut Config,
    i18n: &mut I18n,
) -> bool {
    let mut modified = false;
    let title = i18n.t("general.title").to_string();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            crate::set_accessible_name(ui, ui.id(), &title);
            ui.heading(&title);
            ui.add_space(4.0);

            ui.columns(2, |columns| {
                // ==============================================================
                // COLUMN 0: Hardware, Appearance, OSD & Peripherals
                // ==============================================================
                let col0 = &mut columns[0];
                let g0 = col0.group(|ui| {
                    crate::set_accessible_name(ui, ui.id(), "Hardware & Appearance");

                    // 1. UI Language Selector (F7.3)
                    ui.horizontal(|ui| {
                        let label = i18n.t("general.language");
                        ui.label(label);
                        let current_lang = i18n.current_language();
                        let mut selected_lang = current_lang.to_string();

                        let combo = egui::ComboBox::from_id_salt("ui_language_combo")
                            .selected_text(if current_lang == "it" {
                                "Italiano"
                            } else {
                                "English"
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut selected_lang,
                                    "en".to_string(),
                                    "English",
                                );
                                ui.selectable_value(
                                    &mut selected_lang,
                                    "it".to_string(),
                                    "Italiano",
                                );
                            });
                        let combo_id = combo.response.id;
                        combo.response.on_hover_text(label);
                        crate::set_accessible_name(ui, combo_id, label);

                        if selected_lang != config.ui_language {
                            config.ui_language = selected_lang.clone();
                            i18n.set_language(&selected_lang);
                            modified = true;
                        }
                    });

                    // 2. Color Theme Selector
                    ui.horizontal(|ui| {
                        let label = i18n.t("general.theme");
                        ui.label(label);
                        let current_theme_str = match config.theme {
                            ThemeMode::System => i18n.t("general.theme_system"),
                            ThemeMode::Dark => i18n.t("general.theme_dark"),
                            ThemeMode::Light => i18n.t("general.theme_light"),
                        };
                        let combo = egui::ComboBox::from_id_salt("ui_theme_combo")
                            .selected_text(current_theme_str)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut config.theme,
                                    ThemeMode::System,
                                    i18n.t("general.theme_system"),
                                );
                                ui.selectable_value(
                                    &mut config.theme,
                                    ThemeMode::Dark,
                                    i18n.t("general.theme_dark"),
                                );
                                ui.selectable_value(
                                    &mut config.theme,
                                    ThemeMode::Light,
                                    i18n.t("general.theme_light"),
                                );
                            });
                        let res = combo.response.on_hover_text(label);
                        crate::set_accessible_name(ui, res.id, label);

                        if res.changed() {
                            modified = true;
                        }
                    });

                    // 3. Apple Keyboard Model Selector & Auto-Detection
                    ui.horizontal(|ui| {
                        let label = i18n.t("general.keyboard_model");
                        ui.label(label);
                        let combo = egui::ComboBox::from_id_salt("ui_keyboard_model_combo")
                            .selected_text(config.keyboard_model.display_name())
                            .show_ui(ui, |ui| {
                                for m in AppleModelId::all() {
                                    ui.selectable_value(
                                        &mut config.keyboard_model,
                                        *m,
                                        m.display_name(),
                                    );
                                }
                            });
                        let res = combo
                            .response
                            .on_hover_text(i18n.t("general.keyboard_model_tooltip"));
                        crate::set_accessible_name(ui, res.id, label);
                        if res.changed() {
                            modified = true;
                        }
                    });

                    let auto_detect_label = format!("🔍 {}", i18n.t("general.auto_detect_model"));
                    let auto_btn = ui
                        .button(&auto_detect_label)
                        .on_hover_text(i18n.t("general.auto_detect_model_tooltip"));
                    crate::set_accessible_name(ui, auto_btn.id, &auto_detect_label);
                    if auto_btn.clicked()
                        && let Some(detected) = magicbridge_agent::detect_connected_apple_model()
                    {
                        config.keyboard_model = detected;
                        modified = true;
                    }

                    // 4. Corner / Lock Key Workflow (Touch ID / Lock / Eject)
                    if let Some(prof) = config.active_profile_mut() {
                        ui.horizontal(|ui| {
                            let label = i18n.t("general.corner_key_action");
                            ui.label(label);
                            let current_label = match prof.corner_key_action {
                                CornerKeyAction::LockWorkstation => {
                                    i18n.t("general.corner_key_lock")
                                }
                                CornerKeyAction::TaskView => i18n.t("general.corner_key_task_view"),
                                CornerKeyAction::ShowDesktop => {
                                    i18n.t("general.corner_key_show_desktop")
                                }
                                CornerKeyAction::Sleep => i18n.t("general.corner_key_sleep"),
                                CornerKeyAction::PassThrough => {
                                    i18n.t("general.corner_key_passthrough")
                                }
                            };
                            let combo = egui::ComboBox::from_id_salt("ui_corner_key_action_combo")
                                .selected_text(current_label)
                                .show_ui(ui, |ui| {
                                    for action in CornerKeyAction::all() {
                                        let alabel = match action {
                                            CornerKeyAction::LockWorkstation => {
                                                i18n.t("general.corner_key_lock")
                                            }
                                            CornerKeyAction::TaskView => {
                                                i18n.t("general.corner_key_task_view")
                                            }
                                            CornerKeyAction::ShowDesktop => {
                                                i18n.t("general.corner_key_show_desktop")
                                            }
                                            CornerKeyAction::Sleep => {
                                                i18n.t("general.corner_key_sleep")
                                            }
                                            CornerKeyAction::PassThrough => {
                                                i18n.t("general.corner_key_passthrough")
                                            }
                                        };
                                        ui.selectable_value(
                                            &mut prof.corner_key_action,
                                            *action,
                                            alabel,
                                        );
                                    }
                                });
                            let res = combo
                                .response
                                .on_hover_text(i18n.t("general.corner_key_action_tooltip"));
                            crate::set_accessible_name(ui, res.id, label);
                            if res.changed() {
                                modified = true;
                            }
                        });
                    }

                    ui.separator();

                    // 5. On-Screen Display (OSD) Overlay HUD (M15.1)
                    let osd_label = i18n.t("general.osd_enabled");
                    if ui.checkbox(&mut config.osd_enabled, osd_label).changed() {
                        modified = true;
                    }
                    if config.osd_enabled {
                        ui.horizontal(|ui| {
                            let dlabel = i18n.t("general.osd_duration");
                            ui.label(dlabel);
                            let res = ui
                                .add(
                                    egui::Slider::new(&mut config.osd_duration_ms, 500..=3000)
                                        .suffix(" ms")
                                        .step_by(100.0),
                                )
                                .on_hover_text(dlabel);
                            crate::set_accessible_name(ui, res.id, dlabel);
                            if res.changed() {
                                modified = true;
                            }
                        });
                    }

                    ui.separator();

                    // 6. Apple Peripheral Ecosystem (M18)
                    ui.label(egui::RichText::new(i18n.t("general.peripherals_title")).strong());
                    if ui
                        .checkbox(
                            &mut config.magic_mouse_smooth_scroll,
                            i18n.t("general.mouse_smooth_scroll"),
                        )
                        .changed()
                    {
                        modified = true;
                    }
                    if ui
                        .checkbox(
                            &mut config.magic_mouse_natural_scroll,
                            i18n.t("general.mouse_natural_scroll"),
                        )
                        .changed()
                    {
                        modified = true;
                    }
                    ui.horizontal(|ui| {
                        let slabel = i18n.t("general.mouse_scroll_speed");
                        ui.label(slabel);
                        let res = ui
                            .add(
                                egui::Slider::new(
                                    &mut config.magic_mouse_scroll_speed_percent,
                                    50..=200,
                                )
                                .suffix("%")
                                .step_by(5.0),
                            )
                            .on_hover_text(slabel);
                        crate::set_accessible_name(ui, res.id, slabel);
                        if res.changed() {
                            modified = true;
                        }
                    });

                    ui.separator();

                    // 7. Installation & Maintenance
                    ui.label(egui::RichText::new(i18n.t("general.maintenance_title")).strong());
                    let is_installed = crate::is_installed_location();
                    if is_installed {
                        ui.horizontal(|ui| {
                            ui.colored_label(egui::Color32::from_rgb(40, 160, 60), "✔");
                            ui.label(i18n.t("general.installed_desc"));
                        });
                        let uninstall_btn = egui::Button::new(
                            egui::RichText::new(i18n.t("general.uninstall_button"))
                                .color(egui::Color32::from_rgb(220, 50, 40))
                                .strong(),
                        );
                        if ui.add(uninstall_btn).clicked()
                            && let Ok(current_exe) = std::env::current_exe()
                        {
                            let _ = std::process::Command::new(current_exe)
                                .arg("--uninstall")
                                .spawn();
                        }
                    } else {
                        ui.horizontal(|ui| {
                            ui.colored_label(egui::Color32::from_rgb(0, 120, 215), "ℹ");
                            ui.label(i18n.t("general.portable_desc"));
                        });
                        let install_btn = egui::Button::new(
                            egui::RichText::new(i18n.t("general.install_button"))
                                .color(egui::Color32::WHITE)
                                .strong(),
                        )
                        .fill(egui::Color32::from_rgb(0, 120, 215));
                        if ui.add(install_btn).clicked()
                            && let Ok(current_exe) = std::env::current_exe()
                        {
                            let _ = std::process::Command::new(current_exe)
                                .arg("--install")
                                .spawn();
                        }
                    }
                });
                crate::set_accessible_name(col0, g0.response.id, "Hardware & Appearance");

                // ==============================================================
                // COLUMN 1: System Integration, Hotkeys, Game Mode & Profiles
                // ==============================================================
                let col1 = &mut columns[1];
                let g1 = col1.group(|ui| {
                    crate::set_accessible_name(ui, ui.id(), "System & Automation");

                    // 1. Autostart with Windows (F8.1)
                    let autostart_active = autostart::is_autostart_enabled().unwrap_or(false);
                    let mut autostart_toggle = autostart_active;
                    if ui
                        .checkbox(&mut autostart_toggle, i18n.t("general.autostart"))
                        .changed()
                    {
                        if autostart_toggle {
                            if let Some(agent_exe) = SettingsIpc::find_agent_exe() {
                                let _ = autostart::enable_autostart(&agent_exe);
                            }
                        } else {
                            let _ = autostart::disable_autostart();
                        }
                        config.autostart = autostart_toggle;
                        modified = true;
                    }

                    // 2. Global Pause Hotkey (F9.1)
                    ui.horizontal(|ui| {
                        let label = i18n.t("general.pause_hotkey");
                        ui.label(label);
                        let mut hotkey_str = config.pause_hotkey.clone().unwrap_or_default();
                        let res = ui
                            .add(
                                egui::TextEdit::singleline(&mut hotkey_str)
                                    .hint_text("e.g. Pause, F12")
                                    .desired_width(120.0),
                            )
                            .on_hover_text(label);
                        crate::set_accessible_name(ui, res.id, label);
                        if res.changed() {
                            config.pause_hotkey = if hotkey_str.trim().is_empty() {
                                None
                            } else {
                                Some(hotkey_str.trim().to_string())
                            };
                            modified = true;
                        }
                    });

                    // 3. Debug Logging (NFR-Q2)
                    if ui
                        .checkbox(&mut config.debug_logging, i18n.t("general.debug_logging"))
                        .changed()
                    {
                        modified = true;
                    }

                    ui.separator();

                    // 4. Game Mode & Monitored Executables (F9.2)
                    if ui
                        .checkbox(&mut config.game_mode_enabled, i18n.t("general.game_mode"))
                        .changed()
                    {
                        modified = true;
                    }
                    ui.horizontal(|ui| {
                        let label = i18n.t("general.game_processes");
                        ui.label(label);
                        let res = ui
                            .add(
                                egui::TextEdit::singleline(&mut state.game_processes_text)
                                    .hint_text("e.g. game.exe, steam.exe")
                                    .desired_width(180.0),
                            )
                            .on_hover_text(label);
                        crate::set_accessible_name(ui, res.id, label);
                        if res.changed() {
                            config.game_mode_processes = state
                                .game_processes_text
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                            modified = true;
                        }
                    });

                    ui.separator();

                    // 5. Per-Application Profile Auto-Switching (M16)
                    ui.label(egui::RichText::new(i18n.t("general.app_profiles_title")).strong());
                    if config.app_profiles.is_empty() {
                        ui.label(
                            egui::RichText::new(i18n.t("general.no_app_profiles"))
                                .italics()
                                .color(egui::Color32::from_rgb(140, 150, 160)),
                        );
                    } else {
                        let mut to_remove = None;
                        for (app, prof) in &config.app_profiles {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(app).strong());
                                ui.label("➔");
                                ui.label(prof);
                                if ui
                                    .small_button(format!(
                                        "🗑 {}",
                                        i18n.t("general.delete_app_binding")
                                    ))
                                    .clicked()
                                {
                                    to_remove = Some(app.clone());
                                }
                            });
                        }
                        if let Some(app) = to_remove {
                            config.app_profiles.remove(&app);
                            modified = true;
                        }
                    }

                    ui.horizontal(|ui| {
                        let res = ui.add(
                            egui::TextEdit::singleline(&mut state.new_app_process)
                                .hint_text(i18n.t("general.app_process_hint"))
                                .desired_width(130.0),
                        );
                        crate::set_accessible_name(
                            ui,
                            res.id,
                            i18n.t("general.app_profiles_title"),
                        );

                        let profiles: Vec<String> = config.profiles.keys().cloned().collect();
                        if state.new_app_profile.is_empty() && !profiles.is_empty() {
                            state.new_app_profile = profiles[0].clone();
                        }

                        let combo = egui::ComboBox::from_id_salt("new_app_profile_combo")
                            .selected_text(&state.new_app_profile)
                            .show_ui(ui, |ui| {
                                for p in &profiles {
                                    ui.selectable_value(&mut state.new_app_profile, p.clone(), p);
                                }
                            });
                        let combo_id = combo.response.id;
                        combo
                            .response
                            .on_hover_text(i18n.t("general.app_profiles_title"));
                        crate::set_accessible_name(
                            ui,
                            combo_id,
                            i18n.t("general.app_profiles_title"),
                        );

                        let add_btn = ui.button(i18n.t("general.add_app_profile"));
                        if add_btn.clicked() {
                            let trimmed = state.new_app_process.trim().to_string();
                            if !trimmed.is_empty() && !state.new_app_profile.is_empty() {
                                config
                                    .app_profiles
                                    .insert(trimmed, state.new_app_profile.clone());
                                state.new_app_process.clear();
                                modified = true;
                            }
                        }
                    });

                    ui.separator();

                    // 6. Configuration Import & Export (F7.6)
                    ui.label(egui::RichText::new(i18n.t("general.config_management")).strong());
                    ui.horizontal(|ui| {
                        // Export Config
                        if ui.button(i18n.t("general.export_config")).clicked() {
                            let default_export_path =
                                PathBuf::from("MagicBridge_Config_Export.toml");
                            if let Ok(toml_str) = config.to_toml_string()
                                && let Ok(()) = fs::write(&default_export_path, toml_str)
                            {
                                state.status_message = Some(i18n.t_format(
                                    "general.config_exported",
                                    &[("path", &default_export_path.display().to_string())],
                                ));
                            }
                        }

                        // Import Config
                        if ui.button(i18n.t("general.import_config")).clicked() {
                            let default_import_path =
                                PathBuf::from("MagicBridge_Config_Export.toml");
                            if default_import_path.is_file()
                                && let Ok(contents) = fs::read_to_string(&default_import_path)
                                && let Ok(imported_config) = Config::from_toml_str(&contents)
                            {
                                *config = imported_config;
                                modified = true;
                                state.status_message =
                                    Some(i18n.t("general.config_imported").to_string());
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        // Export Active Profile
                        if ui.button(i18n.t("general.export_profile")).clicked()
                            && let Some(profile) = config.active_profile()
                        {
                            let filename =
                                format!("Profile_{}.toml", profile.name.replace(' ', "_"));
                            let path = PathBuf::from(&filename);
                            if let Ok(toml_str) = toml::to_string_pretty(profile)
                                && let Ok(()) = fs::write(&path, toml_str)
                            {
                                state.status_message = Some(i18n.t_format(
                                    "general.config_exported",
                                    &[("path", &path.display().to_string())],
                                ));
                            }
                        }

                        // Import Profile
                        if ui.button(i18n.t("general.import_profile")).clicked()
                            && let Some(profile) = config.active_profile()
                        {
                            let filename =
                                format!("Profile_{}.toml", profile.name.replace(' ', "_"));
                            let path = PathBuf::from(&filename);
                            if path.is_file()
                                && let Ok(contents) = fs::read_to_string(&path)
                                && let Ok(imported_prof) = toml::from_str::<NamedProfile>(&contents)
                            {
                                config.add_profile(imported_prof);
                                modified = true;
                                state.status_message =
                                    Some(i18n.t("general.config_imported").to_string());
                            }
                        }
                    });

                    if let Some(ref msg) = state.status_message {
                        ui.add_space(2.0);
                        ui.colored_label(egui::Color32::from_rgb(40, 160, 60), msg);
                    }
                });
                crate::set_accessible_name(col1, g1.response.id, "System & Automation");
            });
        });

    modified
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_general_view_state_defaults() {
        let state = GeneralViewState::default();
        assert!(state.pause_hotkey_text.is_empty());
        assert!(state.game_processes_text.is_empty());
        assert!(state.new_app_process.is_empty());
        assert!(state.new_app_profile.is_empty());
        assert!(state.status_message.is_none());

        let mut config = Config::default();
        config
            .app_profiles
            .insert("code.exe".to_string(), "Mac Layout".to_string());
        let from_cfg = GeneralViewState::new_from_config(&config);
        assert!(!from_cfg.new_app_profile.is_empty());
    }
}
