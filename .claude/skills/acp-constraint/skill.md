---
name: acp-constraint
description: Add or review project constraints in `agent-context/constraints/`. Use for non-negotiable rules that future agents must not violate.
---

# acp-constraint

Use this skill for hard rules, not general preferences.

## Rules

- Do not edit `agent-context/constraints/` casually.
- Constraints should be stable, durable, and important enough to survive sessions.
- Keep the index current.

## When to use

- security or privacy requirements
- architecture boundaries that must not be crossed
- release-blocking policies
- hard workflow rules that agents must preserve

## Workflow

1. inspect `agent-context/constraints/INDEX.md`
2. create a new constraint file with a clear slug
3. include:
   - constraint title
   - severity
   - scope
   - rationale
   - examples or implications if needed
4. update `agent-context/constraints/INDEX.md`
5. if the rule is critical, make sure `AGENTS.md` remains aligned with it

## Output

Report:

- the constraint file path
- severity
- any required follow-up sync
