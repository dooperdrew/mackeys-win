//! Native Windows GUI Installer, Uninstaller, and Process Manager for MacKeys.
//!
//! Provides a progressive, multi-step Windows Setup Wizard:
//! - Step 0: Welcome & Mode Selection (Install, or if already installed: Reinstall vs Uninstall)
//! - Step 1: Clean Installation Options (collapsible Advanced Options for custom path / shortcuts / autostart)
//! - Step 2: Real Animated Progress Bar (file copying, shortcuts, registry integration)
//! - Step 3: Integrated First-Run Setup (hardware detection, layout auto-detection, modifier presets, Fn key)
//! - Step 4: Completion Screen with direct "Open Settings Panel" action
//!
//! And a clean Uninstaller Wizard that completely removes all binaries, shortcuts, registry entries, and user data.

use std::path::{Path, PathBuf};
use std::process::Command;

use eframe::{
    App, NativeOptions,
    egui::{self, Color32, CornerRadius, ProgressBar, RichText, Stroke, ViewportBuilder},
};
use magicbridge_core::{
    AppleModelId, Config, FnKeyChoice, KeyboardLayout, ModifierPreset, TopRowMode, atomic_write,
    default_config_path,
};
use windows::Win32::Foundation::{ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::System::Registry::{
    HKEY_CURRENT_USER, KEY_SET_VALUE, REG_SZ, RegCloseKey, RegCreateKeyW, RegDeleteKeyW,
    RegDeleteValueW, RegOpenKeyExW, RegSetValueExW,
};
use windows::core::PCWSTR;

/// Default installation directory under `%LOCALAPPDATA%`.
pub fn default_install_dir() -> PathBuf {
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local_appdata)
            .join("Programs")
            .join("MacKeys")
    } else {
        PathBuf::from(r"C:\Program Files\MacKeys")
    }
}

/// Returns true if the currently running executable is executing from the installed location.
#[must_use]
pub fn is_installed_location() -> bool {
    let current = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return false,
    };

    let target_dir = default_install_dir();
    current.starts_with(&target_dir)
}

/// Checks if the background daemon is alive; if not, launches it via `--daemon`.
pub fn ensure_daemon_running() {
    let ipc = crate::ipc::SettingsIpc::new();
    if ipc.query_status().is_some() {
        return;
    }

    let installed_exe = default_install_dir().join("mackeys.exe");
    let legacy_exe = default_install_dir().join("magicbridge.exe");
    let exe_to_run = if installed_exe.exists() {
        installed_exe
    } else if legacy_exe.exists() {
        legacy_exe
    } else if let Ok(exe_path) = std::env::current_exe() {
        exe_path
    } else {
        return;
    };

    let mut cmd = Command::new(&exe_to_run);
    cmd.arg("--daemon");
    let _ = cmd.spawn();
}

/// Options selected in the installer wizard GUI.
#[derive(Debug, Clone)]
pub struct InstallOptions {
    pub install_dir: PathBuf,
    pub create_desktop_shortcut: bool,
    pub create_start_menu_shortcut: bool,
    pub autostart_on_login: bool,
    pub launch_on_finish: bool,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            install_dir: default_install_dir(),
            create_desktop_shortcut: true,
            create_start_menu_shortcut: true,
            autostart_on_login: true,
            launch_on_finish: true,
        }
    }
}

/// Creates a Windows `.lnk` shortcut file pointing to the target executable.
pub fn create_shortcut(
    target_exe: &Path,
    shortcut_path: &Path,
    description: &str,
    args: &str,
) -> Result<(), String> {
    if let Some(parent) = shortcut_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let ps_cmd = format!(
        "$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut('{}'); $s.TargetPath = '{}'; $s.Arguments = '{}'; $s.Description = '{}'; $s.WorkingDirectory = '{}'; $s.Save()",
        shortcut_path.display(),
        target_exe.display(),
        args,
        description,
        target_exe
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_default()
    );

    let status = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
        .status()
        .map_err(|e| format!("Failed to run PowerShell shortcut creator: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "PowerShell shortcut creation exited with code {:?}",
            status.code()
        ))
    }
}

