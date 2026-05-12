# Shelf 프로젝트 구조·실행 파이프라인

문서 기준: 저장소 루트 `/Users/jack/Private/shelf` 기준으로 재조사한 내용입니다.

---

## 1. 한 줄 요약

| 계층 | 역할 |
|------|------|
| **`frontend/`** | Tauri 웹뷰가 로드하는 **정적 UI** (`index.html` 단일 진입 + 인라인 스타일/스크립트). |
| **`src-tauri/`** | **Rust** 앱 셸, SQLite, Tauri `invoke` 커맨드, Vault 암호화·macOS 인증. |
| **루트 `package.json`** | `@tauri-apps/cli`, `serve`; **`npm run tauri`** 로 로컬 CLI 버전 고정. |

실제 제품 빌드 경로는 **`frontend` ↔ `src-tauri` 만**입니다.

---

## 2. 디렉터리 맵

```
shelf/
├── frontend/
│   └── index.html          # UI 전부 (할 일·캘린더·Vault, 상태 머신, invoke 브릿지)
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json     # 창·번들·devUrl·frontendDist
│   ├── capabilities/
│   │   └── default.json    # 창 API 권한 (set-size, start-dragging 등)
│   └── src/
│       ├── lib.rs          # 진입, DB 초기화, invoke 핸들러 등록
│       ├── main.rs
│       ├── commands/       # todo, event, vault, project, settings
│       ├── db/             # rusqlite + 마이그레이션
│       ├── recurrence/     # 반복 규칙·다음 occurrence
│       └── vault/          # crypto, auth (macOS)
├── docs/                   # 기획·설계 문서
├── package.json
└── README.md
```

### 레거시 가능성

루트에 **`src/`**(SvelteKit 라우트·컴포넌트)·**`src/app.html`** 등이 보인다면, 과거 템플릿 잔존물일 수 있습니다. **`tauri.conf.json`의 `frontendDist`는 `../frontend`** 이므로, 별도 빌드 파이프가 없으면 **Tauri는 해당 폴더를 쓰지 않습니다.**

루트에 **`vite.config.js` / `svelte.config.js`** 가 없다면 SvelteKit 빌드는 현재 연결되어 있지 않은 상태입니다.

---

## 3. 개발·빌드 흐름

| 명령 | 동작 |
|------|------|
| `npm run dev` | `serve`로 `frontend`만 포트 **1420** (Tauri 없음). |
| **`npm run tauri:dev`** (권장) | **`concurrently`**로 **`dev:sync`**(browser-sync, 로그 접두사 `sync`)와 **`dev:tauri`**(`RUST_LOG=info` + `tauri dev`, 접두사 `tauri`)를 한 터미널에서 실행. `wait-on`으로 1420 준비 후 Tauri 기동. |
| `npm run tauri -- dev` | `tauri.conf.json`의 `beforeDevCommand`가 비어 있으면 프론트 서버를 따로 띄워야 함 → **`tauri:dev` 사용 권장.** |
| `npm run tauri build` | `frontendDist` 복사 후 Rust 릴리즈·번들 (macOS면 `.app` / `.dmg` 등). |

CLI는 **`npm`/프로젝트 로컬**을 쓰면 `package-lock.json`과 버전이 맞습니다. 전역 `cargo install tauri-cli` 후 `cargo tauri`는 동작하지만 버전 분리에 유의합니다.

---

## 4. Rust 쪽 개요

- **`lib.rs`**: `db::init(app_data_dir)` → `~/Library/Application Support/.../shelf.db` 등 경로에 SQLite.
- **`invoke_handler`**: 할 일·이벤트·Vault·프로젝트·설정·`get_platform`.
- **`commands/event.rs`**: `get_events` 시 반복 일정의 `recurrence_next` 갱신(투두와 유사).
- **Vault**: `vault/` + `commands/vault.rs`, Touch ID 등은 macOS 모듈.

---

## 5. UI shell 상태 용어 (코드·기획 공통)

| 용어 | 코드 상 `curState` / 동작 | 창 크기 소스 |
|------|---------------------------|--------------|
| **폴드 상태** | `pill` — 필만 보이고 컴팩트·확장 패널 닫힘 | `SIZES.pill` — 기본 **420×52** (`SHELL_DIMS.foldLogicalHeight`) |
| **컴팩트 상태** | `panel` — 좁은 패널만 열림 | `SIZES.panel` — 기본 **600×342** (`compactLogicalHeight`) |
| **익스팬드(확장) 상태** | `expanded` — 넓은 패널 | `SIZES.expanded[탭]` — 기본 높이 **600** + 우하단 핸들로 **높이·폭 변경 가능** |

