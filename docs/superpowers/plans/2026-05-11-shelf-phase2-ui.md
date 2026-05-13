# Shelf Phase 2 — UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement tab-specific compact panel content, tab-aware expanded sub-views, todo priority field, and add-task/add-project modals to match the Phase 2 mockup.

**Architecture:** CompactPanel becomes self-contained (no slot), rendering CompactTodo/CompactCal/CompactVault based on `$activeTab`. ExpandedDashboard switches between ExpandedTodo/ExpandedCalendar/ExpandedVault sub-views. Todos gain a `priority` INTEGER field via DB migration v3. Two modal components (AddTodoModal, AddProjectModal) handle creation flows with priority/project/color selection.

**Tech Stack:** Tauri v2, SvelteKit, Svelte 4 syntax (`$:`, `on:click`, `<slot>`) — **except `src/routes/+layout.svelte` which uses Svelte 5 runes** (`$props()`, `{@render children()}`). Rust/rusqlite, SQLite, TypeScript.

---

## Codebase Context

### Key patterns
- **Rust DB layer**: `*_with(conn)` private functions for testability, `OnceCell<Mutex<Connection>>` global DB
- **Tauri commands**: always `#[tauri::command]` (never `use tauri::command; #[command]`)
- **Svelte stores**: `writable`, load* functions call `invoke`, update store after every mutation
- **Window states**: `pill` (52px) → `panel` (332px) → `expanded` (612px) via `setState()` in `src/lib/stores/window.ts`
- **Active tab**: `$activeTab` store (values: `'todo' | 'cal' | 'vault'`)
- **DB migrations**: `PRAGMA user_version` in `src-tauri/src/db/mod.rs`, currently at v2

### Priority encoding
- `0` = none (no dot)
- `1` = low (blue `#4A9EFF`)
- `2` = medium (amber `#F59E0B`)
- `3` = high (red `#EF4444`)

### Existing files to NOT modify unless listed
- `src-tauri/src/db/projects.rs`, `events.rs`, `vault.rs`, `settings.rs`
- `src/lib/stores/events.ts`, `projects.ts`, `vault.ts`, `window.ts`
- `src/lib/components/vault/*`, `src/lib/components/cal/CalendarView.svelte`
- `src/lib/components/dashboard/DashboardHeader.svelte`

---

## File Structure

**New files to create:**
- `src/lib/components/compact/CompactTodo.svelte`
- `src/lib/components/compact/CompactCal.svelte`
- `src/lib/components/compact/CompactVault.svelte`
- `src/lib/components/dashboard/ExpandedTodo.svelte`
- `src/lib/components/dashboard/ExpandedCalendar.svelte`
- `src/lib/components/dashboard/ExpandedVault.svelte`
- `src/lib/components/modals/AddTodoModal.svelte`
- `src/lib/components/modals/AddProjectModal.svelte`

**Files to modify:**
- `src-tauri/src/db/mod.rs` — migration v3: `todos.priority`
- `src-tauri/src/db/todos.rs` — add `priority` to struct + queries
- `src-tauri/src/commands/todo.rs` — add `priority` param to `create_todo`, `update_todo`
- `src/lib/stores/todos.ts` — add `priority` to interface + function signatures
- `src/lib/components/CompactPanel.svelte` — replace `<slot>` with tab-based rendering
- `src/lib/components/dashboard/ExpandedDashboard.svelte` — tab-aware sub-view routing
- `src/routes/+layout.svelte` — remove `{@render children()}` from panel mode

---

## Task 1: DB Migration v3 — priority field for todos

**Files:**
- Modify: `src-tauri/src/db/mod.rs`
- Modify: `src-tauri/src/db/todos.rs`

### Step 1: Add migration v3 in `src-tauri/src/db/mod.rs`

Add the following block after the existing `if version < 2 { ... }` block (around line 79):

```rust
    if version < 3 {
        conn.execute_batch("
            ALTER TABLE todos ADD COLUMN priority INTEGER NOT NULL DEFAULT 0;
            PRAGMA user_version = 3;
        ")?;
    }
```

The full `run_migrations` function should end with:
```rust
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
```

### Step 2: Update Todo struct and queries in `src-tauri/src/db/todos.rs`

Replace the entire file with:

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
    pub priority: i64,
    pub created_at: String,
}

pub fn get_all() -> Result<Vec<Todo>> {
    let conn = DB.get().unwrap().lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, title, note, done, due_date, recurrence, recurrence_next,
                project_id, category, priority, created_at
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
            priority:         row.get(9)?,
            created_at:       row.get(10)?,
        })
    })?.collect::<Result<Vec<_>>>()?;
    Ok(todos)
}

