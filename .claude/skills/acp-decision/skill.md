---
name: acp-decision
description: Record an architectural decision in `agent-context/decisions/` and update the decisions index. Use when a significant technical decision has been made and future agents need the rationale.
---

# acp-decision

Use this skill whenever a real architectural decision has been made.

## Rules

- Do not edit `agent-context/decisions/` casually.
- Create or update ADRs deliberately and keep `INDEX.md` in sync.
- Prefer concise ADRs that explain context, decision, alternatives, and consequences.

## When to use

- choosing between multiple valid technical approaches
- introducing a new project-wide convention
- changing a previously accepted architectural decision
- recording a decision that future agents will need for continuity

## Workflow

1. inspect the latest ADR number in `agent-context/decisions/INDEX.md`
2. create a new file named `YYYY-MM-DD-<slug>.md`
3. use the standard ADR structure:
   - title
   - date
   - status
   - deciders
   - context
   - decision
   - alternatives considered
   - consequences
   - implementation notes
   - references
4. if this replaces an old ADR, mark the older ADR as superseded
5. update `agent-context/decisions/INDEX.md`

## Output

Report:

- the new ADR path
- ADR number/title
- whether any earlier ADR was superseded
