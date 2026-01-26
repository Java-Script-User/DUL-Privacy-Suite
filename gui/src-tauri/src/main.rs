#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Tauri entry
    gui_lib::run()
}
