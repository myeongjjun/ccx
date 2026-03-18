# ADR-007: Support Workset Isolation via Window Minimization

- **Date**: 2026-03-18
- **Status**: accepted
- **Deciders**: user, Codex
- **Supersedes**: none

## Context

`promote`는 관련 iTerm2 창들을 최근 전환 흐름 앞으로 끌어오는 데는 유용하지만, 실제 업무 전환 관점에서는 아직 부족했다.

- 사용자는 한 업무를 여러 Claude Code / Codex / shell 창 묶음으로 다룬다.
- 단순 focus 순서 재배열만으로는 unrelated 창이 계속 남아서 `Cmd+\`` 전환이 noisy할 수 있다.
- 업무 전환 시점에는 "관련 창만 남기고 나머지는 접는다"는 동작이 더 직접적이다.

동시에, 이 동작은 일회성 숨김이 아니라 반복 가능한 workset switch가 되어야 한다. 즉 `isolate A` 후 `isolate B`를 실행할 때, 이전에 minimize된 창들도 다시 inventory에서 찾아 복구할 수 있어야 한다.

실측 결과 iTerm2의 AppleScript `windows` inventory에는 miniaturized window도 계속 남아 있다. 따라서 minimize를 사용해도 세션 탐색과 후속 복구가 가능하다.

## Decision

`ccx`는 `promote`와 별도로 **window minimization 기반의 `isolate` 작업 모드**를 지원한다.

구체적으로:

1. session collection은 window의 `miniaturized` 상태를 snapshot에 포함한다.
2. `isolate <query>`는 검색 결과 상위 unique window들을 visible workset으로 남긴다.
3. matching window가 이미 minimize되어 있으면 restore하고, non-matching window는 minimize한다.
4. 마지막에는 best-ranked matching window를 focus한다.
5. minimized window도 subsequent collect/search 대상에 계속 포함된다는 전제를 제품 동작으로 채택한다.

## Alternatives Considered

| Alternative | Pros | Cons |
|-------------|------|------|
| `promote`만 유지 | 구현과 UX가 더 단순 | unrelated 창이 계속 남아 workset 전환 느낌이 약함 |
| 앱 전체 hide/show 활용 | 가시성 제어는 가능 | app-level 동작이라 창 단위 workset 제어와 맞지 않음 |
| dedicated restore stack을 별도로 저장 | 복구 제어가 더 정밀 | 현재 iTerm inventory만으로도 가능한 문제에 상태 관리가 추가됨 |

## Consequences

### Positive
- 업무 전환 시 visible iTerm window 집합을 직접 정리할 수 있다.
- repeated isolate 호출로 다른 workset으로 이동할 수 있다.
- minimize된 창도 inventory에 남으므로 별도 hidden-state DB 없이 구현 가능하다.

### Negative
- `isolate`는 `promote`보다 더 파괴적인 동작이다.
- 사용자가 열어둔 unrelated 창이 갑자기 minimize될 수 있다.
- macOS / iTerm scripting behavior에 더 강하게 의존한다.

### Neutral
- `promote`는 유지된다.
- 향후 dedicated restore/undo 흐름이 필요해지면 이 ADR 위에 추가 결정이 생길 수 있다.

## Implementation Notes

- collected session JSON에는 per-window `miniaturized` boolean을 포함한다.
- `isolate --dry-run`은 restore / keep-visible / minimize / leave-minimized / focus 구분을 노출한다.
- 실행 시에는 matching window를 unminimize하고 non-matching window를 minimize한 뒤, 최상위 window를 focus한다.

## References

- [PRD](../../PRD.md)
- [ADR-002](2026-03-18-terminal-native-tui-primary-surface.md)
- [ADR-006](2026-03-18-adopt-cache-first-cli-defaults.md)
