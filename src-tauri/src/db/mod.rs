pub mod todos;
pub mod events;

use once_cell::sync::OnceCell;
use rusqlite::{Connection, Result};
use std::sync::Mutex;

pub static DB: OnceCell<Mutex<Connection>> = OnceCell::new();

pub fn init(app_dir: &str) -> Result<()> {
    let path = format!("{}/shelf.db", app_dir);
    let conn = Connection::open(&path)?;
    run_migrations(&conn)?;
    DB.set(Mutex::new(conn)).ok();
    Ok(())
}

fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            note TEXT DEFAULT '',
            done INTEGER DEFAULT 0,
            due_date TEXT,
            recurrence TEXT DEFAULT 'none',
            recurrence_next TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            start_at TEXT NOT NULL,
            end_at TEXT,
            recurrence TEXT DEFAULT 'none',
            recurrence_next TEXT,
            todo_id INTEGER REFERENCES todos(id),
            created_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS vault_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content BLOB NOT NULL,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now'))
        );
    ")?;
    Ok(())
}
