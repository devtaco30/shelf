//! Shelf 메인 창과 분리된 **설정 전용 웹뷰 창** — 패널 반투명 미리보기 시 메인 UI와 겹치지 않도록 OS 창 단위로 분리한다.

use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WindowEvent};
use tauri::webview::WebviewWindowBuilder;

const SHELF_SETTINGS_WEBVIEW_LABEL: &str = "settings";

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShelfSettingsWindowDonePayload {
    pub confirmed: bool,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub translucent: Option<bool>,
    #[serde(default)]
    pub opacity: Option<f64>,
}

/// 메인 창 오른쪽(논리 좌표)에 설정 창을 띄운다. 이미 있으면 포커스만 옮긴다.
#[tauri::command]
pub async fn shelf_open_settings_window(app: AppHandle) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window(SHELF_SETTINGS_WEBVIEW_LABEL) {
        existing.show().map_err(|e| e.to_string())?;
        existing.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
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
        SHELF_SETTINGS_WEBVIEW_LABEL,
        WebviewUrl::App("settings.html".into()),
    )
    .title("Shelf 설정")
    .inner_size(400.0, 520.0)
    .min_inner_size(320.0, 380.0)
    .decorations(false)
    .transparent(true)
    .resizable(true)
    .position(logical_left, logical_top)
    .build()
    .map_err(|e| e.to_string())?;

    // 멀티 모니터·Mission Control 이후 배율/포커스가 바뀔 때 WebView가 비어 보이는 경우 완화
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

/// 설정 창에서 메인 웹뷰로만 이벤트를 보낸다(테마·패널 면 미리보기).
#[tauri::command]
pub fn shelf_emit_to_main_window(app: AppHandle, channel: String, payload: Value) -> Result<(), String> {
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| "main webview 없음".to_string())?;
    main
        .emit(&channel, payload)
        .map_err(|e| e.to_string())
}

/// 설정 창을 닫기 전에 메인에 완료 여부(및 적용 시 최종 값)를 알린다.
#[tauri::command]
pub async fn shelf_finish_settings_window(app: AppHandle, payload: ShelfSettingsWindowDonePayload) -> Result<(), String> {
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| "main webview 없음".to_string())?;
    let v = serde_json::to_value(&payload).map_err(|e| e.to_string())?;
    main.emit("shelf-settings-window-done", v)
        .map_err(|e| e.to_string())?;
    if let Some(w) = app.get_webview_window(SHELF_SETTINGS_WEBVIEW_LABEL) {
        w.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}
