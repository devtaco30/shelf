# Shelf v2 Phase 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 3단계 창 상태(pill/panel/expanded) UI 재설계 + projects/settings DB + category 시스템 구현

**Architecture:** Pill(52px 좌측 고정) + CompactPanel(슬라이드) + ExpandedDashboard(풀 대시보드) 세 컴포넌트가 `windowState` store 값에 따라 `+layout.svelte`에서 조합됨. Rust 백엔드에 projects CRUD + settings key-value 추가. DB는 `PRAGMA user_version`으로 마이그레이션 버전 관리.

**Tech Stack:** Tauri v2, SvelteKit (Svelte 4/5 혼용), Rust + rusqlite (bundled), SQLite

---

## File Map

| 작업 | 파일 |
|------|------|
| **Create** | `src-tauri/src/db/settings.rs` |
| **Create** | `src-tauri/src/db/projects.rs` |
| **Create** | `src-tauri/src/commands/project.rs` |
| **Create** | `src-tauri/src/commands/settings.rs` |
| **Create** | `src/lib/stores/projects.ts` |
| **Create** | `src/lib/components/Pill.svelte` |
| **Create** | `src/lib/components/CompactPanel.svelte` |
| **Create** | `src/lib/components/dashboard/ExpandedDashboard.svelte` |
| **Create** | `src/lib/components/dashboard/DashboardHeader.svelte` |
| **Create** | `src/lib/components/dashboard/TodayTasks.svelte` |
| **Modify** | `src-tauri/tauri.conf.json` — 초기 너비 52px |
| **Modify** | `src-tauri/src/db/mod.rs` — user_version 마이그레이션 |
| **Modify** | `src-tauri/src/db/todos.rs` — category, project_id 필드 |
| **Modify** | `src-tauri/src/db/events.rs` — category 필드 |
| **Modify** | `src-tauri/src/commands/mod.rs` — 신규 모듈 등록 |
| **Modify** | `src-tauri/src/commands/todo.rs` — category 파라미터 |
| **Modify** | `src-tauri/src/commands/event.rs` — category 파라미터 |
| **Modify** | `src-tauri/src/lib.rs` — 신규 커맨드 등록 |
| **Modify** | `src/lib/stores/window.ts` — 3상태 관리 재작성 |
| **Modify** | `src/lib/stores/todos.ts` — category, project_id 필드 |
| **Modify** | `src/lib/stores/events.ts` — category 필드 |
| **Modify** | `src/routes/+layout.svelte` — Pill + CompactPanel + ExpandedDashboard |
| **Modify** | `src/lib/components/todo/TodoForm.svelte` — 카테고리 드롭다운 |
| **Delete** | `src/lib/components/Sidebar.svelte` |

---

### Task 1: tauri.conf.json 초기 창 크기 조정

**Files:**
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: windows[0] 너비를 pill 크기로 변경**

`src-tauri/tauri.conf.json`의 `"app" > "windows"[0]` 섹션을 아래로 교체:

```json
{
  "title": "Shelf",
  "width": 52,
  "height": 600,
  "minWidth": 52,
  "minHeight": 400,
  "alwaysOnTop": true,
  "decorations": false,
  "transparent": true,
  "resizable": true,
  "x": 100,
  "y": 100
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/tauri.conf.json
git commit -m "chore: 초기 창 너비 pill 크기(52px)로 변경"
```

---

### Task 2: DB 마이그레이션 v1 + v2 (`db/mod.rs`)

**Files:**
- Modify: `src-tauri/src/db/mod.rs`

- [ ] **Step 1: db/mod.rs 전체 교체**

```rust
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

    Ok(())
}
```

- [ ] **Step 2: 컴파일 확인 (settings/projects 모듈 없어서 에러 예상)**

```bash
cd src-tauri && cargo build 2>&1 | grep "^error"
```

Expected: `settings`, `projects` 모듈 파일 없음 에러. Task 3에서 해결.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/db/mod.rs
git commit -m "feat: DB 마이그레이션 버전 관리 — settings/projects 테이블 추가"
```

---

### Task 3: `db/settings.rs` + `db/projects.rs`

**Files:**
- Create: `src-tauri/src/db/settings.rs`
- Create: `src-tauri/src/db/projects.rs`

- [ ] **Step 1: db/settings.rs 작성**

```rust
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
```

- [ ] **Step 2: db/projects.rs 작성**

```rust
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
```

- [ ] **Step 3: 테스트 실행**

```bash
cd src-tauri && cargo test db::settings::tests db::projects::tests -- --nocapture 2>&1
```

Expected: 7개 테스트 모두 `test ... ok`

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/db/settings.rs src-tauri/src/db/projects.rs
git commit -m "feat: db/settings.rs, db/projects.rs — CRUD + 유닛 테스트"
```

---

### Task 4: `db/todos.rs`, `db/events.rs` category 필드 추가

**Files:**
- Modify: `src-tauri/src/db/todos.rs`
- Modify: `src-tauri/src/db/events.rs`

