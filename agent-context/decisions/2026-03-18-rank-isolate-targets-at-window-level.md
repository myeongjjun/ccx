# ADR-008: Rank Isolate Targets at the Window Level

- **Date**: 2026-03-18
- **Status**: superseded
- **Deciders**: user, Codex
- **Supersedes**: none
- **Superseded by**: ADR-009

## Context

`isolate`는 이미 window minimization 기반 workset 전환으로 자리잡았지만, 어떤 창이 visible 상태로 남는지는 아직 session-level 검색 결과 truncation의 부산물이었다.

- 검색은 session score 기준으로 정렬되고, 기존 `isolate`는 상위 N session만 본 뒤 unique window를 남겼다.
- 그 결과 하나의 window에 상위 session이 몰리면 visible window 수가 우연히 줄고, 반대로 약한 match도 상위 N 안에만 들면 남을 수 있었다.
- 사용자는 visible workset이 "점수 상위 window"라는 직관으로 결정되길 원했고, 필요하다면 근소한 2등 window만 함께 남기는 쪽을 선호했다.

## Decision

`isolate <query>`는 **window-level ranking**으로 visible workset을 선택한다.

구체적으로:

1. `isolate`는 matching session 전체를 score 순으로 본다.
2. 각 window는 가장 높은 score를 받은 session 대표값으로 ranking된다.
3. 기본값으로 상위 2개 window까지만 visible 상태 후보로 남긴다.
4. 2등 이하 window는 best window 대비 상대 score gap을 통과할 때만 visible 상태로 유지한다.
5. best-ranked window는 마지막에 focus하고, 나머지 non-selected window는 minimize한다.

기본 relative gap ratio는 `0.7`이며, `--gap-ratio`로 조정할 수 있다.

후속 ADR-009에서 cutoff score 동점 window는 limit 밖이어도 유지하도록 refinement되었다.

## Alternatives Considered

| Alternative | Pros | Cons |
|-------------|------|------|
| 기존 session-level top-N 유지 | 구현이 이미 존재함 | visible window 수와 relevance가 session 분포에 좌우됨 |
| top-1 window만 항상 유지 | 동작이 가장 단순하고 예측 가능함 | 비슷한 score의 보조 workset을 함께 남길 수 없음 |
| 절대 score threshold 사용 | "충분히 좋은 match만 유지"라는 의미가 직관적일 수 있음 | query마다 score scale이 달라 cutoff tuning이 불안정함 |

## Consequences

### Positive
- `isolate` 결과가 session 중복의 부산물이 아니라 window relevance를 직접 반영한다.
- 기본 visible window 수가 더 예측 가능해진다.
- 2등 후보가 충분히 가깝지 않으면 자동으로 1-window isolate로 수렴한다.

### Negative
- 상대 score gap이라는 heuristic이 추가되어 tuning 포인트가 생긴다.
- `--limit` 의미가 session count가 아니라 visible window count로 해석된다.

### Neutral
- minimized window를 inventory에 남기고 반복 isolate에서 복구하는 기본 모델은 유지된다.
- `promote`의 ranking 방식은 이번 결정 범위에 포함되지 않는다.

## Implementation Notes

- `isolate`는 search results 전체를 받아 window별 최고 score representative를 고른다.
- default `--limit`은 `2`, default `--gap-ratio`는 `0.7`이다.
- dry-run은 selected window에 대한 focus / keep-visible / restore와 나머지 minimize 상태를 계속 노출한다.

## References

- [ADR-007](2026-03-18-support-workset-isolation-via-window-minimization.md)
- [README](../../README.md)
