mod db;
mod recurrence;

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
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("Tauri 실행 오류");
}