- [ ] **Step 1: db/todos.rs 전체 교체**

```rust
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
```

- [ ] **Step 2: db/events.rs 전체 교체**

```rust
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
    pub recurrence_next: Option<String>,
    pub todo_id: Option<i64>,
    pub category: String,
    pub created_at: String,
}

pub fn get_all() -> Result<Vec<Event>> {
    let conn = DB.get().unwrap().lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, title, start_at, end_at, recurrence, recurrence_next,
                todo_id, category, created_at
         FROM events ORDER BY start_at ASC",
    )?;
    let events = stmt.query_map([], |row| {
        Ok(Event {
            id:              row.get(0)?,
            title:           row.get(1)?,
            start_at:        row.get(2)?,
            end_at:          row.get(3)?,
            recurrence:      row.get(4)?,
            recurrence_next: row.get(5)?,
            todo_id:         row.get(6)?,
            category:        row.get(7)?,
            created_at:      row.get(8)?,
        })
    })?.collect::<Result<Vec<_>>>()?;
    Ok(events)
}

pub fn create(title: &str, start_at: &str, end_at: Option<&str>, recurrence: &str, category: &str) -> Result<i64> {
    let conn = DB.get().unwrap().lock().unwrap();
    conn.execute(
        "INSERT INTO events (title, start_at, end_at, recurrence, category)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![title, start_at, end_at, recurrence, category],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn delete(id: i64) -> Result<()> {
    let conn = DB.get().unwrap().lock().unwrap();
    conn.execute("DELETE FROM events WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}

pub fn update_recurrence_next(id: i64, next: &str) -> Result<()> {
    let conn = DB.get().unwrap().lock().unwrap();
    conn.execute(
        "UPDATE events SET recurrence_next = ?1 WHERE id = ?2",
        rusqlite::params![next, id],
    )?;
    Ok(())
}
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/db/todos.rs src-tauri/src/db/events.rs
git commit -m "feat: todos/events DB 구조체에 category, project_id 필드 추가"
```

---

### Task 5: Rust 커맨드 추가 및 수정

**Files:**
- Create: `src-tauri/src/commands/project.rs`
- Create: `src-tauri/src/commands/settings.rs`
- Modify: `src-tauri/src/commands/todo.rs`
- Modify: `src-tauri/src/commands/event.rs`

- [ ] **Step 1: commands/project.rs 작성**

```rust
use crate::db::projects;
use tauri::command;

#[command]
pub fn get_projects() -> Result<Vec<projects::Project>, String> {
    projects::get_all().map_err(|e| e.to_string())
}

#[command]
pub fn create_project(
    name: String,
    color: String,
    category: String,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<i64, String> {
    projects::create(&name, &color, &category, start_date.as_deref(), end_date.as_deref())
        .map_err(|e| e.to_string())
}

#[command]
pub fn update_project(
    id: i64,
    name: String,
    color: String,
    category: String,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<(), String> {
    projects::update(id, &name, &color, &category, start_date.as_deref(), end_date.as_deref())
        .map_err(|e| e.to_string())
}

#[command]
pub fn delete_project(id: i64) -> Result<(), String> {
    projects::delete(id).map_err(|e| e.to_string())
}

#[command]
pub fn archive_project(id: i64) -> Result<(), String> {
    projects::archive(id).map_err(|e| e.to_string())
}
```

- [ ] **Step 2: commands/settings.rs 작성**

```rust
use crate::db::settings;
use tauri::command;

#[command]
pub fn get_settings() -> Result<settings::Settings, String> {
    settings::get().map_err(|e| e.to_string())
}

#[command]
pub fn set_setting(key: String, value: String) -> Result<(), String> {
    settings::set(&key, &value).map_err(|e| e.to_string())
}
```

- [ ] **Step 3: commands/todo.rs — create_todo, update_todo에 category 추가**

`create_todo` 함수 교체:

```rust
#[tauri::command]
pub fn create_todo(
    title: String,
    note: String,
    due_date: Option<String>,
    recurrence: String,
    category: String,
) -> Result<i64, String> {
    todos::create(&title, &note, due_date.as_deref(), &recurrence, &category)
        .map_err(|e| e.to_string())
}
```

`update_todo` 함수 교체:

```rust
#[tauri::command]
pub fn update_todo(
    id: i64,
    title: String,
    note: String,
    due_date: Option<String>,
    category: String,
) -> Result<(), String> {
    todos::update(id, &title, &note, due_date.as_deref(), &category)
        .map_err(|e| e.to_string())
}
```

`get_todos`, `toggle_todo`, `delete_todo` 함수는 변경 없음.

- [ ] **Step 4: commands/event.rs — create_event에 category 추가**

`create_event` 함수 교체:

