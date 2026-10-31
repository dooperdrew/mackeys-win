//! Hidden Win32 message window and message pump (F8.2, NFR-P2).

use windows::Win32::Devices::HumanInterfaceDevice::GUID_DEVINTERFACE_HID;
use windows::Win32::Foundation::{
    ERROR_CLASS_ALREADY_EXISTS, GetLastError, HWND, LPARAM, LRESULT, WPARAM,
};
use windows::Win32::Graphics::Gdi::HBRUSH;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CREATESTRUCTW, CreateWindowExW, DBT_DEVICEARRIVAL, DBT_DEVICEREMOVECOMPLETE, DefWindowProcW,
    DestroyWindow, DispatchMessageW, GWLP_USERDATA, GetMessageW, GetWindowLongPtrW, HCURSOR, HICON,
    IsWindow, KillTimer, MSG, PBT_APMRESUMEAUTOMATIC, PBT_APMRESUMESUSPEND, PBT_APMSUSPEND,
    PostQuitMessage, RegisterClassExW, RegisterWindowMessageW, SetTimer, SetWindowLongPtrW,
    TranslateMessage, WINDOW_EX_STYLE, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_DEVICECHANGE,
    WM_LBUTTONDBLCLK, WM_NCCREATE, WM_NCDESTROY, WM_POWERBROADCAST, WM_RBUTTONUP, WM_TIMER,
    WNDCLASS_STYLES, WNDCLASSEXW, WS_OVERLAPPED,
};
use windows::core::{PCWSTR, w};

use crate::device_notify::{
    DeviceNotificationHandle, is_apple_keyboard_path, parse_device_interface_path,
    probe_connected_apple_keyboards,
};
use crate::hook::HookHandle;
use crate::state::DaemonState;
use crate::tray::{
    IDM_OPEN_SETTINGS, IDM_PROFILE_BASE, IDM_QUIT, IDM_START_WITH_WINDOWS, IDM_TOGGLE_ENABLED,
    IDM_TOPROW_FKEYS_FIRST, IDM_TOPROW_MEDIA_FIRST, TrayIcon, WM_TRAY_CALLBACK,
};
use magicbridge_core::TopRowMode;

/// Dedicated window class name for the agent's hidden window.
pub const WINDOW_CLASS_NAME: PCWSTR = w!("MagicBridge_MsgWndClass");

/// TaskbarCreated broadcast message string.
pub const TASKBAR_CREATED_STR: PCWSTR = w!("TaskbarCreated");

/// WM_CONTEXTMENU message constant.
pub const WM_CONTEXTMENU_VAL: u32 = 0x007B;

/// Timer ID for low-frequency periodic battery polling (F6.3).
pub const IDT_BATTERY_TIMER: usize = 100;

/// Internal state owned by the window procedure.
struct WindowData {
    state: DaemonState,
    tray: Option<TrayIcon>,
    device_notify: Option<DeviceNotificationHandle>,
    taskbar_created_msg: u32,
    hook: Option<HookHandle>,
    low_battery_notified: bool,
    critical_battery_notified: bool,
    last_charging_state: Option<bool>,
    battery_timer_minutes: u32,
    game_mode_tracker: Option<crate::game_mode::GameModeTracker>,
    osd: Option<crate::osd::OsdOverlay>,
}

impl WindowData {
    /// Synchronizes the compiled mapping table with the active profile.
    fn recompile_active_table(&self) {
        if let Some(profile) = self.state.config.active_profile() {
            let table = magicbridge_core::compile_profile(profile);
            crate::hook::set_active_table(table);
        }
    }