/// Executes installation into the destination folder and registers in Windows.
pub fn perform_install(options: &InstallOptions) -> Result<(), String> {
    let current_exe =
        std::env::current_exe().map_err(|e| format!("Cannot locate current exe: {e}"))?;
    std::fs::create_dir_all(&options.install_dir)
        .map_err(|e| format!("Failed to create install directory: {e}"))?;

    let dest_exe = options.install_dir.join("mackeys.exe");

    // Terminate running instances before overwrite if present
    if dest_exe.exists() || options.install_dir.join("magicbridge.exe").exists() {
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "mackeys.exe", "/IM", "magicbridge.exe"])
            .status();
        std::thread::sleep(std::time::Duration::from_millis(300));
    }

    // Copy executable
    std::fs::copy(&current_exe, &dest_exe)
        .map_err(|e| format!("Failed to copy binary to destination: {e}"))?;

    // Create Start Menu Shortcuts in dedicated "MacKeys" folder:
    // 1. "MacKeys" -> opens Settings
    // 2. "Uninstall MacKeys" -> runs magicbridge.exe --uninstall
    if options.create_start_menu_shortcut
        && let Ok(appdata) = std::env::var("APPDATA")
    {
        let start_menu_folder =
            PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs\MacKeys");
        let _ = std::fs::create_dir_all(&start_menu_folder);

        let app_shortcut = start_menu_folder.join("MacKeys.lnk");
        let _ = create_shortcut(
            &dest_exe,
            &app_shortcut,
            "MacKeys for Apple Magic Keyboard",
            "",
        );

        let uninstall_shortcut = start_menu_folder.join("Uninstall MacKeys.lnk");
        let _ = create_shortcut(
            &dest_exe,
            &uninstall_shortcut,
            "Uninstall MacKeys",
            "--uninstall",
        );
    }

    // Create Desktop Shortcut
    if options.create_desktop_shortcut
        && let Ok(userprofile) = std::env::var("USERPROFILE")
    {
        let desktop_path = PathBuf::from(userprofile)
            .join("Desktop")
            .join("MacKeys.lnk");
        let _ = create_shortcut(
            &dest_exe,
            &desktop_path,
            "MacKeys for Apple Magic Keyboard",
            "",
        );
    }

    // Autostart in HKCU Run
    if options.autostart_on_login {
        let dest_str = format!("\"{}\" --daemon", dest_exe.display());
        let _ = write_hkcu_string(
            r"Software\Microsoft\Windows\CurrentVersion\Run",
            "MacKeys",
            &dest_str,
        );
    }

    // Register in Windows Settings "Installed apps"
    let uninstall_key = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\MacKeys";
    let uninstall_str = format!("\"{}\" --uninstall", dest_exe.display());
    let icon_str = format!("\"{}\",0", dest_exe.display());

    let _ = write_hkcu_string(
        uninstall_key,
        "DisplayName",
        "MacKeys (Apple Magic Keyboard for Windows)",
    );
    let _ = write_hkcu_string(uninstall_key, "DisplayVersion", env!("CARGO_PKG_VERSION"));
    let _ = write_hkcu_string(uninstall_key, "Publisher", "MacKeys Open Source Project");
    let _ = write_hkcu_string(
        uninstall_key,
        "InstallLocation",
        &options.install_dir.display().to_string(),
    );
    let _ = write_hkcu_string(uninstall_key, "UninstallString", &uninstall_str);
    let _ = write_hkcu_string(uninstall_key, "DisplayIcon", &icon_str);

    // Launch installed app if requested
    if options.launch_on_finish {
        let mut cmd = Command::new(&dest_exe);
        cmd.arg("--daemon");
        let _ = cmd.spawn();
    }

    Ok(())
}

/// Performs clean uninstallation of MacKeys.
pub fn perform_uninstall(remove_user_data: bool) -> Result<(), String> {
    // 1. Terminate running instances
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "mackeys.exe", "/IM", "magicbridge.exe"])
        .status();

    // 2. Remove autostart entry
    let _ = delete_hkcu_value(r"Software\Microsoft\Windows\CurrentVersion\Run", "MacKeys");
    let _ = delete_hkcu_value(
        r"Software\Microsoft\Windows\CurrentVersion\Run",
        "MagicBridge",
    );

    // 3. Remove uninstall entry
    let _ = delete_hkcu_key(r"Software\Microsoft\Windows\CurrentVersion\Uninstall\MacKeys");
    let _ = delete_hkcu_key(r"Software\Microsoft\Windows\CurrentVersion\Uninstall\MagicBridge");

    // 4. Remove Start Menu folder and Desktop Shortcuts
    if let Ok(appdata) = std::env::var("APPDATA") {
        let start_menu_folder =
            PathBuf::from(appdata.clone()).join(r"Microsoft\Windows\Start Menu\Programs\MacKeys");
        let _ = std::fs::remove_dir_all(start_menu_folder);
        let legacy_folder = PathBuf::from(appdata.clone())
            .join(r"Microsoft\Windows\Start Menu\Programs\MagicBridge");
        let _ = std::fs::remove_dir_all(legacy_folder);
        // Also remove legacy single shortcut if present
        let legacy_shortcut =
            PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs\MacKeys.lnk");
        let _ = std::fs::remove_file(legacy_shortcut);
    }
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let desktop_path = PathBuf::from(userprofile.clone())
            .join("Desktop")
            .join("MacKeys.lnk");
        let _ = std::fs::remove_file(desktop_path);
        let legacy_desktop = PathBuf::from(userprofile)
            .join("Desktop")
            .join("MagicBridge.lnk");
        let _ = std::fs::remove_file(legacy_desktop);
    }

    // 5. Remove user configuration if requested
    if remove_user_data && let Ok(config_dir) = magicbridge_core::default_config_dir() {
        let _ = std::fs::remove_dir_all(config_dir);
    }

    // 6. Remove installation directory
    let install_dir = default_install_dir();
    if install_dir.exists() {
        let ps_cmd = format!(
            "Start-Sleep -Seconds 1; Remove-Item -Path '{}' -Recurse -Force",
            install_dir.display()
        );
        let _ = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
            .spawn();
    }

    Ok(())
}

