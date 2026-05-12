use crate::db::DB;
use rusqlite::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultItem {
    pub id: i64,
    pub title: String,
    pub item_type: String,
    pub created_at: String,
    pub updated_at: String,
    // content는 복호화 후에만 반환 — DB에서 직접 노출하지 않는다
}

pub fn get_all() -> Result<Vec<VaultItem>> {
    let conn = DB.get().unwrap().lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, title, item_type, created_at, updated_at FROM vault_items ORDER BY updated_at DESC",
    )?;
    let items = stmt
        .query_map([], |row| {
            Ok(VaultItem {
                id: row.get(0)?,
                title: row.get(1)?,
                item_type: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(items)
}

pub fn create(title: &str, item_type: &str, encrypted_content: &[u8]) -> Result<i64> {
    let conn = DB.get().unwrap().lock().unwrap();
    conn.execute(
        "INSERT INTO vault_items (title, item_type, content) VALUES (?1, ?2, ?3)",
        rusqlite::params![title, item_type, encrypted_content],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_content(id: i64) -> Result<Vec<u8>> {
    let conn = DB.get().unwrap().lock().unwrap();
    conn.query_row(
        "SELECT content FROM vault_items WHERE id = ?1",
        rusqlite::params![id],
        |row| row.get(0),
    )
}

pub fn update(id: i64, title: &str, encrypted_content: &[u8]) -> Result<()> {
    let conn = DB.get().unwrap().lock().unwrap();
    conn.execute(
        "UPDATE vault_items SET title = ?1, content = ?2, updated_at = datetime('now') WHERE id = ?3",
        rusqlite::params![title, encrypted_content, id],
    )?;
    Ok(())
}

/// 제목 변경 없이 암호화된 내용만 교체한다 (마이그레이션 전용).
pub fn update_content(id: i64, encrypted_content: &[u8]) -> Result<()> {
    let conn = DB.get().unwrap().lock().unwrap();
    conn.execute(
        "UPDATE vault_items SET content = ?1 WHERE id = ?2",
        rusqlite::params![encrypted_content, id],
    )?;
    Ok(())
}

pub fn delete(id: i64) -> Result<()> {
    let conn = DB.get().unwrap().lock().unwrap();
    conn.execute("DELETE FROM vault_items WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}