    /// Synchronizes the foreground WinEvent hook for Game Mode and Per-App Auto-Switching (F9.2, M16).
    fn sync_game_mode_tracker(&mut self, hwnd: HWND) {
        let needs_tracker = self.state.config.game_mode_enabled
            || !self.state.config.game_mode_processes.is_empty()
            || !self.state.config.app_profiles.is_empty();
        if needs_tracker && self.game_mode_tracker.is_none() {
            match crate::game_mode::GameModeTracker::install(hwnd) {
                Ok(tracker) => self.game_mode_tracker = Some(tracker),
                Err(e) => eprintln!("Failed to install GameModeTracker: {e}"),
            }
        } else if !needs_tracker {
            self.game_mode_tracker = None;
            self.state.game_mode_active = false;
        }
    }

    /// Configures or resets the periodic battery polling timer (F6.3, NFR-P2).
    fn setup_battery_timer(&mut self, hwnd: HWND) {
        let minutes = self.state.config.battery_poll_minutes.max(1);
        self.battery_timer_minutes = minutes;
        let interval_ms = minutes.saturating_mul(60 * 1000);
        // SAFETY: SetTimer registers a periodic timer for hwnd with interval_ms.
        unsafe {
            let _ = SetTimer(Some(hwnd), IDT_BATTERY_TIMER, interval_ms, None);
        }
    }

    /// Queries battery and updates tray tooltip + low battery notification (F6.1..F6.3).
    fn poll_battery(&mut self) {
        if !self.state.is_connected {
            self.state.battery_percent = None;
            self.state.is_charging = false;
            self.low_battery_notified = false;
            self.critical_battery_notified = false;
            self.last_charging_state = None;
            if let Some(ref mut tray) = self.tray {
                tray.update(&self.state);
            }
            return;
        }

        if let Some(status) = crate::battery::query_battery(&self.state.connected_devices) {
            let old_pct = self.state.battery_percent;
            self.state.battery_percent = Some(status.percentage);
            self.state.is_charging = status.is_charging;

            if let Some(ref mut tray) = self.tray {
                tray.update(&self.state);
            }

            self.check_low_battery_notification(old_pct, status);
        }
    }

    /// Checks battery notifications: cable connect/disconnect and 2-tier low battery (M15.2, F6.2).
    fn check_low_battery_notification(
        &mut self,
        _old_pct: Option<u8>,
        status: crate::battery::BatteryStatus,
    ) {
        // 1. Cable connect / disconnect notifications (M15.2)
        if let Some(was_charging) = self.last_charging_state {
            if !was_charging
                && status.is_charging
                && let Some(ref tray) = self.tray
            {
                let title = "MagicBridge — Charging";
                let msg = format!(
                    "Magic Keyboard connected to power ({}%).",
                    status.percentage
                );
                tray.show_notification(title, &msg);
            } else if was_charging
                && !status.is_charging
                && let Some(ref tray) = self.tray
            {
                let title = "MagicBridge — Disconnected";
                let msg = format!(
                    "Magic Keyboard running on battery ({}%).",
                    status.percentage
                );
                tray.show_notification(title, &msg);
            }
        }
        self.last_charging_state = Some(status.is_charging);

        // 2. Low and critical battery notifications (M15.2)
        if !status.is_charging {
            if status.percentage <= 10 && !self.critical_battery_notified {
                self.critical_battery_notified = true;
                self.low_battery_notified = true;
                if let Some(ref tray) = self.tray {
                    let title = "MagicBridge — Critical Battery";
                    let msg = format!(
                        "Magic Keyboard battery is critically low ({}%)! Connect charger immediately.",
                        status.percentage
                    );
                    tray.show_notification(title, &msg);
                }
            } else if status.percentage <= self.state.config.low_battery_threshold_percent
                && !self.low_battery_notified
            {
                self.low_battery_notified = true;
                if let Some(ref tray) = self.tray {
                    let title = "MagicBridge — Low Battery";
                    let msg = format!(
                        "Magic Keyboard battery is low ({}%). Connect a charging cable.",
                        status.percentage
                    );
                    tray.show_notification(title, &msg);
                }
            } else if status.percentage > self.state.config.low_battery_threshold_percent {
                self.low_battery_notified = false;
                self.critical_battery_notified = false;
            }
        } else {
            // Reset latches when charging
            self.low_battery_notified = false;
            self.critical_battery_notified = false;
        }
    }

