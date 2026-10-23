#!/usr/bin/env bash
# PreToolUse hook for Bash. Blocks risky commands; exit 2 sends the message back to Claude.
set -uo pipefail

input=$(cat)
cmd=$(printf '%s' "$input" | jq -r '.tool_input.command // empty')
[ -z "$cmd" ] && exit 0

root="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
bootstrapping=false
[ -f "$root/.claude/bootstrap-in-progress" ] && bootstrapping=true

block() {
  echo "Blocked by .claude/hooks/guard-bash.sh: $1" >&2
  exit 2
}

# Force pushes, in any form.
if grep -Eq 'git[[:space:]]+push([^;&|]*)[[:space:]](--force|--force-with-lease|-f)([[:space:]=]|$)' <<<"$cmd"; then
  block "force pushes aren't allowed. Ask Ellie if history really needs rewriting."
fi

# Skipping hooks or checks.
if grep -Eq -- '(^|[[:space:]])--no-verify([[:space:]]|$)' <<<"$cmd"; then
  block "--no-verify skips checks. Fix whatever is failing instead."
fi

# Commits and pushes on or to main.
if grep -Eq '(^|[;&|[:space:]])git[[:space:]]+(commit|push)([[:space:]]|$)' <<<"$cmd" && [ "$bootstrapping" = false ]; then
  branch=$(git -C "$root" branch --show-current 2>/dev/null || true)
  if [ "$branch" = "main" ]; then
    block "you're on main. Create a branch first: prd/NNN-slug, fix/slug or docs/slug."
  fi
  if grep -Eq 'git[[:space:]]+push[^;&|]*[[:space:]](HEAD:)?(refs/heads/)?main([[:space:]]|$)' <<<"$cmd"; then
    block "pushing to main isn't allowed. Push your branch and open a PR."
  fi
fi

# Discarding work.
if grep -Eq 'git[[:space:]]+reset[[:space:]]+(.*[[:space:]])?--hard' <<<"$cmd"; then
  block "git reset --hard discards work. Ask Ellie first."
fi
if grep -Eq 'git[[:space:]]+clean[[:space:]]+(.*[[:space:]])?-[a-zA-Z]*f' <<<"$cmd"; then
  block "git clean -f deletes untracked files. Ask Ellie first."
fi

# Deleting root or home.
if grep -Eq 'rm[[:space:]]+(-[a-zA-Z]+[[:space:]]+)*(/|~|~/|\$HOME|\$HOME/|"\$HOME"|/\*)([[:space:]]|$)' <<<"$cmd"; then
  block "that would delete / or your home directory."
fi

exit 0
