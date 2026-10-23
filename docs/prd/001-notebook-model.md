---
id: PRD-001
title: Notebook model and `gyeoldaero check`
status: Draft
depends_on: [PRD-000]
branch:
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
     body: String, frontmatter: serde_yaml::Mapping (or equivalent), links: Vec<Link>,
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

## Open questions

- Frontmatter parser: `serde_yaml` is unmaintained; `serde_yaml_ng` or a TOML-frontmatter
  alternative? (Default proposal: `serde_yaml_ng`.)
- Should non-ASCII slugs (e.g. Hangul folder names) be allowed in v1, or warned about?
  (Default proposal: allowed, percent-encoded in URLs, no warning.)
