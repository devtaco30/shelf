mod commands;
mod db;
mod recurrence;
mod vault;

use tauri::Manager;

#[tauri::command]
fn get_platform() -> String {
    std::env::consts::OS.to_string()
}

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

            // macOS 창 그림자 제거 (transparent window에서 외곽 테두리 방지)
            #[cfg(target_os = "macos")]
            if let Some(window) = app.get_webview_window("main") {
                window.set_shadow(false).ok();
            }

            // 자정마다 반복 일정 체크 (get_todos 호출 시 자동 처리되므로 sleep만 담당)
            std::thread::spawn(|| {
                loop {
                    let now = chrono::Local::now();
                    let tomorrow = (now + chrono::Duration::days(1))
                        .date_naive()
                        .and_hms_opt(0, 0, 0)
                        .unwrap();
                    let secs_until_midnight =
                        (tomorrow - now.naive_local()).num_seconds();
                    std::thread::sleep(std::time::Duration::from_secs(
                        secs_until_midnight.max(0) as u64,
                    ));
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::todo::get_todos,
            commands::todo::create_todo,
            commands::todo::update_todo,
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
            commands::vault::update_vault_item,
            commands::vault::get_vault_content,
            commands::vault::delete_vault_item,
            commands::vault::is_vault_unlocked,
            commands::vault::vault_initialized,
            commands::project::get_projects,
            commands::project::create_project,
            commands::project::update_project,
            commands::project::delete_project,
            commands::project::archive_project,
            commands::settings::get_settings,
            commands::settings::set_setting,
            get_platform,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 실행 오류");
}
