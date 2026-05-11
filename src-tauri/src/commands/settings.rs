use crate::db::settings;
use tauri::command;

#[command]
pub fn get_settings() -> Result<settings::Settings, String> {
    settings::get().map_err(|e| e.to_string())
}

#[command]
pub fn set_setting(key: String, value: String) -> Result<(), String> {
    settings::set(&key, &value).map_err(|e| e.to_string())
}