    /// Updates the hook active state based on connection, enabled status, and game mode.
    ///
    /// Per NFR-S2, remapping is active while an Apple keyboard is connected
    /// and automatically paused when none is connected.
    /// Per F9.2, remapping is paused while a game is active in the foreground.
    fn update_hook_state(&mut self) {
        let should_be_active =
            self.state.enabled && self.state.is_connected && !self.state.game_mode_active;
        if should_be_active {
            if self.hook.is_none() {
                match HookHandle::install() {
                    Ok(h) => {
                        self.hook = Some(h);
                    }
                    Err(e) => {
                        eprintln!("Failed to install keyboard hook: {e}");
                    }
                }
            } else if let Some(ref h) = self.hook {
                h.set_active(true);
            }
        } else if let Some(ref h) = self.hook {
            // Deactivate hook and release any held synthetic keys (NFR-S3).
            h.set_active(false);
            if !self.state.is_connected {
                let h = self.hook.take();
                drop(h);
            }
        }
    }
}

/// Hidden Win32 message window.
#[derive(Debug)]
pub struct MessageWindow {
    hwnd: HWND,
}

impl MessageWindow {
    /// Registers the window class and creates the hidden message window.
    pub fn create(state: DaemonState) -> Result<Self, windows::core::Error> {
        // SAFETY: RegisterWindowMessageW registers or retrieves the message ID for TaskbarCreated.
        let taskbar_created_msg = unsafe { RegisterWindowMessageW(TASKBAR_CREATED_STR) };

        // SAFETY: GetModuleHandleW(None) returns the HINSTANCE of the current process module.
        let hinstance = unsafe { GetModuleHandleW(None)? };

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: WNDCLASS_STYLES(0),
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance.into(),
            hIcon: HICON::default(),
            hCursor: HCURSOR::default(),
            hbrBackground: HBRUSH::default(),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: WINDOW_CLASS_NAME,
            hIconSm: HICON::default(),
        };

        // SAFETY: RegisterClassExW registers the window class.
        let atom = unsafe { RegisterClassExW(&wc) };
        if atom == 0 {
            // SAFETY: Queries the last error code for class registration.
            let err = unsafe { GetLastError() };
            if err != ERROR_CLASS_ALREADY_EXISTS {
                return Err(windows::core::Error::from_thread());
            }
        }

        let osd = crate::osd::OsdOverlay::new().ok();
        let window_data = Box::new(WindowData {
            state,
            tray: None,
            device_notify: None,
            taskbar_created_msg,
            hook: None,
            low_battery_notified: false,
            critical_battery_notified: false,
            last_charging_state: None,
            battery_timer_minutes: 0,
            game_mode_tracker: None,
            osd,
        });
        let raw_ptr = Box::into_raw(window_data);

        // SAFETY: CreateWindowExW creates a hidden top-level window (WS_OVERLAPPED without WS_VISIBLE).
        // It passes `raw_ptr` in lpParam to be set in GWLP_USERDATA by WM_NCCREATE.
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                WINDOW_CLASS_NAME,
                w!("MagicBridge Agent"),
                WS_OVERLAPPED,
                0,
                0,
                0,
                0,
                None,
                None,
                Some(hinstance.into()),
                Some(raw_ptr as *const std::ffi::c_void),
            )
        };

        match hwnd {
            Ok(h) => Ok(Self { hwnd: h }),
            Err(e) => {
                // SAFETY: If window creation failed, reclaim the allocated WindowData to prevent leak.
                unsafe {
                    let _ = Box::from_raw(raw_ptr);
                }
                Err(e)
            }
        }
    }

    /// Returns the raw window handle.
    #[must_use]
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }
}

