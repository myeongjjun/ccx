# ADR-003: Use Rust for the Primary Implementation

- **Date**: 2026-03-18
- **Status**: accepted
- **Deciders**: user, Codex
- **Supersedes**: none

## Context

The project has already converged on a terminal-native TUI as its primary surface. That changes the implementation constraints materially:

- fast startup matters
- terminal control becomes a first-class concern
- single-binary distribution becomes more valuable
- the tool is closer to a long-lived developer utility than a short-lived scripting prototype

The initial scaffold was built in Python to validate search rules, session metadata shape, and focus-transfer boundaries quickly. That validation work is useful, but the user now prefers to move the primary implementation to Rust rather than continue deepening the Python version.

## Decision

Adopt Rust as the primary implementation language for `ccx`.

The Python code remains in the repository temporarily as a reference implementation and behavior oracle while Rust reaches functional parity. New primary-path implementation work should target the Rust codebase first.

The Rust migration should prioritize:

- session models and JSON snapshot handling
- query parsing and ranking
- terminal-native TUI interaction
- iTerm2 focus transfer through `osascript`

## Alternatives Considered

| Alternative | Pros | Cons |
|-------------|------|------|
| Continue in Python until feature complete | Lower immediate migration cost | Duplicates design effort if Rust remains the likely long-term target |
| Separate overlay app before language pivot | Could match launcher language in PRD literally | Expands scope away from the iTerm2-centered workflow |
| Hybrid long-term Python core with small native shell | Less rewrite pressure | Splits ownership and keeps terminal performance concerns in the wrong layer |

## Consequences

### Positive
- The implementation target now matches the product's terminal-native direction better.
- Packaging and startup behavior can converge toward a single binary.
- Search, TUI, and local execution concerns can be handled in one cohesive runtime.

### Negative
- The migration temporarily required maintaining transitional structure and references.
- Immediate compile verification is blocked until a Rust toolchain is available in the environment.

### Neutral
- The Python code was useful during migration as a behavioral reference before retirement.

## Implementation Notes

Migration outcome:

- the Rust crate is rooted at `Cargo.toml` with sources under `src/`
- the temporary Python reference implementation was retired after Rust became the only primary path
- behavior parity work preceded broader expansion during the migration phase

## References

- [PRD](../../PRD.md)
- [ADR-001](./2026-03-18-repo-local-harness-and-python-mvp-scaffold.md)
- [ADR-012](./2026-03-18-retire-python-reference-and-standardize-rust-src-layout.md)
- [ADR-002](./2026-03-18-terminal-native-tui-primary-surface.md)
