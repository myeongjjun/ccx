# ADR-011: Keep Shared Ranking and Make Isolate Workset-Aware

- **Date**: 2026-03-18
- **Status**: accepted
- **Deciders**: user, Codex
- **Supersedes**: none

## Context

`ccx` now has several commands that depend on the same session search quality:

- `search`
- `pick`
- `promote`
- `isolate`
- `tui`

While debugging `isolate`, two pressures appeared at the same time:

1. `isolate` sometimes needed different behavior from plain ranked session search because workset switching should keep related windows together.
2. Splitting the ranking logic per feature would quickly make behavior inconsistent and hard to maintain.

The desired rule is:

- ranking stays shared across commands
- `isolate` may still apply additional workset-specific selection semantics after ranking

## Decision

`ccx` keeps a single shared ranking model in `search`, and `isolate` implements its special behavior only in the selection layer.

Concretely:

1. `search` remains the common source of scoring and ordering for all commands.
2. `isolate` does not introduce a separate feature-specific ranker.
3. `isolate` may expand its kept window set when ranked seed windows share the same matched repo/path workset identity.
4. Collection and ranking improvements should improve all commands; `isolate`-only behavior belongs in `src/isolate.rs`.

## Alternatives Considered

| Alternative | Pros | Cons |
|-------------|------|------|
| Separate ranking rules for `isolate` | Can optimize isolate behavior directly | Search behavior diverges by command and becomes hard to reason about |
| Keep strict ranked windows only | Simpler implementation | Same workset windows can be dropped even when users expect them to stay together |
| Encode workset expansion inside ranking | One pipeline only | Ranking becomes polluted with command-specific semantics |

## Consequences

### Positive
- Search behavior stays consistent across commands.
- `isolate` can remain repo/path aware without forking the ranking model.
- Future debugging has a clear boundary: ranking vs selection.

### Negative
- `isolate` selection logic becomes more sophisticated than plain top-K windows.
- Some isolate bugs require checking both `search` output and `isolate` expansion behavior.

### Neutral
- Existing tie and gap behavior still applies inside the isolate seed window stage.
- Collection quality remains a separate concern from isolate selection semantics.

## Implementation Notes

- Shared ranking lives in `src/search.rs`.
- Workset-aware expansion for `isolate` lives in `src/isolate.rs`.
- Query-matched repo/path identity is the basis for extra kept windows, not a second scoring pipeline.

## References

- [ADR-007](2026-03-18-support-workset-isolation-via-window-minimization.md)
- [ADR-009](2026-03-18-keep-cutoff-ties-in-isolate-window-selection.md)
- [README](../../README.md)