```rust
#[tauri::command]
pub fn create_event(
    title: String,
    start_at: String,
    end_at: Option<String>,
    recurrence: String,
    category: String,
) -> Result<i64, String> {
    events::create(&title, &start_at, end_at.as_deref(), &recurrence, &category)
        .map_err(|e| e.to_string())
}
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/project.rs src-tauri/src/commands/settings.rs \
        src-tauri/src/commands/todo.rs src-tauri/src/commands/event.rs
git commit -m "feat: projects/settings 커맨드 신규, todo/event category 파라미터 추가"
```

---

### Task 6: `commands/mod.rs` + `lib.rs` 커맨드 등록

**Files:**
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: commands/mod.rs에 신규 모듈 추가**

```rust
pub mod todo;
pub mod event;
pub mod vault;
pub mod project;
pub mod settings;
```

- [ ] **Step 2: lib.rs에 get_platform 커맨드 추가 + invoke_handler 갱신**

`src-tauri/src/lib.rs`에서 `pub fn run()` 위에 추가:

```rust
#[tauri::command]
fn get_platform() -> String {
    std::env::consts::OS.to_string()
}
```

`invoke_handler` 내 신규 커맨드 추가:

```rust
.invoke_handler(tauri::generate_handler![
    commands::todo::get_todos,
    commands::todo::create_todo,
    commands::todo::update_todo,
    commands::todo::toggle_todo,
    commands::todo::delete_todo,
    commands::event::get_events,
    commands::event::create_event,
    commands::event::delete_event,
    commands::vault::setup_vault,
    commands::vault::unlock_vault,
    commands::vault::lock_vault,
    commands::vault::get_vault_items,
    commands::vault::create_vault_item,
    commands::vault::update_vault_item,
    commands::vault::get_vault_content,
    commands::vault::delete_vault_item,
    commands::vault::is_vault_unlocked,
    commands::vault::vault_initialized,
    // 신규
    commands::project::get_projects,
    commands::project::create_project,
    commands::project::update_project,
    commands::project::delete_project,
    commands::project::archive_project,
    commands::settings::get_settings,
    commands::settings::set_setting,
    get_platform,
])
```

- [ ] **Step 3: 빌드 확인**

```bash
cd src-tauri && cargo build 2>&1 | grep "^error"
```

Expected: 에러 없음

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: projects/settings/get_platform Tauri 커맨드 등록"
```

---

### Task 7: Frontend 스토어 업데이트

**Files:**
- Modify: `src/lib/stores/window.ts`
- Modify: `src/lib/stores/todos.ts`
- Modify: `src/lib/stores/events.ts`
- Create: `src/lib/stores/projects.ts`

- [ ] **Step 1: window.ts 전체 재작성**

```typescript
import { writable, get } from 'svelte/store';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';

export type WindowState = 'pill' | 'panel' | 'expanded';
export type Tab = 'todo' | 'cal' | 'vault';

const PILL_W     = 52;
const PANEL_W    = 332;  // 52 + 280
const EXPANDED_W = 612;  // 52 + 560
const HEIGHT     = 600;

export const windowState = writable<WindowState>('pill');
export const activeTab   = writable<Tab>('todo');

export async function setState(next: WindowState): Promise<void> {
  const widths: Record<WindowState, number> = {
    pill: PILL_W, panel: PANEL_W, expanded: EXPANDED_W,
  };
  await getCurrentWindow().setSize(new LogicalSize(widths[next], HEIGHT));
  windowState.set(next);
}

export async function openTab(tab: Tab): Promise<void> {
  const currentState = get(windowState);
  const currentTab   = get(activeTab);

  if (currentState === 'panel' && currentTab === tab) {
    await setState('pill');
    return;
  }

  activeTab.set(tab);
  await setState('panel');
}
```

- [ ] **Step 2: todos.ts — Todo 인터페이스 + createTodo/updateTodo 시그니처 수정**

```typescript
import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface Todo {
  id: number;
  title: string;
  note: string;
  done: boolean;
  due_date: string | null;
  recurrence: string;
  recurrence_next: string | null;
  project_id: number | null;
  category: string;
  created_at: string;
}

export const todos = writable<Todo[]>([]);

export async function loadTodos(): Promise<void> {
  const result = await invoke<Todo[]>('get_todos');
  todos.set(result);
}

export async function createTodo(
  title: string,
  note: string,
  due_date: string | null,
  recurrence: string,
  category: string = '작업',
): Promise<void> {
  await invoke('create_todo', { title, note, dueDate: due_date, recurrence, category });
  await loadTodos();
}

export async function updateTodo(
  id: number,
  title: string,
  note: string,
  due_date: string | null,
  category: string = '작업',
): Promise<void> {
  await invoke('update_todo', { id, title, note, dueDate: due_date, category });
  await loadTodos();
}

export async function toggleTodo(id: number, done: boolean): Promise<void> {
  await invoke('toggle_todo', { id, done });
  await loadTodos();
}

export async function deleteTodo(id: number): Promise<void> {
  await invoke('delete_todo', { id });
  await loadTodos();
}
```

- [ ] **Step 3: events.ts — CalEvent 인터페이스 + createEvent 시그니처 수정**

```typescript
import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface CalEvent {
  id: number;
  title: string;
  start_at: string;
  end_at: string | null;
  recurrence: string;
  recurrence_next: string | null;
  todo_id: number | null;
  category: string;
  created_at: string;
}

