//! MagicBridge background agent library.
//!
//! Provides the autostart manager, input hooks, HID battery reader,
//! tray interface, IPC daemon, game mode manager, and diagnostic logger.

pub mod action_worker;
pub mod autostart;
pub mod battery;
pub mod device_notify;
pub mod game_mode;
pub mod hook;
pub mod injector;
pub mod ipc;
pub mod logger;
pub mod osd;
pub mod single_instance;
pub mod state;
pub mod tray;
pub mod tray_icon;
pub mod window;

pub use action_worker::{ActionWorkerHandle, dispatch_action, start_action_worker};
pub use battery::{
    BatteryStatus, query_battery, read_battery_from_hid_path, read_bluetooth_pnp_battery,
};
pub use device_notify::{detect_connected_apple_model, probe_connected_apple_keyboards};
pub use game_mode::{
    GameModeTracker, WM_GAME_MODE_EVENT, get_foreground_process_name, is_foreground_game,
    is_fullscreen_rect, is_ignored_shell_class, matches_process_list,
};
pub use hook::{
    HookAction, HookHandle, KeyState, clear_active_table, decode_remap, encode_remap,
    get_active_custom_action, has_active_table, is_fn_down, is_hook_active, process_keyboard_event,
    release_all_active_keys, set_active_table, set_hook_active,
};
pub use injector::{build_keyboard_input, send_key_down, send_key_up, send_keyboard_event};
pub use ipc::{
    AtomicMappingTable, DaemonStatus, IpcClient, IpcError, IpcRequest, IpcResponse, IpcServer,
    IpcServerHandle,
};
pub use logger::{
    get_recent_logs, is_enabled as is_logging_enabled, log, set_enabled as set_logging_enabled,
};
pub use osd::OsdOverlay;

/// Runs the complete background daemon message pump, tray icon, and hook engine.
///
/// Returns 0 on clean exit or 1 on initialization error.
pub fn run_daemon() -> i32 {
    // 1. Single-instance guard (F8.3): exit cleanly if already running.
    let _guard = match single_instance::SingleInstanceGuard::acquire() {
        Ok(guard) => guard,
        Err(single_instance::SingleInstanceError::AlreadyRunning) => {
            // Already running: clean exit with code 0.
            return 0;
        }
        Err(e) => {
            eprintln!("Failed to acquire single instance guard: {e}");
            return 1;
        }
    };

    // 2. Load configuration from default path or use defaults.
    let config_path = magicbridge_core::default_config_path().ok();
    let config = match config_path.as_ref() {
        Some(path) => magicbridge_core::Config::load_or_default(path).unwrap_or_default(),
        None => magicbridge_core::Config::default(),
    };

    // Initialize diagnostic debug logger (NFR-Q2).
    let log_path = magicbridge_core::default_config_dir()
        .map(|d| d.join("debug.log"))
        .ok();
    logger::init(config.debug_logging, log_path);
    logger::log("INFO", "main", "MagicBridge Agent starting up");

    // 3. Initialize daemon state and atomic mapping table (Milestone 2.1 & 2.5).
    let state = state::DaemonState::new(config, config_path);
    let table = state.compile_active_profile();
    hook::set_active_table(table.clone());
    let atomic_table = ipc::AtomicMappingTable::new(table);
    let shared_state = std::sync::Arc::new(std::sync::Mutex::new(state.clone()));

    // Start background action worker for custom actions (F5.1..F5.2, NFR-P3).
    let _action_worker = action_worker::start_action_worker();

    // 4. Create hidden Win32 message window & system tray icon.
    let window = match window::MessageWindow::create(state, Some(shared_state.clone())) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Failed to create background message window: {e}");
            return 1;
        }
    };

    // 5. Start Named Pipe IPC server (\\.\pipe\MagicBridge_Agent_IPC).
    let _ipc_server = match ipc::IpcServer::start_default(
        shared_state,
        atomic_table,
        Some(ipc::NotifyHwnd(window.hwnd())),
    ) {
        Ok(srv) => Some(srv),
        Err(e) => {
            eprintln!("[MagicBridge] Warning: failed to start Named Pipe IPC server: {e}");
            None
        }
    };

    // 6. Run Win32 message pump (event-driven, 0% idle CPU per NFR-P2).
    window::run_message_pump()
}
