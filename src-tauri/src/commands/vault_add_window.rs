//! 메인 웹뷰와 분리된 **보안 항목 추가·수정 전용 창** — Vault 폼을 OS 창 단위로 띄운다.

use crate::commands::vault as vault_items_cmd;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Mutex;
use tauri::webview::WebviewWindowBuilder;
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WindowEvent};
use tauri::window::Color;

const SHELF_VAULT_FORM_WEBVIEW_LABEL: &str = "shelf-vault-form";

pub struct ShelfVaultFormBootstrap(pub Mutex<Option<Value>>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShelfVaultFormDone {
    pub cancelled: bool,
    /// 생성·수정 성공 시 해당 항목 id (취소면 None)
    #[serde(default)]
    pub saved_item_id: Option<i64>,
    #[serde(default)]
    pub deleted: Option<bool>,
}

#[tauri::command]
pub fn shelf_consume_vault_form_bootstrap(
    state: State<'_, ShelfVaultFormBootstrap>,
) -> Result<Option<Value>, String> {
    Ok(state.0.lock().map_err(|e| e.to_string())?.take())
}

#[tauri::command]
pub async fn shelf_open_vault_form_window(
    app: AppHandle,
    state: State<'_, ShelfVaultFormBootstrap>,
    bootstrap: Value,
) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window(SHELF_VAULT_FORM_WEBVIEW_LABEL) {
        let _ = existing.close();
        std::thread::sleep(std::time::Duration::from_millis(80));
    }

    {
        let mut slot = state.0.lock().map_err(|e| e.to_string())?;
        *slot = Some(bootstrap);
    }

    let main = app
        .get_webview_window("main")
        .ok_or_else(|| "main webview 없음".to_string())?;

    let outer_pos = main.outer_position().map_err(|e| e.to_string())?;
    let outer_size = main.outer_size().map_err(|e| e.to_string())?;
    let scale = main.scale_factor().map_err(|e| e.to_string())?;

    let margin_logical = 16.0_f64;
    let logical_left =
        outer_pos.x as f64 / scale + outer_size.width as f64 / scale + margin_logical;
    let logical_top = outer_pos.y as f64 / scale;

    let win = WebviewWindowBuilder::new(
        &app,
        SHELF_VAULT_FORM_WEBVIEW_LABEL,
        WebviewUrl::App("vault-add-window.html".into()),
    )
    .title("보안 항목")
    .inner_size(520.0, 620.0)
    .min_inner_size(400.0, 480.0)
    .decorations(false)
    .transparent(true)
    .background_color(Color(0, 0, 0, 0))
    .resizable(true)
    .position(logical_left, logical_top)
    .build()
    .map_err(|e| e.to_string())?;

    let win_for_events = win.clone();
    win.on_window_event(move |event| {
        if matches!(
            event,
            WindowEvent::ScaleFactorChanged { .. } | WindowEvent::Focused(true)
        ) {
            let _ = win_for_events.show();
        }
    });

    #[cfg(target_os = "macos")]
    {
        let _ = win.set_shadow(false);
    }

    Ok(())
}

/// Vault 탭 이탈·앱 접기 등 — 부트스트랩 없이 창만 닫을 때 (완료 이벤트 없음).
#[tauri::command]
pub fn shelf_close_vault_form_window(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window(SHELF_VAULT_FORM_WEBVIEW_LABEL) {
        w.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 신규 항목: DB 저장 → 메인에 `shelf-vault-form-window-done` emit (**창 유지** — 연속 추가).
#[tauri::command(rename_all = "snake_case")]
pub fn shelf_create_vault_item_from_window(
    app: AppHandle,
    title: String,
    content: String,
    item_type: String,
) -> Result<i64, String> {
    let id = vault_items_cmd::create_vault_item(title, content, item_type)?;
    let payload = ShelfVaultFormDone {
        cancelled: false,
        saved_item_id: Some(id),
        deleted: None,
    };
    let v = serde_json::to_value(&payload).map_err(|e| e.to_string())?;
    app.emit("shelf-vault-form-window-done", v)
        .map_err(|e| e.to_string())?;
    Ok(id)
}

/// 수정 반영 또는 삭제: 단일 IPC → emit → 지연 후 창 닫기 (`delete_vault_item == true` 면 삭제만 수행).
#[tauri::command(rename_all = "snake_case")]
pub fn shelf_commit_vault_edit_from_window(
    app: AppHandle,
    delete_vault_item: bool,
    id: i64,
    title: String,
    content: String,
) -> Result<(), String> {
    if delete_vault_item {
        vault_items_cmd::delete_vault_item(id)?;
    } else {
        vault_items_cmd::update_vault_item(id, title, content)?;
    }
    let payload = ShelfVaultFormDone {
        cancelled: false,
        saved_item_id: Some(id),
        deleted: Some(delete_vault_item),
    };
    let v = serde_json::to_value(&payload).map_err(|e| e.to_string())?;
    app.emit("shelf-vault-form-window-done", v)
        .map_err(|e| e.to_string())?;
    let app_for_close = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(80));
        if let Some(w) = app_for_close.get_webview_window(SHELF_VAULT_FORM_WEBVIEW_LABEL) {
            let _ = w.close();
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn shelf_finish_vault_form_window(
    app: AppHandle,
    payload: ShelfVaultFormDone,
) -> Result<(), String> {
    let v = serde_json::to_value(&payload).map_err(|e| e.to_string())?;
    app.emit("shelf-vault-form-window-done", v)
        .map_err(|e| e.to_string())?;
    if let Some(w) = app.get_webview_window(SHELF_VAULT_FORM_WEBVIEW_LABEL) {
        w.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}
