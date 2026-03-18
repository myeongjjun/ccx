# ADR-006: Adopt Cache-First CLI Defaults

- **Date**: 2026-03-18
- **Status**: accepted
- **Deciders**: user, Codex
- **Supersedes**: [ADR-004](2026-03-18-session-collection-caching-strategy.md)

## Context

실측 기준으로 iTerm2 수집 경로는 여전히 느리다.

- `collect --output ...`: 약 2.65초 ~ 2.74초
- `search --cache`: 약 0.00초 ~ 0.01초
- `pick --cache --dry-run`: 약 0.00초
- `promote --cache --dry-run`: 약 0.00초 ~ 0.01초

즉, `collect`를 사용자 hot path에 두면 PRD의 "검색 후 즉시 이동" 경험을 해친다. 반면 캐시를 읽고 검색/랭킹하는 경로는 이미 충분히 빠르다.

기존 ADR-004는 launchd 기반 주기적 collector를 MVP 기본으로 제안했지만, 배포 직전 기준으로는 다음 문제가 있었다.

- launchd 설치/제거 UX가 아직 구현되지 않음
- CLI 기본 동작이 아직 launchd 전제를 충족하지 않음
- 기본 UX를 개선하려는 현재 목표에 비해 설치 복잡도가 앞서감

## Decision

배포 전 기본 동작은 **cache-first CLI**로 확정한다.

구체적으로:

1. `search`, `pick`, `promote`, `tui`는 기본적으로 managed cache를 읽는다.
2. 캐시가 없으면 동기적으로 `collect`를 실행해 캐시를 생성한 뒤 계속 진행한다.
3. 캐시가 오래되었으면 현재 호출은 기존 캐시로 즉시 처리하고, 백그라운드 refresh를 트리거한다.
4. 사용자는 필요할 때 `--refresh-now`로 강제 동기 갱신하거나, `--no-refresh`로 자동 refresh를 막을 수 있다.
5. 기본 캐시 경로는 repo-local 경로가 아니라 macOS 표준 cache 경로를 사용한다.

현재 stale refresh는 launchd 없이 현재 바이너리가 백그라운드 `collect`를 spawn하는 방식으로 구현한다.

## Alternatives Considered

| Alternative | Pros | Cons |
|-------------|------|------|
| ADR-004대로 launchd collector를 바로 기본화 | 검색 path에서 수집 비용 완전 제거, 자동 최신화 명확 | 설치/운영 UX가 아직 없고 배포 범위를 키움 |
| one-shot collect 유지 | 구현이 가장 단순 | 측정상 약 2.7초로 hot path에 너무 느림 |
| 수동 `--cache`만 유지 | 검색 자체는 빠름 | 기본 UX가 사용 습관에 의존하고 stale 상태가 자주 남음 |

## Consequences

### Positive
- 기본 CLI UX가 즉시 응답 중심으로 바뀐다.
- launchd 없이도 첫 배포에서 실용적인 최신화 전략을 제공한다.
- launchd collector를 나중에 붙이더라도 cache-first 표면은 그대로 유지할 수 있다.

### Negative
- stale window 동안 닫힌 세션이나 새 세션 반영이 늦을 수 있다.
- 첫 실행이나 강제 refresh는 여전히 `collect` 지연을 겪는다.
- background refresh 상태를 별도 UI 없이 조용히 처리하므로 관찰 가능성이 낮다.

### Neutral
- `collect` 커맨드는 유지된다.
- launchd 기반 collector는 폐기된 것이 아니라 향후 재도입 가능한 확장안으로 남는다.

## Implementation Notes

- 기본 cache path: `~/Library/Caches/ccx/latest-sessions.json` on macOS
- stale threshold: 15초
- managed cache를 쓰지 않는 명시적 `--sessions` 입력은 기존처럼 그대로 유지
- 향후 launchd를 도입하면 이 ADR을 supersede 하거나, background refresh 구현만 교체하면 된다

## References

- [PRD](../../PRD.md)
- [ADR-003](2026-03-18-use-rust-for-primary-implementation.md)
- [ADR-004](2026-03-18-session-collection-caching-strategy.md)
