use crate::db::DB;
use rusqlite::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub id: i64,
    pub title: String,
    pub start_at: String,
    pub end_at: Option<String>,
    pub recurrence: String,
    pub todo_id: Option<i64>,
    pub created_at: String,
}

pub fn get_all() -> Result<Vec<Event>> {
    let conn = DB.get().unwrap().lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, title, start_at, end_at, recurrence, todo_id, created_at FROM events ORDER BY start_at",
    )?;
    let events = stmt
        .query_map([], |row| {
            Ok(Event {
                id: row.get(0)?,
                title: row.get(1)?,
                start_at: row.get(2)?,
                end_at: row.get(3)?,
                recurrence: row.get(4)?,
                todo_id: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(events)
}

pub fn create(title: &str, start_at: &str, end_at: Option<&str>, recurrence: &str) -> Result<i64> {
    let conn = DB.get().unwrap().lock().unwrap();
    conn.execute(
        "INSERT INTO events (title, start_at, end_at, recurrence) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![title, start_at, end_at, recurrence],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn delete(id: i64) -> Result<()> {
    let conn = DB.get().unwrap().lock().unwrap();
    conn.execute("DELETE FROM events WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}
