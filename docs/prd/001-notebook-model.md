---
id: PRD-001
title: Notebook model and `gyeoldaero check`
status: In Progress
depends_on: [PRD-000]
branch: prd/001-notebook-model
---

# PRD-001: Notebook model and `gyeoldaero check`

## Problem

Nothing understands a notebook yet. Every later feature (rendering, watching, `new`)
needs one reliable model of the folder tree: which folders are notes, their order, their
bodies, and their links.

## Outcome

`gyeoldaero-core` can scan a notebook into a model, and `gyeoldaero check <path>` reports problems
in a notebook in plain language, exiting non-zero on errors.

## Scope

- Scanning, ignore rules, symlink handling.
- Folder names → order prefix + slug; sibling ordering.
- `index.md` parsing: frontmatter (`title`, unknown fields preserved) and body.
- Wiki link extraction and resolution (absolute and relative).
- Diagnostics and `gyeoldaero.toml` parsing.
- `gyeoldaero check` subcommand.

## Non-goals

- Markdown → HTML (PRD-002).
- Watching for changes (PRD-004).
- Backlinks presentation (model stores outgoing links only).

## Requirements

1. Implement the rules in `architecture.md` → *Notebook model* exactly: scanning rules,
   folder names/slugs/order, frontmatter, links, diagnostics.
2. Public API (shape, names may be refined in the plan):
   - `Notebook::load(root: &Path, config: &Config) -> Result<(Notebook, Vec<Diagnostic>), CoreError>`
   - `Notebook::root() -> NoteId`, `get(NoteId) -> &Note`, `children(NoteId) -> &[NoteId]`
     (already ordered), `parent(NoteId) -> Option<NoteId>`, `by_path(&str) -> Option<NoteId>`.
   - `Note { slug, order: Option<u32>, dir_name, path: NotePath, title: Option<String>,
     body: String, frontmatter: serde_yaml_ng::Mapping, links: Vec<Link>,
     assets: Vec<PathBuf> }`
   - `Link { target: String, label: Option<String>, resolved: Option<NoteId> }`
   - `Diagnostic { level: Error|Warning, path, message }` with `Display`.
   - `Config::load(root) -> Result<Config, CoreError>` with defaults when the file is absent.
3. Link extraction must ignore `[[…]]` inside inline code and fenced code blocks.
4. `gyeoldaero check [PATH]` (default: current directory) prints diagnostics grouped by note
   path, then a one-line summary (`3 errors, 1 warning in 21 notes`). Exit codes: 0 clean
   or warnings only, 1 errors, 2 unusable root.
5. Scanning a notebook of 5,000 notes completes in under 1 s on a laptop (release build).
   Add a benchmark-style ignored test or a `cargo bench` if cheap.
6. Frontmatter is parsed with `serde_yaml_ng`.
7. Non-ASCII slugs (e.g. Hangul folder names) are allowed and produce no diagnostic.
8. Slugs and link targets are compared after Unicode NFC normalization and then
   lowercasing (`str::to_lowercase`): sibling duplicate detection, `by_path` lookups and
   link resolution all compare these keys, so a folder name stored in NFD (as macOS often
   writes it) matches a link typed in NFC, and `Caches` / `caches` are the same slug
   (they would collide in lowercased URLs). The slug is still displayed as written.

## Acceptance criteria

- [ ] AC1 — The fixture notebook loads with zero diagnostics, and the loaded tree (slugs,
      order, bodies) equals the mockup's `TREE`. *(unit, against `tests/fixtures/notebook`)*
- [ ] AC2 — Ordering: prefixed by number, then unprefixed by natural sort, ties by slug.
      *(unit, table-driven)*
- [ ] AC3 — Duplicate slugs among siblings produce an error diagnostic naming both folders. *(unit)*
- [ ] AC4 — Every ignore rule in the architecture doc is covered (hidden, `.git`,
      Syncthing temp files, `4913`, swap files, output dir inside notebook). *(unit)*
- [ ] AC5 — Symlinks are not followed and produce a warning. *(unit, unix-only test)*
- [ ] AC6 — Directory without `index.md` → note with empty body + warning; stray `.md`
      files → warning. *(unit)*
- [ ] AC7 — Frontmatter: `title` used; unknown fields preserved; malformed YAML → error
      and the whole file becomes the body. *(unit)*
- [ ] AC8 — Links: absolute, `./`, `../`, with labels; broken link → error; links in code
      are ignored. *(unit)*
- [ ] AC9 — `gyeoldaero check` output and exit codes match Requirement 4 on a fixture with
      known problems (`tests/fixtures/broken/`). *(cli)*
