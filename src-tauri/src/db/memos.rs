use crate::db::DB;
use rusqlite::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Memo {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
}

/// 목록·검색용: 제목 + 본문 앞부분 미리보기
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoSummary {
    pub id: i64,
    pub title: String,
    pub preview: String,
    pub updated_at: String,
}

fn preview_sql_expr() -> &'static str {
    "substr(replace(replace(ifnull(body,''), char(10), ' '), char(13), ' '), 1, 120)"
}

pub fn list_summaries(search: Option<&str>) -> Result<Vec<MemoSummary>> {
    let conn = DB.get().unwrap().lock().unwrap();
    let q = search.map(str::trim).filter(|s| !s.is_empty());
    let preview = preview_sql_expr();
    let rows = if let Some(needle) = q {
        let sql = format!(
            "SELECT id, title, {preview} AS preview, updated_at
             FROM memos
             WHERE title LIKE '%' || ?1 || '%' OR body LIKE '%' || ?1 || '%'
             ORDER BY updated_at DESC"
        );
        let mut stmt = conn.prepare(&sql)?;
        let mapped = stmt.query_map(rusqlite::params![needle], |row| {
            Ok(MemoSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                preview: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?;
        mapped.collect::<Result<Vec<_>>>()?
    } else {
        let sql = format!(
            "SELECT id, title, {preview} AS preview, updated_at
             FROM memos
             ORDER BY updated_at DESC"
        );
        let mut stmt = conn.prepare(&sql)?;
        let mapped = stmt.query_map([], |row| {
            Ok(MemoSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                preview: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?;
        mapped.collect::<Result<Vec<_>>>()?
    };
    Ok(rows)
}

pub fn count_all() -> Result<i64> {
    let conn = DB.get().unwrap().lock().unwrap();
    let n: i64 = conn.query_row("SELECT COUNT(*) FROM memos", [], |r| r.get(0))?;
    Ok(n)
}

pub fn get_by_id(id: i64) -> Result<Memo> {
    let conn = DB.get().unwrap().lock().unwrap();
    conn.query_row(
        "SELECT id, title, body, created_at, updated_at FROM memos WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(Memo {
                id: row.get(0)?,
                title: row.get(1)?,
                body: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        },
    )
}

pub fn create(title: &str, body: &str) -> Result<i64> {
    let conn = DB.get().unwrap().lock().unwrap();
    conn.execute(
        "INSERT INTO memos (title, body) VALUES (?1, ?2)",
        rusqlite::params![title, body],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update(id: i64, title: &str, body: &str) -> Result<()> {
    let conn = DB.get().unwrap().lock().unwrap();
    let n = conn.execute(
        "UPDATE memos SET title = ?1, body = ?2, updated_at = datetime('now','localtime') WHERE id = ?3",
        rusqlite::params![title, body, id],
    )?;
    if n == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    Ok(())
}

pub fn delete(id: i64) -> Result<()> {
    let conn = DB.get().unwrap().lock().unwrap();
    let n = conn.execute("DELETE FROM memos WHERE id = ?1", rusqlite::params![id])?;
    if n == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    Ok(())
}
