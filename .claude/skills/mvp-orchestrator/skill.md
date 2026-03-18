---
name: mvp-orchestrator
description: "Top-level repo-local harness for delivering the ccx MVP across collection, ranking, and launcher UX."
---

# MVP Orchestrator

Use this skill to coordinate the `ccx` project work.

## Scenario Routing

- session collection or activation issue: use `session-discovery` + `session-inventory`
- relevance or search quality issue: use `search-ranking` + `query-ranking`
- overlay behavior or keyboard flow issue: use `launcher-experience` + `launcher-flow`

## Delivery Phases

1. define or update the session contract
2. implement or refine search indexing and ranking
3. validate launcher interaction expectations
4. add tests for PRD scenarios before broadening scope

## Data Flow

- `session-discovery` produces raw session metadata
- `search-ranking` transforms metadata into ranked results
- `launcher-experience` consumes ranked results for the UI contract

## Guardrails

- do not turn the project into a terminal replacement
- keep MVP centered on search quality and instant focus transfer
- avoid preview-heavy or orchestration-heavy features until the core jump flow is solid
