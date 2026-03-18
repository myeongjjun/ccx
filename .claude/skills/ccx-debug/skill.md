---
name: ccx-debug
description: Run the local ccx isolate debug cycle, inspect `--debug` output, classify whether a failure comes from collection, ranking, isolate selection, or iTerm apply behavior, then patch and rerun until the cycle stabilizes. Use when iterating on `ccx` search/isolate behavior in this repository.
---

# ccx-debug

Use this skill when working on `ccx` behavior in this repository, especially for:

- `ccx isolate ...` returning the wrong keep set
- `ccx --debug isolate ...` showing mismatches or partial apply
- search/ranking issues such as repo/path confusion
- collection quality issues where `cwd`, `repo_name`, or `foreground_command` are missing

## Core cycle

Run the same local loop each time:

```bash
bash scripts/isolate-cycle.sh "ccx"
```

The helper does:

1. unminimize all iTerm windows
2. `cargo install --path . --force`
3. `ccx --debug isolate "<query>"`
4. save the combined output to a temp log file

Use a different query by replacing `"ccx"`.

## Required validation before each live cycle

Run:

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```

Only run the live isolate cycle after both pass.

## How to read the debug output

Focus on these lines:

- `isolate query=... keep_windows=... focus_window_id=...`
- `applying isolate keep_window_ids=[...]`
- `isolate apply status=...`
- `live before visible=...`
- `live after visible=...`
- `live mismatch count=...`
- `same-process mismatch ids=[...]`
- `window error window_id=... phase=... message=...`

Interpretation:

- Keep set wrong:
  - likely `src/search.rs` or `src/isolate.rs`
- Correct keep set but `partial:` or mismatches:
  - likely `src/iterm.rs`
- Missing `cwd` / wrong repo clustering:
  - likely `src/iterm.rs` collection or `src/classifier.rs`
- Focus target wrong:
  - inspect ranking order first, then isolate plan selection

## File map

- `scripts/isolate-cycle.sh`
  - repeatable local debug loop
- `src/iterm.rs`
  - iTerm AppleScript collection, isolate apply, same-process reporting
- `src/classifier.rs`
  - normalize collected metadata into `SessionRecord`
- `src/search.rs`
  - shared ranking logic for search/pick/promote/isolate
- `src/isolate.rs`
  - isolate-specific selection policy on top of shared ranking
- `src/main.rs`
  - debug logging and command flow

## Working rules

- Keep shared ranking in `src/search.rs` common across features.
- If `isolate` needs different behavior, change selection policy in `src/isolate.rs`, not the ranking API per feature.
- Prefer minimal fixes with an immediate rerun of the cycle.
- Do not document this workflow in public-facing README content unless explicitly asked.

## Typical patch directions

### Collection problem

- fill missing `cwd` / `foreground_command`
- improve AppleScript collection
- add `tty`-based fallback if AppleScript metadata is incomplete

### Ranking problem

- tighten exact repo/path matching
- reduce noisy query-independent hint boosts
- add focused search tests for the bad comparison

### Isolate selection problem

- keep shared ranking intact
- adjust only how ranked results expand into kept windows/workset clusters

### Apply/runtime problem

- stabilize AppleScript loops before mutating window state
- use same-process after-state reports
- log per-window errors and mismatch ids

## Done criteria

The cycle is in a good state when:

- `isolate apply status=ok`
- `live mismatch count=0`
- keep windows match the intended workset for the query

If apply still returns `partial`, narrow it to specific `window_id`s and keep iterating from `src/iterm.rs`.
