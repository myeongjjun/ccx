#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
usage: scripts/isolate-cycle.sh [query]

Runs the local isolate-debug cycle:
  1. Unminimize all iTerm2 windows
  2. cargo install --path . --force
  3. ccx --debug isolate "<query>"

Environment:
  CCX_BIN       Override the binary to run (default: ccx)
  CCX_LOG_FILE  Override the output log file path
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if [[ $# -gt 1 ]]; then
  usage >&2
  exit 1
fi

query="${1:-ccx}"
ccx_bin="${CCX_BIN:-ccx}"
log_file="${CCX_LOG_FILE:-}"

if [[ -z "$log_file" ]]; then
  log_file="$(mktemp -t ccx-isolate-cycle.XXXXXX.log)"
fi

echo "[cycle] unminimizing all iTerm windows"
osascript <<'EOF'
tell application "iTerm"
  repeat with windowRef in windows
    try
      if miniaturized of windowRef then
        set miniaturized of windowRef to false
      end if
    end try
  end repeat
end tell
EOF

echo "[cycle] installing latest ccx from $(pwd)"
cargo install --path . --force

echo "[cycle] running: ${ccx_bin} --debug isolate ${query}"
"$ccx_bin" --debug isolate "$query" 2>&1 | tee "$log_file"

echo "[cycle] debug log saved to $log_file"
