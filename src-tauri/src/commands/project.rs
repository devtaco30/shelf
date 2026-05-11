use crate::db::projects;
use tauri::command;

#[command]
pub fn get_projects() -> Result<Vec<projects::Project>, String> {
    projects::get_all().map_err(|e| e.to_string())
}

#[command]
pub fn create_project(
    name: String,
    color: String,
    category: String,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<i64, String> {
    projects::create(&name, &color, &category, start_date.as_deref(), end_date.as_deref())
        .map_err(|e| e.to_string())
}

#[command]
pub fn update_project(
    id: i64,
    name: String,
    color: String,
    category: String,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<(), String> {
    projects::update(id, &name, &color, &category, start_date.as_deref(), end_date.as_deref())
        .map_err(|e| e.to_string())
}

#[command]
pub fn delete_project(id: i64) -> Result<(), String> {
    projects::delete(id).map_err(|e| e.to_string())
}

#[command]
pub fn archive_project(id: i64) -> Result<(), String> {
    projects::archive(id).map_err(|e| e.to_string())
}
