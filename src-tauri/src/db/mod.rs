pub mod todos;
pub mod events;
pub mod vault;
pub mod settings;
pub mod projects;

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
    let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;

    if version < 1 {
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
            CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            INSERT OR IGNORE INTO settings (key, value) VALUES ('name', '나');
            INSERT OR IGNORE INTO settings (key, value) VALUES ('bio', '나의 작업 대시보드');
            CREATE TABLE IF NOT EXISTS projects (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                name       TEXT    NOT NULL,
                color      TEXT    NOT NULL DEFAULT '#7C3AED',
                category   TEXT    NOT NULL DEFAULT '프로젝트',
                start_date TEXT,
                end_date   TEXT,
                archived   INTEGER NOT NULL DEFAULT 0,
                created_at TEXT    NOT NULL DEFAULT (datetime('now'))
            );
            PRAGMA user_version = 1;
        ")?;
    }

    if version < 2 {
        conn.execute_batch("
            ALTER TABLE todos  ADD COLUMN project_id INTEGER REFERENCES projects(id);
            ALTER TABLE todos  ADD COLUMN category   TEXT NOT NULL DEFAULT '작업';
            ALTER TABLE events ADD COLUMN category   TEXT NOT NULL DEFAULT '작업';
            PRAGMA user_version = 2;
        ")?;
    }

    if version < 3 {
        conn.execute_batch("
            ALTER TABLE todos ADD COLUMN priority INTEGER NOT NULL DEFAULT 0;
            PRAGMA user_version = 3;
        ")?;
    }

    Ok(())
}
