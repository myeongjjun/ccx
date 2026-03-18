# ADR-005: CLI Naming Evaluation

- **Date**: 2026-03-18
- **Status**: accepted
- **Deciders**: user, Claude Code, Codex GPT-5.2
- **Supersedes**: none

## Context

현재 공개 전 프로젝트 이름과 CLI 이름을 확정해야 한다.

GitHub public 공개 전에 이름이 적절한지, 기존 유명 CLI/패키지와 충돌하지 않는지 검토가 필요하다.

### 좋은 CLI 이름의 기준

매일 수십 번 치는 커맨드라면 길이가 핵심이다. 성공한 CLI 도구들의 패턴:

| 도구 | 글자수 | 특징 |
|------|:------:|------|
| `rg` (ripgrep) | 2 | 짧고 빠름 |
| `fd` (find) | 2 | 기능 축약 |
| `fzf` (fuzzy finder) | 3 | 약어 |
| `zj` (zellij) | 2 | 극단적 축약 |
| `uv` (Python pkg) | 2 | 브랜드형 |

### 충돌 스캔 방법

1차: Codex GPT-5.2 웹 검색으로 PyPI, npm, crates.io, Homebrew 스캔
2차: Claude Code가 npm, crates.io 직접 웹 검색으로 교차 검증

## Evaluation Summary

짧고 반복 타이핑되는 CLI라는 점을 우선해 `ccx` 단독 이름을 재검토했다.

### 레지스트리 충돌 검증 (Claude Code 직접 웹 검색)

| 레지스트리 | `ccx` 존재 여부 | 상태 |
|-----------|----------------|------|
| **npm** | `ccx` 패키지 존재 | v1.0.0, **7년 전** 마지막 발행 — 사실상 abandoned |
| **crates.io** | 없음 | `cctx` (Claude Code context manager)만 존재 |
| **PyPI** | 없음 | |
| **Homebrew** | 없음 | |
| **로컬 머신** | `which ccx` → not found | |

### `ccx` 단독 평가

| 기준 | 점수 | 비고 |
|------|:----:|------|
| 글자수 | **3** | fzf, rg급 |
| 타이핑 | 5/5 | 하이픈 없음, 왼손 홈로우 근처 |
| 기억성 | 5/5 | cc+cx → ccx 즉시 연상 |
| 명확성 | 4/5 | cc/cx 사용자라면 즉시 이해, 외부인은 불투명 |
| 발음 | 자연스러움 | "씨씨엑스" |
| 충돌 | 거의 없음 | npm abandoned 패키지만, crates.io 깨끗 |
| 미래확장 | 3/5 | cc+cx에 묶이지만 브랜드로 전환 가능 |

## Final Comparison

| 이름 | 글자수 | 타이핑 | 명확성 | 확장성 | 충돌 | 총평 |
|------|:------:|:------:|:------:|:------:|:----:|------|
| **`ccx`** | **3** | **5** | 4 | 3 | 거의 없음 | **가장 실용적. 매일 치는 커맨드로 최적** |
| `termhop` | 7 | 5 | 3 | **5** | 없음 | 확장성 최고, AI 뉘앙스 약함 |
| `ccx-focus` | 9 | 3 | **5** | 2 | 없음 | 명확성 최고, 너무 길다 |
| 이전 작업명 | 8 | 3 | 4 | 2 | 없음 | 반복 사용 비용이 큼 |

## Decision

프로젝트 이름, 기본 CLI 이름, Rust package/bin 이름, Python package 이름을 모두 **`ccx`** 로 통일한다.

이전 작업명은 내부 설명용으로는 유용했지만, 공개 브랜드와 반복 타이핑되는 커맨드로는 너무 길다. 이 프로젝트는 `fzf` 같은 짧고 선점 가능한 CLI 도구를 지향하므로, 길이와 재사용 빈도를 우선하는 이름을 선택한다.

## Consequences

### 이름 변경 시 영향 범위

- `pyproject.toml` — package name, scripts entry
- `Cargo.toml` — package name, bin name
- `src/` 디렉토리 — Python 모듈 이름 정리
- `README.md`, `PRD.md` — 문서 내 참조
- `.claude/skills/` — 스킬 정의 내 참조
- cache 경로 — 이전 이름 기반 디렉터리명 정리

### Positive
- 공개 전 이름 확정으로 rename 비용 최소화
- 짧은 실행명으로 반복 사용 비용을 줄일 수 있음
- 공개 브랜드와 실제 명령이 일치함

### Negative
- 이름 변경 시 기존 작업물 전체 업데이트 필요
- 기존 작업명 기준 코드/문서/하네스 수정 비용

## References

- npm `ccx` 패키지: v1.0.0, 7년 전 발행, abandoned — [npmjs.com/package/ccx](https://www.npmjs.com/package/ccx)
- crates.io `cctx`: Claude Code context manager — [crates.io/crates/cctx](https://crates.io/crates/cctx)
- [ADR-003](2026-03-18-use-rust-for-primary-implementation.md) — Rust 포팅 (Cargo.toml 이름 변경 포함)
