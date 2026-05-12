# Shelf

macOS용 데스크톱 앱입니다. [Tauri 2](https://v2.tauri.app/) 기반이며, UI는 `frontend/` 정적 자산(단일 `index.html` 중심), 데이터·IPC·창 제어는 `src-tauri/` Rust입니다. 로컬에는 SQLite를 사용합니다.

## 요구 사항

- **Node.js** 18 이상(권장 LTS)
- **Rust** toolchain (`rustup` 권장)
- **macOS** — 현재 구성·Vault 등은 macOS 전제에 가깝습니다.

## 클론 후 설치

```bash
git clone git@github.com:devtaco30/shelf.git
cd shelf
npm ci
```

## 개발

| 명령 | 설명 |
|------|------|
| `npm run dev` | `frontend`만 **1420** 포트로 서빙 (Tauri 없음). |
| **`npm run tauri:dev`** | **권장.** browser-sync로 프론트 핫리로드 + `tauri dev`를 한 터미널에서 실행합니다. |
| `npm run tauri -- dev` | `beforeDevCommand`가 비어 있으므로, 별도로 1420 서버가 떠 있어야 합니다. **`tauri:dev` 사용을 권장**합니다. |

Rust 로그는 `tauri:dev` 경로에서 기본 `RUST_LOG=info`입니다.

## 빌드

```bash
npm run tauri build
```

`tauri.conf.json`의 `frontendDist`는 `../frontend`입니다.

## 기능 개요(코드 기준)

- 할 일·이벤트(반복 규칙)·캘린더 뷰
- 메모(SQLite `memos` + IPC)
- Vault(암호화·macOS 인증 연동)
- 프로젝트·설정 IPC

자세한 디렉터리 맵·창 상태(`pill` / `panel` / `expanded`)·디버깅 메모는 [`docs/project-structure.md`](docs/project-structure.md)를 참고하세요.

## 창 레이아웃 디버그(선택)

Web Inspector에서 `http://localhost:1420/?debugWindow=1` 로 열거나, `localStorage.setItem('shelfDebugWindow','1')` 후 새로고침하면 창·레이아웃 관련 로그를 볼 수 있습니다.

## 라이선스

MIT (`package.json` 기준).
