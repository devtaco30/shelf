//! 메인 웹뷰와 분리된 **새 태스크 전용 창** — 컴팩트 패널·WKWebView 클립과 무관하게 OS 창 단위로 폼을 띄운다.

use crate::db::todos;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Mutex;
use tauri::webview::WebviewWindowBuilder;
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WindowEvent};
use tauri::window::Color;

const SHELF_TASK_ADD_WEBVIEW_LABEL: &str = "shelf-task-add";

/// `task-add-window.html` 이 로드 직후 한 번만 읽어 가는 부트스트랩 JSON.
pub struct ShelfTaskAddBootstrap(pub Mutex<Option<Value>>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShelfTaskAddFormDone {
    pub cancelled: bool,
    #[serde(default)]
    pub created_id: Option<i64>,
}

#[tauri::command]
pub fn shelf_consume_task_add_bootstrap(
    state: State<'_, ShelfTaskAddBootstrap>,
) -> Result<Option<Value>, String> {
    Ok(state.0.lock().map_err(|e| e.to_string())?.take())
}

/// 메인 창 오른쪽에 새 태스크 창을 연다. 이미 있으면 닫고 다시 연다.
#[tauri::command]
pub async fn shelf_open_task_add_form_window(
    app: AppHandle,
    state: State<'_, ShelfTaskAddBootstrap>,
    bootstrap: Value,
) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window(SHELF_TASK_ADD_WEBVIEW_LABEL) {
        let _ = existing.close();
        // 동일 라벨로 즉시 재생성할 때 이전 인스턴스 해제를 기다린다.
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
        SHELF_TASK_ADD_WEBVIEW_LABEL,
        WebviewUrl::App("task-add-window.html".into()),
    )
    .title("새 태스크")
    .inner_size(420.0, 460.0)
    .min_inner_size(340.0, 380.0)
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

/// 할 일 탭 이탈·앱 접기 등으로 부트스트랩 없이 창만 닫을 때 (완료 이벤트 없음).
#[tauri::command]
pub fn shelf_close_task_add_form_window(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window(SHELF_TASK_ADD_WEBVIEW_LABEL) {
        w.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// `task-add-window` 전용: DB 저장 → 메인에 `shelf-task-add-form-window-done` emit (**창은 닫지 않음** — 연속 추가 시 폼만 클라이언트에서 초기화).
/// 보조 웹뷰에서 `create_todo` 직후 두 번째 `invoke(shelf_finish_…)` 가 Rust까지 안 오는 환경이 있어 단일 커맨드로 통합.
#[tauri::command(rename_all = "snake_case")]
pub fn shelf_create_task_from_add_window(
    app: AppHandle,
    title: String,
    note: String,
    due_date: Option<String>,
    recurrence: String,
    category: String,
    priority: i64,
    project_id: i64,
) -> Result<i64, String> {
    eprintln!(
        "[Shelf][shelf_create_task_from_add_window] IPC 수신 title_len={} project_id={} priority={} due_date={:?}",
        title.len(),
        project_id,
        priority,
        due_date
    );
    let proj = if project_id > 0 { Some(project_id) } else { None };
    let id = todos::create(
        &title,
        &note,
        due_date.as_deref(),
        &recurrence,
        &category,
        priority,
        proj,
    )
    .map_err(|e| e.to_string())?;
    eprintln!("[Shelf][shelf_create_task_from_add_window] DB INSERT rowid={id}");

    let payload = ShelfTaskAddFormDone {
        cancelled: false,
        created_id: Some(id),
    };
    let v = serde_json::to_value(&payload).map_err(|e| e.to_string())?;
    app.emit("shelf-task-add-form-window-done", v)
        .map_err(|e| e.to_string())?;
    eprintln!("[Shelf][shelf_create_task_from_add_window] emit OK → 메인 리로드 (창 유지)");

    Ok(id)
}

/// 보조 웹뷰에서 호출 — `async fn` 이지만 본문에 await 없음인 상태면 일부 환경에서 스케줄만 되고 실행 안 되는 사례가 있어 **동기 커맨드**로 둔다.
#[tauri::command]
pub fn shelf_finish_task_add_form_window(app: AppHandle, payload: ShelfTaskAddFormDone) -> Result<(), String> {
    eprintln!(
        "[Shelf][shelf_finish_task_add_form_window] cancelled={} created_id={:?}",
        payload.cancelled, payload.created_id
    );
    let v = serde_json::to_value(&payload).map_err(|e| e.to_string())?;
    // WebviewWindow::emit 만으로는 메인 리스너가 안 받는 환경이 있어 앱 전역 브로드캐스트 사용
    app.emit("shelf-task-add-form-window-done", v)
        .map_err(|e| e.to_string())?;
    eprintln!("[Shelf][shelf_finish_task_add_form_window] emit OK → 메인 index 리스너로 전달 예정");
    // invoke 응답이 JS로 돌아간 뒤 닫히도록 살짝 지연 (웹뷰 즉시 종료 시 IPC 완료 순서 꼬임 완화)
    let app_for_close = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(80));
        if let Some(w) = app_for_close.get_webview_window(SHELF_TASK_ADD_WEBVIEW_LABEL) {
            let _ = w.close();
        }
    });
    Ok(())
}
