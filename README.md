# Shelf

macOS용 Tauri 2 앱입니다. UI는 `frontend/` 정적 파일(주로 `index.html`), 로직·저장소는 `src-tauri/` Rust입니다.

## 개발

- 프론트만: `npm run dev` — `frontend`를 1420 포트로 서빙합니다.
- 앱 전체(권장): **`npm run tauri:dev`** — **browser-sync**(프론트 서버 로그, 접두사 `sync`)와 **`tauri dev`**(Cargo/Rust 로그, 접두사 `tauri`)를 한 터미널에서 동시에 띄웁니다. Rust 쪽은 기본으로 `RUST_LOG=info`를 넣어 두었습니다.
- 예전 방식: `npm run tauri -- dev` — `beforeDevCommand`가 비어 있으므로 **먼저 `npm run dev:sync`를 다른 터미널에서** 띄우거나, **`npm run tauri:dev`만** 쓰는 편이 안전합니다.

## 창 레이아웃 디버그 (선택)

Web Inspector 콘솔에서 화면 작업 영역 높이·창 논리 높이·위치 등을 로깅하려면 앱을 **`http://localhost:1420/?debugWindow=1`** 로 열거나, 한 번 `localStorage.setItem('shelfDebugWindow','1')` 후 새로고침한다.

## 빌드

```bash
npm run tauri build
```
