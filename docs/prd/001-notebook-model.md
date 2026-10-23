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
     A relative `build.out` in `gyeoldaero.toml` resolves against the notebook root.
3. Link extraction must ignore `[[…]]` inside inline code and fenced code blocks.
4. `gyeoldaero check [PATH]` (default: current directory) prints diagnostics grouped by note
   path, then a one-line summary (`3 errors, 1 warning in 21 notes`). Every count is plural
   except exactly 1 (`0 warnings`, `1 warning`, `2 errors`). Exit codes: 0 clean or
   warnings only, 1 errors, 2 unusable root.
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

- [ ] AC1 — With zero diagnostics, the fixture loads with the same notes, nesting and
      bodies as the mockup's `TREE`; sibling order follows AC2.
      *(unit, against `tests/fixtures/notebook`)*
- [ ] AC2 — Ordering: prefixed by number, then unprefixed by natural sort, ties by slug.
      *(unit, table-driven)*
- [ ] AC3 — Duplicate slugs among siblings produce an error diagnostic naming both folders. *(unit)*
- [ ] AC4 — Every ignore rule in the architecture doc is covered (hidden, `.git`,
      Syncthing temp files, `4913`, swap files, output dir inside notebook). *(unit)*
- [ ] AC5 — Symlinks are not followed and produce a warning. *(unit, unix-only test)*
- [ ] AC6 — Directory without `index.md` → note with empty body + warning; stray `.md`
      files → warning. *(unit)*
- [ ] AC7 — Frontmatter: `title` used; unknown fields preserved; malformed YAML → error
      and the whole file becomes the body; an opening `---` with no closing `---` → warning
      and the whole file is the body; CRLF line endings are accepted; a numeric or boolean
      `title` becomes a string, a list or map `title` is an error. *(unit)*
- [ ] AC8 — Links: absolute, `./`, `../`, with labels; broken link → error; links in code
      are ignored; a link written with order prefixes (`[[010-b]]`) resolves and gives a
      warning telling the user to link by slug. *(unit)*
- [ ] AC9 — `gyeoldaero check` output and exit codes match Requirement 4 on a fixture with
      known problems (`tests/fixtures/broken/`). *(cli)*
- [ ] AC10 — Performance target met. *(ignored test or bench, run in the PR description)*
- [ ] AC11 — Unicode: a Hangul slug loads with no diagnostic; an NFD-named folder resolves
      from an NFC link and an NFC `by_path` lookup; NFC and NFD spellings of the same slug
      among siblings are reported as duplicates; so are `Caches` and `caches`; a link
      differing only in case resolves. *(unit)*
- [ ] AC12 — Config: defaults when `gyeoldaero.toml` is absent; a relative `build.out`
      resolves against the notebook root. *(unit)*

## Decisions

Open questions answered by Ellie on 2026-10-23:

- Frontmatter parser: **`serde_yaml_ng`**.
- Non-ASCII slugs: **allowed, no warning** (percent-encoded in URLs, per
  `architecture.md`). Added: slugs and link targets are compared after Unicode NFC
  normalization (Requirement 8, AC11).
- Slug case (raised during planning): comparison is **case-insensitive** after NFC, to
  match the lowercased URL form (Requirement 8, AC11).

Answered by Ellie on 2026-10-23, after the failing tests were written:

- **AC1** no longer requires the mockup's hand-picked sibling order; order follows AC2.
- **Fixture**: the top-level notes get order prefixes (`010-cpu-architecture`,
  `020-korean-infosec-law`, `030-homelab`, `040-inbox`) so the root order matches the
  mockup. Deeper levels stay as they are, so unprefixed sorting stays covered.
- **Prefixed links** (`[[010-b]]`) resolve, with a warning to link by slug: prefixed links
  break when notes are renumbered (AC8).
- **Output path**: a relative `build.out` in `gyeoldaero.toml` resolves against the
  notebook root; a relative `--out` on the command line resolves against the current
  directory (the flag arrives with PRD-002) (AC12).
- **Frontmatter**: an opening `---` with no closing `---` is a warning and the whole file
  is the body (a leading `---` can be a Markdown horizontal rule); CRLF line endings are
  accepted everywhere; a numeric or boolean `title` becomes a string, a list or map is an
  error (AC7).
- **Summary line**: plural for every count except exactly 1 (Requirement 4).
- **General rule** for cases the specs don't cover: be lenient with what people type, warn
  where something could break silently later, and error only when a note can't be
  understood. Recorded in `architecture.md` → *Diagnostics*.

