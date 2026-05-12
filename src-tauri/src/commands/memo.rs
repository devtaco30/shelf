use crate::db::memos::{self, Memo, MemoSummary};

#[tauri::command(rename_all = "snake_case")]
pub fn get_memos(query: Option<String>) -> Result<Vec<MemoSummary>, String> {
    let q = query.as_deref();
    memos::list_summaries(q).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_memo(id: i64) -> Result<Memo, String> {
    memos::get_by_id(id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_memo_count() -> Result<i64, String> {
    memos::count_all().map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn create_memo(title: String, body: String) -> Result<i64, String> {
    memos::create(&title, &body).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_memo(id: i64, title: String, body: String) -> Result<(), String> {
    memos::update(id, &title, &body).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_memo(id: i64) -> Result<(), String> {
    memos::delete(id).map_err(|e| e.to_string())
}
