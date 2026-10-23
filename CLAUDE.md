# CLAUDE.md — gyeoldaero (결대로)

gyeoldaero is a self-hosted, terminal-first notebook where **every note is a folder** with an
`index.md`. A Rust binary scans the notebook, renders one static HTML page per note, watches
for changes, and serves the site (typically over Tailscale to a phone). Writing happens in
the user's own editor; gyeoldaero never edits notes in a UI.

Owner: Ellie. Open source, MIT.

## Read before working

| File | What it is | Who changes it |
|---|---|---|
| `docs/bible.md` | What gyeoldaero is and why. Highest authority. | Ellie only |
| `docs/architecture.md` | Crates, model rules, build/watch/serve, security. | Via PRD or ADR |
| `docs/renderer-spec.md` | Exact page structure and interactions. | Via PRD |
| `docs/reference/mockup.html` | Approved interactive mockup. Behavioral reference. | Never |
| `docs/workflow.md` | PRD lifecycle, branches, definition of done. | Via PR |
| `docs/prd/` | Units of work. `README.md` is the index. | See workflow |
| `docs/adr/` | Technical decisions. | `adr` skill |
| `docs/dev-commands.md` | How to build, test, run. | Keep current |

## Hard rules

1. **Specs before code.** Only implement PRDs with status `Accepted`. Never set a PRD to
   `Accepted` yourself. If code needs to differ from a spec, stop and propose the spec
   change to Ellie.
2. **Never commit to `main`.** One PRD = one branch `prd/NNN-slug` = one PR. Ellie merges.
3. **Tests first.** Every acceptance criterion gets a test that fails before the change.
4. **Untrusted content.** Note content and file names are untrusted input: escape
   everything, no raw HTML, no symlink following, no writes outside the output dir.
5. **No external requests** from rendered pages. Fonts and assets are bundled.
6. **Crate boundaries** (one-way): `cli → serve → render → core`. Core knows nothing of
   HTML; render knows nothing of HTTP or watching; cli only wires things together.
7. **No `unwrap()`/`expect()`** on paths that handle user input in library crates. Use
   `thiserror` errors in libraries, `anyhow` in the binary.
8. Snapshot changes (`insta`) are reviewed one by one, never bulk-accepted.

## Workflow (summary of `docs/workflow.md`)

- New idea → `prd-write` skill → `spec-reviewer` agent → Ellie accepts.
- Start → `prd-start` skill (branch, status, plan, tasks).
- Tests → `test-author` agent. Implement in small conventional commits (`Refs: PRD-NNN`).
- Review → `code-reviewer` agent; `security-reviewer` for parsing, rendering, paths, server.
- Ship → `prd-ship` skill (checks, checklist, docs, PR).
- Decisions the specs don't cover → `adr` skill.

## Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo insta review                      # snapshot changes
cargo run -p gyeoldaero-cli -- <subcommand>  # build | serve | check | new
```

More in `docs/dev-commands.md`.

## Hooks in this repo

- **SessionStart** prints the branch, dirty files and PRD statuses.
- **PreToolUse (Bash)** blocks force pushes, `--no-verify`, commits or pushes on `main`,
  `git reset --hard`, `git clean -f`, and deleting `/` or home.
- **PostToolUse (Edit/Write)** runs `rustfmt` on edited `.rs` files.
- **Stop** runs fmt check, clippy and tests when Rust files changed, and sends you back to
  fix failures before finishing.

If a hook blocks something you believe is right, explain why to Ellie instead of working
around it.

## Style

- Rust 2024, stable. Small modules, explicit types at crate boundaries, doc comments on
  public items.
- Prefer clear and slightly redundant over clever. Correctness before caching.
- User-facing messages are plain sentences that say what went wrong and how to fix it.
- Keep chat updates short; the PRD, task list and PR are the record.
