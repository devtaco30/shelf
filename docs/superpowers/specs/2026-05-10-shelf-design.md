# Shelf — 디자인 스펙

**날짜**: 2026-05-10  
**상태**: 확정  

---

## 개요

Shelf는 macOS용 개인 생산성 앱이다. 할 일 관리, 캘린더, 보안 메모를 하나의 플로팅 윈도우에서 제공한다. 항상 최상단에 고정되며, 화면 가장자리로 접어두면 책갈피만 노출되는 슬라이드오버 방식으로 동작한다. Windows 지원을 고려한 크로스플랫폼 구조로 설계한다.

---

## 기술 스택

| 레이어 | 기술 |
|--------|------|
| Frontend | Svelte 5 + TypeScript |
| Backend | Tauri 2 (Rust) |
| DB | SQLite (`tauri-plugin-sql`) |
| 암호화 | `aes-gcm` Rust crate |
| 생체인증 | `tauri-plugin-biometric` |
| 키 저장 | `keyring` Rust crate (OS 네이티브 키체인 추상화) |

---

## UI/UX

### 레이아웃

```
┌──────────────────┬──┐
│                  │📋│  ← Todo 책갈피
│   콘텐츠 영역    │📅│  ← Cal 책갈피
│                  │🔒│  ← Vault 책갈피
└──────────────────┴──┘
```

탭은 우측 책갈피 형태로 배치한다. 상단 탭 바 없음.

### 윈도우 동작

**펼쳐진 상태 (기본)**
- 항상 최상단 고정 (always-on-top)
- 크기/위치 자유롭게 드래그 조정
- 마지막 위치와 크기를 로컬에 저장해 재실행 시 복원

**접힌 상태 (슬라이드오버)**
- 창을 화면 오른쪽 끝으로 드래그하면 자동으로 접힘
- 우측 책갈피(아이콘)만 화면 가장자리에 노출
- 책갈피 클릭 시 해당 탭으로 바로 펼쳐짐
- 단축키 `⌘ + Shift + S`로 접기/펼치기 토글

---

## 탭 구성

### 1. Todo 탭 (`📋`)

할 일 목록 관리. 반복 일정을 지원한다.

**반복 일정 처리 규칙**
- 마감일이 지났고 완료되지 않은 반복 항목 → 해당 회차 드롭 (과거 항목 누적 없음)
- 드롭 후 `recurrence_next` 기준으로 다음 회차 자동 생성
- 체크 시점: 앱 실행 시 + 자정마다 백그라운드 실행

### 2. Cal 탭 (`📅`)

월/주 단위 캘린더 뷰. Todo 항목과 별도 이벤트를 함께 표시한다.

### 3. Vault 탭 (`🔒`)

민감한 정보(니모닉, 비밀번호 등)를 암호화해 저장하는 보안 메모.  
탭 접근 시 인증이 필요하다.

---

## 데이터 모델 (SQLite)

### `todos`
```sql
id            INTEGER PRIMARY KEY
title         TEXT NOT NULL
note          TEXT
done          BOOLEAN DEFAULT FALSE
due_date      DATETIME
recurrence    TEXT  -- none | daily | weekly | monthly
recurrence_next DATETIME
created_at    DATETIME DEFAULT CURRENT_TIMESTAMP
```

### `events`
```sql
id            INTEGER PRIMARY KEY
title         TEXT NOT NULL
start_at      DATETIME NOT NULL
end_at        DATETIME
recurrence    TEXT  -- none | daily | weekly | monthly
recurrence_next DATETIME
todo_id       INTEGER REFERENCES todos(id)
created_at    DATETIME DEFAULT CURRENT_TIMESTAMP
```

### `vault_items`
```sql
id            INTEGER PRIMARY KEY
title         TEXT NOT NULL
content       BLOB NOT NULL  -- AES-256-GCM 암호화된 바이너리
created_at    DATETIME DEFAULT CURRENT_TIMESTAMP
updated_at    DATETIME DEFAULT CURRENT_TIMESTAMP
```

---

## 보안 설계

### Vault 접근 인증 흐름

```
Vault 탭 클릭
     │
     ▼
Touch ID 요청
     │
 성공 ──────────────→ 복호화 후 열람
     │
 실패/취소
     │
     ▼
비밀번호 입력 폼
     │
 성공 ──────────────→ 복호화 후 열람
     │
 실패 (3회 초과)
     │
     ▼
30초 잠금
```

### 암호화

- 알고리즘: AES-256-GCM
- 암호화 키: OS 네이티브 키체인에 저장
  - macOS → Keychain
  - Windows → Windows Credential Manager
- 비밀번호 저장: PBKDF2로 키 파생, 평문 저장 없음

### 자동 잠금

- 기본 5분 비활성 시 Vault 자동 잠금
- 다른 탭으로 이동해도 Vault는 잠긴 상태 유지
- 설정에서 자동 잠금 시간 변경 가능

### 최초 설정

1. 앱 첫 실행 시 Vault 비밀번호 설정
2. Touch ID 등록 여부 선택 (선택 사항)

### 크로스플랫폼 추상화

```
AuthProvider (공통 인터페이스)
├── MacOSAuthProvider   → Touch ID + Keychain
└── WindowsAuthProvider → Windows Hello + Credential Manager
```

Svelte UI는 플랫폼을 몰라도 된다. 플랫폼 차이는 Rust 레이어에서만 처리.

---

## 프로젝트 구조

```
shelf/
├── src/                        # Svelte UI
│   ├── lib/
│   │   ├── components/         # Tab, TodoItem, CalendarView 등
│   │   └── stores/             # 전역 상태
│   └── routes/                 # Todo / Cal / Vault 페이지
├── src-tauri/                  # Rust 백엔드
│   ├── src/
│   │   ├── auth/               # MacOSAuthProvider, WindowsAuthProvider
│   │   ├── db/                 # SQLite 쿼리
│   │   ├── vault/              # 암호화/복호화
│   │   └── recurrence/         # 반복 일정 체크 로직
│   └── tauri.conf.json
└── docs/
    └── superpowers/specs/
        └── 2026-05-10-shelf-design.md
```

---

## 개발 순서

1. **플로팅 윈도우 + 슬라이드오버** — 기본 shell, always-on-top, 책갈피 UI
2. **Todo 탭** — CRUD + 반복 일정 드롭 로직
3. **Cal 탭** — 달력 뷰 + Todo/이벤트 연동
4. **Vault 탭** — AES-256-GCM 암호화 + 생체인증

---

## 미결 사항

- 자동 잠금 시간 기본값 (5분) — 추후 설정 UI에서 변경 가능
- Cal 탭 기본 뷰 (월/주) — 구현 시 결정
- Windows 지원 시점 — macOS v1 완성 후 결정
