# `fork-add-modal-state` 브랜치에서의 모달·셸 레이아웃 (조사 기록)

조사 시점: 로컬 브랜치 `fork-add-modal-state` 최신 커밋 `f1b252b` (`docs: shell 상태별 기본 크기 표기 갱신`).  
조사 후 작업 트리는 `main`으로 복귀·스태시 복원함.

## 요약

이 브랜치에서는 **태스크/Vault/프로젝트 모달이 여전히 `#app` 안**에 있고, **`html, body` 모두 `overflow: hidden`** 인 구조가 `main`과 유사하다. 다만 **컴팩트(`panel`) 상태의 논리 창 높이가 `600px`** 로 잡혀 있어, 이후 `main`에서 보이는 것처럼 **짧은 높이(예: ~340px)에서 모달이 세로로 잘리는 상황이 상대적으로 덜 드러난다.**

## DOM 구조

- `<div id="app">` … 컴팩트 패널·확장 패널 등 … 그 **안에** `<!-- 모달들 -->` 블록이 들어 있음.
- `</div><!-- /app -->` 직전에 **삭제 토스트**(`del-toast`)도 함께 위치.

즉, 오버레이를 `#app` 밖으로 빼지 않았고, 토스트도 아직 `#app` 내부다.

## CSS (모달 관련)

| 항목 | `fork-add-modal-state` 내용 |
|------|------------------------------|
| `html, body` | `width/height: 100%`, **`overflow: hidden`**, `background: transparent` |
| `#app` (초기 CSS) | `position: relative`, **`622px × 600px`** (실행 후 JS가 상태별로 덮어씀) |
| `.modal-overlay` | `position: fixed; inset: 0`, `display: flex`, **`align-items: center`**, **`justify-content: center`**, `z-index: 100` — **오버레이 자체에는 `overflow-y` 없음** |
| `.modal` | `width: 300px`, **`max-height: 92%`**, **`overflow-y: auto`** |

## JS: 창·셸 크기 (`resizeWindow`)

`SIZES` (논리 픽셀):

| 상태 | 크기 |
|------|------|
| `pill` | 52 × **600** |
| `panel` | **342 × 600** |
| `expanded` (탭별) | **622 × 600** |

`resizeWindow`가 `#app`의 `width`/`height`와 Tauri `setSize`를 이 값에 맞춘다.

**포인트:** 컴팩트 패널이 **세로 600px** 이라, 같은 모달 마크업이라도 **모달 카드가 창 높이를 넘지 않거나**, 넘어도 **`92%` + `.modal` 내부 스크롤**로 커버하기 쉽다. 반면 `main` 쪽에서 컴팩트 높이를 **로그/목록에 맞춰 훨씬 낮게**(예: 340px 근처) 잡으면, **`#app` 내부 + `overflow: hidden` 체인**에서 모달이 **시각적으로 잘리는** 문제가 부각될 수 있다.

## `main`과의 차이 (정성)

| 구분 | `fork-add-modal-state` | 이후 `main` (대화 맥락 기준) |
|------|-------------------------|------------------------------|
| 컴팩트 창 높이 | 600px 고정 계열 | 상태·탭별로 더 낮게 조정되는 경우 있음 |
| 모달 DOM 위치 | `#app` 내부 | 일부 작업에서 `#app` 밖으로 이동·루트 `overflow` 분리 등 시도 |
| 테마·토스트 | 라이트 베이스 UI, 토스트도 `#app` 안 | 다크 글래스·토스트 `#app` 밖 등 진화 |

## 참고 파일 (해당 브랜치 기준)

- `frontend/index.html`: 상단 스타일(~L237 모달), `SIZES`·`resizeWindow`(~L774~), 모달 HTML(~L502~696).

이 문서는 **레그레션/디자인 비교**용으로만 두었으며, 현재 `main`의 정답 구현을 대체하지 않는다.