export const events = writable<CalEvent[]>([]);

export async function loadEvents(): Promise<void> {
  const result = await invoke<CalEvent[]>('get_events');
  events.set(result);
}

export async function createEvent(
  title: string,
  start_at: string,
  end_at: string | null,
  recurrence: string,
  category: string = '작업',
): Promise<void> {
  await invoke('create_event', { title, startAt: start_at, endAt: end_at, recurrence, category });
  await loadEvents();
}

export async function deleteEvent(id: number): Promise<void> {
  await invoke('delete_event', { id });
  await loadEvents();
}
```

- [ ] **Step 4: projects.ts 신규 작성**

```typescript
import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface Project {
  id: number;
  name: string;
  color: string;
  category: string;
  start_date: string | null;
  end_date: string | null;
  archived: boolean;
  created_at: string;
}

export const CATEGORY_COLORS: Record<string, { bg: string; text: string }> = {
  '작업':      { bg: '#E8F4FD', text: '#1A6FA8' },
  '클라이언트': { bg: '#FFF0F0', text: '#C0392B' },
  '개인':      { bg: '#F0FFF4', text: '#27AE60' },
  'work':      { bg: '#FFF8E1', text: '#E67E22' },
  '사일':      { bg: '#F3F0FF', text: '#6C5CE7' },
};

export const PROJECT_COLORS = [
  '#7C3AED', '#10B981', '#EF4444', '#F59E0B',
  '#06B6D4', '#EC4899', '#6B7280', '#4F46E5',
];

export const projects = writable<Project[]>([]);

export async function loadProjects(): Promise<void> {
  const result = await invoke<Project[]>('get_projects');
  projects.set(result);
}

export async function createProject(
  name: string,
  color: string,
  category: string,
  start_date: string | null,
  end_date: string | null,
): Promise<void> {
  await invoke('create_project', { name, color, category, startDate: start_date, endDate: end_date });
  await loadProjects();
}

export async function updateProject(
  id: number,
  name: string,
  color: string,
  category: string,
  start_date: string | null,
  end_date: string | null,
): Promise<void> {
  await invoke('update_project', { id, name, color, category, startDate: start_date, endDate: end_date });
  await loadProjects();
}

export async function deleteProject(id: number): Promise<void> {
  await invoke('delete_project', { id });
  await loadProjects();
}

export async function archiveProject(id: number): Promise<void> {
  await invoke('archive_project', { id });
  await loadProjects();
}
```

- [ ] **Step 5: 타입 체크**

```bash
npm run check 2>&1 | head -40
```

Expected: ExpandedDashboard 미생성으로 인한 에러만 남음. 나머지 스토어 에러 없음.

- [ ] **Step 6: Commit**

```bash
git add src/lib/stores/window.ts src/lib/stores/todos.ts \
        src/lib/stores/events.ts src/lib/stores/projects.ts
