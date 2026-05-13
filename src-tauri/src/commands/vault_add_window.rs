//! 메인 웹뷰와 분리된 **보안 항목 추가·수정 전용 창** — Vault 폼을 OS 창 단위로 띄운다.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Mutex;
use tauri::webview::WebviewWindowBuilder;
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WindowEvent};

const SHELF_VAULT_FORM_WEBVIEW_LABEL: &str = "shelf-vault-form";

pub struct ShelfVaultFormBootstrap(pub Mutex<Option<Value>>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShelfVaultFormDone {
    pub cancelled: bool,
    /// 생성·수정 성공 시 해당 항목 id (취소면 None)
    #[serde(default)]
    pub saved_item_id: Option<i64>,
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

#[tauri::command]
pub async fn shelf_finish_vault_form_window(
    app: AppHandle,
    payload: ShelfVaultFormDone,
) -> Result<(), String> {
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| "main webview 없음".to_string())?;
    let v = serde_json::to_value(&payload).map_err(|e| e.to_string())?;
    main.emit("shelf-vault-form-window-done", v)
        .map_err(|e| e.to_string())?;
    if let Some(w) = app.get_webview_window(SHELF_VAULT_FORM_WEBVIEW_LABEL) {
        w.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}
