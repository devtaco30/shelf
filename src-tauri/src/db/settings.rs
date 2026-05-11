use crate::db::DB;
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub name: String,
    pub bio: String,
}

fn get_with(conn: &Connection) -> Result<Settings> {
    let mut name = String::from("나");
    let mut bio  = String::from("나의 작업 대시보드");
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    for row in rows {
        let (key, value) = row?;
        match key.as_str() {
            "name" => name = value,
            "bio"  => bio  = value,
            _      => {}
        }
    }
    Ok(Settings { name, bio })
}

fn set_with(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

pub fn get() -> Result<Settings> {
    let conn = DB.get().unwrap().lock().unwrap();
    get_with(&conn)
}

pub fn set(key: &str, value: &str) -> Result<()> {
    let conn = DB.get().unwrap().lock().unwrap();
    set_with(&conn, key, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("
            CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            INSERT INTO settings (key, value) VALUES ('name', '나');
            INSERT INTO settings (key, value) VALUES ('bio', '나의 작업 대시보드');
        ").unwrap();
        conn
    }

    #[test]
    fn get_returns_defaults() {
        let conn = setup();
        let s = get_with(&conn).unwrap();
        assert_eq!(s.name, "나");
        assert_eq!(s.bio,  "나의 작업 대시보드");
    }

    #[test]
    fn set_updates_value() {
        let conn = setup();
        set_with(&conn, "name", "Jack").unwrap();
        let s = get_with(&conn).unwrap();
        assert_eq!(s.name, "Jack");
    }

    #[test]
    fn set_upserts_on_duplicate() {
        let conn = setup();
        set_with(&conn, "name", "First").unwrap();
        set_with(&conn, "name", "Second").unwrap();
        let s = get_with(&conn).unwrap();
        assert_eq!(s.name, "Second");
    }
}