// ----------------------------------------------------------------------------
// Win32 Registry Helpers
// ----------------------------------------------------------------------------

fn encode_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn write_hkcu_string(sub_key: &str, value_name: &str, data: &str) -> Result<(), WIN32_ERROR> {
    let sub_key_w = encode_wide(sub_key);
    let value_name_w = encode_wide(value_name);
    let data_w = encode_wide(data);

    let mut hkey = windows::Win32::System::Registry::HKEY::default();
    // SAFETY: RegCreateKeyW opens or creates sub_key in HKCU.
    let status = unsafe {
        RegCreateKeyW(
            HKEY_CURRENT_USER,
            PCWSTR::from_raw(sub_key_w.as_ptr()),
            &mut hkey,
        )
    };
    if status != ERROR_SUCCESS {
        return Err(status);
    }

    let byte_count = (data_w.len() * std::mem::size_of::<u16>()) as u32;
    // SAFETY: data_w is a valid null-terminated UTF-16 buffer with byte length byte_count.
    let byte_slice =
        unsafe { std::slice::from_raw_parts(data_w.as_ptr() as *const u8, byte_count as usize) };
    // SAFETY: hkey is an open HKCU registry key handle with KEY_SET_VALUE access permissions.
    let set_status = unsafe {
        RegSetValueExW(
            hkey,
            PCWSTR::from_raw(value_name_w.as_ptr()),
            None,
            REG_SZ,
            Some(byte_slice),
        )
    };

    // SAFETY: RegCloseKey closes the opened registry key.
    unsafe {
        let _ = RegCloseKey(hkey);
    }

    if set_status != ERROR_SUCCESS {
        Err(set_status)
    } else {
        Ok(())
    }
}

fn delete_hkcu_value(sub_key: &str, value_name: &str) -> Result<(), WIN32_ERROR> {
    let sub_key_w = encode_wide(sub_key);
    let value_name_w = encode_wide(value_name);

    let mut hkey = windows::Win32::System::Registry::HKEY::default();
    // SAFETY: RegOpenKeyExW opens subkey with KEY_SET_VALUE.
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR::from_raw(sub_key_w.as_ptr()),
            None,
            KEY_SET_VALUE,
            &mut hkey,
        )
    };
    if status != ERROR_SUCCESS {
        return Err(status);
    }

    // SAFETY: RegDeleteValueW removes the named value.
    let del_status = unsafe { RegDeleteValueW(hkey, PCWSTR::from_raw(value_name_w.as_ptr())) };

    // SAFETY: RegCloseKey closes registry handle.
    unsafe {
        let _ = RegCloseKey(hkey);
    }

    if del_status != ERROR_SUCCESS {
        Err(del_status)
    } else {
        Ok(())
    }
}

fn delete_hkcu_key(sub_key: &str) -> Result<(), WIN32_ERROR> {
    let sub_key_w = encode_wide(sub_key);
    // SAFETY: RegDeleteKeyW deletes the entire subkey from HKCU.
    let status = unsafe { RegDeleteKeyW(HKEY_CURRENT_USER, PCWSTR::from_raw(sub_key_w.as_ptr())) };
    if status != ERROR_SUCCESS {
        Err(status)
    } else {
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// Progressive Windows Setup Wizard Application
// ----------------------------------------------------------------------------

/// Setup wizard steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SetupStep {
    #[default]
    Welcome,
    Options,
    Installing,
    FirstSetup,
    Finished,
}

/// Action chosen on Step 0 if MacKeys is already installed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WelcomeAction {
    #[default]
    InstallOrUpgrade,
    Uninstall,
}

