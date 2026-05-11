use crate::db::DB;
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub category: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub archived: bool,
    pub created_at: String,
}

fn get_all_with(conn: &Connection) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, color, category, start_date, end_date, archived, created_at
         FROM projects WHERE archived = 0 ORDER BY created_at DESC",
    )?;
    let items = stmt.query_map([], |row| {
        Ok(Project {
            id:         row.get(0)?,
            name:       row.get(1)?,
            color:      row.get(2)?,
            category:   row.get(3)?,
            start_date: row.get(4)?,
            end_date:   row.get(5)?,
            archived:   row.get::<_, i64>(6)? != 0,
            created_at: row.get(7)?,
        })
    })?.collect::<Result<Vec<_>>>()?;
    Ok(items)
}

fn create_with(
    conn: &Connection,
    name: &str,
    color: &str,
    category: &str,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO projects (name, color, category, start_date, end_date)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![name, color, category, start_date, end_date],
    )?;
    Ok(conn.last_insert_rowid())
}

fn update_with(
    conn: &Connection,
    id: i64,
    name: &str,
    color: &str,
    category: &str,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<()> {
    conn.execute(
        "UPDATE projects SET name=?1, color=?2, category=?3, start_date=?4, end_date=?5 WHERE id=?6",
        rusqlite::params![name, color, category, start_date, end_date, id],
    )?;
    Ok(())
}

fn delete_with(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM projects WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}

fn archive_with(conn: &Connection, id: i64) -> Result<()> {
    conn.execute(
        "UPDATE projects SET archived = 1 WHERE id = ?1",
        rusqlite::params![id],
    )?;
    Ok(())
}

pub fn get_all() -> Result<Vec<Project>> {
    let conn = DB.get().unwrap().lock().unwrap();
    get_all_with(&conn)
}

pub fn create(name: &str, color: &str, category: &str, start_date: Option<&str>, end_date: Option<&str>) -> Result<i64> {
    let conn = DB.get().unwrap().lock().unwrap();
    create_with(&conn, name, color, category, start_date, end_date)
}

pub fn update(id: i64, name: &str, color: &str, category: &str, start_date: Option<&str>, end_date: Option<&str>) -> Result<()> {
    let conn = DB.get().unwrap().lock().unwrap();
    update_with(&conn, id, name, color, category, start_date, end_date)
}

pub fn delete(id: i64) -> Result<()> {
    let conn = DB.get().unwrap().lock().unwrap();
    delete_with(&conn, id)
}

pub fn archive(id: i64) -> Result<()> {
    let conn = DB.get().unwrap().lock().unwrap();
    archive_with(&conn, id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("
            CREATE TABLE projects (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                name       TEXT    NOT NULL,
                color      TEXT    NOT NULL DEFAULT '#7C3AED',
                category   TEXT    NOT NULL DEFAULT '프로젝트',
                start_date TEXT,
                end_date   TEXT,
                archived   INTEGER NOT NULL DEFAULT 0,
                created_at TEXT    NOT NULL DEFAULT (datetime('now'))
            );
        ").unwrap();
        conn
    }

    #[test]
    fn create_and_get() {
        let conn = setup();
        let id = create_with(&conn, "테스트", "#7C3AED", "프로젝트", None, None).unwrap();
        assert!(id > 0);
        let list = get_all_with(&conn).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "테스트");
        assert!(!list[0].archived);
    }

    #[test]
    fn update_project() {
        let conn = setup();
        let id = create_with(&conn, "원래", "#7C3AED", "프로젝트", None, None).unwrap();
        update_with(&conn, id, "새 이름", "#10B981", "작업", Some("2026-05-01"), Some("2026-05-31")).unwrap();
        let list = get_all_with(&conn).unwrap();
        assert_eq!(list[0].name, "새 이름");
        assert_eq!(list[0].start_date, Some("2026-05-01".to_string()));
    }

    #[test]
    fn archive_hides_from_list() {
        let conn = setup();
        let id = create_with(&conn, "대상", "#7C3AED", "프로젝트", None, None).unwrap();
        archive_with(&conn, id).unwrap();
        let list = get_all_with(&conn).unwrap();
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn delete_project() {
        let conn = setup();
        let id = create_with(&conn, "삭제", "#7C3AED", "프로젝트", None, None).unwrap();
        delete_with(&conn, id).unwrap();
        assert_eq!(get_all_with(&conn).unwrap().len(), 0);
    }
}
