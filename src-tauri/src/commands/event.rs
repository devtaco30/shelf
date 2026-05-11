use crate::db::events::{self, Event};

#[tauri::command]
pub fn get_events() -> Result<Vec<Event>, String> {
    events::get_all().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_event(
    title: String,
    start_at: String,
    end_at: Option<String>,
    recurrence: String,
    category: String,
) -> Result<i64, String> {
    events::create(&title, &start_at, end_at.as_deref(), &recurrence, &category)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_event(id: i64) -> Result<(), String> {
    events::delete(id).map_err(|e| e.to_string())
}