/// Progressive Windows GUI Setup Wizard Application.
pub struct InstallerApp {
    pub step: SetupStep,
    pub welcome_action: WelcomeAction,
    pub already_installed: bool,
    pub show_advanced_options: bool,
    pub options: InstallOptions,
    // Step 2 install simulation / state
    pub install_progress: f32,
    pub install_status_text: String,
    pub install_error: Option<String>,
    // Step 3 first-run settings
    pub selected_model: AppleModelId,
    pub selected_layout: KeyboardLayout,
    pub selected_modifier: ModifierPreset,
    pub selected_fn_key: FnKeyChoice,
    pub media_keys_first: bool,
    pub is_connected_probe: bool,
    pub battery_probe: Option<u8>,
    pub custom_path_str: String,
}

impl Default for InstallerApp {
    fn default() -> Self {
        let default_dir = default_install_dir();
        let already_installed = default_dir.join("mackeys.exe").exists()
            || default_dir.join("magicbridge.exe").exists();
        let detected_layout = crate::detect_windows_keyboard_layout();

        let mut app = Self {
            step: SetupStep::Welcome,
            welcome_action: WelcomeAction::InstallOrUpgrade,
            already_installed,
            show_advanced_options: false,
            options: InstallOptions::default(),
            install_progress: 0.0,
            install_status_text: String::from("Ready to install"),
            install_error: None,
            selected_model: magicbridge_core::AppleModelId::default(),
            selected_layout: detected_layout,
            selected_modifier: ModifierPreset::WindowsLayout,
            selected_fn_key: FnKeyChoice::RightOption,
            media_keys_first: true,
            is_connected_probe: false,
            battery_probe: None,
            custom_path_str: default_dir.display().to_string(),
        };
        app.refresh_hardware_probe();
        app
    }
}

impl App for InstallerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // 1. Wizard Steps Breadcrumb Bar
            render_wizard_breadcrumbs(ui, self.step);
            ui.add_space(8.0);

            // 2. Step View Dispatch
            match self.step {
                SetupStep::Welcome => self.render_step_welcome(ui, ctx),
                SetupStep::Options => self.render_step_options(ui, ctx),
                SetupStep::Installing => self.render_step_installing(ui, ctx),
                SetupStep::FirstSetup => self.render_step_first_setup(ui, ctx),
                SetupStep::Finished => self.render_step_finished(ui, ctx),
            }
        });
    }
}

