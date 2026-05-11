# Shelf v2 — 설계 문서

> 작성일: 2026-05-11  
> 스택: Tauri v2 + SvelteKit + Rust + SQLite | macOS (향후 Windows 지원 예정)  
> 참고: `docs/plan/shelf-v2-spec.md`, `docs/plan/mockup.html`

---

## 1. 목표

단순 Todo + Calendar 조합을 **프리랜서/개인 작업자용 프로젝트 스케줄러**로 확장한다.  
핵심: 프로젝트 기간 캘린더 시각화 + 오늘 할 일·일정 대시보드.

향후 Windows까지 출시할 계획이므로, 플랫폼 종속 코드는 분기 처리한다.

---

## 2. UI 구조 — 3단계 창 상태

```
[💊 pill, 52px] ──▶ [📋 panel, 332px] ──▶ [⬜ expanded, 612px]
                 ◀─────────────────────────────────────────────
```

| 상태 | 총 너비 | 구성 |
|------|---------|------|
| `pill` | 52px | 필 단독 |
| `panel` | 332px (52 + 280) | 필 + 컴팩트 패널 슬라이드 인 |
| `expanded` | 612px (52 + 560) | 필 + 풀 대시보드 |

높이는 모든 상태에서 600px 고정.

### 전환 규칙

- 탭 아이콘 클릭 → `panel` + 해당 탭 활성
- 같은 아이콘 재클릭 → `pill`
- `⬜ 확장` 클릭 → `expanded`
- `▼ 컴팩트` 클릭 → `panel`
- `×` 닫기 클릭 → `pill`

### 스냅 동작

제거. v2에서는 드래그로만 위치 이동.

---

## 3. 컴포넌트 구조 (접근법 B — 클린 분리)

```
+layout.svelte          ← 얇은 껍데기, 상태에 따라 컴포넌트 조합
  ├── Pill.svelte        ← 52px 좌측 고정 (traffic lights + 탭 + 확장 버튼)
  ├── CompactPanel.svelte ← 280px 슬라이드 패널 (헤더 + <slot>)
  │     └── 기존 라우트 그대로 (/todo, /cal, /vault)
  └── ExpandedDashboard.svelte ← 560px 풀 대시보드
        ├── DashboardHeader.svelte  (프로필 + 통계 + 날짜)
        ├── CalendarView.svelte     (기존 확장)
        ├── TodaySchedule.svelte    (오늘 일정 — Phase 2)
        └── TodayTasks.svelte       (오늘 할 일)
```

### 기존 컴포넌트 처리

| 파일 | 처리 |
|------|------|
| `Sidebar.svelte` | `Pill.svelte`로 대체 후 삭제 |
| `/routes/todo`, `/routes/cal`, `/routes/vault` | CompactPanel 내부에서 그대로 사용 |
| `CalendarView.svelte` | Phase 2에서 프로젝트 바 추가 확장 |

### 플랫폼 분기 (Traffic Lights)

```svelte
<!-- Pill.svelte -->
{#if platform === 'macos'}
  <div class="traffic-lights"><!-- 빨강/노랑 닷 --></div>
{:else}
  <button class="win-close" onclick={closeWindow}>×</button>
{/if}
```

`platform`은 Rust 커맨드 `get_platform() → String`으로 감지 (`std::env::consts::OS` 사용).  
외부 플러그인 없이 해결. 앱 초기화 시 한 번 호출 후 store에 저장.

### Svelte 5 (Runes 모드)

코드베이스가 Svelte 5 runes 모드 사용 중. `<slot>` 대신 snippet 사용:

```svelte
<!-- CompactPanel.svelte -->
<script lang="ts">
  let { children } = $props()
</script>
<div class="panel">
  <header>...</header>
  {@render children()}
</div>
```

---

## 4. 상태 관리

### window.ts 변경

```typescript
// 제거: collapsed, EXPANDED_WIDTH, COLLAPSED_WIDTH, initWindowListener(스냅 로직)
// 추가:
export type WindowState = 'pill' | 'panel' | 'expanded'
export type Tab = 'todo' | 'cal' | 'vault'

export const windowState = writable<WindowState>('pill')
export const activeTab   = writable<Tab>('todo')

const PILL_W     = 52
const PANEL_W    = 332   // 52 + 280
const EXPANDED_W = 612   // 52 + 560
const HEIGHT     = 600

export async function setState(next: WindowState): Promise<void>
export async function openTab(tab: Tab): Promise<void>
```

창 상태는 메모리 전용 (DB 저장 없음). 앱 재시작 시 항상 `pill`로 초기화.

---

## 5. 데이터 모델

### 마이그레이션 전략

`PRAGMA user_version`으로 버전 관리. 기존 코드에 버전 관리 없으므로 v0 → v1 → v2 순 적용.

### v1 — 신규 테이블