pub fn create(
    title: &str,
    note: &str,
    due_date: Option<&str>,
    recurrence: &str,
    category: &str,
    priority: i64,
) -> Result<i64> {
    let conn = DB.get().unwrap().lock().unwrap();
    conn.execute(
        "INSERT INTO todos (title, note, due_date, recurrence, category, priority)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![title, note, due_date, recurrence, category, priority],
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

pub fn update(
    id: i64,
    title: &str,
    note: &str,
    due_date: Option<&str>,
    category: &str,
    priority: i64,
) -> Result<()> {
    let conn = DB.get().unwrap().lock().unwrap();
    conn.execute(
        "UPDATE todos SET title=?1, note=?2, due_date=?3, category=?4, priority=?5 WHERE id=?6",
        rusqlite::params![title, note, due_date, category, priority, id],
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

### Step 3: Build to verify

```bash
cd /Users/jack/Private/shelf/src-tauri && cargo check 2>&1 | head -40
```

Expected: no errors (warnings about unused variables are OK).

- [ ] Step 1: Add migration v3 block in `src-tauri/src/db/mod.rs`
- [ ] Step 2: Replace `src-tauri/src/db/todos.rs` with updated struct + queries
- [ ] Step 3: Run `cd /Users/jack/Private/shelf/src-tauri && cargo check`
- [ ] **Step 4: Commit**
```bash
git add src-tauri/src/db/mod.rs src-tauri/src/db/todos.rs
git commit -m "feat: DB 마이그레이션 v3 — todos.priority 필드 추가"
```

---

## Task 2: Rust command — priority in todo create/update

**Files:**
- Modify: `src-tauri/src/commands/todo.rs`

### Step 1: Update `create_todo` and `update_todo` to include `priority`

Replace the entire file with:

```rust
use crate::db::todos::{self, Todo};
use crate::recurrence::{next_occurrence, Recurrence};
use chrono::Local;

#[tauri::command]
pub fn get_todos() -> Result<Vec<Todo>, String> {
    let mut todos = todos::get_all().map_err(|e| e.to_string())?;
    let now = Local::now().naive_local();

    for todo in &mut todos {
        if todo.done || todo.recurrence == "none" {
            continue;
        }
        let r = Recurrence::from_str(&todo.recurrence);
        let from = todo
            .recurrence_next
            .as_deref()
            .or(todo.due_date.as_deref())
            .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok());

        if let Some(from_dt) = from {
            if from_dt < now {
                if let Some(next) = next_occurrence(from_dt, &r, now) {
                    let next_str = next.format("%Y-%m-%d %H:%M:%S").to_string();
                    todos::update_recurrence_next(todo.id, &next_str).ok();
                    todo.recurrence_next = Some(next_str);
                }
            }
        }
    }
    Ok(todos)
}

#[tauri::command]
pub fn create_todo(
    title: String,
    note: String,
    due_date: Option<String>,
    recurrence: String,
    category: String,
    priority: i64,
) -> Result<i64, String> {
    todos::create(&title, &note, due_date.as_deref(), &recurrence, &category, priority)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_todo(
    id: i64,
    title: String,
    note: String,
    due_date: Option<String>,
    category: String,
    priority: i64,
) -> Result<(), String> {
    todos::update(id, &title, &note, due_date.as_deref(), &category, priority)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_todo(id: i64, done: bool) -> Result<(), String> {
    todos::toggle_done(id, done).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_todo(id: i64) -> Result<(), String> {
    todos::delete(id).map_err(|e| e.to_string())
}
```

### Step 2: Build to verify

```bash
cd /Users/jack/Private/shelf/src-tauri && cargo check 2>&1 | head -40
```

Expected: no errors.

- [ ] Step 1: Replace `src-tauri/src/commands/todo.rs`
- [ ] Step 2: Run `cd /Users/jack/Private/shelf/src-tauri && cargo check`
- [ ] **Step 3: Commit**
```bash
git add src-tauri/src/commands/todo.rs
git commit -m "feat: create_todo/update_todo에 priority 파라미터 추가"
```

---

## Task 3: Frontend store — priority in todos

**Files:**
- Modify: `src/lib/stores/todos.ts`

### Step 1: Update `src/lib/stores/todos.ts`

Replace the entire file with:

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
  priority: number;
  created_at: string;
}

export const PRIORITY_COLORS: Record<number, string | null> = {
  0: null,
  1: '#4A9EFF',
  2: '#F59E0B',
  3: '#EF4444',
};

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
  priority: number = 0,
): Promise<void> {
  await invoke('create_todo', { title, note, dueDate: due_date, recurrence, category, priority });
  await loadTodos();
}

export async function updateTodo(
  id: number,
  title: string,
  note: string,
  due_date: string | null,
  category: string = '작업',
  priority: number = 0,
): Promise<void> {
  await invoke('update_todo', { id, title, note, dueDate: due_date, category, priority });
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

### Step 2: Verify TypeScript

```bash
cd /Users/jack/Private/shelf && npx tsc --noEmit 2>&1 | head -30
```

Expected: 0 errors.

- [ ] Step 1: Replace `src/lib/stores/todos.ts`
- [ ] Step 2: Run `npx tsc --noEmit`
- [ ] **Step 3: Commit**
```bash
git add src/lib/stores/todos.ts
git commit -m "feat: todos 스토어에 priority 필드 추가"
```

---

## Task 4: CompactPanel — tab-based self-contained rendering

**Files:**
- Modify: `src/lib/components/CompactPanel.svelte`
- Modify: `src/routes/+layout.svelte`

### Step 1: Replace `CompactPanel.svelte`

The new CompactPanel renders tab content directly based on `$activeTab` — no `<slot>` needed.
Note: CompactVault.svelte will be created in Task 7. Use a placeholder `<!-- vault -->` for now if needed. Actually since we'll create CompactVault right after this (Task 7), just import all three and implement them together. For this task, create placeholder `<p>로딩중...</p>` that will be replaced.

Actually, to avoid import errors, create the three compact components as empty stubs first, then flesh them out in Tasks 5-7. Create the stubs:

**Create `src/lib/components/compact/CompactTodo.svelte` stub:**
```svelte
<p style="padding:12px;font-size:12px;color:#aaa">할 일 (구현 예정)</p>
```

**Create `src/lib/components/compact/CompactCal.svelte` stub:**
```svelte
<p style="padding:12px;font-size:12px;color:#aaa">캘린더 (구현 예정)</p>
```

**Create `src/lib/components/compact/CompactVault.svelte` stub:**
```svelte
<p style="padding:12px;font-size:12px;color:#aaa">Vault (구현 예정)</p>
```

**Replace `src/lib/components/CompactPanel.svelte`:**
```svelte
<script lang="ts">
  import { activeTab, setState } from '$lib/stores/window';
  import type { Tab } from '$lib/stores/window';
  import CompactTodo  from './compact/CompactTodo.svelte';
  import CompactCal   from './compact/CompactCal.svelte';
  import CompactVault from './compact/CompactVault.svelte';

  const TAB_TITLES: Record<Tab, string> = {
    todo: '할 일', cal: '캘린더', vault: 'Vault',
  };

  async function handleExpand(): Promise<void> { await setState('expanded'); }
  async function handleClose(): Promise<void>  { await setState('pill'); }
</script>

<div class="compact-panel">
  <header class="cp-header">
    <span class="cp-title">{TAB_TITLES[$activeTab]}</span>
    <div class="cp-actions">
      <button class="btn-expand" on:click={handleExpand}>확장</button>
      <button class="btn-close"  on:click={handleClose}>«</button>
    </div>
  </header>

  <div class="cp-body">
    {#if $activeTab === 'todo'}
      <CompactTodo />
    {:else if $activeTab === 'cal'}
      <CompactCal />
    {:else}
      <CompactVault />
    {/if}
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
    padding: 11px 13px 10px; flex-shrink: 0;
    border-bottom: 0.5px solid #F0F0F0;
  }

  .cp-title { font-size: 13px; font-weight: 600; }

  .cp-actions { display: flex; align-items: center; gap: 6px; }

  .btn-expand {
    padding: 3px 8px; border-radius: 6px;
    background: #AAED3A; border: none;
    font-size: 10px; font-weight: 600; cursor: pointer; color: #111;
  }
  .btn-expand:hover { background: #9bde2a; }

  .btn-close {
    background: none; border: none; cursor: pointer;
    color: #bbb; font-size: 13px; line-height: 1;
  }
  .btn-close:hover { color: #333; }

  .cp-body {
    flex: 1; overflow-y: auto;
  }
</style>
```

### Step 2: Update `src/routes/+layout.svelte` — remove children from panel mode

The current layout renders `{@render children()}` inside CompactPanel. Since CompactPanel is now self-contained, remove the children render in panel mode. The `children` snippet is still rendered in expanded mode as it was (ExpandedDashboard handles its own content, so we can just not render children at all).

Replace the entire `src/routes/+layout.svelte`:
```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
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
      <CompactPanel />
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

Note: `children` is destructured but intentionally unused — Svelte 5 requires layouts to accept it. The `{@render children()}` is removed because both CompactPanel and ExpandedDashboard are now fully self-contained.

### Step 3: Type-check

```bash
cd /Users/jack/Private/shelf && npx tsc --noEmit 2>&1 | head -30
```

Expected: 0 errors.

- [ ] Step 1: Create 3 stub compact components in `src/lib/components/compact/`
- [ ] Step 2: Replace `src/lib/components/CompactPanel.svelte`
- [ ] Step 3: Replace `src/routes/+layout.svelte`
- [ ] Step 4: Run `npx tsc --noEmit`
- [ ] **Step 5: Commit**
```bash
git add src/lib/components/CompactPanel.svelte src/routes/+layout.svelte \
        src/lib/components/compact/CompactTodo.svelte \
        src/lib/components/compact/CompactCal.svelte \
        src/lib/components/compact/CompactVault.svelte
git commit -m "refactor: CompactPanel 자체 탭 렌더링으로 전환 (slot 제거)"
```

---

## Task 5: CompactTodo component

**Files:**
- Replace: `src/lib/components/compact/CompactTodo.svelte` (was stub)

### Step 1: Implement `CompactTodo.svelte`

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { todos, loadTodos, toggleTodo, PRIORITY_COLORS } from '$lib/stores/todos';
  import { events, loadEvents } from '$lib/stores/events';
  import { projects, loadProjects } from '$lib/stores/projects';
  import { CATEGORY_COLORS } from '$lib/stores/projects';
  import AddTodoModal from '$lib/components/modals/AddTodoModal.svelte';

  const todayPrefix = new Date().toISOString().slice(0, 10);

  let showAddModal = false;

  $: todayTodos = $todos.filter(t => t.due_date?.startsWith(todayPrefix));
  $: doneTodayCount = todayTodos.filter(t => t.done).length;
  $: activeProjectCount = $projects.length;

  $: nextEventTime = (() => {
    const now = new Date();
    const upcoming = $events
      .filter(e => e.start_at.startsWith(todayPrefix))
      .map(e => new Date(e.start_at.replace(' ', 'T')))
      .filter(d => d >= now)
      .sort((a, b) => a.getTime() - b.getTime());
    if (!upcoming.length) return null;
    const h = String(upcoming[0].getHours()).padStart(2, '0');
    const m = String(upcoming[0].getMinutes()).padStart(2, '0');
    return `${h}:${m}`;
  })();

  function badgeStyle(category: string): string {
    const c = CATEGORY_COLORS[category] ?? { bg: '#F5F5F5', text: '#666' };
    return `background:${c.bg};color:${c.text}`;
  }

  onMount(async () => {
    await Promise.all([loadTodos(), loadEvents(), loadProjects()]);
  });
</script>

{#if showAddModal}
  <AddTodoModal on:close={() => (showAddModal = false)} />
{/if}

<div class="compact-todo">
  <!-- Stats row -->
  <div class="stats-row">
    <div class="stat-card">
      <span class="stat-num">{activeProjectCount}</span>
      <span class="stat-label">프로젝트</span>
    </div>
    <div class="stat-card">
      <span class="stat-num">{doneTodayCount}</span>
      <span class="stat-label">완료</span>
    </div>
    <div class="stat-card">
      <span class="stat-num">{nextEventTime ?? '없음'}</span>
      <span class="stat-label">다음 일정</span>
    </div>
  </div>

  <!-- Section header -->
  <div class="section-header">
    <span class="section-title">오늘 할 일</span>
    <button class="btn-add-task" on:click={() => (showAddModal = true)}>+</button>
  </div>

  <!-- Task list -->
  <ul class="task-list">
    {#each todayTodos as todo (todo.id)}
      {@const dotColor = PRIORITY_COLORS[todo.priority]}
      <li class="task-item">
        {#if dotColor}
          <span class="priority-dot" style="background:{dotColor}"></span>
        {:else}
          <span class="priority-dot empty"></span>
        {/if}
        <button
          class="checkbox" class:done={todo.done}
          on:click={() => toggleTodo(todo.id, !todo.done)}
          aria-label={todo.done ? '완료 취소' : '완료'}
        ></button>
        <span class="task-title" class:done={todo.done}>{todo.title}</span>
        <span class="badge" style={badgeStyle(todo.category)}>{todo.category}</span>
      </li>
    {:else}
      <li class="empty">오늘 할 일이 없어요</li>
    {/each}
  </ul>
</div>

<style>
  .compact-todo { display: flex; flex-direction: column; height: 100%; }

  .stats-row {
    display: grid; grid-template-columns: 1fr 1fr 1fr;
    gap: 6px; padding: 10px 12px; background: #FAFAFA;
    border-bottom: 0.5px solid #F0F0F0;
  }
  .stat-card {
    display: flex; flex-direction: column; align-items: center;
    padding: 6px 4px; background: #fff; border-radius: 8px;
    border: 0.5px solid #EFEFEF;
  }
  .stat-num   { font-size: 16px; font-weight: 700; line-height: 1; }
  .stat-label { font-size: 9px; color: #aaa; margin-top: 2px; }

  .section-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 10px 12px 6px;
  }
  .section-title { font-size: 11px; font-weight: 600; color: #888; }
  .btn-add-task {
    width: 20px; height: 20px; border-radius: 50%;
    background: #AAED3A; border: none; font-size: 14px; line-height: 1;
    cursor: pointer; display: flex; align-items: center; justify-content: center;
    font-weight: 700; color: #111;
  }
  .btn-add-task:hover { background: #9bde2a; }

  .task-list { list-style: none; padding: 0 12px; margin: 0; flex: 1; overflow-y: auto; }
  .task-item {
    display: flex; align-items: center; gap: 5px;
    padding: 6px 0; border-bottom: 0.5px solid #F5F5F5;
  }
  .task-item:last-child { border: none; }

  .priority-dot {
    width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0;
  }
  .priority-dot.empty { background: transparent; }

  .checkbox {
    width: 14px; height: 14px; border-radius: 3px;
    border: 1.5px solid #ddd; background: transparent;
    cursor: pointer; flex-shrink: 0; padding: 0; transition: 0.15s;
  }
  .checkbox.done { background: #AAED3A; border-color: #AAED3A; }

  .task-title      { flex: 1; font-size: 12px; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .task-title.done { text-decoration: line-through; color: #bbb; }

  .badge {
    padding: 2px 6px; border-radius: 20px;
    font-size: 9px; font-weight: 600; white-space: nowrap; flex-shrink: 0;
  }

  .empty { font-size: 12px; color: #ccc; text-align: center; padding: 20px 0; }
</style>
```

**Important:** `AddTodoModal` will be created in Task 11. For now, CompactTodo imports it — this will compile only after Task 11. To avoid blocking, create an empty stub for AddTodoModal first:

Create `src/lib/components/modals/AddTodoModal.svelte`:
```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  const dispatch = createEventDispatcher<{ close: void }>();
</script>

<div class="overlay" on:click={() => dispatch('close')} role="dialog" aria-modal="true">
  <div class="modal" on:click|stopPropagation role="presentation">
    <p style="padding:20px;font-size:13px;color:#aaa">할 일 추가 (구현 예정)</p>
    <button on:click={() => dispatch('close')}>닫기</button>
  </div>
</div>

<style>
  .overlay { position:fixed;inset:0;background:rgba(0,0,0,0.3);display:flex;align-items:center;justify-content:center;z-index:100; }
  .modal { background:#fff;border-radius:12px;padding:16px;min-width:280px; }
  button { padding:6px 14px;border:none;border-radius:6px;background:#111;color:#fff;cursor:pointer;width:100%;margin-top:8px; }
</style>
```

### Step 2: Type-check

```bash
cd /Users/jack/Private/shelf && npx tsc --noEmit 2>&1 | head -30
```

- [ ] Step 1: Create `src/lib/components/modals/AddTodoModal.svelte` stub
- [ ] Step 2: Replace `src/lib/components/compact/CompactTodo.svelte` with full implementation
- [ ] Step 3: Run `npx tsc --noEmit`
- [ ] **Step 4: Commit**
```bash
git add src/lib/components/compact/CompactTodo.svelte \
        src/lib/components/modals/AddTodoModal.svelte
git commit -m "feat: CompactTodo 컴포넌트 구현 (stats + 오늘 할 일 목록)"
```

---

## Task 6: CompactCal component

**Files:**
- Replace: `src/lib/components/compact/CompactCal.svelte` (was stub)

### Step 1: Implement `CompactCal.svelte`

Project duration bars appear below each date number when a project's `start_date <= date <= end_date`.

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { events, loadEvents } from '$lib/stores/events';
  import { projects, loadProjects } from '$lib/stores/projects';

  let currentDate = new Date();
  $: year  = currentDate.getFullYear();
  $: month = currentDate.getMonth();
  $: firstDay    = new Date(year, month, 1).getDay();
  $: daysInMonth = new Date(year, month + 1, 0).getDate();
  $: days = Array.from({ length: daysInMonth }, (_, i) => i + 1);

  const MONTH_NAMES = ['1월','2월','3월','4월','5월','6월','7월','8월','9월','10월','11월','12월'];
  const todayPrefix = new Date().toISOString().slice(0, 10);

  function dateStr(day: number): string {
    return `${year}-${String(month + 1).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
  }

  function barsForDay(day: number): { color: string }[] {
    const d = dateStr(day);
    return $projects
      .filter(p => p.start_date && p.end_date && p.start_date <= d && d <= p.end_date)
      .map(p => ({ color: p.color }))
      .slice(0, 3);
  }

  $: todayEvents = $events
    .filter(e => e.start_at.startsWith(todayPrefix))
    .sort((a, b) => a.start_at.localeCompare(b.start_at));

  function formatTime(startAt: string): string {
    const t = startAt.slice(11, 16);
    return t || '';
  }

  function prevMonth(): void { currentDate = new Date(year, month - 1, 1); }
  function nextMonth(): void { currentDate = new Date(year, month + 1, 1); }

  onMount(async () => {
    await Promise.all([loadEvents(), loadProjects()]);
  });
</script>

<div class="compact-cal">
  <!-- Calendar -->
  <div class="cal-header">
    <button class="nav" on:click={prevMonth}>‹</button>
    <span class="month-label">{year}년 {MONTH_NAMES[month]}</span>
    <button class="nav" on:click={nextMonth}>›</button>
  </div>

  <div class="grid">
    {#each ['일','월','화','수','목','금','토'] as d}
      <div class="day-label">{d}</div>
    {/each}
    {#each { length: firstDay } as _}
      <div></div>
    {/each}
    {#each days as day}
      {@const prefix = dateStr(day)}
      {@const isToday = prefix === todayPrefix}
      {@const bars = barsForDay(day)}
      <div class="day" class:today={isToday}>
        <span class="day-num">{day}</span>
        <div class="bars">
          {#each bars as bar}
            <div class="bar" style="background:{bar.color}"></div>
          {/each}
        </div>
      </div>
    {/each}
  </div>

  <!-- Today schedule -->
  <div class="schedule-section">
    <p class="section-title">오늘 일정</p>
    {#each todayEvents as ev}
      <div class="sched-item">
        <span class="sched-dot"></span>
        <span class="sched-time">{formatTime(ev.start_at)}</span>
        <span class="sched-title">{ev.title}</span>
      </div>
    {:else}
      <p class="empty">오늘 일정이 없어요</p>
    {/each}
  </div>
</div>

<style>
  .compact-cal { display: flex; flex-direction: column; height: 100%; }

  .cal-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 8px 12px; flex-shrink: 0;
  }
  .month-label { font-size: 12px; font-weight: 600; }
  .nav { background: none; border: none; cursor: pointer; font-size: 16px; color: #888; padding: 0 4px; }
  .nav:hover { color: #111; }

  .grid {
    display: grid; grid-template-columns: repeat(7, 1fr);
    gap: 1px; padding: 0 8px; flex-shrink: 0;
  }
  .day-label { text-align: center; font-size: 9px; color: #aaa; padding: 2px 0; }

  .day {
    display: flex; flex-direction: column; align-items: center;
    padding: 2px 1px; border-radius: 4px; min-height: 26px;
  }
  .day.today { background: rgba(170, 237, 58, 0.15); }
  .day.today .day-num { font-weight: 700; }

  .day-num { font-size: 10px; line-height: 1.2; }

  .bars { display: flex; flex-direction: column; gap: 1px; width: 100%; margin-top: 1px; }
  .bar { height: 2px; border-radius: 1px; width: 100%; }

  .schedule-section {
    padding: 10px 12px; border-top: 0.5px solid #F0F0F0;
    flex: 1; overflow-y: auto;
  }
  .section-title { font-size: 10px; font-weight: 600; color: #888; margin: 0 0 6px; }

  .sched-item {
    display: flex; align-items: center; gap: 6px;
    padding: 4px 0; border-bottom: 0.5px solid #F5F5F5;
  }
  .sched-item:last-child { border: none; }
  .sched-dot { width: 6px; height: 6px; border-radius: 50%; background: #AAED3A; flex-shrink: 0; }
  .sched-time { font-size: 10px; color: #888; width: 32px; flex-shrink: 0; }
  .sched-title { font-size: 11px; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .empty { font-size: 11px; color: #ccc; text-align: center; padding: 12px 0; margin: 0; }
</style>
```

### Step 2: Type-check

```bash
cd /Users/jack/Private/shelf && npx tsc --noEmit 2>&1 | head -30
```

- [ ] Step 1: Replace `src/lib/components/compact/CompactCal.svelte`
- [ ] Step 2: Run `npx tsc --noEmit`
- [ ] **Step 3: Commit**
```bash
git add src/lib/components/compact/CompactCal.svelte
git commit -m "feat: CompactCal 컴포넌트 구현 (프로젝트 바 달력 + 오늘 일정)"
```

---

## Task 7: CompactVault component

**Files:**
- Replace: `src/lib/components/compact/CompactVault.svelte` (was stub)

### Step 1: Implement `CompactVault.svelte`

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { vaultUnlocked, vaultInitialized, checkVaultState } from '$lib/stores/vault';
  import VaultLock from '$lib/components/vault/VaultLock.svelte';
  import VaultList from '$lib/components/vault/VaultList.svelte';

  onMount(checkVaultState);
</script>

<div class="compact-vault">
  {#if $vaultUnlocked}
    <VaultList />
  {:else}
    <VaultLock />
  {/if}
</div>

<style>
  .compact-vault { height: 100%; overflow-y: auto; }
</style>
```

### Step 2: Type-check

```bash
cd /Users/jack/Private/shelf && npx tsc --noEmit 2>&1 | head -30
```

- [ ] Step 1: Replace `src/lib/components/compact/CompactVault.svelte`
- [ ] Step 2: Run `npx tsc --noEmit`
- [ ] **Step 3: Commit**
```bash
git add src/lib/components/compact/CompactVault.svelte
git commit -m "feat: CompactVault 컴포넌트 구현"
```

---

## Task 8: ExpandedDashboard — tab-aware sub-view routing

**Files:**
- Replace: `src/lib/components/dashboard/ExpandedDashboard.svelte`
- Create stubs: `src/lib/components/dashboard/ExpandedTodo.svelte`, `ExpandedCalendar.svelte`, `ExpandedVault.svelte`

### Step 1: Create stub sub-view components

**`src/lib/components/dashboard/ExpandedTodo.svelte` stub:**
```svelte
<p style="padding:20px;font-size:13px;color:#aaa">할 일 확장 뷰 (구현 예정)</p>
```

**`src/lib/components/dashboard/ExpandedCalendar.svelte` stub:**
```svelte
<p style="padding:20px;font-size:13px;color:#aaa">캘린더 확장 뷰 (구현 예정)</p>
```

**`src/lib/components/dashboard/ExpandedVault.svelte` stub:**
```svelte
<p style="padding:20px;font-size:13px;color:#aaa">Vault 확장 뷰 (구현 예정)</p>
```

### Step 2: Replace `ExpandedDashboard.svelte`

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { loadTodos } from '$lib/stores/todos';
  import { loadEvents } from '$lib/stores/events';
  import { loadProjects } from '$lib/stores/projects';
  import { checkVaultState } from '$lib/stores/vault';
  import { activeTab } from '$lib/stores/window';
  import DashboardHeader  from './DashboardHeader.svelte';
  import ExpandedTodo     from './ExpandedTodo.svelte';
  import ExpandedCalendar from './ExpandedCalendar.svelte';
  import ExpandedVault    from './ExpandedVault.svelte';

  onMount(async () => {
    await Promise.all([loadTodos(), loadEvents(), loadProjects(), checkVaultState()]);
  });
</script>

<div class="expanded">
  <DashboardHeader />

  <div class="ex-content">
    {#if $activeTab === 'todo'}
      <ExpandedTodo />
    {:else if $activeTab === 'cal'}
      <ExpandedCalendar />
    {:else}
      <ExpandedVault />
    {/if}
  </div>
</div>

<style>
  .expanded {
    width: 560px; height: 100vh; background: #fff;
    display: flex; flex-direction: column; overflow: hidden;
  }
  .ex-content {
    flex: 1; overflow: hidden; display: flex; flex-direction: column;
  }
</style>
```

### Step 3: Type-check

```bash
cd /Users/jack/Private/shelf && npx tsc --noEmit 2>&1 | head -30
```

- [ ] Step 1: Create 3 stub expanded sub-view components
- [ ] Step 2: Replace `src/lib/components/dashboard/ExpandedDashboard.svelte`
- [ ] Step 3: Run `npx tsc --noEmit`
- [ ] **Step 4: Commit**
```bash
git add src/lib/components/dashboard/ExpandedDashboard.svelte \
        src/lib/components/dashboard/ExpandedTodo.svelte \
        src/lib/components/dashboard/ExpandedCalendar.svelte \
        src/lib/components/dashboard/ExpandedVault.svelte
git commit -m "refactor: ExpandedDashboard 탭 인식 서브뷰 라우팅으로 전환"
```

---

## Task 9: ExpandedTodo sub-view

**Files:**
- Replace: `src/lib/components/dashboard/ExpandedTodo.svelte` (was stub)

The layout is two-panel:
- **Left (200px)**: Project list — colored bar + name + category + task count badge + "+ 새 프로젝트" button (opens AddProjectModal)
- **Right (360px)**: Task list — priority dot + checkbox + title + category badge + due date. Inline add row at bottom.

### Step 1: Create `AddProjectModal.svelte` stub (full implementation in Task 12)

Create `src/lib/components/modals/AddProjectModal.svelte`:
```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  const dispatch = createEventDispatcher<{ close: void }>();
</script>

<div class="overlay" on:click={() => dispatch('close')} role="dialog" aria-modal="true">
  <div class="modal" on:click|stopPropagation role="presentation">
    <p style="padding:20px;font-size:13px;color:#aaa">프로젝트 추가 (구현 예정)</p>
    <button on:click={() => dispatch('close')}>닫기</button>
  </div>
</div>

<style>
  .overlay { position:fixed;inset:0;background:rgba(0,0,0,0.3);display:flex;align-items:center;justify-content:center;z-index:100; }
  .modal { background:#fff;border-radius:12px;padding:16px;min-width:280px; }
  button { padding:6px 14px;border:none;border-radius:6px;background:#111;color:#fff;cursor:pointer;width:100%;margin-top:8px; }
</style>
```

### Step 2: Implement `ExpandedTodo.svelte`

```svelte
<script lang="ts">
  import { todos, toggleTodo, createTodo, PRIORITY_COLORS } from '$lib/stores/todos';
  import { projects } from '$lib/stores/projects';
  import { CATEGORY_COLORS } from '$lib/stores/projects';
  import AddProjectModal from '$lib/components/modals/AddProjectModal.svelte';

  const CATEGORIES = ['작업', '클라이언트', '개인', 'work', '사일'];
  const todayPrefix = new Date().toISOString().slice(0, 10);

  let showAddProject = false;

  let selectedProjectId: number | null = null;

  let newTitle    = '';
  let newCategory = '작업';
  let newPriority = 0;

  $: filteredTodos = selectedProjectId === null
    ? $todos
    : $todos.filter(t => t.project_id === selectedProjectId);

  $: taskCountForProject = (pid: number) => $todos.filter(t => t.project_id === pid).length;

  function badgeStyle(category: string): string {
    const c = CATEGORY_COLORS[category] ?? { bg: '#F5F5F5', text: '#666' };
    return `background:${c.bg};color:${c.text}`;
  }

  async function handleAddTodo(): Promise<void> {
    if (!newTitle.trim()) return;
    await createTodo(newTitle.trim(), '', todayPrefix, 'none', newCategory, newPriority);
    newTitle = '';
    newPriority = 0;
  }
</script>

{#if showAddProject}
  <AddProjectModal on:close={() => (showAddProject = false)} />
{/if}

<div class="expanded-todo">
  <!-- Left: Project panel -->
  <aside class="project-panel">
    <p class="panel-title">프로젝트</p>

    <button
      class="proj-item all-item"
      class:active={selectedProjectId === null}
      on:click={() => (selectedProjectId = null)}
    >
      <span class="proj-bar" style="background:#888"></span>
      <span class="proj-name">전체</span>
      <span class="task-count">{$todos.length}</span>
    </button>

    {#each $projects as proj (proj.id)}
      <button
        class="proj-item"
        class:active={selectedProjectId === proj.id}
        on:click={() => (selectedProjectId = proj.id)}
      >
        <span class="proj-bar" style="background:{proj.color}"></span>
        <div class="proj-info">
          <span class="proj-name">{proj.name}</span>
          <span class="proj-cat">{proj.category}</span>
        </div>
        <span class="task-count">{taskCountForProject(proj.id)}</span>
      </button>
    {/each}

    <button class="btn-new-project" on:click={() => (showAddProject = true)}>
      + 새 프로젝트
    </button>
  </aside>

  <!-- Right: Task panel -->
  <main class="task-panel">
    <ul class="task-list">
      {#each filteredTodos as todo (todo.id)}
        {@const dotColor = PRIORITY_COLORS[todo.priority]}
        <li class="task-item">
          {#if dotColor}
            <span class="priority-dot" style="background:{dotColor}"></span>
          {:else}
            <span class="priority-dot empty"></span>
          {/if}
          <button
            class="checkbox" class:done={todo.done}
            on:click={() => toggleTodo(todo.id, !todo.done)}
            aria-label={todo.done ? '완료 취소' : '완료'}
          ></button>
          <span class="task-title" class:done={todo.done}>{todo.title}</span>
          <span class="badge" style={badgeStyle(todo.category)}>{todo.category}</span>
          {#if todo.due_date}
            <span class="due-date">{todo.due_date.slice(5, 10)}</span>
          {/if}
        </li>
      {:else}
        <li class="empty">할 일이 없어요</li>
      {/each}
    </ul>

    <!-- Inline add row -->
    <div class="add-row">
      <div class="priority-btns">
        {#each [0, 1, 2, 3] as p}
          {@const color = PRIORITY_COLORS[p]}
          <button
            class="prio-btn" class:selected={newPriority === p}
            style={color ? `background:${color}` : 'background:#ddd'}
            on:click={() => (newPriority = p)}
            title={['없음','낮음','보통','높음'][p]}
          ></button>
        {/each}
      </div>
      <input
        class="add-input"
        bind:value={newTitle}
        placeholder="할 일 추가..."
        on:keydown={(e) => e.key === 'Enter' && handleAddTodo()}
      />
      <select class="cat-select" bind:value={newCategory}>
        {#each CATEGORIES as cat}<option value={cat}>{cat}</option>{/each}
      </select>
    </div>
  </main>
</div>

<style>
  .expanded-todo {
    display: flex; flex: 1; overflow: hidden;
  }

  /* Left project panel */
  .project-panel {
    width: 190px; flex-shrink: 0;
    border-right: 0.5px solid #F0F0F0;
    display: flex; flex-direction: column;
    padding: 12px 8px; overflow-y: auto;
    background: #FAFAFA;
  }
  .panel-title { font-size: 10px; font-weight: 600; color: #aaa; margin: 0 0 8px 4px; }

  .proj-item {
    display: flex; align-items: center; gap: 7px;
    padding: 7px 8px; border-radius: 8px;
    border: none; background: transparent; cursor: pointer;
    text-align: left; width: 100%; margin-bottom: 2px;
    transition: background 0.1s;
  }
  .proj-item:hover  { background: rgba(0,0,0,0.05); }
  .proj-item.active { background: rgba(0,0,0,0.08); }

  .proj-bar { width: 3px; height: 28px; border-radius: 2px; flex-shrink: 0; }
  .all-item .proj-bar { height: 14px; }

  .proj-info { flex: 1; min-width: 0; display: flex; flex-direction: column; }
  .proj-name { font-size: 12px; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .proj-cat  { font-size: 9px; color: #aaa; }

  .task-count {
    font-size: 10px; color: #888;
    background: #eee; border-radius: 10px;
    padding: 1px 6px; flex-shrink: 0;
  }

  .btn-new-project {
    margin-top: 8px; padding: 7px 8px;
    border: 1px dashed #ddd; border-radius: 8px;
    background: none; cursor: pointer; color: #888;
    font-size: 11px; text-align: center; width: 100%;
    transition: border-color 0.15s, color 0.15s;
  }
  .btn-new-project:hover { border-color: #AAED3A; color: #111; }

  /* Right task panel */
  .task-panel {
    flex: 1; display: flex; flex-direction: column; overflow: hidden;
  }

  .task-list {
    flex: 1; list-style: none; padding: 8px 14px; margin: 0;
    overflow-y: auto;
  }
  .task-item {
    display: flex; align-items: center; gap: 6px;
    padding: 7px 0; border-bottom: 0.5px solid #F5F5F5;
  }
  .task-item:last-child { border: none; }

  .priority-dot {
    width: 7px; height: 7px; border-radius: 50%; flex-shrink: 0;
  }
  .priority-dot.empty { background: transparent; }

  .checkbox {
    width: 15px; height: 15px; border-radius: 4px;
    border: 1.5px solid #ddd; background: transparent;
    cursor: pointer; flex-shrink: 0; padding: 0; transition: 0.15s;
  }
  .checkbox.done { background: #AAED3A; border-color: #AAED3A; }

  .task-title      { flex: 1; font-size: 13px; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .task-title.done { text-decoration: line-through; color: #bbb; }

  .badge {
    padding: 2px 7px; border-radius: 20px;
    font-size: 9px; font-weight: 600; white-space: nowrap; flex-shrink: 0;
  }
  .due-date { font-size: 10px; color: #aaa; flex-shrink: 0; }

  .empty { font-size: 12px; color: #ccc; text-align: center; padding: 30px 0; }

  /* Add row */
  .add-row {
    display: flex; align-items: center; gap: 6px;
    padding: 10px 14px; border-top: 0.5px solid #F0F0F0;
    flex-shrink: 0;
  }

  .priority-btns { display: flex; gap: 4px; }
  .prio-btn {
    width: 10px; height: 10px; border-radius: 50%;
    border: 1.5px solid transparent; cursor: pointer; padding: 0;
    transition: transform 0.1s;
  }
  .prio-btn.selected { border-color: #111; transform: scale(1.2); }

  .add-input {
    flex: 1; border: 0.5px solid #eee; border-radius: 6px;
    padding: 6px 8px; font-size: 12px; background: #F7F7F7; outline: none;
  }
  .add-input:focus { border-color: #AAED3A; background: #fff; }

  .cat-select {
    border: 0.5px solid #eee; border-radius: 6px;
    padding: 5px 4px; font-size: 11px; background: #F7F7F7;
    outline: none; max-width: 70px;
  }
</style>
```

### Step 3: Type-check

```bash
cd /Users/jack/Private/shelf && npx tsc --noEmit 2>&1 | head -30
```

- [ ] Step 1: Create `src/lib/components/modals/AddProjectModal.svelte` stub
- [ ] Step 2: Replace `src/lib/components/dashboard/ExpandedTodo.svelte`
- [ ] Step 3: Run `npx tsc --noEmit`
- [ ] **Step 4: Commit**
```bash
git add src/lib/components/dashboard/ExpandedTodo.svelte \
        src/lib/components/modals/AddProjectModal.svelte
git commit -m "feat: ExpandedTodo 서브뷰 구현 (프로젝트 패널 + 태스크 패널)"
```

---

## Task 10: ExpandedCalendar sub-view

**Files:**
- Replace: `src/lib/components/dashboard/ExpandedCalendar.svelte` (was stub)

Two-panel layout:
- **Left**: Calendar with project bars + project chips row below the grid + inline event add at bottom
- **Right (180px)**: "오늘 일정" list — colored bar + time + title

### Step 1: Implement `ExpandedCalendar.svelte`

```svelte
<script lang="ts">
  import { events, createEvent } from '$lib/stores/events';
  import { projects } from '$lib/stores/projects';

  let currentDate = new Date();
  $: year  = currentDate.getFullYear();
  $: month = currentDate.getMonth();
  $: firstDay    = new Date(year, month, 1).getDay();
  $: daysInMonth = new Date(year, month + 1, 0).getDate();
  $: days = Array.from({ length: daysInMonth }, (_, i) => i + 1);

  const MONTH_NAMES = ['1월','2월','3월','4월','5월','6월','7월','8월','9월','10월','11월','12월'];
  const todayPrefix = new Date().toISOString().slice(0, 10);

  function dateStr(day: number): string {
    return `${year}-${String(month + 1).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
  }

  function barsForDay(day: number): { color: string; name: string }[] {
    const d = dateStr(day);
    return $projects
      .filter(p => p.start_date && p.end_date && p.start_date <= d && d <= p.end_date)
      .map(p => ({ color: p.color, name: p.name }))
      .slice(0, 2);
  }

  $: todayEvents = $events
    .filter(e => e.start_at.startsWith(todayPrefix))
    .sort((a, b) => a.start_at.localeCompare(b.start_at));

  function getProjectColor(categoryOrTitle: string): string {
    const match = $projects.find(p => p.name === categoryOrTitle || p.category === categoryOrTitle);
    return match ? match.color : '#AAED3A';
  }

  function formatTime(startAt: string): string {
    return startAt.slice(11, 16) || '';
  }

  let newEventTitle = '';
  let newEventDate  = todayPrefix;

  async function handleAddEvent(): Promise<void> {
    if (!newEventTitle.trim() || !newEventDate) return;
    await createEvent(newEventTitle.trim(), `${newEventDate} 09:00:00`, null, 'none');
    newEventTitle = '';
  }

  function prevMonth(): void { currentDate = new Date(year, month - 1, 1); }
  function nextMonth(): void { currentDate = new Date(year, month + 1, 1); }
</script>

<div class="expanded-cal">
  <!-- Left: Calendar + chips + add -->
  <div class="cal-panel">
    <div class="cal-header">
      <button class="nav" on:click={prevMonth}>‹</button>
      <span class="month-label">{year}년 {MONTH_NAMES[month]}</span>
      <button class="nav" on:click={nextMonth}>›</button>
    </div>

    <div class="grid">
      {#each ['일','월','화','수','목','금','토'] as d}
        <div class="day-label">{d}</div>
      {/each}
      {#each { length: firstDay } as _}
        <div></div>
      {/each}
      {#each days as day}
        {@const prefix = dateStr(day)}
        {@const isToday = prefix === todayPrefix}
        {@const bars = barsForDay(day)}
        <div class="day" class:today={isToday}>
          <span class="day-num">{day}</span>
          <div class="bars">
            {#each bars as bar}
              <div class="bar" style="background:{bar.color}"></div>
            {/each}
          </div>
        </div>
      {/each}
    </div>

    <!-- Project chips -->
    <div class="proj-chips">
      {#each $projects as proj (proj.id)}
        <span class="chip" style="border-color:{proj.color};color:{proj.color}">
          {proj.name}
        </span>
      {/each}
    </div>

    <!-- Add event -->
    <div class="add-row">
      <input type="date" class="date-input" bind:value={newEventDate} />
      <input
        class="title-input"
        bind:value={newEventTitle}
        placeholder="일정 추가..."
        on:keydown={(e) => e.key === 'Enter' && handleAddEvent()}
      />
      <button class="btn-add" on:click={handleAddEvent}>+</button>
    </div>
  </div>

  <!-- Right: Today schedule -->
  <aside class="schedule-panel">
    <p class="panel-title">오늘 일정</p>
    {#each todayEvents as ev (ev.id)}
      {@const color = getProjectColor(ev.category)}
      <div class="sched-item">
        <div class="sched-bar" style="background:{color}"></div>
        <div class="sched-content">
          <span class="sched-time">{formatTime(ev.start_at)}</span>
          <span class="sched-title">{ev.title}</span>
        </div>
      </div>
    {:else}
      <p class="empty">오늘 일정이 없어요</p>
    {/each}
  </aside>
</div>

<style>
  .expanded-cal {
    display: flex; flex: 1; overflow: hidden;
  }

  /* Left calendar panel */
  .cal-panel {
    flex: 1; display: flex; flex-direction: column;
    padding: 12px 14px; border-right: 0.5px solid #F0F0F0; overflow: hidden;
  }

  .cal-header {
    display: flex; align-items: center; justify-content: space-between;
    margin-bottom: 8px; flex-shrink: 0;
  }
  .month-label { font-size: 13px; font-weight: 600; }
  .nav { background: none; border: none; cursor: pointer; font-size: 18px; color: #888; }
  .nav:hover { color: #111; }

  .grid {
    display: grid; grid-template-columns: repeat(7, 1fr);
    gap: 2px; flex-shrink: 0;
  }
  .day-label { text-align: center; font-size: 10px; color: #aaa; padding: 2px; }

  .day {
    display: flex; flex-direction: column; align-items: center;
    padding: 3px 1px; border-radius: 4px; min-height: 32px;
  }
  .day.today { background: rgba(170, 237, 58, 0.15); }
  .day.today .day-num { font-weight: 700; }
  .day-num { font-size: 11px; line-height: 1.3; }

  .bars { display: flex; flex-direction: column; gap: 1px; width: 90%; margin-top: 2px; }
  .bar  { height: 2px; border-radius: 1px; width: 100%; }

  .proj-chips {
    display: flex; flex-wrap: wrap; gap: 5px;
    margin: 10px 0; flex-shrink: 0;
  }
  .chip {
    padding: 3px 8px; border-radius: 20px; font-size: 10px;
    border: 1px solid; background: transparent; font-weight: 500;
  }

  .add-row {
    display: flex; gap: 6px; margin-top: auto; flex-shrink: 0;
    padding-top: 10px; border-top: 0.5px solid #F0F0F0;
  }
  .date-input {
    border: 0.5px solid #eee; border-radius: 6px;
    padding: 5px 6px; font-size: 11px; background: #F7F7F7;
    outline: none; width: 110px;
  }
  .title-input {
    flex: 1; border: 0.5px solid #eee; border-radius: 6px;
    padding: 5px 8px; font-size: 12px; background: #F7F7F7; outline: none;
  }
  .title-input:focus { border-color: #AAED3A; background: #fff; }
  .btn-add {
    width: 28px; height: 28px; border-radius: 6px;
    background: #AAED3A; border: none; font-size: 18px; line-height: 1;
    cursor: pointer; display: flex; align-items: center; justify-content: center;
    font-weight: 700; color: #111;
  }
  .btn-add:hover { background: #9bde2a; }

  /* Right schedule panel */
  .schedule-panel {
    width: 180px; flex-shrink: 0;
    padding: 12px 12px; overflow-y: auto;
    background: #FAFAFA;
  }
  .panel-title { font-size: 10px; font-weight: 600; color: #aaa; margin: 0 0 10px; }

  .sched-item {
    display: flex; gap: 8px; align-items: flex-start;
    margin-bottom: 10px;
  }
  .sched-bar { width: 3px; border-radius: 2px; min-height: 36px; flex-shrink: 0; }
  .sched-content { display: flex; flex-direction: column; gap: 2px; }
  .sched-time  { font-size: 10px; color: #888; }
  .sched-title { font-size: 12px; font-weight: 500; }

  .empty { font-size: 11px; color: #ccc; text-align: center; margin-top: 20px; }
</style>
```

### Step 2: Type-check

```bash
cd /Users/jack/Private/shelf && npx tsc --noEmit 2>&1 | head -30
```

- [ ] Step 1: Replace `src/lib/components/dashboard/ExpandedCalendar.svelte`
- [ ] Step 2: Run `npx tsc --noEmit`
- [ ] **Step 3: Commit**
```bash
git add src/lib/components/dashboard/ExpandedCalendar.svelte
git commit -m "feat: ExpandedCalendar 서브뷰 구현 (달력 + 프로젝트 바 + 오늘 일정)"
```

---

## Task 11: ExpandedVault sub-view

**Files:**
- Replace: `src/lib/components/dashboard/ExpandedVault.svelte` (was stub)

Simple wrapper: shows VaultLock or VaultList depending on vault state. Vault already has its own styling.

### Step 1: Implement `ExpandedVault.svelte`

```svelte
<script lang="ts">
  import { vaultUnlocked, vaultInitialized, checkVaultState } from '$lib/stores/vault';
  import VaultLock from '$lib/components/vault/VaultLock.svelte';
  import VaultList from '$lib/components/vault/VaultList.svelte';
  import { onMount } from 'svelte';

  onMount(checkVaultState);
</script>

<div class="expanded-vault">
  {#if $vaultUnlocked}
    <VaultList />
  {:else}
    <VaultLock />
  {/if}
</div>

<style>
  .expanded-vault { flex: 1; overflow-y: auto; }
</style>
```

### Step 2: Type-check

```bash
cd /Users/jack/Private/shelf && npx tsc --noEmit 2>&1 | head -30
```

- [ ] Step 1: Replace `src/lib/components/dashboard/ExpandedVault.svelte`
- [ ] Step 2: Run `npx tsc --noEmit`
- [ ] **Step 3: Commit**
```bash
git add src/lib/components/dashboard/ExpandedVault.svelte
git commit -m "feat: ExpandedVault 서브뷰 구현"
```

---

## Task 12: AddTodoModal — full implementation

**Files:**
- Replace: `src/lib/components/modals/AddTodoModal.svelte` (was stub from Task 5)

The modal shows:
- Title input
- Project chip selector (tap to toggle project assignment)
- Priority buttons (4 circles: none/low/medium/high)
- Due date input
- Submit button

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { createTodo } from '$lib/stores/todos';
  import { projects } from '$lib/stores/projects';
  import { PRIORITY_COLORS } from '$lib/stores/todos';

  const dispatch = createEventDispatcher<{ close: void }>();

  const CATEGORIES = ['작업', '클라이언트', '개인', 'work', '사일'];

  let title       = '';
  let category    = '작업';
  let priority    = 0;
  let dueDate     = new Date().toISOString().slice(0, 10);
  let projectId: number | null = null;
  let loading     = false;

  function selectProject(id: number): void {
    projectId = projectId === id ? null : id;
  }

  async function handleSubmit(): Promise<void> {
    if (!title.trim()) return;
    loading = true;
    try {
      await createTodo(title.trim(), '', dueDate || null, 'none', category, priority);
      dispatch('close');
    } finally {
      loading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === 'Escape') dispatch('close');
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="overlay" on:click={() => dispatch('close')}>
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="modal" on:click|stopPropagation>
    <h3 class="modal-title">할 일 추가</h3>

    <!-- Title -->
    <input
      class="title-input"
      bind:value={title}
      placeholder="무엇을 해야 하나요?"
      autofocus
      on:keydown={(e) => e.key === 'Enter' && handleSubmit()}
    />

    <!-- Project chips -->
    {#if $projects.length > 0}
      <div class="section-label">프로젝트</div>
      <div class="project-chips">
        {#each $projects as proj (proj.id)}
          <button
            class="proj-chip"
            class:selected={projectId === proj.id}
            style="--color:{proj.color}"
            on:click={() => selectProject(proj.id)}
          >
            {proj.name}
          </button>
        {/each}
      </div>
    {/if}

    <!-- Priority -->
    <div class="section-label">우선순위</div>
    <div class="priority-row">
      {#each [0, 1, 2, 3] as p}
        {@const color = PRIORITY_COLORS[p]}
        {@const labels = ['없음', '낮음', '보통', '높음']}
        <button
          class="prio-btn" class:selected={priority === p}
          on:click={() => (priority = p)}
        >
          <span
            class="prio-dot"
            style={color ? `background:${color}` : 'background:#ddd'}
          ></span>
          <span class="prio-label">{labels[p]}</span>
        </button>
      {/each}
    </div>

    <!-- Category -->
    <div class="section-label">카테고리</div>
    <div class="cat-row">
      {#each CATEGORIES as cat}
        <button
          class="cat-btn" class:selected={category === cat}
          on:click={() => (category = cat)}
        >{cat}</button>
      {/each}
    </div>

    <!-- Due date -->
    <div class="section-label">기한</div>
    <input type="date" class="date-input" bind:value={dueDate} />

    <!-- Actions -->
    <div class="actions">
      <button class="btn-cancel" on:click={() => dispatch('close')}>취소</button>
      <button class="btn-submit" on:click={handleSubmit} disabled={loading || !title.trim()}>
        {loading ? '추가 중...' : '추가'}
      </button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.4);
    display: flex; align-items: center; justify-content: center;
    z-index: 200;
  }

  .modal {
    background: #fff; border-radius: 14px;
    padding: 20px; width: 300px;
    box-shadow: 0 16px 48px rgba(0,0,0,0.15);
    display: flex; flex-direction: column; gap: 10px;
  }

  .modal-title { margin: 0; font-size: 15px; font-weight: 700; }

  .title-input {
    width: 100%; box-sizing: border-box;
    border: 1px solid #eee; border-radius: 8px;
    padding: 10px 12px; font-size: 14px; outline: none;
  }
  .title-input:focus { border-color: #AAED3A; }

  .section-label { font-size: 10px; font-weight: 600; color: #aaa; margin-bottom: -4px; }

  .project-chips { display: flex; flex-wrap: wrap; gap: 6px; }
  .proj-chip {
    padding: 4px 10px; border-radius: 20px;
    border: 1.5px solid var(--color); color: var(--color);
    background: transparent; cursor: pointer; font-size: 11px; font-weight: 500;
    transition: background 0.1s;
  }
  .proj-chip.selected { background: var(--color); color: #fff; }

  .priority-row { display: flex; gap: 8px; }
  .prio-btn {
    display: flex; flex-direction: column; align-items: center; gap: 4px;
    padding: 6px 10px; border-radius: 8px;
    border: 1.5px solid #eee; background: transparent; cursor: pointer;
    transition: border-color 0.1s;
  }
  .prio-btn.selected { border-color: #111; }
  .prio-dot { width: 10px; height: 10px; border-radius: 50%; }
  .prio-label { font-size: 9px; color: #888; }

  .cat-row { display: flex; flex-wrap: wrap; gap: 5px; }
  .cat-btn {
    padding: 4px 9px; border-radius: 6px;
    border: 1px solid #eee; background: transparent; cursor: pointer;
    font-size: 11px; color: #555;
    transition: border-color 0.1s, background 0.1s;
  }
  .cat-btn.selected { border-color: #111; background: #111; color: #fff; }

  .date-input {
    border: 1px solid #eee; border-radius: 8px;
    padding: 7px 10px; font-size: 12px; outline: none; width: 100%;
    box-sizing: border-box;
  }

  .actions { display: flex; gap: 8px; margin-top: 4px; }
  .btn-cancel {
    flex: 1; padding: 9px; border-radius: 8px;
    border: 1px solid #eee; background: #F5F5F5; cursor: pointer;
    font-size: 13px;
  }
  .btn-submit {
    flex: 1; padding: 9px; border-radius: 8px;
    border: none; background: #111; color: #fff; cursor: pointer;
    font-size: 13px; font-weight: 600;
  }
  .btn-submit:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-cancel:hover { background: #eee; }
  .btn-submit:not(:disabled):hover { background: #333; }
</style>
```

### Step 2: Type-check

```bash
cd /Users/jack/Private/shelf && npx tsc --noEmit 2>&1 | head -30
```

- [ ] Step 1: Replace `src/lib/components/modals/AddTodoModal.svelte`
- [ ] Step 2: Run `npx tsc --noEmit`
- [ ] **Step 3: Commit**
```bash
git add src/lib/components/modals/AddTodoModal.svelte
git commit -m "feat: AddTodoModal 구현 (우선순위, 프로젝트, 카테고리, 기한)"
```

---

## Task 13: AddProjectModal — full implementation

**Files:**
- Replace: `src/lib/components/modals/AddProjectModal.svelte` (was stub from Task 9)

The modal shows:
- Name input
- Color swatches (PROJECT_COLORS)
- 개인/회사 toggle (maps to category field)
- Date range (start/end date)
- Submit button

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { createProject, PROJECT_COLORS } from '$lib/stores/projects';

  const dispatch = createEventDispatcher<{ close: void }>();

  let name      = '';
  let color     = PROJECT_COLORS[0];
  let category  = '개인';
  let startDate = '';
  let endDate   = '';
  let loading   = false;

  async function handleSubmit(): Promise<void> {
    if (!name.trim()) return;
    loading = true;
    try {
      await createProject(name.trim(), color, category, startDate || null, endDate || null);
      dispatch('close');
    } finally {
      loading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === 'Escape') dispatch('close');
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="overlay" on:click={() => dispatch('close')}>
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="modal" on:click|stopPropagation>
    <h3 class="modal-title">새 프로젝트</h3>

    <!-- Name -->
    <input
      class="name-input"
      bind:value={name}
      placeholder="프로젝트 이름"
      autofocus
      on:keydown={(e) => e.key === 'Enter' && handleSubmit()}
    />

    <!-- Color swatches -->
    <div class="section-label">색상</div>
    <div class="color-grid">
      {#each PROJECT_COLORS as c}
        <button
          class="swatch" class:selected={color === c}
          style="background:{c}"
          on:click={() => (color = c)}
          aria-label={c}
        ></button>
      {/each}
    </div>

    <!-- Category toggle -->
    <div class="section-label">유형</div>
    <div class="toggle-row">
      <button
        class="toggle-btn" class:active={category === '개인'}
        on:click={() => (category = '개인')}
      >개인</button>
      <button
        class="toggle-btn" class:active={category === '회사'}
        on:click={() => (category = '회사')}
      >회사</button>
    </div>

    <!-- Date range -->
    <div class="section-label">기간</div>
    <div class="date-row">
      <input type="date" class="date-input" bind:value={startDate} />
      <span class="date-sep">~</span>
      <input type="date" class="date-input" bind:value={endDate} />
    </div>

    <!-- Actions -->
    <div class="actions">
      <button class="btn-cancel" on:click={() => dispatch('close')}>취소</button>
      <button class="btn-submit" on:click={handleSubmit} disabled={loading || !name.trim()}>
        {loading ? '추가 중...' : '만들기'}
      </button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.4);
    display: flex; align-items: center; justify-content: center;
    z-index: 200;
  }

  .modal {
    background: #fff; border-radius: 14px;
    padding: 20px; width: 280px;
    box-shadow: 0 16px 48px rgba(0,0,0,0.15);
    display: flex; flex-direction: column; gap: 10px;
  }

  .modal-title { margin: 0; font-size: 15px; font-weight: 700; }

  .name-input {
    width: 100%; box-sizing: border-box;
    border: 1px solid #eee; border-radius: 8px;
    padding: 10px 12px; font-size: 14px; outline: none;
  }
  .name-input:focus { border-color: #AAED3A; }

  .section-label { font-size: 10px; font-weight: 600; color: #aaa; margin-bottom: -4px; }

  .color-grid { display: flex; flex-wrap: wrap; gap: 8px; }
  .swatch {
    width: 26px; height: 26px; border-radius: 50%;
    border: 2px solid transparent; cursor: pointer; padding: 0;
    transition: transform 0.1s, border-color 0.1s;
  }
  .swatch:hover    { transform: scale(1.1); }
  .swatch.selected { border-color: #111; transform: scale(1.15); }

  .toggle-row { display: flex; border: 1px solid #eee; border-radius: 8px; overflow: hidden; }
  .toggle-btn {
    flex: 1; padding: 8px; border: none; background: transparent;
    cursor: pointer; font-size: 13px; transition: background 0.1s;
  }
  .toggle-btn.active { background: #111; color: #fff; font-weight: 600; }
  .toggle-btn:not(.active):hover { background: #F5F5F5; }

  .date-row { display: flex; align-items: center; gap: 6px; }
  .date-input {
    flex: 1; border: 1px solid #eee; border-radius: 8px;
    padding: 7px 8px; font-size: 11px; outline: none; box-sizing: border-box;
  }
  .date-sep { color: #aaa; font-size: 12px; flex-shrink: 0; }

  .actions { display: flex; gap: 8px; margin-top: 4px; }
  .btn-cancel {
    flex: 1; padding: 9px; border-radius: 8px;
    border: 1px solid #eee; background: #F5F5F5; cursor: pointer;
    font-size: 13px;
  }
  .btn-submit {
    flex: 1; padding: 9px; border-radius: 8px;
    border: none; background: #111; color: #fff; cursor: pointer;
    font-size: 13px; font-weight: 600;
  }
  .btn-submit:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-cancel:hover { background: #eee; }
  .btn-submit:not(:disabled):hover { background: #333; }
</style>
```

### Step 2: Type-check

```bash
cd /Users/jack/Private/shelf && npx tsc --noEmit 2>&1 | head -30
```

### Step 3: Final build check (Rust + TypeScript)

```bash
cd /Users/jack/Private/shelf/src-tauri && cargo check 2>&1 | grep -E "^error" | head -20
cd /Users/jack/Private/shelf && npx tsc --noEmit 2>&1 | head -30
```

Both must return 0 errors.

- [ ] Step 1: Replace `src/lib/components/modals/AddProjectModal.svelte`
- [ ] Step 2: Run `npx tsc --noEmit`
- [ ] Step 3: Run final Rust + TypeScript check
- [ ] **Step 4: Commit**
```bash
git add src/lib/components/modals/AddProjectModal.svelte
git commit -m "feat: AddProjectModal 구현 (이름, 색상, 유형, 기간)"
```

---

## Self-Review

### 1. Spec coverage check

| Requirement | Task |
|---|---|
| DB priority field + migration v3 | Task 1 |
| Rust commands with priority | Task 2 |
| Frontend store priority | Task 3 |
| CompactPanel tab-based routing | Task 4 |
| CompactTodo (stats + task list + add) | Task 5 |
| CompactCal (mini cal with bars + schedule) | Task 6 |
| CompactVault | Task 7 |
| ExpandedDashboard tab-aware | Task 8 |
| ExpandedTodo (project panel + task panel) | Task 9 |
| ExpandedCalendar (cal + chips + schedule) | Task 10 |
| ExpandedVault | Task 11 |
| AddTodoModal (priority, project, category, due date) | Task 12 |
| AddProjectModal (name, color, type, dates) | Task 13 |

All requirements covered.

### 2. Placeholder scan

No TBD, TODO, or "implement later" in any task code blocks.

### 3. Type consistency

- `PRIORITY_COLORS` defined in `todos.ts` (Task 3), imported in CompactTodo (Task 5), ExpandedTodo (Task 9), AddTodoModal (Task 12)
- `PROJECT_COLORS` already in `projects.ts`, imported in AddProjectModal (Task 13)
- `CATEGORY_COLORS` already in `projects.ts`, imported in CompactTodo (Task 5), ExpandedTodo (Task 9)
- `createTodo` signature: `(title, note, due_date, recurrence, category, priority)` — consistent in Tasks 3, 5, 12
- `createProject` signature: `(name, color, category, start_date, end_date)` — consistent in Tasks 3, 13
