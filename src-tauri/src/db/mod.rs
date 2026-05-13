pub mod todos;
pub mod events;
pub mod vault;
pub mod settings;
pub mod projects;
pub mod memos;

use crate::vault::auth::{AuthProvider, PlatformAuthProvider};
use once_cell::sync::OnceCell;
use rand::RngCore;
use rusqlite::{Connection, Result};
use std::sync::Mutex;

pub static DB: OnceCell<Mutex<Connection>> = OnceCell::new();

pub fn init(app_dir: &str) -> Result<()> {
    let path = format!("{}/shelf.db", app_dir);
    let provider = PlatformAuthProvider;

    // DB 암호화 키: Keychain에서 로드, 없으면 최초 실행으로 판단해 랜덤 32바이트 생성
    let db_key: Vec<u8> = match provider.load_db_key() {
        Ok(k) => k,
        Err(_) => {
            let mut key = vec![0u8; 32];
            rand::thread_rng().fill_bytes(&mut key);
            provider
                .store_db_key(&key)
                .expect("DB 키 Keychain 저장 실패");
            key
        }
    };
    let hex_key = hex::encode(&db_key);

    let conn = if std::path::Path::new(&path).exists() {
        // 기존 DB: PRAGMA key 설정 후 테스트 쿼리로 암호화 여부 판단
        let conn = Connection::open(&path)?;
        conn.execute_batch(&format!("PRAGMA key = \"x'{hex_key}'\";"))?;
        match conn.query_row("SELECT count(*) FROM sqlite_master", [], |r| {
            r.get::<_, i64>(0)
        }) {
            Ok(_) => conn, // 이미 암호화된 DB
            Err(_) => {
                // 평문 DB → 마이그레이션
                drop(conn);
                migrate_plain_to_encrypted(&path, &hex_key)
                    .map_err(rusqlite::Error::InvalidParameterName)?
            }
        }
    } else {
        // 신규: 암호화 DB로 바로 생성
        let conn = Connection::open(&path)?;
        conn.execute_batch(&format!("PRAGMA key = \"x'{hex_key}'\";"))?;
        conn
    };

    run_migrations(&conn)?;
    DB.set(Mutex::new(conn)).ok();
    Ok(())
}

/// 평문 SQLite DB를 SQLCipher 암호화 DB로 변환한다.
/// 원본을 .plain_backup으로 이동 후 sqlcipher_export로 복사, 성공 시 백업 삭제.
fn migrate_plain_to_encrypted(path: &str, hex_key: &str) -> std::result::Result<Connection, String> {
    let backup_path = format!("{}.plain_backup", path);
    std::fs::rename(path, &backup_path).map_err(|e| format!("백업 이동 실패: {e}"))?;

    // 평문 DB 열기 (key 미설정 = 평문 모드)
    let plain_conn =
        Connection::open(&backup_path).map_err(|e| format!("평문 DB 열기 실패: {e}"))?;

    // ATTACH 문자열 내 작은따옴표 이스케이프 (SQL 단일 인용)
    let attach_path = path.replace('\'', "''");

    // ATTACH로 새 암호화 DB 연결 후 sqlcipher_export로 전체 복사
    plain_conn
        .execute_batch(&format!(
            "ATTACH DATABASE '{attach_path}' AS encrypted KEY \"x'{hex_key}'\"; \
             SELECT sqlcipher_export('encrypted'); \
             DETACH DATABASE encrypted;"
        ))
        .map_err(|e| format!("sqlcipher_export 실패: {e}"))?;
    drop(plain_conn);

    // 백업 삭제
    let _ = std::fs::remove_file(&backup_path);

    // 새 암호화 DB 반환
    let conn = Connection::open(path).map_err(|e| format!("암호화 DB 열기 실패: {e}"))?;
    conn
        .execute_batch(&format!("PRAGMA key = \"x'{hex_key}'\";"))
        .map_err(|e| format!("PRAGMA key 설정 실패: {e}"))?;
    Ok(conn)
}

/// `table_name`은 고정 식별자만 전달한다 (SQL 식별자 위치).
fn sqlite_table_has_column(conn: &Connection, table_name: &str, column_name: &str) -> Result<bool> {
    let sql = format!("SELECT COUNT(*) FROM pragma_table_info('{table_name}') WHERE name = ?");
    let n: i64 = conn.query_row(&sql, [column_name], |r| r.get(0))?;
    Ok(n > 0)
}

fn run_migrations(conn: &Connection) -> Result<()> {
    // 매 단계마다 DB에서 user_version을 다시 읽는다. 한 번만 읽으면 이전 블록이 올린 버전을
    // 반영하지 못해 동일 세션에서 중복 ALTER가 날 수 있다.
    loop {
        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version >= 6 {
            break;
        }

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
            continue;
        }

        if version < 2 {
            if !sqlite_table_has_column(conn, "todos", "project_id")? {
                conn.execute(
                    "ALTER TABLE todos ADD COLUMN project_id INTEGER REFERENCES projects(id)",
                    [],
                )?;
            }
            if !sqlite_table_has_column(conn, "todos", "category")? {
                conn.execute(
                    "ALTER TABLE todos ADD COLUMN category TEXT NOT NULL DEFAULT '작업'",
                    [],
                )?;
            }
            if !sqlite_table_has_column(conn, "events", "category")? {
                conn.execute(
                    "ALTER TABLE events ADD COLUMN category TEXT NOT NULL DEFAULT '작업'",
                    [],
                )?;
            }
            conn.execute_batch("PRAGMA user_version = 2")?;
            continue;
        }

        if version < 3 {
            if !sqlite_table_has_column(conn, "todos", "priority")? {
                conn.execute(
                    "ALTER TABLE todos ADD COLUMN priority INTEGER NOT NULL DEFAULT 0",
                    [],
                )?;
            }
            conn.execute_batch("PRAGMA user_version = 3")?;
            continue;
        }

        if version < 4 {
            if !sqlite_table_has_column(conn, "todos", "completed_at")? {
                conn.execute("ALTER TABLE todos ADD COLUMN completed_at TEXT", [])?;
            }
            conn.execute_batch("PRAGMA user_version = 4")?;
            continue;
        }

        if version < 5 {
            if !sqlite_table_has_column(conn, "vault_items", "item_type")? {
                conn.execute(
                    "ALTER TABLE vault_items ADD COLUMN item_type TEXT NOT NULL DEFAULT 'memo'",
                    [],
                )?;
            }
            conn.execute_batch("PRAGMA user_version = 5")?;
            continue;
        }

        if version < 6 {
            conn.execute_batch("
            CREATE TABLE IF NOT EXISTS memos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL DEFAULT '',
                body TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
            );
            CREATE INDEX IF NOT EXISTS idx_memos_updated_at ON memos(updated_at);
            PRAGMA user_version = 6;
        ")?;
            continue;
        }
    }

    Ok(())
}
