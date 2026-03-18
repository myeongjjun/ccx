# ADR-012: Retire the Python Reference and Standardize Rust Sources Under `src/`

- **Date**: 2026-03-18
- **Status**: accepted
- **Deciders**: user, Codex
- **Supersedes**: ADR-001

## Context

`ccx` has now converged on the Rust implementation as the real product path.

The repository still contains:

- a Rust implementation under `rust-src/`
- an older Python reference implementation under `src/ccx/`
- Python packaging metadata that no longer reflects the actual shipping binary

That layout made sense during migration, but it now adds confusion:

- `src/` points at code that is no longer the primary path
- `rust-src/` looks transitional even though it is the real implementation
- release docs and package metadata imply two active implementations when only the Rust CLI matters

## Decision

Retire the Python reference implementation from the repository and standardize the Rust crate on the conventional `src/` layout.

Concretely:

1. Move Rust sources from `rust-src/` to `src/`.
2. Remove the unused Python package and Python test suite.
3. Remove Python packaging files that describe a no-longer-supported Python CLI path.
4. Update docs, workflows, and local skills to reference the Rust-first structure only.

## Alternatives Considered

| Alternative | Pros | Cons |
|-------------|------|------|
| Keep both Rust and Python implementations | Historical reference remains in-tree | Ongoing confusion about what is real, extra maintenance cost, misleading packaging metadata |
| Keep `rust-src/` and remove Python only | Less file movement | Retains a transitional layout name for the primary codebase |
| Archive Python code in a separate branch only | Keeps current repo clean | Requires explicit archive discipline, but that is acceptable once Rust is clearly primary |

## Consequences

### Positive
- Repository structure matches the actual product implementation.
- Public release packaging becomes less confusing.
- New contributors will find the Rust code in the standard location immediately.

### Negative
- Python reference history is no longer available in the main working tree.
- Some docs, skills, and old ADR implementation notes need updating.

### Neutral
- Historical Python work still exists in git history if it is ever needed for reference.

## Implementation Notes

- Rust sources should live under `src/`.
- Public packaging should describe the Rust binary only.
- Old Python-specific release/documentation references should be removed rather than kept as stale historical notes in the main README.

## References

- [ADR-001](./2026-03-18-repo-local-harness-and-python-mvp-scaffold.md)
- [ADR-003](./2026-03-18-use-rust-for-primary-implementation.md)