`resizeWindow(state)`는 OS 창에 **`LogicalSize(w,h)`** 를 적용한다. **폴드/컴팩트에서는 반드시 `SIZES.pill` / `SIZES.panel`만 쓰고**, 확장에서만 `SIZES.expanded`를 쓴다.  
(과거 버그: 폴드/컴팩트일 때 `exSize()`(확장 크기)를 참조해, 확장을 키운 뒤 접으면 **창 높이가 확장 때와 같게 남는** 문제가 있었다.)

---

## 6. 프론트(`frontend/index.html`) 개요

- **`invoke(cmd, args)`**: `window.__TAURI__.core.invoke` 래퍼.
- **`getCurrentWindow`, `LogicalSize`**: `resizeWindow`, 리사이즈 핸들에서 **`setSize`** 만 사용 (**`setPosition` 호출 없음**).
- **UI 상태 머신**: 폴드(`pill`) → 컴팩트(`panel`) → 익스팬드(`expanded`); 진입점은 **`setState`**.
- **`data-tauri-drag-region`**: 필·컴팩트/확장 헤더 등. **폴드(`shell-fold`)일 때** `#app` 전체를 덮는 `.fold-drag-shim`에도 부착 — 필이 세로 중앙이라 생기는 **위·아래 투명 영역**에서도 드래그되게 함. 닫힌 `.cpanel`/`.expanel`은 `pointer-events:none`으로 레이어가 드래그를 가로막지 않게 함.
- **기동 시** `resizeWindow('pill')`로 OS 창 크기를 `SIZES.pill`에 맞춤 (세션 복원 등으로 창만 커진 채 시작하는 경우 완화).

---

## 7. 창 설정·권한 (`tauri.conf.json` / capabilities)

| 설정 | 값 | 비고 |
|------|-----|------|
| `decorations` | `false` | 시스템 타이틀바 없음 → 드래그는 **`data-tauri-drag-region`** 에 의존. |
| `transparent` | `true` | macOS에서 그림자 등은 `lib.rs`에서 `set_shadow(false)`. |
| `alwaysOnTop` | `true` | 다른 창보다 위 레이어; “막힌 느낌”과 연관 가능(아래 8절). |
| `resizable` | `true` | 확장 패널 리사이즈 스크립트가 **`setSize`** 로 논리 크기 변경. |
| 초기 위치 | `x: 100`, `y: 100` | 한 번만; 이후 OS가 위치 기억할 수 있음. |

`capabilities/default.json`에는 **`core:window:allow-set-position`**, **`allow-start-dragging`** 등이 포함되어 있어, 권한 부족으로 드래그가 막히는 구성은 아닙니다.

---

## 8. “창이 위로 안 올라간다 / 막힌다” 조사 메모

코드상 **창 Y좌표를 고정하거나 상단으로 클램프하는 Rust/JS 로직은 없음**.

가능한 원인 후보(우선순위 없음):

1. **(수정됨) 폴드·컴팩트에서도 확장과 같은 논리 높이로 `setSize` 되던 버그** — `resizeWindow`가 비확장 상태에서 `exSize()`(확장 크기 객체)를 참조하고 있어, 확장에서 창을 키운 뒤 접어도 OS 창 높이가 줄지 않을 수 있었다. **`logicalSizeForShellState`** 로 상태별로 분리함.
2. **드래그 영역**: 실제로 움직이려면 **`data-tauri-drag-region` 이 있는 영역**(좁은 필 바·헤더 띠)에서 드래그해야 함. 본문·버튼 위에서는 이동하지 않음.
3. **`alwaysOnTop`**: 항상 최상단 레벨이라, 다른 앱 창과의 겹침 체감이 일반 창과 다를 수 있음.
4. **리사이즈 핸들**: 우하단 핸들은 **`nwse-resize`** 로 크기만 변경; 창 위치는 바꾸지 않음.
5. **환경**: 다른 오버레이(메뉴바 드롭다운 아님), 스페이스/디스플레이 경계, 접근성 도구 등은 별도 재현 필요.

**추가 디버깅**: 문제가 남으면 `alwaysOnTop` 을 잠시 끄거나, DevTools로 드래그 영역 히트 테스트 확인.

---

## 9. 변경 시 참고 파일

| 목적 | 파일 |
|------|------|
| 창 크기·항상 위·투명 | `src-tauri/tauri.conf.json` |
| 웹뷰에서 창 API | `src-tauri/capabilities/default.json`, `frontend/index.html` |
| IPC 추가 | `src-tauri/src/lib.rs`, `src-tauri/src/commands/*.rs` |
| DB 스키마 | `src-tauri/src/db/mod.rs` (마이그레이션) |

---

이 문서는 레포 구조 정리용이며, 동작 변경이 있으면 함께 갱신하는 것을 권장합니다.