Answered by Ellie on 2026-10-23, on choices raised during implementation:

- **Default output dir**: `<platform cache dir>/gyeoldaero/<folder-name>-<8 hex chars of a
  hash of the canonical absolute notebook path>`, using `dirs::cache_dir()`. The config
  `name` is display-only. (Adds `dirs` to ADR 0002; the hash is 32-bit FNV-1a, written
  inline, because `std`'s `DefaultHasher` isn't stable across Rust versions.)
- **Files that aren't allowed assets** (`.svg`, `.html`, `.docx`, …): no diagnostic in
  PRD-001. PRD-002 warns when a note's Markdown references a file that isn't an asset.

## Plan

1. **Dependencies and ADR.** Add to `[workspace.dependencies]` and `gyeoldaero-core`:
   `serde` (derive), `serde_yaml_ng` 0.10, `toml`, `thiserror`, `unicode-normalization`, `dirs`,
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
     `out` = `dirs::cache_dir()/gyeoldaero/<folder-name>-<hash>`, `~` expanded). *(AC4: output dir ignore)*
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

## Notes

Choices made where the specs are silent, under the general rule (lenient input, warn
where something could break silently, error only when a note can't be understood):

- **Root note.** Its slug and path are empty; `dir_name` is the root folder's name. It
  counts as a note in the `check` summary, and a missing root `index.md` warns like any
  other folder. A root given as a symlink (e.g. `~/notes`) is used as given: the
  no-symlink rule applies to what is found inside the notebook.
- **Folder names.** `010-` alone (prefix with nothing after it) is not a prefix; the whole
  name is the slug. Natural sort compares comparison keys (NFC + lowercase), so `Apple`
  sorts with `apple`; the folder name on disk is the last tie-break, so order is
  deterministic. A folder or file name that isn't valid UTF-8 can't be addressed by a link:
  it is skipped with an error naming it.
- **Duplicate slugs.** One error per clash naming every folder involved (three clashing
  folders → one error). Every clashing note stays in the tree so nothing disappears;
  `by_path` and links resolve to the first in sibling order. The error is attached to the
  parent note.
- **`index.md`.** Only the exact name `index.md` is the body. `Index.md`, `INDEX.MD`, etc.
  get the stray-file warning with a hint to rename (it would work on macOS and silently
  not on Linux). `.md` is matched case-insensitively for stray detection, and so are asset
  extensions (`photo.PNG` is an asset).
- **File contents.** A UTF-8 byte-order mark is dropped. An `index.md` that isn't valid
  UTF-8 is decoded lossily with a warning. CRLF is normalized to LF when a note is read.
- **Frontmatter.** An empty block (`---` / `---`) is an empty mapping, no diagnostic. A
  block that is valid YAML but not a mapping (e.g. `---`, a line of text, `---`, which is
  also a Markdown rule plus a setext heading) is treated like an unclosed block: warning,
  whole file as body. A `title` that is empty or `null` means no title (the slug is shown),
  no diagnostic. A list or map `title` is an error; the rest of the frontmatter still
  applies.
- **Symlinks.** One warning per symlink, on the note whose folder holds it. A symlinked
  `index.md` gets only the symlink warning, not a second "missing index.md" one.
- **Links.** comrak's wikilink extension does the extraction, so code, escaping and
  trimming match what the renderer will parse (PRD-002 should reuse the same parse
  options). `[[]]` is not a link. `[[a|]]` has no label. A trailing `/`, a leading `/`
  and `.`/`..` segments in the middle of a path are accepted. `![[x]]` (embed syntax) is
  treated as a link. `..` above the root is a broken link. A prefixed link segment is only
  tried after no slug matches.
- **Config.** `gyeoldaero.toml` that can't be parsed stops `check` with exit code 2 (the
  notebook can't be read as configured). Unknown keys are warnings (a typo like `[biuld]`
  would otherwise be silently ignored); they are carried in `Config::diagnostics`.
  `name = ""` falls back to the folder name. `~` and `~/…` are expanded; `~user` is not.
  `bind` stays a string until PRD-005 parses it. When `dirs::cache_dir()` finds no cache
  dir (no `HOME`), the default output falls back to the system temp dir.
- **`check` output.** Diagnostics go to stdout, grouped under the note path (`/` for the
  root) in tree order; hard errors go to stderr.
