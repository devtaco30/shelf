mod commands;
mod db;
mod recurrence;
mod vault;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("앱 데이터 경로를 찾을 수 없음")
                .to_str()
                .unwrap()
                .to_string();
            std::fs::create_dir_all(&app_dir).ok();
            db::init(&app_dir).expect("DB 초기화 실패");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::todo::get_todos,
            commands::todo::create_todo,
            commands::todo::toggle_todo,
            commands::todo::delete_todo,
            commands::event::get_events,
            commands::event::create_event,
            commands::event::delete_event,
            commands::vault::setup_vault,
            commands::vault::unlock_vault,
            commands::vault::lock_vault,
            commands::vault::get_vault_items,
            commands::vault::create_vault_item,
            commands::vault::get_vault_content,
            commands::vault::delete_vault_item,
            commands::vault::is_vault_unlocked,
            commands::vault::vault_initialized,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 실행 오류");
}