```sql
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
-- 초기 프로필: name='나', bio='나의 작업 대시보드'

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
```

### v2 — 기존 테이블 컬럼 추가

```sql
ALTER TABLE todos  ADD COLUMN project_id INTEGER REFERENCES projects(id);
ALTER TABLE todos  ADD COLUMN category   TEXT NOT NULL DEFAULT '작업';
ALTER TABLE events ADD COLUMN category   TEXT NOT NULL DEFAULT '작업';
```

`NOT NULL DEFAULT` 조합이므로 기존 행 영향 없음. `vault_items` 변경 없음.

### TypeScript 인터페이스 추가/변경

```typescript
interface Project {
  id: number
  name: string
  color: string
  category: string
  start_date: string | null
  end_date: string | null
  archived: boolean
  created_at: string
}

interface Settings {
  name: string
  bio: string
}

// Todo — 기존 + 추가
interface Todo {
  // ...기존 필드 유지
  project_id: number | null  // 신규
  category: string           // 신규
}

// Event — 기존 + 추가
interface Event {
  // ...기존 필드 유지
  category: string           // 신규
}
```

---

## 6. Rust 커맨드 추가

```
// 플랫폼 감지
get_platform()                                              → String  // "macos" | "windows" | "linux"

// projects
get_projects()                                              → Vec<Project>
create_project(name, color, category, start_date, end_date) → i64
update_project(id, name, color, category, start_date, end_date) → ()
delete_project(id)                                          → ()
archive_project(id)                                         → ()

// settings (프로필)
get_settings()                  → Settings
set_setting(key, value)         → ()

// todo — 기존 시그니처에 category 추가
create_todo(title, note, due_date, recurrence, category)    → i64
update_todo(id, title, note, due_date, category)            → ()
```

---

## 7. 디자인 시스템

### 색상

| 역할 | Hex |
|------|-----|
| 강조 (라임) | `#AAED3A` |
| 필 배경 | `#1E1E2E` |
| 패널 배경 | `#FFFFFF` |
| 서브 배경 | `#F7F7F7` |
| 테두리 | `#F0F0F0` |

### 카테고리 배지

| 카테고리 | 배경 | 텍스트 |
|----------|------|--------|
| 작업 | `#E8F4FD` | `#1A6FA8` |
| 클라이언트 | `#FFF0F0` | `#C0392B` |
| 개인 | `#F0FFF4` | `#27AE60` |
| work | `#FFF8E1` | `#E67E22` |
| 사일 | `#F3F0FF` | `#6C5CE7` |

### 프로젝트 색상 프리셋 (8종)

`#7C3AED` `#10B981` `#EF4444` `#F59E0B` `#06B6D4` `#EC4899` `#6B7280` `#4F46E5`

---

## 8. Phase 로드맵

### Phase 1 (현재 범위)

- [ ] DB 마이그레이션 v1, v2 (`db/mod.rs`)
- [ ] Rust 커맨드: projects CRUD, settings get/set
- [ ] `projects.ts` store 신규
- [ ] `todos.ts`, `events.ts` category 필드 추가
- [ ] 레이아웃 재설계: `Pill.svelte` + `CompactPanel.svelte` + `+layout.svelte`
- [ ] `window.ts` 3상태 관리, 스냅 제거
- [ ] `ExpandedDashboard.svelte` 골격 + `DashboardHeader.svelte` + `TodayTasks.svelte`
- [ ] `TodoForm.svelte` 카테고리 드롭다운 추가
- [ ] `tauri.conf.json` 창 크기 조정

### Phase 2

- [ ] `CalendarView.svelte` 프로젝트 기간 바 렌더링
- [ ] 날짜 클릭 → 진행 중 프로젝트 팝오버
- [ ] `ProjectChips.svelte` 캘린더 하단 칩 목록
- [ ] `TodaySchedule.svelte` (ExpandedDashboard 우측)

### Phase 3

- [ ] 간트 뷰 (전체 보기 토글)
- [ ] 프로젝트 편집 모달
- [ ] 법정공휴일 데이터
- [ ] Windows 창 컨트롤 UI 완성

---

## 9. 외부 의존성

- 외부 연동 없음 (OAuth, 클라우드 동기화 제외)
- 신규 npm/Cargo 패키지 추가 없음 — `get_platform()` Rust 커맨드로 플랫폼 감지
- 기존 의존성 유지: `rusqlite`, `once_cell`, `serde`

---

## 10. 주의사항

- 기존 `recurrence` 로직 (Rust) 변경 없음
- Vault 패널 기능 변경 없음 (UI 위치만 pill 내 탭으로 이동)
- `macOSPrivateApi: true` — Windows 빌드 시 `tauri.conf.json` 플랫폼별 설정 분리 필요
- `ALTER TABLE ADD COLUMN` 순서: projects 테이블 먼저 생성 후 todos의 `project_id` FK 추가
