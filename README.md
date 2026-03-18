# ccx

`ccx` is a macOS CLI for jumping between iTerm2 work contexts built around Claude Code (`cc`) and Codex (`cx`) sessions.

## Install

### Homebrew

```bash
brew tap myeongjjun/ccx
brew install ccx
```

### From Source

```bash
cargo install --path .
```

After installing, verify that your shell resolves the real `ccx` binary:

```bash
type -a ccx
```

If `ccx` is already an alias or shell function in your environment, remove or rename it before using this tool:

```bash
unalias ccx
hash -r
```

The CLI itself cannot reliably warn about shell aliases after launch, because alias expansion happens before the `ccx` process starts.

## Quick Start

Requirements:

- macOS
- iTerm2
- terminal permission to control iTerm2 through Apple Events / Automation

Install and run the first real workflow:

```bash
ccx
ccx ccx
```

The intended onboarding path is:

- `ccx`
  - opens the interactive launcher
- `ccx <query>`
  - runs isolate shorthand for one work context
  - example: `ccx ccx`

After that, the two primary interactive workflows are:

```bash
ccx
ccx isolate-tui
```

`ccx` is the fast launcher. `ccx <query>` is the fast isolate path. `isolate-tui` is the interactive workset switcher.

## Primary Workflows

| Command | Role | Typical use |
|---|---|---|
| `ccx` | Interactive launcher | Search live results and focus one session |
| `ccx <query>` | Isolate shorthand | Jump straight into one work context |
| `ccx isolate-tui` | Interactive workset switcher | Preview a project workset and isolate it |

`ccx` is cache-first by default:

- it reads `~/Library/Caches/ccx/latest-sessions.json`
- if the cache is missing, it runs `collect` and creates it
- if the cache is stale, it returns immediately and refreshes in the background
- use `--refresh-now` to force a fresh collect before running a command

Source repository:

```text
https://github.com/myeongjjun/ccx
```

## Supporting CLI

| Command | What it does | Typical use |
|---|---|---|
| `ccx search "<query>"` | Show ranked matching sessions | Inspect results or script against JSON output |
| `ccx pick "<query>"` | Focus the single best session | Open one session without changing the current workset |
| `ccx promote "<query>"` | Reorder a workset by ranked relevance | Improve repeated `Cmd+\`` switching |
| `ccx isolate "<query>"` | Keep the matching workset visible and minimize the rest | Scriptable workset switch |
| `ccx collect` | Collect and cache the current iTerm2 snapshot | Refresh the session inventory |
| `ccx focus-script --session <file>` | Print the AppleScript used for focusing a session | Debug automation behavior |
| `ccx schema` | Print a sample session schema | Inspect snapshot shape |

## Supported Environment

`ccx` currently targets:

- macOS
- iTerm2
- local AppleScript / `osascript` automation

If collection or focus fails on first use, check:

- System Settings -> Privacy & Security -> Automation
- System Settings -> Privacy & Security -> Accessibility

Your terminal app may need permission to control iTerm2.

## Usage

Fast launcher:

```bash
ccx
```

The default launcher stays focused on fast session selection: type a query, move through results, and press Enter to focus the selected session.

Fast isolate shorthand:

```bash
ccx ccx
ccx my-project
```

`ccx <query>` is shorthand for `ccx isolate "<query>"`. It is the quickest path to shrinking the current iTerm2 window set down to one matching work context.

Interactive workset switcher:

```bash
ccx isolate-tui
```

`isolate-tui` is a separate workset switcher. It previews the visible workset and, on a best-effort basis, the selected session's live visible contents before you press Enter. The contents panel depends on iTerm2 automation support and may be empty when iTerm2 does not expose session contents.

Scriptable search and focus:

```bash
ccx search "session-keyword"
ccx search "p:/path/to/project r:project-name t:task-keyword" --json --explain
ccx pick "session-keyword"
```

`pick` is the lightweight escape hatch from an isolated workspace. If you are already working inside one isolated workset and need to open one terminal from another project without re-shaping the whole window set, `pick` focuses that session and leaves the rest of the current workspace layout alone.

Workset commands:

```bash
ccx promote "session-keyword"
ccx isolate "session-keyword"
ccx isolate "session-keyword" --dry-run
ccx isolate "session-keyword" --limit 3 --gap-ratio 0.6
```

