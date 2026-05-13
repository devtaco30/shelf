//! 메인 앱 종료·메인 창 닫기 직전에 보조 웹뷰를 정리한다.

use tauri::{AppHandle, Manager};

use super::settings_window::SHELF_SETTINGS_WEBVIEW_LABEL;

const SHELF_TASK_ADD_WEBVIEW_LABEL: &str = "shelf-task-add";
const SHELF_TASK_EDIT_WEBVIEW_LABEL: &str = "shelf-task-edit";
const SHELF_VAULT_FORM_WEBVIEW_LABEL: &str = "shelf-vault-form";
const SHELF_MEMO_FORM_WEBVIEW_LABEL: &str = "shelf-memo-form";

/// `settings`·태스크 추가/수정·Vault·메모 보조 웹뷰가 떠 있으면 닫는다. 오류는 무시하고 다음 라벨로 진행한다.
pub fn close_all_secondary_webviews_impl(app: &AppHandle) {
    for label in [
        SHELF_SETTINGS_WEBVIEW_LABEL,
        SHELF_TASK_ADD_WEBVIEW_LABEL,
        SHELF_TASK_EDIT_WEBVIEW_LABEL,
        SHELF_VAULT_FORM_WEBVIEW_LABEL,
        SHELF_MEMO_FORM_WEBVIEW_LABEL,
    ] {
        if let Some(w) = app.get_webview_window(label) {
            let _ = w.close();
        }
    }
}

#[tauri::command]
pub fn shelf_close_all_secondary_webviews(app: AppHandle) -> Result<(), String> {
    close_all_secondary_webviews_impl(&app);
    Ok(())
}
