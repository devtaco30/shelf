mod commands;
mod db;
mod recurrence;
mod vault;

use tauri::Emitter;
use tauri::Manager;

#[cfg(target_os = "macos")]
fn shelf_macos_append_settings_menu_item(handle: &tauri::AppHandle) -> Result<(), String> {
    use tauri::menu::{Menu, MenuItem, MenuItemKind};

    let menu = Menu::default(handle).map_err(|e| e.to_string())?;
    let items = menu.items().map_err(|e| e.to_string())?;
    let Some(MenuItemKind::Submenu(app_menu)) = items.first() else {
        menu.set_as_app_menu().map_err(|e| e.to_string())?;
        return Ok(());
    };
    let settings_item = MenuItem::with_id(
        handle,
        "open-settings",
        "설정...",
        true,
        Some("CmdOrCtrl+,"),
    )
    .map_err(|e| e.to_string())?;
    let n = app_menu.items().map_err(|e| e.to_string())?.len();
    let insert_pos = n.saturating_sub(1);
    app_menu
        .insert(&settings_item, insert_pos)
        .map_err(|e| e.to_string())?;
    menu.set_as_app_menu().map_err(|e| e.to_string())?;
    Ok(())
}

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

            app.manage(commands::task_add_window::ShelfTaskAddBootstrap(std::sync::Mutex::new(
                None,
            )));
            app.manage(commands::task_edit_window::ShelfTaskEditBootstrap(std::sync::Mutex::new(
                None,
            )));
            app.manage(commands::vault_add_window::ShelfVaultFormBootstrap(
                std::sync::Mutex::new(None),
            ));
            app.manage(commands::memo_add_window::ShelfMemoFormBootstrap(std::sync::Mutex::new(
                None,
            )));

            #[cfg(target_os = "macos")]
            if let Err(e) = shelf_macos_append_settings_menu_item(app.handle()) {
                eprintln!("[Shelf] 앱 메뉴(설정…) 추가 실패: {}", e);
            }

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
        .on_menu_event(|app, event| {
            #[cfg(target_os = "macos")]
            if event.id() == "open-settings" {
                let _ = app.emit("open-settings", ());
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::todo::get_todos,
            commands::todo::create_todo,
            commands::todo::update_todo,
            commands::todo::set_todo_project,
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
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::memo::get_memos,
            commands::memo::get_memo,
            commands::memo::get_memo_count,
            commands::memo::create_memo,
            commands::memo::update_memo,
            commands::memo::delete_memo,
            get_platform,
            commands::settings_window::shelf_open_settings_window,
            commands::settings_window::shelf_emit_to_main_window,
            commands::settings_window::shelf_finish_settings_window,
            commands::task_add_window::shelf_consume_task_add_bootstrap,
            commands::task_add_window::shelf_open_task_add_form_window,
            commands::task_add_window::shelf_close_task_add_form_window,
            commands::task_add_window::shelf_create_task_from_add_window,
            commands::task_add_window::shelf_finish_task_add_form_window,
            commands::task_edit_window::shelf_consume_task_edit_bootstrap,
            commands::task_edit_window::shelf_open_task_edit_form_window,
            commands::task_edit_window::shelf_close_task_edit_form_window,
            commands::task_edit_window::shelf_commit_task_edit_from_window,
            commands::vault_add_window::shelf_consume_vault_form_bootstrap,
            commands::vault_add_window::shelf_open_vault_form_window,
            commands::vault_add_window::shelf_close_vault_form_window,
            commands::vault_add_window::shelf_create_vault_item_from_window,
            commands::vault_add_window::shelf_commit_vault_edit_from_window,
            commands::vault_add_window::shelf_finish_vault_form_window,
            commands::memo_add_window::shelf_consume_memo_form_bootstrap,
            commands::memo_add_window::shelf_open_memo_form_window,
            commands::memo_add_window::shelf_finish_memo_form_window,
            commands::memo_add_window::shelf_memo_form_trace,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 실행 오류");
}
