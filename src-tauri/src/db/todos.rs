use crate::db::DB;
use rusqlite::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub note: String,
    pub done: bool,
    pub due_date: Option<String>,
    pub recurrence: String,
    pub recurrence_next: Option<String>,
    pub project_id: Option<i64>,
    pub category: String,
    pub created_at: String,
}

pub fn get_all() -> Result<Vec<Todo>> {
    let conn = DB.get().unwrap().lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, title, note, done, due_date, recurrence, recurrence_next,
                project_id, category, created_at
         FROM todos ORDER BY created_at DESC",
    )?;
    let todos = stmt.query_map([], |row| {
        Ok(Todo {
            id:               row.get(0)?,
            title:            row.get(1)?,
            note:             row.get(2)?,
            done:             row.get::<_, i64>(3)? != 0,
            due_date:         row.get(4)?,
            recurrence:       row.get(5)?,
            recurrence_next:  row.get(6)?,
            project_id:       row.get(7)?,
            category:         row.get(8)?,
            created_at:       row.get(9)?,
        })
    })?.collect::<Result<Vec<_>>>()?;
    Ok(todos)
}

pub fn create(title: &str, note: &str, due_date: Option<&str>, recurrence: &str, category: &str) -> Result<i64> {
    let conn = DB.get().unwrap().lock().unwrap();
    conn.execute(
        "INSERT INTO todos (title, note, due_date, recurrence, category)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![title, note, due_date, recurrence, category],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn toggle_done(id: i64, done: bool) -> Result<()> {
    let conn = DB.get().unwrap().lock().unwrap();
    conn.execute(
        "UPDATE todos SET done = ?1 WHERE id = ?2",
        rusqlite::params![done as i64, id],
    )?;
    Ok(())
}

pub fn delete(id: i64) -> Result<()> {
    let conn = DB.get().unwrap().lock().unwrap();
    conn.execute("DELETE FROM todos WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}

pub fn update(id: i64, title: &str, note: &str, due_date: Option<&str>, category: &str) -> Result<()> {
    let conn = DB.get().unwrap().lock().unwrap();
    conn.execute(
        "UPDATE todos SET title=?1, note=?2, due_date=?3, category=?4 WHERE id=?5",
        rusqlite::params![title, note, due_date, category, id],
    )?;
    Ok(())
}

pub fn update_recurrence_next(id: i64, next: &str) -> Result<()> {
    let conn = DB.get().unwrap().lock().unwrap();
    conn.execute(
        "UPDATE todos SET recurrence_next = ?1 WHERE id = ?2",
        rusqlite::params![next, id],
    )?;
    Ok(())
}
