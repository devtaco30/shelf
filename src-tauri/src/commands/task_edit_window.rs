//! 태스크 **수정** 전용 보조 창 — `task-add-window` 와 동일하게 **단일 IPC**로 저장·삭제·emit·창 닫기.

use crate::db::todos;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Mutex;
use tauri::webview::WebviewWindowBuilder;
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WindowEvent};

const SHELF_TASK_EDIT_WEBVIEW_LABEL: &str = "shelf-task-edit";

pub struct ShelfTaskEditBootstrap(pub Mutex<Option<Value>>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShelfTaskEditFormDone {
    pub cancelled: bool,
    #[serde(default)]
    pub todo_id: Option<i64>,
    #[serde(default)]
    pub deleted: Option<bool>,
}

#[tauri::command]
pub fn shelf_consume_task_edit_bootstrap(
    state: State<'_, ShelfTaskEditBootstrap>,
) -> Result<Option<Value>, String> {
    Ok(state.0.lock().map_err(|e| e.to_string())?.take())
}

#[tauri::command]
pub async fn shelf_open_task_edit_form_window(
    app: AppHandle,
    state: State<'_, ShelfTaskEditBootstrap>,
    bootstrap: Value,
) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window(SHELF_TASK_EDIT_WEBVIEW_LABEL) {
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
        SHELF_TASK_EDIT_WEBVIEW_LABEL,
        WebviewUrl::App("task-edit-window.html".into()),
    )
    .title("태스크 수정")
    .inner_size(420.0, 520.0)
    .min_inner_size(340.0, 400.0)
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
pub fn shelf_close_task_edit_form_window(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window(SHELF_TASK_EDIT_WEBVIEW_LABEL) {
        w.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// `delete_todo == true` 이면 나머지 필드는 무시하고 삭제만 수행.
#[tauri::command(rename_all = "snake_case")]
pub fn shelf_commit_task_edit_from_window(
    app: AppHandle,
    delete_todo: bool,
    todo_id: i64,
    title: String,
    note: String,
    due_date: Option<String>,
    category: String,
    priority: i64,
    project_id: i64,
) -> Result<(), String> {
    eprintln!(
        "[Shelf][shelf_commit_task_edit_from_window] delete_todo={delete_todo} todo_id={todo_id} title_len={}",
        title.len()
    );

    if delete_todo {
        todos::delete(todo_id).map_err(|e| e.to_string())?;
        eprintln!("[Shelf][shelf_commit_task_edit_from_window] DB DELETE id={todo_id}");
    } else {
        todos::update(
            todo_id,
            &title,
            &note,
            due_date.as_deref(),
            &category,
            priority,
        )
        .map_err(|e| e.to_string())?;
        let proj = if project_id > 0 {
            Some(project_id)
        } else {
            None
        };
        todos::set_project(todo_id, proj).map_err(|e| e.to_string())?;
        eprintln!(
            "[Shelf][shelf_commit_task_edit_from_window] DB UPDATE+project id={todo_id} project_id={project_id}"
        );
    }

    let payload = ShelfTaskEditFormDone {
        cancelled: false,
        todo_id: Some(todo_id),
        deleted: Some(delete_todo),
    };
    let v = serde_json::to_value(&payload).map_err(|e| e.to_string())?;
    app.emit("shelf-task-edit-form-window-done", v)
        .map_err(|e| e.to_string())?;
    eprintln!("[Shelf][shelf_commit_task_edit_from_window] emit OK → 메인 리로드 예정");

    let app_for_close = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(80));
        if let Some(w) = app_for_close.get_webview_window(SHELF_TASK_EDIT_WEBVIEW_LABEL) {
            let _ = w.close();
        }
    });
    Ok(())
}
