# PRD index

Statuses are defined in `docs/workflow.md` §2. Keep this table in sync with each PRD's
front matter. Only Ellie moves a PRD to `Accepted` or `Deferred`.

| PRD | Title | Status | Depends on |
|---|---|---|---|
| [000](000-repo-bootstrap.md) | Repository bootstrap | Shipped | — |
| [001](001-notebook-model.md) | Notebook model and `gyeoldaero check` | In Progress | 000 |
| [002](002-static-renderer.md) | Static renderer and `gyeoldaero build` | Draft | 001 |
| [003](003-interaction-layer.md) | Interaction layer (`gyeoldaero.js`) and e2e tests | Draft | 002 |
| [004](004-build-and-watch.md) | Watching and incremental rebuilds | Draft | 002 |
| [005](005-serve.md) | `gyeoldaero serve` | Draft | 004 |
| [006](006-note-commands.md) | Note commands (`gyeoldaero new`) | Draft | 001 |
| [007](007-theming.md) | Theming | Deferred | 003 |
| [008](008-yazi-plugin.md) | yazi plugin | Deferred | 006 |

Suggested order: 000 → 001 → 002 → 003 → 004 → 005, with 006 any time after 001.
v1 is done when 000–006 are `Shipped`.
