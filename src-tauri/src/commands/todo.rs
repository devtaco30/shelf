use crate::db::todos::{self, Todo};
use crate::recurrence::{next_occurrence, Recurrence};
use chrono::Local;

#[tauri::command]
pub fn get_todos() -> Result<Vec<Todo>, String> {
    let mut todos = todos::get_all().map_err(|e| e.to_string())?;
    let now = Local::now().naive_local();

    for todo in &mut todos {
        if todo.done || todo.recurrence == "none" {
            continue;
        }
        let r = Recurrence::from_str(&todo.recurrence);
        let from = todo
            .recurrence_next
            .as_deref()
            .or(todo.due_date.as_deref())
            .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok());

        if let Some(from_dt) = from {
            if from_dt < now {
                if let Some(next) = next_occurrence(from_dt, &r, now) {
                    let next_str = next.format("%Y-%m-%d %H:%M:%S").to_string();
                    todos::update_recurrence_next(todo.id, &next_str).ok();
                    todo.recurrence_next = Some(next_str);
                }
            }
        }
    }
    Ok(todos)
}

#[tauri::command]
pub fn create_todo(
    title: String,
    note: String,
    due_date: Option<String>,
    recurrence: String,
    category: String,
    priority: i64,
    project_id: Option<i64>,
) -> Result<i64, String> {
    todos::create(&title, &note, due_date.as_deref(), &recurrence, &category, priority, project_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_todo(
    id: i64,
    title: String,
    note: String,
    due_date: Option<String>,
    category: String,
    priority: i64,
) -> Result<(), String> {
    todos::update(id, &title, &note, due_date.as_deref(), &category, priority)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_todo(id: i64, done: bool) -> Result<(), String> {
    todos::toggle_done(id, done).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_todo(id: i64) -> Result<(), String> {
    todos::delete(id).map_err(|e| e.to_string())
}
