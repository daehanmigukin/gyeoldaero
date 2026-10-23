#!/usr/bin/env bash
# PostToolUse hook for Edit/MultiEdit/Write. Formats edited Rust files with rustfmt.
set -uo pipefail

file=$(jq -r '.tool_input.file_path // empty')
case "$file" in
  *.rs) ;;
  *) exit 0 ;;
esac
[ -f "$file" ] || exit 0
command -v rustfmt >/dev/null 2>&1 || exit 0

# rustfmt picks up rustfmt.toml from the file's directory upward.
if ! out=$(rustfmt --edition 2024 "$file" 2>&1); then
  echo "rustfmt couldn't format $file (probably a syntax error):" >&2
  echo "$out" | tail -n 30 >&2
  exit 2
fi
exit 0