git commit -m "feat: window 3상태 store, projects store, category 필드 추가"
```

---

### Task 8: `Pill.svelte` 작성

**Files:**
- Create: `src/lib/components/Pill.svelte`

- [ ] **Step 1: Pill.svelte 작성**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { goto } from '$app/navigation';
  import { windowState, activeTab, openTab, setState } from '$lib/stores/window';
  import type { Tab } from '$lib/stores/window';

  const tabs: { id: Tab; icon: string; label: string }[] = [
    { id: 'todo',  icon: '📋', label: '할 일' },
    { id: 'cal',   icon: '📅', label: '캘린더' },
    { id: 'vault', icon: '🔒', label: 'Vault' },
  ];

  let platform = 'macos';

  async function handleTabClick(tab: Tab) {
    await openTab(tab);
    if ($windowState === 'panel') await goto(`/${tab}`);
  }

  async function closeWindow()    { await getCurrentWindow().close(); }
  async function minimizeWindow() { await getCurrentWindow().minimize(); }

  onMount(async () => {
    platform = await invoke('get_platform');
  });
</script>

<aside class="pill">
  {#if platform === 'macos'}
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="traffic-lights" on:mousedown|stopPropagation>
      <button class="dot close"    on:click={closeWindow}    title="닫기"></button>
      <button class="dot minimize" on:click={minimizeWindow} title="최소화"></button>
    </div>
  {:else}
    <button class="win-close" on:click={closeWindow}>×</button>
  {/if}

  <div class="tabs">
    {#each tabs as tab}
      <button
        class="tab-btn"
        class:active={$windowState === 'panel' && $activeTab === tab.id}
        on:click={() => handleTabClick(tab.id)}
        title={tab.label}
      >
        {tab.icon}
        {#if $windowState === 'panel' && $activeTab === tab.id}
          <span class="indicator"></span>
        {/if}
      </button>
    {/each}
  </div>

  <div class="divider"></div>

  <button class="expand-btn" on:click={() => setState('expanded')} title="확장">⬜</button>
</aside>

<style>
  .pill {
    width: 52px;
    background: #1E1E2E;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 10px 0 12px;
    gap: 4px;
    flex-shrink: 0;
    height: 100vh;
  }

  .traffic-lights {
    display: flex;
    flex-direction: column;
    gap: 5px;
    margin-bottom: 8px;
    padding-top: 2px;
  }

  .dot {
    width: 10px; height: 10px;
    border-radius: 50%; border: none; cursor: pointer; padding: 0;
  }
  .dot.close    { background: #FF5F57; }
  .dot.minimize { background: #FEBC2E; }

  .win-close {
    background: none; border: none;
    color: rgba(255,255,255,0.6); cursor: pointer;
    font-size: 14px; margin-bottom: 8px;
  }

  .tabs {
    display: flex; flex-direction: column;
    align-items: center; gap: 4px; flex: 1;
  }

  .tab-btn {
    width: 36px; height: 36px;
    border: none; border-radius: 10px;
    background: transparent; cursor: pointer;
    font-size: 18px; display: flex; align-items: center; justify-content: center;
    position: relative; transition: background 0.15s;
  }
  .tab-btn:hover  { background: rgba(255,255,255,0.1); }
  .tab-btn.active { background: rgba(255,255,255,0.15); }

  .indicator {
    position: absolute; right: -8px; top: 50%; transform: translateY(-50%);
    width: 3px; height: 18px; background: #AAED3A; border-radius: 2px;
  }

  .divider {
    width: 28px; height: 0.5px;
    background: rgba(255,255,255,0.08); margin: 4px 0;
  }

  .expand-btn {
    width: 28px; height: 28px;
    border: none; border-radius: 7px;
    background: rgba(255,255,255,0.06); cursor: pointer;
    font-size: 13px; color: rgba(255,255,255,0.5);
    display: flex; align-items: center; justify-content: center;
    transition: background 0.15s;
  }
  .expand-btn:hover { background: rgba(255,255,255,0.12); color: #fff; }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/components/Pill.svelte
git commit -m "feat: Pill.svelte — 좌측 52px 필 컴포넌트"
```

---

### Task 9: `CompactPanel.svelte` 작성

**Files:**
- Create: `src/lib/components/CompactPanel.svelte`

- [ ] **Step 1: CompactPanel.svelte 작성**

```svelte
<script lang="ts">
  import { activeTab, setState } from '$lib/stores/window';

  const TAB_TITLES: Record<string, string> = {
    todo: '할 일', cal: '캘린더', vault: 'Vault',
  };

  async function handleExpand() { await setState('expanded'); }
  async function handleClose()  { await setState('pill'); }
</script>

<div class="compact-panel">
  <header class="cp-header">
    <span class="cp-title">{TAB_TITLES[$activeTab] ?? '할 일'}</span>
    <div class="cp-actions">
      <button class="btn-expand" on:click={handleExpand}>⬜ 확장</button>
      <button class="btn-close"  on:click={handleClose}>×</button>
    </div>
  </header>

  <div class="cp-body">
    <slot />
  </div>
</div>

<style>
  .compact-panel {
    width: 280px; height: 100vh;
    background: #fff;
    display: flex; flex-direction: column; overflow: hidden;
  }

  .cp-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 11px 13px 0; flex-shrink: 0;
  }

  .cp-title { font-size: 13px; font-weight: 600; }

  .cp-actions { display: flex; align-items: center; gap: 6px; }

  .btn-expand {
    padding: 3px 8px; border-radius: 6px;
    background: #AAED3A; border: none;
    font-size: 10px; font-weight: 600; cursor: pointer; color: #111;
  }

  .btn-close {
    background: none; border: none; cursor: pointer;
    color: #bbb; font-size: 15px; line-height: 1;
  }
  .btn-close:hover { color: #333; }

  .cp-body {
    flex: 1; overflow-y: auto; padding: 10px 13px 13px;
  }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/components/CompactPanel.svelte
git commit -m "feat: CompactPanel.svelte — 280px 슬라이드 패널"
```

---

### Task 10: `+layout.svelte` 재설계 + `Sidebar.svelte` 삭제

**Files:**
- Modify: `src/routes/+layout.svelte`
- Delete: `src/lib/components/Sidebar.svelte`

