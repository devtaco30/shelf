use crate::db::events::{self, Event};
use crate::recurrence::{next_occurrence, Recurrence};
use chrono::Local;

#[tauri::command]
pub fn get_events() -> Result<Vec<Event>, String> {
    let mut list = events::get_all().map_err(|e| e.to_string())?;
    let now = Local::now().naive_local();

    for ev in &mut list {
        if ev.recurrence == "none" {
            continue;
        }
        let r = Recurrence::from_str(&ev.recurrence);
        let from = ev
            .recurrence_next
            .as_deref()
            .or(Some(ev.start_at.as_str()))
            .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok());

        if let Some(from_dt) = from {
            if from_dt < now {
                if let Some(next) = next_occurrence(from_dt, &r, now) {
                    let next_str = next.format("%Y-%m-%d %H:%M:%S").to_string();
                    events::update_recurrence_next(ev.id, &next_str).ok();
                    ev.recurrence_next = Some(next_str);
                }
            }
        }
    }
    Ok(list)
}

#[tauri::command(rename_all = "snake_case")]
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
