#!/usr/bin/env bash
# Stop hook. When Rust files have uncommitted changes, run fmt check, clippy and tests.
# On failure, exit 2 so Claude keeps working and sees the output.
# If Claude is already continuing because of this hook, don't loop: warn Ellie instead.
set -uo pipefail

input=$(cat)
active=$(printf '%s' "$input" | jq -r '.stop_hook_active // false')

root="${CLAUDE_PROJECT_DIR:-$(pwd)}"
cd "$root" || exit 0
[ -f Cargo.toml ] || exit 0

changed=$(
  {
    git diff --name-only HEAD 2>/dev/null || git diff --name-only --cached 2>/dev/null
    git ls-files --others --exclude-standard 2>/dev/null
  } | grep -E '\.rs$|(^|/)Cargo\.(toml|lock)$' || true
)
[ -z "$changed" ] && exit 0

fail() {
  local step="$1" output="$2"
  if [ "$active" = "true" ]; then
    msg="Checks still failing after a retry: $step. Look at the output before continuing."
    jq -n --arg m "$msg" '{systemMessage: $m}'
    exit 0
  fi
  echo "Checks failed before finishing: $step" >&2
  echo "$output" | tail -n 80 >&2
  echo "Fix this, or explain to Ellie why it can't be fixed in this change." >&2
  exit 2
}

out=$(cargo fmt --all --check 2>&1) || fail "cargo fmt --check" "$out"
out=$(cargo clippy --workspace --all-targets --quiet -- -D warnings 2>&1) || fail "cargo clippy" "$out"
out=$(cargo test --workspace --quiet 2>&1) || fail "cargo test" "$out"
exit 0