- [ ] **Step 1: +layout.svelte 전체 교체**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { goto } from '$app/navigation';
  import { windowState } from '$lib/stores/window';
  import Pill from '$lib/components/Pill.svelte';
  import CompactPanel from '$lib/components/CompactPanel.svelte';
  import ExpandedDashboard from '$lib/components/dashboard/ExpandedDashboard.svelte';

  let { children } = $props();

  let vaultLockTimer: ReturnType<typeof setTimeout>;

  function resetVaultTimer(): void {
    clearTimeout(vaultLockTimer);
    vaultLockTimer = setTimeout(async () => {
      const { lockVault } = await import('$lib/stores/vault');
      await lockVault();
    }, 5 * 60 * 1000);
  }

  async function startDrag(e: MouseEvent): Promise<void> {
    if (e.button !== 0) return;
    await getCurrentWindow().startDragging();
  }

  onMount(async () => {
    await goto('/todo');
    window.addEventListener('mousemove', resetVaultTimer);
    window.addEventListener('keydown', resetVaultTimer);
    resetVaultTimer();
  });
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="app" onmousedown={startDrag}>
  <Pill />

  {#if $windowState === 'panel'}
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div onmousedown={(e) => e.stopPropagation()}>
      <CompactPanel>
        {@render children()}
      </CompactPanel>
    </div>
  {:else if $windowState === 'expanded'}
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div onmousedown={(e) => e.stopPropagation()}>
      <ExpandedDashboard />
    </div>
  {/if}
</div>

<style>
  :global(html), :global(body) {
    margin: 0;
    font-family: -apple-system, 'Apple SD Gothic Neo', sans-serif;
    background: transparent;
  }

  .app {
    display: flex;
    flex-direction: row;
    height: 100vh;
    border-radius: 12px;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0,0,0,0.2);
  }
</style>
```

- [ ] **Step 2: Sidebar.svelte 삭제**

```bash
git rm src/lib/components/Sidebar.svelte
```

- [ ] **Step 3: 타입 체크**

```bash
npm run check 2>&1 | head -30
```

Expected: 에러 없음 (ExpandedDashboard는 다음 Task에서 생성)

- [ ] **Step 4: Commit**

```bash
git add src/routes/+layout.svelte
git commit -m "feat: layout 재설계 — Pill + CompactPanel + ExpandedDashboard 조합"
```

---

### Task 11: Dashboard 컴포넌트 3종

**Files:**
- Create: `src/lib/components/dashboard/DashboardHeader.svelte`
- Create: `src/lib/components/dashboard/TodayTasks.svelte`
- Create: `src/lib/components/dashboard/ExpandedDashboard.svelte`

- [ ] **Step 1: DashboardHeader.svelte 작성**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { todos } from '$lib/stores/todos';
  import { events } from '$lib/stores/events';
  import { projects } from '$lib/stores/projects';
  import { setState } from '$lib/stores/window';

  const today        = new Date();
  const DAY_NAMES    = ['일','월','화','수','목','금','토'];
  const todayPrefix  = today.toISOString().slice(0, 10);
  const dateLabel    = `${today.getFullYear()}.${String(today.getMonth()+1).padStart(2,'0')}.${String(today.getDate()).padStart(2,'0')} ${DAY_NAMES[today.getDay()]}`;

  let profileName = '나';
  let profileBio  = '나의 작업 대시보드';

  $: activeProjectCount = $projects.length;
  $: todayEventCount    = $events.filter(e => e.start_at.startsWith(todayPrefix)).length;
  $: todayDoneCount     = $todos.filter(t => t.done && t.due_date?.startsWith(todayPrefix)).length;

  onMount(async () => {
    const s = await invoke<{ name: string; bio: string }>('get_settings');
    profileName = s.name;
    profileBio  = s.bio;
  });
</script>

<header class="dash-header">
  <div class="profile">
    <div class="avatar">{profileName.charAt(0)}</div>
    <div>
      <div class="profile-name">{profileName} · Shelf</div>
      <div class="profile-bio">{profileBio}</div>
    </div>
  </div>

  <div class="stats">
    <div class="stat"><div class="stat-n">{activeProjectCount}</div><div class="stat-l">진행 프로젝트</div></div>
    <div class="stat"><div class="stat-n">{todayEventCount}</div><div class="stat-l">오늘 일정</div></div>
    <div class="stat"><div class="stat-n">{todayDoneCount}</div><div class="stat-l">완료</div></div>
  </div>

  <span class="date">{dateLabel}</span>

  <div class="actions">
    <button class="btn-compact" on:click={() => setState('panel')}>▼ 컴팩트</button>
    <button class="btn-close"   on:click={() => setState('pill')}>×</button>
  </div>
</header>

<style>
  .dash-header {
    display: flex; align-items: center; gap: 10px;
    padding: 12px 16px; border-bottom: 0.5px solid #F0F0F0; flex-shrink: 0;
  }
  .profile { display: flex; align-items: center; gap: 8px; flex: 1; min-width: 0; }
  .avatar {
    width: 32px; height: 32px; border-radius: 50%;
    background: #111; color: #fff;
    display: flex; align-items: center; justify-content: center;
    font-size: 13px; font-weight: 600; flex-shrink: 0;
  }
  .profile-name { font-size: 13px; font-weight: 600; }
  .profile-bio  { font-size: 10px; color: #aaa; }
  .stats { display: flex; gap: 12px; }
  .stat   { text-align: center; }
  .stat-n { font-size: 18px; font-weight: 700; line-height: 1; }
  .stat-l { font-size: 9px; color: #aaa; }
  .date   { font-size: 11px; color: #aaa; white-space: nowrap; }
  .actions { display: flex; gap: 6px; align-items: center; }
  .btn-compact {
    padding: 4px 10px; border-radius: 6px;
    background: #F5F5F5; border: 0.5px solid #eee; font-size: 10px; cursor: pointer;
  }
  .btn-compact:hover { background: #eee; }
  .btn-close { background: none; border: none; cursor: pointer; color: #bbb; font-size: 16px; }
  .btn-close:hover { color: #333; }
</style>
```

