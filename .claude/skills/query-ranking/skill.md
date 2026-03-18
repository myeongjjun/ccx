---
name: query-ranking
description: "Implements tokenization and ranking for cc/cx session search with explicit PRD-driven boosts."
---

# Query Ranking

## Workflow

1. tokenize path, repo, title, badge, and command fields
2. score exact type matches before path and repo matches
3. add fuzzy matching only after deterministic boosts are applied
4. verify scoring against PRD examples such as `cc auth`, `cx api`, and `path hi`

## Tool Usage

- prefer standard library implementations unless quality is clearly insufficient
- keep ranking logic unit-testable without macOS dependencies

## Output Rules

- every scoring rule must be explainable in a test or example
- include regression coverage for path segments and recency tie-breaks