- [ ] AC10 — Performance target met. *(ignored test or bench, run in the PR description)*
- [ ] AC11 — Unicode: a Hangul slug loads with no diagnostic; an NFD-named folder resolves
      from an NFC link and an NFC `by_path` lookup; NFC and NFD spellings of the same slug
      among siblings are reported as duplicates; so are `Caches` and `caches`; a link
      differing only in case resolves. *(unit)*

## Decisions

Open questions answered by Ellie on 2026-10-23:

- Frontmatter parser: **`serde_yaml_ng`**.
- Non-ASCII slugs: **allowed, no warning** (percent-encoded in URLs, per
  `architecture.md`). Added: slugs and link targets are compared after Unicode NFC
  normalization (Requirement 8, AC11).
- Slug case (raised during planning): comparison is **case-insensitive** after NFC, to
  match the lowercased URL form (Requirement 8, AC11).

## Plan

1. **Dependencies and ADR.** Add to `[workspace.dependencies]` and `gyeoldaero-core`:
   `serde` (derive), `serde_yaml_ng` 0.10, `toml`, `thiserror`, `unicode-normalization`,
   `comrak` (parsing only, for link extraction). Dev: `tempfile`; CLI gains `anyhow`
   and dev `assert_cmd`. `toml`, `serde`, `unicode-normalization` and `assert_cmd` aren't
   in the architecture table, so record them in `docs/adr/0002-notebook-model-deps.md`
   (`adr` skill, status Proposed) and link it from `architecture.md`. *(all ACs)*
2. **Spec sync.** `architecture.md` → *Folder names, slugs, order* and *Links*: add the
   NFC + case-insensitive comparison rule (Req. 8). *(AC11)*
3. **Tests first** with the `test-author` agent, committed failing:
   - AC1: `crates/gyeoldaero-core/tests/fixture.rs` loads `tests/fixtures/notebook` and
     compares it with the mockup's `TREE`, extracted at test time from
     `docs/reference/mockup.html` by a small test-only parser, so the mockup stays the
     single source of truth.
   - AC2–AC8, AC11: table-driven unit tests per module (below), building notebooks in
     `tempfile` dirs; AC5 under `#[cfg(unix)]`.
   - AC9: new fixture `tests/fixtures/broken/` (duplicate slugs, broken link, malformed
     frontmatter, missing `index.md`, stray `.md`) and
     `crates/gyeoldaero-cli/tests/check.rs` with `assert_cmd` for output and exit codes
     0/1/2.
   - AC10: `#[ignore]` test that generates 5,000 notes and asserts `load` < 1 s; run with
     `cargo test --release -p gyeoldaero-core -- --ignored`.
4. **`gyeoldaero-core` modules**, each with its tests:
   - `error.rs`: `CoreError` (unreadable root, I/O, config parse).
   - `diagnostic.rs`: `Diagnostic`, `Level`, `Display`. *(AC3, AC6, AC7, AC8)*
   - `config.rs`: `Config::load`, defaults (`name` = folder name, bind `127.0.0.1:4321`,
     `out` under `$XDG_CACHE_HOME` or `~/.cache`, `~` expanded). *(AC4: output dir ignore)*
   - `name.rs`: prefix split, slug, comparison key (NFC + lowercase), natural-sort ordering. *(AC2, AC11)*
   - `ignore.rs`: hidden entries, `.git`, Syncthing and editor artifacts, output dir.
     *(AC4)*
   - `frontmatter.rs`: split `---` block, `title`, preserved `Mapping`, malformed → error
     diagnostic and whole file as body. *(AC7)*
   - `links.rs`: extract `[[path]]` / `[[path|label]]` from the comrak AST (so code spans
     and fenced blocks are skipped exactly as the renderer will see them), resolve
     absolute, `./`, `../` via comparison keys. *(AC8, AC11)*
   - `notebook.rs`: scan (no symlink following), build the model, the API from Req. 2,
     sibling duplicate detection, missing `index.md` / stray `.md` warnings. *(AC1–AC7)*
5. **`gyeoldaero check [PATH]`** in `gyeoldaero-cli`: load config and notebook, print
   diagnostics grouped by note path, summary line, exit 0/1/2. *(AC9)*
6. **Performance** run in release; record the timing for the PR. *(AC10)*
7. **Docs**: `docs/dev-commands.md` (check command is live, perf test command), README
   status if anything user-facing changes.
8. **Review**: `code-reviewer`; `security-reviewer` (scanning, paths, symlinks).
9. **Ship** with `prd-ship`.

### For Ellie to look at

- **New dependencies** in step 1 (ADR 0002).
