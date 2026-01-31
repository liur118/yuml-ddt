#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use yuml_ddt_lib::commands;
use yuml_ddt_lib::storage;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_yaml_files,
            commands::read_yaml_file,
            commands::save_yaml_file,
            commands::parse_yaml_steps,
            commands::execute_step,
            commands::get_step_list,
            // 存储相关命令
            storage::get_app_data,
            storage::save_app_data,
            storage::add_recent_workspace,
            storage::remove_recent_workspace,
            storage::clear_recent_workspaces,
            storage::get_recent_workspaces,
            storage::update_settings,
            storage::get_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
