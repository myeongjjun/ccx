# ADR-002: Use a Terminal-Native TUI as the Primary User Surface

- **Date**: 2026-03-18
- **Status**: accepted
- **Deciders**: user, Codex
- **Supersedes**: none

## Context

The PRD describes an Open Quickly-like launcher experience and initially leaves room for an overlay-style surface. During implementation planning, two interpretations emerged:

- a separate macOS launcher or overlay app
- a terminal-native interface used from within iTerm2 itself

The user's actual workflow is already centered on iTerm2, and the main problem is not general desktop launchability. The problem is finding and switching between existing iTerm2 sessions while staying focused on terminal work.

Pushing the product toward a separate app would broaden the scope into areas that are not central to the problem:

- cross-environment app lifecycle and focus management
- broader runtime orchestration for Claude Code and Codex environments
- extra surface area beyond iTerm2 session discovery and jump behavior

## Decision

Adopt a terminal-native TUI as the primary user surface for the MVP and near-term product direction.

The architecture is now:

- core search and integration logic remains a Python library and CLI
- the main interactive surface becomes a CLI-launched TUI running inside iTerm2
- iTerm2 control remains externalized to AppleScript or scripting adapters only for metadata collection and focus transfer

This keeps the product scoped as an iTerm2 session finder and switcher rather than a separate launcher application.

## Alternatives Considered

| Alternative | Pros | Cons |
|-------------|------|------|
| Separate macOS launcher app | Can emulate desktop overlay UX closely | Expands scope and weakens alignment with the actual in-terminal workflow |
| Stay CLI-only with no TUI | Lowest implementation cost | Too weak as a daily driver for rapid search and selection |
| iTerm2 plugin or deep extension first | Potentially tighter integration | Higher platform coupling and more implementation risk early |

## Consequences

### Positive
- The product stays aligned with the user's actual work context inside iTerm2.
- The implementation stays focused on session indexing, ranking, and jump behavior.
- The core engine remains reusable while the interactive layer stays lightweight.

### Negative
- The UX will not behave like a global desktop launcher from every app context.
- Some PRD language about overlay behavior is now interpreted through a terminal-native interaction model.

### Neutral
- A separate overlay app can still be explored later if terminal-native usage proves insufficient.

## Implementation Notes

Near-term implementation should prioritize:

- a `tui` command for interactive query and selection
- arrow-key navigation and Enter-to-focus behavior
- reuse of the existing search, cache, and provider layers

## References

- [PRD](../../PRD.md)
- [ADR-001](./2026-03-18-repo-local-harness-and-python-mvp-scaffold.md)
