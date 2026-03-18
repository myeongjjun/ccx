# ADR-010: Adopt Ratatui for a Dedicated Isolate TUI

- **Date**: 2026-03-18
- **Status**: accepted
- **Deciders**: user, Codex
- **Supersedes**: none

## Context

`ccx`에는 이미 `tui`가 있지만, 현재 역할은 검색 결과에서 단일 session을 선택해 focus하는 launcher에 가깝다.

- 기존 `tui`에 `--action isolate`를 넣으면 Enter의 의미가 focus와 workset isolate 사이에서 갈라진다.
- `isolate`는 query 기반으로 여러 window를 남기고 나머지를 minimize하는 workset 동작이라, 단일-session launcher와 UX 모델이 다르다.
- 사용자는 shell 단축키에서 바로 호출할 수 있으면서도, terminal 안에서 GUI처럼 preview가 보이는 isolate 전용 화면을 원한다.

## Decision

`ccx`는 기존 `tui`를 유지하고, 별도의 `isolate-tui` surface를 추가한다.

구체적으로:

1. 기존 `tui`는 Enter = focus인 launcher semantics를 유지한다.
2. `isolate`용 interactive UI는 별도 `isolate-tui` subcommand로 분리한다.
3. 새 화면은 `ratatui`를 사용해 query, visible workset preview, action preview를 동시에 보여준다.
4. CLI `isolate`와 `isolate-tui`는 동일한 isolate planning logic을 공유한다.
5. `ratatui` 도입 범위는 우선 `isolate-tui`에 한정하고, 기존 `tui` 전체 재작성 결정은 이후로 미룬다.

## Alternatives Considered

| Alternative | Pros | Cons |
|-------------|------|------|
| 기존 `tui`에 `--action isolate` 추가 | subcommand 수가 늘지 않음 | focus launcher와 isolate workset switcher semantics가 섞여 UX가 모호해짐 |
| 기존 `crossterm` 직접 렌더링으로 isolate 화면 구현 | 새 의존성이 없음 | richer layout, preview panel, GUI-like polish 구현 비용이 커짐 |
| 기존 `tui` 전체를 `ratatui`로 재작성 | UI stack이 통일됨 | 현재 필요한 범위를 넘어서는 리스크와 변경량이 큼 |

## Consequences

### Positive
- `tui`와 `isolate-tui`의 역할이 분리되어 사용자가 동작을 예측하기 쉽다.
- isolate 전용 화면에서 preview-first UX를 제공할 수 있다.
- isolate planning logic을 공유하면 CLI와 interactive mode 간 결과 불일치를 줄일 수 있다.

### Negative
- subcommand와 UI code path가 하나 더 생긴다.
- `ratatui` dependency가 추가된다.

### Neutral
- 기존 `promote`, `pick`, `search`의 semantics는 이번 결정 범위에 포함되지 않는다.
- 향후 기존 `tui`를 `ratatui`로 옮길지 여부는 별도 결정 사항이다.

## Implementation Notes

- isolate planning은 shared module로 분리한다.
- `isolate-tui`는 query 변경 시 preview를 갱신하고, Enter 시 현재 previewed plan을 적용한다.
- 초기 버전은 `ratatui + crossterm` backend 조합을 사용한다.

## References

- [ADR-009](2026-03-18-keep-cutoff-ties-in-isolate-window-selection.md)
- [README](../../README.md)
