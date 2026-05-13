<div align="center">

# 🧲 Shelf

**macOS를 위한 미니멀 플로팅 워크스페이스**

항상 위에. 항상 방해되지 않게.

[![Release](https://img.shields.io/github/v/release/devtaco30/shelf?style=flat-square&color=lime)](https://github.com/devtaco30/shelf/releases/latest)
[![macOS](https://img.shields.io/badge/macOS-12%2B-black?style=flat-square&logo=apple)](https://github.com/devtaco30/shelf/releases/latest)
[![License](https://img.shields.io/badge/license-MIT-lime?style=flat-square)](LICENSE)

[English](README.md)

<br/>

<img src="assets/screenshot-todo-exp.png" width="80%">

<br/>

<img src="assets/screenshot-cal-exp.png" width="32%">
<img src="assets/screenshot-vault-locked.png" width="32%">
<img src="assets/screenshot-memo-exp.png" width="32%">

</div>

---

## Shelf란?

Shelf는 화면 가장자리에 떠 있는 플로팅 사이드바입니다. 모든 창 위에 떠 있으면서도 방해되지 않는 작은 필 형태로, 필요할 때 클릭 한 번으로 전체 패널로 펼쳐집니다.

할 일, 캘린더, 암호화 볼트, 메모. 모든 것이 한 곳에, 언제나 한 클릭 거리에.

---

## 기능

**태스크** — 프로젝트, 우선순위, 마감일을 설정하여 할 일을 관리합니다. 컬러 라벨로 프로젝트를 구분하고 진행 상황을 한눈에 파악합니다.

**캘린더** — 깔끔한 월간 그리드에서 일정을 확인하고 추가합니다. 반복 일정을 지원합니다.

**볼트** — 패스워드와 카드 정보를 위한 암호화 저장소. AES-256 암호화와 Touch ID로 보호됩니다. 데이터는 기기 밖으로 나가지 않습니다.

**메모** — 빠른 메모를 위한 스크래치패드. 불필요한 과정 없이 바로 입력합니다.

**테마** — Dark Glass(어두운 패널, 라임 포인트)와 Soft Color(밝은 패널, 바이올렛 포인트) 중 선택. 패널 투명도를 조절할 수 있습니다.

---

## 다운로드

→ **[최신 릴리즈 다운로드](https://github.com/devtaco30/shelf/releases/latest)**

`Shelf_x.x.x_aarch64.dmg` (Apple Silicon) 또는 `Shelf_x.x.x_x64.dmg` (Intel)를 다운로드하고, DMG를 열어 Shelf를 Applications 폴더로 드래그합니다.

> **첫 실행 시:** 앱이 공증되지 않아 macOS 보안 경고가 표시될 수 있습니다.
> 앱을 우클릭 → **열기** → **열기**를 선택하세요.

---

## 직접 빌드

**요구 사항:** Node.js 18+, Rust (stable), macOS 12+

```bash
git clone https://github.com/devtaco30/shelf.git
cd shelf
npm ci
npm run tauri build
```

빌드된 `.dmg` 파일은 `src-tauri/target/release/bundle/dmg/`에 생성됩니다.

**개발 모드:**
```bash
npm run tauri:dev
```

---

## 기술 스택

- [Tauri 2](https://v2.tauri.app/) — Rust + WebView
- Vanilla JS / HTML / CSS — 프레임워크 없음
- SQLite — 로컬 데이터 저장
- AES-256-GCM — 볼트 암호화
- macOS Keychain + Touch ID — 키 관리

---

## 라이선스

MIT