impl Drop for MessageWindow {
    fn drop(&mut self) {
        crate::hook::clear_agent_window();
        // SAFETY: Checks if window is still valid before calling DestroyWindow.
        let is_valid = unsafe { IsWindow(Some(self.hwnd)).as_bool() };
        if is_valid {
            // SAFETY: Destroys the window handle.
            unsafe {
                let _ = DestroyWindow(self.hwnd);
            }
        }
    }
}

/// Win32 window procedure handling tray notifications and menu commands.
unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_NCCREATE {
        // SAFETY: In WM_NCCREATE, lparam points to CREATESTRUCTW.
        let cs = unsafe { &*(lparam.0 as *const CREATESTRUCTW) };
        let state_ptr = cs.lpCreateParams as *mut WindowData;
        // SAFETY: Store pointer in GWLP_USERDATA.
        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
        }
        // SAFETY: Delegate to default window procedure.
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }

    // SAFETY: Retrieves our pointer stored in GWLP_USERDATA.
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowData };
    if state_ptr.is_null() {
        // SAFETY: Delegate to default window procedure before NCCREATE or after NCDESTROY.
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }

    // SAFETY: `state_ptr` is valid, non-null, and only accessed on this message loop thread.
    let data = unsafe { &mut *state_ptr };

    match msg {
        WM_CREATE => {
            crate::hook::set_agent_window(hwnd);

            // 1. Initial probe of connected Apple keyboards to populate state before tray creation.
            let devices = probe_connected_apple_keyboards();
            data.state.sync_connected_devices(&devices);

            // 2. Synchronize active mapping table, game mode, and update hook state (NFR-S2, F9.2).
            data.recompile_active_table();
            data.sync_game_mode_tracker(hwnd);
            data.state.game_mode_active = crate::game_mode::is_foreground_game(&data.state.config);
            data.update_hook_state();

            // 3. Register for device interface arrival/removal notifications (F8.4, NFR-S2).
            match DeviceNotificationHandle::register_for_device_interface(
                hwnd,
                &GUID_DEVINTERFACE_HID,
            ) {
                Ok(notify) => data.device_notify = Some(notify),
                Err(e) => eprintln!("Failed to register for device notifications: {e}"),
            }

            // 4. Initialize system tray icon with current status.
            match TrayIcon::create(hwnd, &data.state) {
                Ok(tray) => data.tray = Some(tray),
                Err(e) => eprintln!("Failed to initialize system tray icon: {e}"),
            }

            // 5. Setup periodic battery timer and perform initial battery read (F6.1..F6.3).
            data.setup_battery_timer(hwnd);
            data.poll_battery();
            LRESULT(0)
        }
        WM_DEVICECHANGE => {
            let event = wparam.0 as u32;
            match event {
                DBT_DEVICEARRIVAL => {
                    if let Some(path) = parse_device_interface_path(lparam)
                        && is_apple_keyboard_path(&path)
                    {
                        let info = magicbridge_core::device::parse_hid_path(&path);
                        let transport_str = info.transport.to_string();
                        println!(
                            "[MagicBridge] Apple keyboard connected ({transport_str}): {path}"
                        );
                        data.state.on_device_arrival(&path, &transport_str);
                        data.update_hook_state();
                        data.poll_battery();
                        if let Some(ref tray) = data.tray {
                            let title = "MagicBridge — Keyboard Connected";
                            let msg = format!(
                                "Apple Keyboard connected ({transport_str}). Active profile: {}",
                                data.state.config.active_profile
                            );
                            tray.show_notification(title, &msg);
                        }
                        if let Some(ref mut tray) = data.tray {
                            tray.update(&data.state);
                        }
                    }
                }
                DBT_DEVICEREMOVECOMPLETE => {
                    if let Some(path) = parse_device_interface_path(lparam)
                        && is_apple_keyboard_path(&path)
                    {
                        println!("[MagicBridge] Apple keyboard disconnected: {path}");
                        data.state.on_device_removal(&path);
                        // If our tracked set became empty, re-probe present devices
                        // in case another interface was not tracked or removed out of order.
                        if !data.state.is_connected {
                            let devices = probe_connected_apple_keyboards();
                            data.state.sync_connected_devices(&devices);
                        }
                        data.update_hook_state();
                        data.poll_battery();
                        if let Some(ref tray) = data.tray {
                            tray.show_notification(
                                "MagicBridge — Keyboard Disconnected",
                                "Apple Keyboard disconnected. Remapping paused.",
                            );
                        }
                        if let Some(ref mut tray) = data.tray {
                            tray.update(&data.state);
                        }
                    }
                }
                _ => {}
            }
            LRESULT(1)
        }
        WM_POWERBROADCAST => {
            let event = wparam.0 as u32;
            match event {
                PBT_APMSUSPEND => {
                    println!("[MagicBridge] Power broadcast: system entering suspend / sleep");
                    if let Some(ref h) = data.hook {
                        h.set_active(false);
                    }
                }
                PBT_APMRESUMEAUTOMATIC | PBT_APMRESUMESUSPEND => {
                    println!(
                        "[MagicBridge] Power broadcast: system resumed from suspend (event={event:#x})"
                    );
                    let devices = probe_connected_apple_keyboards();
                    data.state.sync_connected_devices(&devices);
                    data.update_hook_state();
                    data.poll_battery();
                    if let Some(ref mut tray) = data.tray {
                        tray.update(&data.state);
                    }
                }
                _ => {}
            }
            LRESULT(1)
        }
        WM_TIMER => {
            if wparam.0 == IDT_BATTERY_TIMER {
                data.poll_battery();
            }
            LRESULT(0)
        }
        WM_TRAY_CALLBACK => {
            let event = lparam.0 as u32;
            match event {
                WM_RBUTTONUP | WM_CONTEXTMENU_VAL => {
                    if let Some(ref tray) = data.tray {
                        tray.show_context_menu(&data.state);
                    }
                }
                WM_LBUTTONDBLCLK => {
                    launch_settings();
                }
                _ => {}
            }
            LRESULT(0)
        }
        crate::game_mode::WM_GAME_MODE_EVENT => {
            let is_game = crate::game_mode::is_foreground_game(&data.state.config);
            if is_game != data.state.game_mode_active {
                data.state.game_mode_active = is_game;
                crate::logger::log(
                    "INFO",
                    "game_mode",
                    &format!("Foreground game status changed: active={is_game}"),
                );
                data.update_hook_state();
                if data.state.config.osd_enabled
                    && let Some(ref osd) = data.osd
                {
                    osd.show_status(
                        data.state.enabled,
                        is_game,
                        data.state.config.osd_duration_ms,
                    );
                }
                if let Some(ref mut tray) = data.tray {
                    tray.update(&data.state);
                }
            }

            // M16: Per-Application Profile Auto-Switching
            if !is_game
                && !data.state.config.app_profiles.is_empty()
                && let Some(proc_name) = crate::game_mode::get_foreground_process_name()
                && let Some(target_profile) = data.state.config.profile_for_process(&proc_name)
            {
                let target_profile = target_profile.to_string();
                if data.state.config.active_profile != target_profile
                    && data.state.set_active_profile(&target_profile)
                {
                    data.recompile_active_table();
                    if data.state.config.osd_enabled
                        && let Some(ref osd) = data.osd
                    {
                        osd.show_profile(&target_profile, data.state.config.osd_duration_ms);
                    }
                    if let Some(ref mut tray) = data.tray {
                        tray.update(&data.state);
                    }
                }
            }
            LRESULT(0)
        }
        crate::ipc::WM_IPC_STATE_UPDATED => {
            data.recompile_active_table();
            crate::logger::set_enabled(data.state.config.debug_logging);
            data.sync_game_mode_tracker(hwnd);
            data.state.game_mode_active = crate::game_mode::is_foreground_game(&data.state.config);
            data.update_hook_state();
            if data.state.config.battery_poll_minutes != data.battery_timer_minutes {
                data.setup_battery_timer(hwnd);
            }
            data.poll_battery();
            if let Some(ref mut tray) = data.tray {
                tray.update(&data.state);
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            let cmd_id = wparam.0 & 0xFFFF;
            match cmd_id {
                IDM_TOGGLE_ENABLED => {
                    data.state.toggle_enabled();
                    data.update_hook_state();
                    if data.state.config.osd_enabled
                        && let Some(ref osd) = data.osd
                    {
                        osd.show_status(
                            data.state.enabled,
                            data.state.game_mode_active,
                            data.state.config.osd_duration_ms,
                        );
                    }
                    if let Some(ref mut tray) = data.tray {
                        tray.update(&data.state);
                    }
                }
                IDM_TOPROW_MEDIA_FIRST => {
                    data.state.set_top_row_mode(TopRowMode::MediaKeysFirst);
                    data.recompile_active_table();
                    if data.state.config.osd_enabled
                        && let Some(ref osd) = data.osd
                    {
                        osd.show_top_row_mode(
                            TopRowMode::MediaKeysFirst,
                            data.state.config.osd_duration_ms,
                        );
                    }
                    if let Some(ref mut tray) = data.tray {
                        tray.update(&data.state);
                    }
                }
                IDM_TOPROW_FKEYS_FIRST => {
                    data.state.set_top_row_mode(TopRowMode::FKeysFirst);
                    data.recompile_active_table();
                    if data.state.config.osd_enabled
                        && let Some(ref osd) = data.osd
                    {
                        osd.show_top_row_mode(
                            TopRowMode::FKeysFirst,
                            data.state.config.osd_duration_ms,
                        );
                    }
                    if let Some(ref mut tray) = data.tray {
                        tray.update(&data.state);
                    }
                }
                crate::tray::IDM_TOGGLE_TOPROW_MODE => {
                    let new_mode = match data.state.active_top_row_mode() {
                        TopRowMode::MediaKeysFirst => TopRowMode::FKeysFirst,
                        TopRowMode::FKeysFirst => TopRowMode::MediaKeysFirst,
                    };
                    data.state.set_top_row_mode(new_mode);
                    data.recompile_active_table();
                    if data.state.config.osd_enabled
                        && let Some(ref osd) = data.osd
                    {
                        osd.show_top_row_mode(new_mode, data.state.config.osd_duration_ms);
                    }
                    if let Some(ref mut tray) = data.tray {
                        tray.update(&data.state);
                    }
                }
                IDM_OPEN_SETTINGS => {
                    launch_settings();
                }
                IDM_START_WITH_WINDOWS => {
                    data.state.toggle_autostart();
                }
                IDM_QUIT => {
                    // SAFETY: DestroyWindow triggers WM_DESTROY and message loop shutdown.
                    let _ = unsafe { DestroyWindow(hwnd) };
                }
                other if other >= IDM_PROFILE_BASE => {
                    let idx = other - IDM_PROFILE_BASE;
                    let profiles = data.state.profile_names();
                    if let Some(name) = profiles.get(idx) {
                        data.state.set_active_profile(name);
                        data.recompile_active_table();
                        if data.state.config.osd_enabled
                            && let Some(ref osd) = data.osd
                        {
                            osd.show_profile(name, data.state.config.osd_duration_ms);
                        }
                        if let Some(ref mut tray) = data.tray {
                            tray.update(&data.state);
                        }
                    }
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            // Cancel periodic battery polling timer (F6.3).
            // SAFETY: KillTimer cancels IDT_BATTERY_TIMER for this window.
            unsafe {
                let _ = KillTimer(Some(hwnd), IDT_BATTERY_TIMER);
            }
            // Clean up device notification registration, hook, tray icon, and OSD before window is gone.
            data.hook.take();
            data.device_notify.take();
            data.tray.take();
            data.osd.take();
            // SAFETY: PostQuitMessage posts WM_QUIT to exit message loop cleanly.
            unsafe {
                PostQuitMessage(0);
            }
            LRESULT(0)
        }
        WM_NCDESTROY => {
            // Reset GWLP_USERDATA.
            // SAFETY: Setting GWLP_USERDATA to 0 so subsequent calls see null.
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
            // SAFETY: Reclaiming ownership of WindowData originally allocated via Box::into_raw.
            let _ = unsafe { Box::from_raw(state_ptr) };
            // SAFETY: Final non-client destruction.
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        _ if data.taskbar_created_msg != 0 && msg == data.taskbar_created_msg => {
            // Explorer restarted: re-add tray icon.
            if let Some(ref mut tray) = data.tray {
                tray.re_add(&data.state);
            }
            LRESULT(0)
        }
        _ => {
            // SAFETY: Forward all unhandled messages to DefWindowProcW.
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
    }
}

/// Runs the Win32 message pump until WM_QUIT is posted.
pub fn run_message_pump() -> i32 {
    let mut msg = MSG::default();
    loop {
        // SAFETY: GetMessageW retrieves messages from the calling thread's message queue.
        let ret = unsafe { GetMessageW(&mut msg, None, 0, 0) };
        if ret.0 <= 0 {
            // ret.0 == 0 indicates WM_QUIT; ret.0 == -1 indicates an error.
            break;
        }
        // SAFETY: Translates virtual-key messages into character messages.
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    msg.wParam.0 as i32
}

/// Attempts to spawn the settings application (`magicbridge.exe --settings`).
pub fn launch_settings() {
    if let Ok(exe_path) = std::env::current_exe() {
        let mut cmd = std::process::Command::new(&exe_path);
        cmd.arg("--settings");
        if cmd.spawn().is_ok() {
            return;
        }
    }

    // Fallback: look for magicbridge.exe on PATH or in current directory
    let mut command = std::process::Command::new("magicbridge.exe");
    command.arg("--settings");
    let _ = command.spawn();
}

#[cfg(test)]
mod tests {
    use super::*;
    use magicbridge_core::Config;

    #[test]
    fn message_window_create_and_destroy() {
        let config = Config::default();
        let state = DaemonState::new(config, None);

        let window =
            MessageWindow::create(state).expect("Creating hidden message window should succeed");
        assert!(!window.hwnd().is_invalid());

        drop(window);
    }

    #[test]
    fn message_window_handles_power_broadcast_and_device_change() {
        use windows::Win32::UI::WindowsAndMessaging::SendMessageW;

        let config = Config::default();
        let state = DaemonState::new(config, None);

        let window =
            MessageWindow::create(state).expect("Creating hidden message window should succeed");

        // SAFETY: Sending test messages to our own hidden test window on this thread.
        unsafe {
            let res_suspend = SendMessageW(
                window.hwnd(),
                WM_POWERBROADCAST,
                Some(WPARAM(PBT_APMSUSPEND as usize)),
                Some(LPARAM(0)),
            );
            assert_eq!(res_suspend.0, 1);

            let res_resume = SendMessageW(
                window.hwnd(),
                WM_POWERBROADCAST,
                Some(WPARAM(PBT_APMRESUMEAUTOMATIC as usize)),
                Some(LPARAM(0)),
            );
            assert_eq!(res_resume.0, 1);

            let res_devchange = SendMessageW(
                window.hwnd(),
                WM_DEVICECHANGE,
                Some(WPARAM(0)),
                Some(LPARAM(0)),
            );
            assert_eq!(res_devchange.0, 1);
        }

        drop(window);
    }
}
