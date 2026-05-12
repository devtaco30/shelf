use crate::db::settings;

#[tauri::command]
pub fn get_settings() -> Result<settings::Settings, String> {
    settings::get().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting(key: String, value: String) -> Result<(), String> {
    settings::set(&key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_setting(key: String) -> Result<Option<String>, String> {
    settings::get_value(&key).map_err(|e| e.to_string())
}
