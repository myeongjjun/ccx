---
name: session-inventory
description: "Builds the session collection model for iTerm2 metadata, including IDs, cwd, type inference, and activation targets."
---

# Session Inventory

## Workflow

1. identify the minimum metadata needed for MVP search and jump
2. define a stable `SessionRecord` contract
3. isolate macOS and iTerm2 specific collection logic from portable core logic
4. document missing metadata and fallback heuristics

## Tool Usage

- inspect PRD scenarios before changing collection fields
- keep integration code separate from pure parsing and typing code

## Output Rules

- collection design must explain how `session_id`, `window_id`, `tab_id`, `cwd`, `foreground_command`, and `session_type` are obtained
- note reliability risks for AppleScript or iTerm2 APIs
