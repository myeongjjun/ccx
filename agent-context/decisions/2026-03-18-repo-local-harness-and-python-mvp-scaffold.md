# ADR-001: Use Repo-Local Harness and Python MVP Scaffold

- **Date**: 2026-03-18
- **Status**: superseded
- **Deciders**: user, Codex
- **Supersedes**: none
- **Superseded by**: ADR-012

## Context

The project starts from a PRD that defines an iTerm2 companion launcher focused on four MVP responsibilities:

- collect session metadata from iTerm2
- classify Claude Code and Codex sessions
- rank results with path, repo, and type-aware search
- jump to the selected session quickly

The repository already uses ACP conventions, and the user explicitly clarified that harness setup should be scoped to this repository rather than a global skill installation.

## Decision

Adopt a repo-local project structure with two layers:

- repo-local harness files under `.claude/agents/` and `.claude/skills/` for project-specific agent roles and orchestration
- a Python application scaffold under `src/ccx/` and `tests/`, managed with `uv`, to implement the MVP search and platform integration logic

The initial scaffold will keep pure search logic separate from macOS/iTerm2 integration code so ranking can be tested without platform dependencies.

## Alternatives Considered

| Alternative | Pros | Cons |
|-------------|------|------|
| Global harness in home directory | Reusable across projects | Wrong scope for this repo and weaker alignment with the PRD |
| Start with only docs and no runnable scaffold | Lower initial code volume | Delays validation of data model, CLI, and ranking behavior |
| Build the macOS UI first | Earlier demo of visible UX | Forces platform-coupled decisions before search and collection contracts stabilize |

## Consequences

### Positive
- Future agents can find project-specific roles and workflows directly in this repository.
- Search, normalization, and activation logic can evolve incrementally behind a stable Python package layout.
- Unit tests can cover ranking and normalization rules before the UI exists.

### Negative
- The repo now contains both ACP metadata and project-local harness metadata, which requires discipline about file purpose.
- A future global launcher framework would need explicit extraction if cross-project reuse becomes important.

### Neutral
- The first scaffold stops short of a full UI and focuses on contracts, ranking, and platform boundaries.

## Implementation Notes

Current scaffold includes:

- `pyproject.toml` with `uv` and `pytest`
- `src/ccx/` for models, tokenizer, ranking, normalization, and AppleScript builders
- `tests/` for search and platform-adjacent unit coverage
- `.claude/agents/` and `.claude/skills/` for repo-local harness definitions

## References

- [PRD](../../PRD.md)
- [AGENTS](../../AGENTS.md)
