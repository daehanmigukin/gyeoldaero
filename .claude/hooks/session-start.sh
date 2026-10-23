#!/usr/bin/env bash
# SessionStart hook. Stdout is added to Claude's context.
set -uo pipefail
cat >/dev/null || true

root="${CLAUDE_PROJECT_DIR:-$(pwd)}"
cd "$root" || exit 0

echo "## gyeoldaero session context"
echo "Branch: $(git branch --show-current 2>/dev/null || echo 'unknown')"

dirty=$(git status --short 2>/dev/null | head -n 20)
if [ -n "$dirty" ]; then
  echo "Uncommitted changes:"
  echo "$dirty" | sed 's/^/  /'
fi

if [ -f docs/prd/README.md ]; then
  echo
  echo "PRD status (docs/prd/README.md):"
  grep -E '^\| \[[0-9]+\]' docs/prd/README.md \
    | sed -E 's/\[([0-9]+)\]\([^)]*\)/PRD-\1/' \
    | sed 's/^/  /'
fi

if [ -f .claude/bootstrap-in-progress ]; then
  echo
  echo "Bootstrap (PRD-000) in progress: commits on main are allowed until .claude/bootstrap-in-progress is deleted."
fi

echo
echo "Rules: read CLAUDE.md. Only Accepted PRDs get implemented; never commit to main."
exit 0