impl InstallerApp {
    /// Refreshes keyboard connection status and battery state from IPC or direct HID probing.
    pub fn refresh_hardware_probe(&mut self) {
        let ipc = crate::ipc::SettingsIpc::new();
        let daemon_status = ipc.query_status();
        let hardware_detected = magicbridge_agent::detect_connected_apple_model();
        let keyboards = magicbridge_agent::probe_connected_apple_keyboards();

        self.is_connected_probe = daemon_status.as_ref().is_some_and(|s| s.is_connected)
            || hardware_detected.is_some()
            || !keyboards.is_empty();

        if let Some(m) = hardware_detected {
            self.selected_model = m;
        }

        self.battery_probe = daemon_status
            .as_ref()
            .and_then(|s| s.battery_percent)
            .or_else(|| {
                magicbridge_agent::query_battery(&std::collections::HashSet::new())
                    .map(|b| b.percentage)
            });
    }
    fn render_step_welcome(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Welcome to the MacKeys Setup Wizard");
        ui.add_space(4.0);
        ui.label(
            "MacKeys makes the Apple Magic Keyboard with Numeric Keypad (model A1843) \
             behave as a first-class Windows keyboard with native media controls, user-mode \
             Fn layer, modifier presets, and live battery reporting.",
        );
        ui.add_space(12.0);

        if self.already_installed {
            egui::Frame::group(ui.style())
                .fill(Color32::from_rgb(240, 245, 255))
                .corner_radius(CornerRadius::same(6))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(180, 205, 240)))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("ℹ MacKeys is currently installed on this computer.")
                                .strong()
                                .color(Color32::from_rgb(0, 100, 200)),
                        );
                        ui.add_space(4.0);
                        ui.radio_value(
                            &mut self.welcome_action,
                            WelcomeAction::InstallOrUpgrade,
                            "Reinstall or Upgrade MacKeys (Recommended)",
                        );
                        ui.radio_value(
                            &mut self.welcome_action,
                            WelcomeAction::Uninstall,
                            "Uninstall MacKeys completely from this computer",
                        );
                    });
                });
        } else {
            egui::Frame::group(ui.style())
                .fill(Color32::from_rgb(245, 248, 250))
                .corner_radius(CornerRadius::same(6))
                .show(ui, |ui| {
                    ui.label("This setup wizard will install MacKeys onto your computer and guide you through first-run keyboard configuration.");
                });
        }

        ui.add_space(20.0);
        ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }

                let next_text =
                    if self.already_installed && self.welcome_action == WelcomeAction::Uninstall {
                        "Next (Uninstall) >"
                    } else {
                        "Next >"
                    };

                let next_btn =
                    egui::Button::new(RichText::new(next_text).strong().color(Color32::WHITE))
                        .fill(Color32::from_rgb(0, 120, 215))
                        .corner_radius(CornerRadius::same(6));

                if ui.add(next_btn).clicked() {
                    if self.already_installed && self.welcome_action == WelcomeAction::Uninstall {
                        // Launch uninstaller window
                        let _ = run_uninstaller_gui();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    } else {
                        self.step = SetupStep::Options;
                    }
                }
            });
            ui.separator();
        });
    }

    fn render_step_options(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Installation Options");
        ui.label("Select the installation options for MacKeys:");
        ui.add_space(8.0);

        // Basic Options (visible to all users)
        egui::Frame::group(ui.style())
            .fill(Color32::from_rgb(245, 248, 252))
            .corner_radius(CornerRadius::same(6))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Installation Summary").strong());
                    ui.label("• Installs to your personal user account (%LOCALAPPDATA%\\Programs\\MacKeys)");
                    ui.label("• Zero administrator privileges required, leaves Windows system files untouched");
                    ui.add_space(6.0);
                    ui.checkbox(
                        &mut self.options.create_desktop_shortcut,
                        "Create Desktop Shortcut",
                    );
                    ui.checkbox(
                        &mut self.options.autostart_on_login,
                        "Start MacKeys automatically at Windows login",
                    );
                });
            });

        ui.add_space(8.0);

        // Advanced Options Collapsible (masked by default as requested by user)
        let toggle_label = if self.show_advanced_options {
            "▾ Hide Advanced Options"
        } else {
            "▸ Show Advanced Options (Custom Path, Shortcuts)"
        };
        if ui.button(RichText::new(toggle_label).small()).clicked() {
            self.show_advanced_options = !self.show_advanced_options;
        }

        if self.show_advanced_options {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Install Folder:");
                    if ui.text_edit_singleline(&mut self.custom_path_str).changed() {
                        self.options.install_dir = PathBuf::from(&self.custom_path_str);
                    }
                });
                ui.checkbox(
                    &mut self.options.create_start_menu_shortcut,
                    "Create Start Menu shortcuts (MacKeys & Uninstaller)",
                );
            });
        }

        ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }

                let install_btn = egui::Button::new(
                    RichText::new("Install Now >")
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(0, 120, 215))
                .corner_radius(CornerRadius::same(6));

                if ui.add(install_btn).clicked() {
                    self.step = SetupStep::Installing;
                }

                if ui.button("< Back").clicked() {
                    self.step = SetupStep::Welcome;
                }
            });
            ui.separator();
        });
    }

    fn render_step_installing(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Installing MacKeys...");
        ui.label("Please wait while the setup wizard installs MacKeys on your computer.");
        ui.add_space(16.0);

        if self.install_progress < 1.0 {
            self.install_progress += 0.25;
            ctx.request_repaint_after(std::time::Duration::from_millis(150));

            if self.install_progress >= 0.25 && self.install_progress < 0.5 {
                self.install_status_text = String::from("Copying application binaries...");
            } else if self.install_progress >= 0.5 && self.install_progress < 0.75 {
                self.install_status_text =
                    String::from("Creating Start Menu and Desktop shortcuts...");
            } else if self.install_progress >= 0.75 && self.install_progress < 1.0 {
                self.install_status_text =
                    String::from("Registering in Windows Settings and starting services...");
            }
        }

        if self.install_progress >= 1.0 && self.install_error.is_none() {
            match perform_install(&self.options) {
                Ok(()) => {
                    self.step = SetupStep::FirstSetup;
                }
                Err(e) => {
                    self.install_error = Some(e);
                }
            }
        }

        ui.add(
            ProgressBar::new(self.install_progress.clamp(0.0, 1.0))
                .show_percentage()
                .animate(true),
        );
        ui.add_space(8.0);
        ui.label(RichText::new(&self.install_status_text).italics());

        if let Some(ref err) = self.install_error {
            ui.add_space(8.0);
            ui.colored_label(
                Color32::from_rgb(220, 50, 40),
                format!("Installation failed: {err}"),
            );
        }
    }

    fn render_step_first_setup(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        if !self.is_connected_probe {
            self.refresh_hardware_probe();
        }

        ui.heading("First-Run Keyboard Setup");
        ui.label("Configure your initial keyboard preferences (you can change them anytime in Settings):");
        ui.add_space(6.0);

        // Hardware detection indicator
        egui::Frame::group(ui.style())
            .fill(if self.is_connected_probe {
                Color32::from_rgb(240, 248, 242)
            } else {
                Color32::from_rgb(255, 250, 240)
            })
            .corner_radius(CornerRadius::same(6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if self.is_connected_probe {
                        ui.colored_label(
                            Color32::from_rgb(40, 160, 60),
                            format!("✔ Connected: {}", self.selected_model.display_name()),
                        );
                        if let Some(pct) = self.battery_probe {
                            ui.label(format!("(Battery: {pct}%)"));
                        }
                    } else {
                        ui.colored_label(
                            Color32::from_rgb(200, 140, 0),
                            "⚡ Keyboard not connected yet (connect via USB or Bluetooth)",
                        );
                    }
                });
            });

        ui.add_space(6.0);

        // 0. Keyboard Model Selection
        ui.horizontal(|ui| {
            ui.label(RichText::new("Apple Model:").strong());
            let combo = egui::ComboBox::from_id_salt("wizard_setup_model")
                .selected_text(self.selected_model.display_name())
                .show_ui(ui, |ui| {
                    for m in AppleModelId::all() {
                        ui.selectable_value(&mut self.selected_model, *m, m.display_name());
                    }
                });
            combo
                .response
                .on_hover_text("Select your Apple keyboard model or click Auto-Detect");

            if ui.button("🔍 Auto-Detect Model").clicked() {
                self.refresh_hardware_probe();
            }
        });

        ui.add_space(4.0);

        // 1. Keyboard Layout Selection
        ui.horizontal(|ui| {
            ui.label(RichText::new("Keyboard Layout:").strong());
            let current_str = format!("{}", self.selected_layout);
            let combo = egui::ComboBox::from_id_salt("wizard_setup_layout")
                .selected_text(current_str)
                .show_ui(ui, |ui| {
                    ui.label(RichText::new("── Detected on this PC ──").small().weak());
                    for (name, l) in crate::get_installed_windows_layouts() {
                        let label = format!("✔ {name}");
                        ui.selectable_value(&mut self.selected_layout, l, label);
                    }
                    ui.separator();
                    ui.label(
                        RichText::new("── All Windows Keyboard Layouts ──")
                            .small()
                            .weak(),
                    );
                    ui.selectable_value(
                        &mut self.selected_layout,
                        KeyboardLayout::IsoItalian,
                        "ISO Italian",
                    );
                    ui.selectable_value(
                        &mut self.selected_layout,
                        KeyboardLayout::AnsiUs,
                        "ANSI US",
                    );
                    ui.selectable_value(&mut self.selected_layout, KeyboardLayout::IsoUk, "ISO UK");
                    ui.selectable_value(
                        &mut self.selected_layout,
                        KeyboardLayout::IsoGerman,
                        "ISO German",
                    );
                    ui.selectable_value(
                        &mut self.selected_layout,
                        KeyboardLayout::IsoFrench,
                        "ISO French",
                    );
                    ui.selectable_value(
                        &mut self.selected_layout,
                        KeyboardLayout::IsoSpanish,
                        "ISO Spanish",
                    );
                    ui.selectable_value(
                        &mut self.selected_layout,
                        KeyboardLayout::IsoPortuguese,
                        "ISO Portuguese",
                    );
                    ui.selectable_value(
                        &mut self.selected_layout,
                        KeyboardLayout::IsoSwiss,
                        "ISO Swiss",
                    );
                    ui.selectable_value(
                        &mut self.selected_layout,
                        KeyboardLayout::IsoNordic,
                        "ISO Nordic",
                    );
                    ui.selectable_value(
                        &mut self.selected_layout,
                        KeyboardLayout::JapaneseJis,
                        "Japanese JIS",
                    );
                    ui.selectable_value(&mut self.selected_layout, KeyboardLayout::Dutch, "Dutch");
                    ui.selectable_value(
                        &mut self.selected_layout,
                        KeyboardLayout::Polish,
                        "Polish (Programmers)",
                    );
                    ui.selectable_value(
                        &mut self.selected_layout,
                        KeyboardLayout::Russian,
                        "Russian",
                    );
                    ui.selectable_value(
                        &mut self.selected_layout,
                        KeyboardLayout::Ukrainian,
                        "Ukrainian",
                    );
                    ui.selectable_value(
                        &mut self.selected_layout,
                        KeyboardLayout::Turkish,
                        "Turkish (Q / F)",
                    );
                    ui.selectable_value(&mut self.selected_layout, KeyboardLayout::Czech, "Czech");
                    ui.selectable_value(
                        &mut self.selected_layout,
                        KeyboardLayout::Hungarian,
                        "Hungarian",
                    );
                    ui.selectable_value(&mut self.selected_layout, KeyboardLayout::Greek, "Greek");
                    ui.selectable_value(
                        &mut self.selected_layout,
                        KeyboardLayout::Arabic,
                        "Arabic",
                    );
                    ui.selectable_value(
                        &mut self.selected_layout,
                        KeyboardLayout::Hebrew,
                        "Hebrew",
                    );
                    ui.selectable_value(
                        &mut self.selected_layout,
                        KeyboardLayout::Korean,
                        "Korean",
                    );
                    ui.selectable_value(
                        &mut self.selected_layout,
                        KeyboardLayout::Chinese,
                        "Chinese",
                    );
                });
            combo
                .response
                .on_hover_text("Select your physical keyboard layout");

            if ui.button("🔍 Auto-Detect from Windows").clicked() {
                self.selected_layout = crate::detect_windows_keyboard_layout();
            }
        });

        ui.add_space(4.0);

        // 2. Modifier Key Preset Selection
        ui.horizontal(|ui| {
            ui.label(RichText::new("Modifier Preset:").strong());
            ui.radio_value(
                &mut self.selected_modifier,
                ModifierPreset::WindowsLayout,
                "Windows Layout (Cmd ↔ Alt swapped for standard PC feel)",
            );
            ui.radio_value(
                &mut self.selected_modifier,
                ModifierPreset::MacLayout,
                "Mac Layout",
            );
        });

        ui.add_space(4.0);

        // 3. Fn Trigger Key Choice
        ui.horizontal(|ui| {
            ui.label(RichText::new("Fn Trigger Key:").strong());
            ui.radio_value(
                &mut self.selected_fn_key,
                FnKeyChoice::RightOption,
                "Right Option / AltGr",
            );
            ui.radio_value(
                &mut self.selected_fn_key,
                FnKeyChoice::RightControl,
                "Right Control",
            );
            ui.radio_value(&mut self.selected_fn_key, FnKeyChoice::F19, "F19");
        });

        ui.add_space(4.0);
        ui.checkbox(
            &mut self.media_keys_first,
            "Top row keys default to Media controls (Play/Pause, Volume, Next/Prev Track)",
        );

        ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
            ui.horizontal(|ui| {
                let save_btn = egui::Button::new(
                    RichText::new("Save & Continue >")
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(0, 120, 215))
                .corner_radius(CornerRadius::same(6));

                if ui.add(save_btn).clicked() {
                    // Save initial configuration
                    let mut config = Config {
                        keyboard_model: self.selected_model,
                        ..Default::default()
                    };
                    if let Some(profile) = config.active_profile_mut() {
                        profile.layout = self.selected_layout.clone();
                        profile.modifier_preset = self.selected_modifier;
                        profile.fn_key = self.selected_fn_key;
                        profile.top_row_mode = if self.media_keys_first {
                            TopRowMode::MediaKeysFirst
                        } else {
                            TopRowMode::FKeysFirst
                        };
                    }
                    config.autostart = self.options.autostart_on_login;
                    let config_path =
                        default_config_path().unwrap_or_else(|_| PathBuf::from("config.toml"));
                    if let Ok(toml_str) = config.to_toml_string() {
                        let _ = atomic_write(&config_path, toml_str);
                    }

                    // Reload running daemon or ensure it is running
                    let ipc = crate::ipc::SettingsIpc::new();
                    let _ = ipc.reload_config();
                    ensure_daemon_running();

                    self.step = SetupStep::Finished;
                }

                if ui.button("< Back").clicked() {
                    self.step = SetupStep::Options;
                }
            });
            ui.separator();
        });
    }

    fn render_step_finished(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("🎉 Setup Completed Successfully!");
        ui.add_space(4.0);
        ui.label("MacKeys is now installed and configured on your computer.");
        ui.label("The background keyboard engine is active and will run in your system tray.");
        ui.add_space(16.0);

        egui::Frame::group(ui.style())
            .fill(Color32::from_rgb(240, 248, 255))
            .corner_radius(CornerRadius::same(6))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("What would you like to do next?").strong());
                    ui.label("• Open the Settings panel to customize individual keys, view battery status, or run hardware verification tests.");
                    ui.label("• Or simply close this wizard — your keyboard is already working with your chosen layout!");
                });
            });

        ui.add_space(20.0);

        ui.horizontal(|ui| {
            let open_settings_btn = egui::Button::new(
                RichText::new("🚀 Open MacKeys Settings Panel")
                    .size(15.0)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(Color32::from_rgb(0, 120, 215))
            .corner_radius(CornerRadius::same(6));

            if ui.add(open_settings_btn).clicked() {
                let dest_exe = self.options.install_dir.join("mackeys.exe");
                let exe_to_run = if dest_exe.exists() {
                    dest_exe
                } else {
                    self.options.install_dir.join("magicbridge.exe")
                };
                let mut cmd = Command::new(&exe_to_run);
                cmd.arg("--settings");
                let _ = cmd.spawn();
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            if ui
                .button(RichText::new("Finish & Close").size(14.0))
                .clicked()
            {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }
}

/// Standalone GUI Uninstaller application.
#[derive(Debug, Clone, Default)]
pub struct UninstallerApp {
    pub remove_user_data: bool,
    pub status_message: Option<Result<String, String>>,
    pub is_finished: bool,
    pub progress: f32,
}

impl App for UninstallerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);
            ui.heading("🗑 MacKeys Uninstaller");
            ui.add_space(8.0);

            if self.is_finished {
                ui.colored_label(
                    Color32::from_rgb(40, 160, 60),
                    RichText::new("✔ MacKeys was successfully and cleanly removed.").size(16.0).strong(),
                );
                ui.add_space(10.0);
                ui.label("All application files, shortcuts, and background entries have been purged.");
                ui.add_space(16.0);
                if ui.button(RichText::new("Close").size(14.0)).clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                return;
            }

            ui.label("Are you sure you want to completely remove MacKeys and all its shortcuts from your computer?");
            ui.add_space(8.0);
            ui.checkbox(&mut self.remove_user_data, "Also remove custom key profiles and user configuration files (%APPDATA%\\MacKeys)");

            if let Some(ref res) = self.status_message {
                ui.add_space(8.0);
                match res {
                    Ok(msg) => {
                        ui.colored_label(Color32::from_rgb(40, 160, 60), msg);
                    }
                    Err(err) => {
                        ui.colored_label(Color32::from_rgb(220, 50, 40), format!("Error: {err}"));
                    }
                }
            }

            ui.add_space(20.0);
            ui.horizontal(|ui| {
                let uninstall_btn = egui::Button::new(
                    RichText::new("Uninstall MacKeys")
                        .size(14.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(210, 45, 40))
                .corner_radius(CornerRadius::same(6));

                if ui.add(uninstall_btn).clicked() {
                    match perform_uninstall(self.remove_user_data) {
                        Ok(()) => {
                            self.is_finished = true;
                            self.status_message = Some(Ok("Uninstalled successfully!".to_string()));
                        }
                        Err(e) => {
                            self.status_message = Some(Err(e));
                        }
                    }
                }

                if ui.button("Cancel").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });
    }
}

/// Helper to render wizard steps breadcrumbs header.
fn render_wizard_breadcrumbs(ui: &mut egui::Ui, current_step: SetupStep) {
    let steps = [
        (SetupStep::Welcome, "1. Welcome"),
        (SetupStep::Options, "2. Options"),
        (SetupStep::Installing, "3. Installing"),
        (SetupStep::FirstSetup, "4. Initial Setup"),
        (SetupStep::Finished, "5. Ready"),
    ];

    ui.horizontal(|ui| {
        for (i, (step, label)) in steps.iter().enumerate() {
            let is_current = current_step == *step;
            let text = if is_current {
                RichText::new(*label)
                    .strong()
                    .color(Color32::from_rgb(0, 120, 215))
            } else {
                RichText::new(*label).color(Color32::from_rgb(130, 135, 140))
            };

            ui.label(text);
            if i < steps.len() - 1 {
                ui.label(RichText::new("›").color(Color32::from_rgb(180, 185, 190)));
            }
        }
    });
    ui.separator();
}

/// Runs the native GUI Installer wizard window.
pub fn run_installer_gui() -> Result<(), eframe::Error> {
    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([640.0, 450.0])
            .with_min_inner_size([600.0, 420.0])
            .with_title("MacKeys Setup Wizard"),
        ..Default::default()
    };
    eframe::run_native(
        "MacKeys Setup Wizard",
        native_options,
        Box::new(|cc| {
            crate::setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(InstallerApp::default()))
        }),
    )
}

/// Runs the native GUI Uninstaller window.
pub fn run_uninstaller_gui() -> Result<(), eframe::Error> {
    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([540.0, 320.0])
            .with_resizable(false)
            .with_title("MacKeys Uninstaller"),
        ..Default::default()
    };
    eframe::run_native(
        "MacKeys Uninstaller",
        native_options,
        Box::new(|cc| {
            crate::setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(UninstallerApp::default()))
        }),
    )
}