`isolate` ranks matching windows by their best session score, keeps the top 2 windows by default, and only keeps lower-ranked windows when they stay within the configured relative score gap. Windows tied with the cutoff score are also kept even if that means exceeding `--limit`. `--limit` controls the cutoff rank, and `--gap-ratio` tunes how close additional windows must be to the best match.

`isolate` still works across repeated calls because minimized iTerm2 windows remain in the collected inventory. A later isolate can restore a previously minimized workset if it matches the new query.

Refresh and debug:

```bash
ccx search "session-keyword" --refresh-now
ccx --debug isolate "ccx" --refresh-now --dry-run
CCX_DEBUG=1 ccx search "ccx"
```

`--debug` writes diagnostics to stderr, including cache/refresh decisions, collector activity, search result counts, and isolate apply status.

Inspect the collection script without running it:

```bash
ccx collect --dry-run
```

## Why ccx?

When you run many Claude Code or Codex sessions at once, generic window switchers lose track of what each session is doing. `ccx` is purpose-built for that workflow.

| Feature | Cmd+Tab / Cmd+\` | iTerm2 Open Quickly | tmux | Raycast / Alfred | **ccx** |
|---|---|---|---|---|---|
| Semantic search (project, type, path) | — | ~ | — | ~ | ✓ |
| AI-session aware (CC / CX scoring) | — | — | — | — | ✓ |
| Field-specific queries (`p:`, `r:`, `t:`) | — | — | — | — | ✓ |
| Workset promotion (reorder by relevance) | — | — | — | — | ✓ |
| Isolate workset (minimize the rest) | — | — | — | — | ✓ |
| Cache-first, instant results | — | — | ✓ | ~ | ✓ |
| Interactive TUI with live preview | — | ~ | ~ | ~ | ✓ |
| CLI-composable (pipe, script) | — | — | ✓ | — | ✓ |
| Cross-app window switching | ✓ | — | — | ✓ | — |
| Works outside iTerm2 | ✓ | — | ✓ | ✓ | — |

`ccx` does not replace `Cmd+Tab` or Raycast. It stays inside iTerm2 and optimizes for semantic work-context switching instead of flat window-title switching.

## Notes

- `ccx` is iTerm2-specific. It does not try to manage terminals outside iTerm2.
- Session metadata quality depends on what iTerm2 exposes through AppleScript. When fields such as `foreground_command` are missing, `ccx` falls back to title and other collected metadata.
- `isolate` uses the shared search ranking and then applies isolate-specific workset selection. Search quality improvements therefore affect multiple commands, not just `isolate`.

## Open Source Components

Core open-source building blocks:

- `clap` for the Rust CLI surface
- `crossterm` for the terminal UI
- `ratatui` for the isolate preview layout and widgets
- `serde` and `serde_json` for session snapshot serialization
- `anyhow` for error handling
- `unicode-width` for terminal-safe line layout

macOS integration uses the system `osascript` executable and iTerm2's AppleScript support.

## Development

```bash
cargo clippy --all-targets -- -D warnings
cargo test
```

## Release Automation

The repository includes a tag-driven GitHub Actions workflow at [`./.github/workflows/release.yml`](./.github/workflows/release.yml).

Default flow:

1. push a tag such as `v0.1.0`
2. validate that `Cargo.toml` matches that version
3. create a GitHub Release if one does not already exist
4. compute the source tarball checksum for that tag
5. update `Formula/ccx.rb` in a separate Homebrew tap repo if configured

Required GitHub configuration for tap updates:

- repository variable `HOMEBREW_TAP_REPOSITORY`
  - value: `myeongjjun/homebrew-ccx`
- repository secret `HOMEBREW_TAP_TOKEN`
  - token with write access to `myeongjjun/homebrew-ccx`

The tap formula renderer lives at [`./scripts/render-homebrew-formula.sh`](./scripts/render-homebrew-formula.sh).

Practical pre-release checklist:

1. Make sure `Cargo.toml` has the release version.
2. Run `cargo clippy --all-targets -- -D warnings`.
3. Run `cargo test`.
4. Tag the release, for example `v0.1.0`.
5. Confirm the tap repository and token are configured if you want automatic formula updates.
