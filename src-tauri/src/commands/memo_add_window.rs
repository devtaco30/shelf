//! 메인 웹뷰와 분리된 **메모 추가·수정 전용 창**

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Mutex;
use tauri::webview::WebviewWindowBuilder;
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WindowEvent};

const SHELF_MEMO_FORM_WEBVIEW_LABEL: &str = "shelf-memo-form";

pub struct ShelfMemoFormBootstrap(pub Mutex<Option<Value>>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShelfMemoFormDone {
    pub cancelled: bool,
    #[serde(default)]
    pub saved_memo_id: Option<i64>,
}

/// 보조 웹뷰 콘솔은 메인 DevTools에 안 보일 때가 많음 — `npm run tauri dev` 터미널로 동일 줄 출력
#[tauri::command]
pub fn shelf_memo_form_trace(line: String) -> Result<(), String> {
    eprintln!("[ShelfMemoFormWin] {}", line);
    Ok(())
}

#[tauri::command]
pub fn shelf_consume_memo_form_bootstrap(
    state: State<'_, ShelfMemoFormBootstrap>,
) -> Result<Option<Value>, String> {
    Ok(state.0.lock().map_err(|e| e.to_string())?.take())
}

#[tauri::command]
pub async fn shelf_open_memo_form_window(
    app: AppHandle,
    state: State<'_, ShelfMemoFormBootstrap>,
    bootstrap: Value,
) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window(SHELF_MEMO_FORM_WEBVIEW_LABEL) {
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
        SHELF_MEMO_FORM_WEBVIEW_LABEL,
        WebviewUrl::App("memo-add-window.html".into()),
    )
    .title("메모")
    .inner_size(440.0, 520.0)
    .min_inner_size(360.0, 420.0)
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

/// 메모 탭 이탈·앱 접기 등 — 완료 이벤트 없이 보조 창만 닫음 (`shelf-vault-form`과 동일 패턴).
#[tauri::command]
pub fn shelf_close_memo_form_window(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window(SHELF_MEMO_FORM_WEBVIEW_LABEL) {
        w.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn shelf_finish_memo_form_window(
    app: AppHandle,
    payload: ShelfMemoFormDone,
) -> Result<(), String> {
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| "main webview 없음".to_string())?;
    let v = serde_json::to_value(&payload).map_err(|e| e.to_string())?;
    main.emit("shelf-memo-form-window-done", v)
        .map_err(|e| e.to_string())?;
    if let Some(w) = app.get_webview_window(SHELF_MEMO_FORM_WEBVIEW_LABEL) {
        w.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}
