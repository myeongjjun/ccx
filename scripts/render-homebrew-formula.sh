#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 5 ]]; then
  echo "usage: $0 <owner> <repo> <tag> <sha256> <output>" >&2
  exit 1
fi

owner="$1"
repo="$2"
tag="$3"
sha256="$4"
output="$5"
version="${tag#v}"

mkdir -p "$(dirname "$output")"

cat >"$output" <<EOF
class Ccx < Formula
  desc "iTerm2 work-context launcher for Claude Code and Codex sessions"
  homepage "https://github.com/${owner}/${repo}"
  url "https://github.com/${owner}/${repo}/archive/refs/tags/${tag}.tar.gz"
  sha256 "${sha256}"
  version "${version}"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  def caveats
    <<~EOS
      First run:
        ccx
        ccx ccx

      Main workflows:
        ccx
        ccx <query>
        ccx isolate-tui
    EOS
  end

  test do
    assert_match "ccx", shell_output("#{bin}/ccx --help")
  end
end
EOF
