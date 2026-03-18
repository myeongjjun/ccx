# ADR-009: Keep Cutoff Ties in Isolate Window Selection

- **Date**: 2026-03-18
- **Status**: accepted
- **Deciders**: user, Codex
- **Supersedes**: ADR-008

## Context

ADR-008 introduced window-level ranking for `isolate`, but its initial implementation still enforced the cutoff as a hard top-K boundary.

- In practice, users can have several equally relevant windows for the same query.
- A hard cutoff made one of several tied windows disappear purely because of secondary tie-break ordering.
- The desired UX is that `--limit` defines the cutoff rank, not a strict cap when multiple windows have the same score at that boundary.

## Decision

`isolate` keeps all windows whose score is tied with the cutoff-ranked window, provided they also satisfy the configured relative gap threshold.

Concretely:

1. `--limit` determines the cutoff-ranked window score.
2. Any lower-ranked window with the same score as that cutoff remains visible.
3. The relative `--gap-ratio` filter still applies, so weak ties below the best-score gap are not retained.

## Alternatives Considered

| Alternative | Pros | Cons |
|-------------|------|------|
| Strict top-K cap | Result count is fully bounded | Tied windows can disappear arbitrarily based on tie-breaks |
| Increase default limit | More tied windows survive | Also keeps more non-tied windows and weakens isolation |
| Remove limit entirely | No arbitrary cutoff behavior | Visible workset can grow too large for the switching use case |

## Consequences

### Positive
- Equally relevant windows are treated consistently.
- `isolate` behaves closer to user expectation when several workset windows are tied.

### Negative
- Visible window count can exceed `--limit` when ties occur.
- Dry-run output may now show more windows than the numeric limit suggests.

### Neutral
- Relative gap filtering remains the main control for pruning weaker matches.
- Secondary ordering still matters for focus choice among tied windows.

## Implementation Notes

- The cutoff score is taken from the window at rank `--limit`.
- Windows beyond that rank are retained only when their score equals the cutoff score and they pass `--gap-ratio`.

## References

- [ADR-008](2026-03-18-rank-isolate-targets-at-window-level.md)
- [README](../../README.md)