- [ ] **Step 2: TodayTasks.svelte 작성**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { todos, loadTodos, createTodo, toggleTodo, deleteTodo } from '$lib/stores/todos';
  import { CATEGORY_COLORS } from '$lib/stores/projects';

  const CATEGORIES = ['작업', '클라이언트', '개인', 'work', '사일'];
  const todayPrefix = new Date().toISOString().slice(0, 10);

  $: todayTodos = $todos.filter(t => t.due_date?.startsWith(todayPrefix));

  let newTitle    = '';
  let newCategory = '작업';

  async function handleAdd() {
    if (!newTitle.trim()) return;
    await createTodo(newTitle.trim(), '', todayPrefix, 'none', newCategory);
    newTitle = '';
  }

  function badgeStyle(category: string): string {
    const c = CATEGORY_COLORS[category] ?? { bg: '#F5F5F5', text: '#666' };
    return `background:${c.bg};color:${c.text}`;
  }

  onMount(loadTodos);
</script>

<section class="today-tasks">
  <p class="section-title">오늘 할 일</p>

  <ul class="task-list">
    {#each todayTodos as todo (todo.id)}
      <li class="task-item">
        <button
          class="checkbox" class:done={todo.done}
          on:click={() => toggleTodo(todo.id, !todo.done)}
          aria-label={todo.done ? '완료 취소' : '완료'}
        ></button>
        <span class="task-title" class:done={todo.done}>{todo.title}</span>
        <span class="badge" style={badgeStyle(todo.category)}>{todo.category}</span>
        <button class="btn-del" on:click={() => deleteTodo(todo.id)} aria-label="삭제">×</button>
      </li>
    {/each}
  </ul>

  <div class="task-input">
    <select bind:value={newCategory} class="cat-select">
      {#each CATEGORIES as cat}<option value={cat}>{cat}</option>{/each}
    </select>
    <input
      bind:value={newTitle}
      placeholder="할 일 추가..."
      on:keydown={(e) => e.key === 'Enter' && handleAdd()}
      class="title-input"
    />
    <button on:click={handleAdd} class="btn-add">추가</button>
  </div>
</section>

<style>
  .today-tasks { padding: 12px 14px; border-top: 0.5px solid #F0F0F0; }
  .section-title { font-size: 11px; font-weight: 600; color: #aaa; margin: 0 0 7px; }
  .task-list { list-style: none; padding: 0; margin: 0 0 8px; }
  .task-item {
    display: flex; align-items: center; gap: 6px;
    padding: 5px 0; border-bottom: 0.5px solid #F0F0F0;
  }
  .task-item:last-child { border: none; }
  .checkbox {
    width: 14px; height: 14px; border-radius: 3px;
    border: 1.5px solid #ddd; background: transparent;
    cursor: pointer; flex-shrink: 0; padding: 0; transition: 0.15s;
  }
  .checkbox.done { background: #AAED3A; border-color: #AAED3A; }
  .task-title      { flex: 1; font-size: 12px; }
  .task-title.done { text-decoration: line-through; color: #bbb; }
  .badge { padding: 2px 7px; border-radius: 20px; font-size: 9px; font-weight: 600; white-space: nowrap; }
  .btn-del { background: none; border: none; cursor: pointer; color: #ccc; font-size: 14px; line-height: 1; }
  .btn-del:hover { color: #333; }
  .task-input { display: flex; gap: 5px; }
  .cat-select {
    border: 0.5px solid #eee; border-radius: 6px;
    padding: 4px 6px; font-size: 11px; background: #F7F7F7; outline: none;
  }
  .title-input {
    flex: 1; border: 0.5px solid #eee; border-radius: 6px;
    padding: 5px 8px; font-size: 11px; background: #F7F7F7; outline: none;
  }
  .btn-add {
    padding: 5px 10px; border-radius: 6px;
    background: #111; color: #fff; border: none; font-size: 11px; cursor: pointer;
  }
  .btn-add:hover { background: #333; }
</style>
```

- [ ] **Step 3: ExpandedDashboard.svelte 작성**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { loadTodos } from '$lib/stores/todos';
  import { loadEvents } from '$lib/stores/events';
  import { loadProjects } from '$lib/stores/projects';
  import DashboardHeader from './DashboardHeader.svelte';
  import TodayTasks from './TodayTasks.svelte';
  import CalendarView from '$lib/components/cal/CalendarView.svelte';

  onMount(async () => {
    await Promise.all([loadTodos(), loadEvents(), loadProjects()]);
  });
</script>

<div class="expanded">
  <DashboardHeader />

  <div class="ex-body">
    <div class="ex-left">
      <CalendarView />
    </div>
    <div class="ex-right">
      <!-- TodaySchedule — Phase 2에서 구현 -->
      <p class="placeholder">오늘 일정<br><span>Phase 2에서 추가됩니다</span></p>
    </div>
  </div>

  <TodayTasks />
</div>

<style>
  .expanded {
    width: 560px; height: 100vh; background: #fff;
    display: flex; flex-direction: column; overflow: hidden;
  }
  .ex-body {
    display: grid; grid-template-columns: 1fr 200px;
    flex: 1; overflow: hidden; border-bottom: 0.5px solid #F0F0F0;
  }
  .ex-left { padding: 14px; border-right: 0.5px solid #F0F0F0; overflow-y: auto; }
  .ex-right { padding: 14px; overflow-y: auto; }
  .placeholder { font-size: 11px; color: #aaa; text-align: center; padding-top: 40px; line-height: 2; }
  .placeholder span { font-size: 10px; color: #ccc; }
</style>
```

- [ ] **Step 4: 타입 체크**

```bash
npm run check 2>&1 | head -30
```

Expected: 에러 없음

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/dashboard/
git commit -m "feat: ExpandedDashboard, DashboardHeader, TodayTasks 컴포넌트"
```

---

### Task 12: `TodoForm.svelte` 카테고리 드롭다운 추가

**Files:**
- Modify: `src/lib/components/todo/TodoForm.svelte`

현재 파일: `createTodo(title, note, due_date, recurrence)` 4인수 호출, Svelte 4 문법.

- [ ] **Step 1: category 상태 변수 + 드롭다운 추가**

`<script>` 블록 상단의 `let` 변수들 아래에 추가:

```svelte
import { CATEGORY_COLORS } from '$lib/stores/projects';
const CATEGORIES = ['작업', '클라이언트', '개인', 'work', '사일'];
let category = '작업';
```

`submit()` 함수 내 `createTodo` 호출을 변경:

```svelte
// 변경 전:
await createTodo(title.trim(), note.trim(), combineDatetime(), recurrence);
// 변경 후:
await createTodo(title.trim(), note.trim(), combineDatetime(), recurrence, category);
```

`submit()` 의 리셋 줄에 `category = '작업'` 추가:

```svelte
title = ''; note = ''; date = ''; time = ''; useTime = false; recurrence = 'none'; category = '작업'; open = false;
```

`recurrence` select 아래에 카테고리 select 추가:

```svelte
<select bind:value={category}>
  {#each CATEGORIES as cat}
    <option value={cat}>{cat}</option>
  {/each}
</select>
```

- [ ] **Step 2: 타입 체크**

```bash
npm run check 2>&1 | head -20
```

Expected: 에러 없음

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/todo/TodoForm.svelte
git commit -m "feat: TodoForm에 카테고리 드롭다운 추가"
```

---

### Task 13: 통합 검증 (tauri dev + 수동)

- [ ] **Step 1: 개발 서버 실행**

```bash
npm run tauri dev
```

- [ ] **Step 2: Pill 상태 확인**

앱 실행 시 창 너비 52px. 확인 항목:
- traffic lights (빨강/노랑 닷) 보임
- 📋 📅 🔒 아이콘 3개 보임
- ⬜ 확장 버튼 하단 보임

- [ ] **Step 3: Panel 전환 확인**

- 📋 클릭 → 창 332px 확장, "할 일" 헤더 + 기존 TodoList 렌더
- 같은 아이콘 재클릭 → 창 52px 복귀
- 📅 클릭 → 캘린더 렌더, 📋 클릭 시 "할 일"로 전환
- `⬜ 확장` 버튼 클릭 → Expanded 상태 전환

- [ ] **Step 4: Expanded 상태 확인**

- 창 612px
- 헤더: 아바타 + 이름("나") + 통계 3개 + 날짜
- 좌측: 기존 CalendarView 렌더
- 우측: "Phase 2에서 추가됩니다" 플레이스홀더
- 하단: 오늘 할 일 섹션 + 카테고리 드롭다운 + 추가 버튼
- `▼ 컴팩트` → panel, `×` → pill 전환 확인

- [ ] **Step 5: DB 마이그레이션 확인**

```bash
sqlite3 ~/Library/Application\ Support/com.shelf.app/shelf.db \
  "PRAGMA user_version; SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;"
```

Expected:
```
2
events
projects
settings
todos
vault_items
```

- [ ] **Step 6: category 기능 확인**

- TodoForm에서 카테고리 선택 후 할 일 추가
- TodayTasks에 해당 카테고리 배지(색상) 표시 확인

- [ ] **Step 7: Commit**

```bash
git add .
git commit -m "chore: Phase 1 통합 검증 완료"
```
