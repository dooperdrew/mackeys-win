//! MacKeys background agent entrypoint.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let exit_code = magicbridge_agent::run_daemon();
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
}
