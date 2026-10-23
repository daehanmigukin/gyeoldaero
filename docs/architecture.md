# gyeoldaero — Architecture

How gyeoldaero is built. Subordinate to `bible.md`. Changes to this document go through a PRD
or an ADR (`docs/adr/`), never silently alongside code.

---

## Shape

One Cargo workspace, one shipped binary (`gyeoldaero`), four crates with one-way dependencies:

```
gyeoldaero-cli ──▶ gyeoldaero-serve ──▶ gyeoldaero-render ──▶ gyeoldaero-core
    │                                   ▲            ▲
    └───────────────────────────────────┴────────────┘
```

| Crate | Kind | Owns | Must not |
|---|---|---|---|
| `gyeoldaero-core` | lib | Scanning a notebook folder into an in-memory model; parsing `index.md` (frontmatter + body); order prefixes; path and link resolution; validation diagnostics; config (`gyeoldaero.toml`). | Know about HTML, HTTP, or file watching. |
| `gyeoldaero-render` | lib | Turning the model into pages: Markdown → HTML, templates, bundled assets (CSS, JS, fonts), output layout, atomic writes. | Know about HTTP or watching. Do filesystem scanning. |
| `gyeoldaero-serve` | lib | Watching the notebook, debouncing, deciding what to rebuild, serving the output directory over HTTP. | Contain rendering or parsing logic. |
| `gyeoldaero-cli` | bin `gyeoldaero` | Argument parsing, subcommands (`serve`, `build`, `check`, `new`), logging setup, exit codes. | Contain business logic beyond wiring. |

Directory layout:

```
gyeoldaero/
├── Cargo.toml              # workspace
├── crates/
│   ├── gyeoldaero-core/
│   ├── gyeoldaero-render/
│   │   └── assets/         # gyeoldaero.css, gyeoldaero.js, fonts/, templates/
│   ├── gyeoldaero-serve/
│   └── gyeoldaero-cli/
├── tests/
│   ├── fixtures/notebook/  # sample notebook (content of the mockup)
│   └── e2e/                # Playwright tests for the interaction layer
├── docs/
└── .claude/
```

## Recommended dependencies

Pinned choices; changing one needs an ADR.

| Need | Crate | Why |
|---|---|---|
| CLI | `clap` (derive) | Standard. |
| Markdown | `comrak` | GFM, built-in wikilink extension, raw HTML can be disabled. Also used by core for link extraction ([ADR 0002](adr/0002-notebook-model-deps.md)). |
| Frontmatter | hand-split `---` + `serde_yaml_ng` | Small surface. Chosen in PRD-001. |
| Config | `toml` | `gyeoldaero.toml`, read as a table so unknown settings can be reported. [ADR 0002](adr/0002-notebook-model-deps.md). |
| Platform dirs | `dirs` | Cache dir (default output) and `~`. [ADR 0002](adr/0002-notebook-model-deps.md). |
| Unicode | `unicode-normalization` | NFC comparison keys. [ADR 0002](adr/0002-notebook-model-deps.md). |
| Templates | `minijinja` | Runtime templates, so themes can override them later. Default templates embedded with `include_str!`. |
| Watching | `notify` + `notify-debouncer-full` | Cross-platform, debounced events. |
| HTTP | `axum` + `tower-http` (`ServeDir`, headers) | Small, well-maintained. |
| Async runtime | `tokio` | Required by axum. Keep core and render synchronous. |
| Errors | `thiserror` in libs, `anyhow` in the binary | Conventional split. |
| Logging | `tracing` + `tracing-subscriber` | Structured, leveled. |
| Snapshot tests | `insta` | Golden tests for rendered HTML. |
| Temp dirs in tests | `tempfile` | |
| CLI tests | `assert_cmd` | [ADR 0002](adr/0002-notebook-model-deps.md). |

Edition 2024. Toolchain: stable, pinned in `rust-toolchain.toml`.

---

## Notebook model (`gyeoldaero-core`)

### Scanning rules

- The notebook root is a note. Every directory below it is a note, except ignored ones.
- A note's body comes from `index.md`. A directory without `index.md` is still a note with
  an empty body; `gyeoldaero check` reports it as a warning.
- Other `.md` files in a note folder are ignored in v1 and reported by `check` (they are
  probably meant to be notes of their own).
- Non-Markdown files with an allowed extension are assets of that note (see Security).
- Ignored entirely: hidden entries (starting with `.`), `.git`, Syncthing artifacts
  (`.stfolder`, `.stversions`, `.syncthing.*.tmp`, `~syncthing~*`), editor artifacts
  (`*.swp`, `*.swo`, `*~`, `4913`), and the output directory if it sits inside the
  notebook.
- Symlinks are not followed. They are reported by `check`.
- A folder whose slug would start with `.` (e.g. `001-..`, `002-.git`) is an error and is
  skipped: such a slug would be hidden, or a path traversal once written as a URL path.
- Limits: nesting deeper than 64 levels below the root is an error and the deeper folders
  are skipped; an `index.md` over 8 MiB is an error and the note gets an empty body.

### Folder names, slugs, order

- Folder name = optional order prefix + slug. Prefix regex: `^(\d{1,4})-`.
  `020-caches` → order 20, slug `caches`.
- Slugs must match `^[a-z0-9][a-z0-9-]*$` after lowercasing for URLs. Other characters
  (including Hangul) are allowed in folder names; their URL form is percent-encoded and
  `check` does not complain. Display always uses the slug as written.
- Slugs are compared by a key: Unicode NFC normalization, then lowercasing. So an NFD
  folder name matches an NFC link, and `Caches` and `caches` are the same slug.
  Duplicate detection, path lookups and link resolution all use this key (PRD-001).
- Two siblings with the same slug (e.g. `010-x` and `x`) are an error.
- Sibling order: prefixed notes first, by number, then unprefixed notes by natural sort
  (so `note-2` < `note-10`). Ties on number break by slug.

### Frontmatter

Optional YAML block at the top of `index.md`, delimited by `---` lines. v1 fields:

| Field | Type | Effect |
|---|---|---|
| `title` | string | Shown instead of the slug in headings and rows. A number or boolean is converted to a string; a list or map is a `check` error. |

Unknown fields are preserved in the model and ignored by the renderer, so later features
can use them. Malformed frontmatter is a `check` error; the note renders with the whole
file as body. An opening `---` with no closing `---` is not frontmatter (a leading `---`
can be a Markdown horizontal rule): the whole file is the body and `check` warns.
CRLF line endings are accepted everywhere (frontmatter, body, config).

### Links

- Syntax: `[[path]]` or `[[path|label]]`. Path is from the notebook root, using slugs
  (prefixes not needed). `[[./child]]` and `[[../sibling]]` are relative to the current note.
- A link written with order prefixes (`[[010-caches]]`) still resolves, but `check` warns
  and asks for the slug: prefixed links break when notes are renumbered.
- A link that doesn't resolve renders as a visibly broken link (no `href`) and is an error
  in `check`.
- The model keeps outgoing links per note so backlinks can be added without rescanning.

### Diagnostics

`gyeoldaero-core` returns diagnostics (level, note path, folder on disk, message) instead of
failing on content problems. Only an unusable notebook root or `gyeoldaero.toml` is a hard
error. Below the root, an I/O error (unreadable folder or file) is an error diagnostic on
its note and the scan continues; an `index.md` that disappears mid-scan counts as missing.
`gyeoldaero check` prints diagnostics and exits non-zero on any error-level diagnostic. It
escapes control and bidi characters in what it prints, since file names and link targets
are untrusted.

For cases the rules above don't cover: be lenient with what people type, warn where
something could break silently later, and error only when a note can't be understood.

---

## Rendering (`gyeoldaero-render`)

- One HTML file per note. Output mirrors note paths:
  `out/index.html`, `out/cpu-architecture/index.html`, `out/cpu-architecture/caches/index.html`.
- URLs end with `/`. Internal links are relative so the site works under any base path.
- Each page contains: path bar, the note's heading and body, and each child's title and
  body inside a collapsible element. Exact structure: `renderer-spec.md`.
- Bundled static files under `out/_gyeoldaero/`: `gyeoldaero.css`, `gyeoldaero.js`, `fonts/*.woff2`,
  font license files. Asset URLs carry a content hash query (`gyeoldaero.css?v=<hash>`) for
  cache busting.
- Assets in note folders are copied next to that note's `index.html`.
- All writes are atomic: write to a temp file in the same directory, then rename.
- Markdown raw HTML is **disabled** (escaped). A config switch to allow it may come later.

## Build and watch (`gyeoldaero-serve`)

- `gyeoldaero build`: scan, render everything, write to the output directory, exit.
- `gyeoldaero serve`: full build, then watch the notebook and serve the output.
- Watch events are debounced (300 ms default). Events on ignored paths are dropped before
  debouncing.
- Rebuild rules:
  - Body change in note N → re-render N and N's parent (the parent shows N's body).
  - Any structural change (note created, deleted, renamed, moved, order prefix changed,
    `title` changed) → full rebuild. Correct first; optimize later only with measurements.
  - Asset change → copy or remove that asset.
- If a rebuild fails, keep serving the last good output and log the error.
- Rebuilds are serialized; events arriving during a rebuild trigger one more rebuild after.

## Serving

- `axum` serving the output directory. Default bind `127.0.0.1:4321`; `--bind` to change
  (e.g. to the Tailscale address). The docs recommend `tailscale serve` for HTTPS.
- Headers on every response: a strict `Content-Security-Policy` (`default-src 'self'`,
  no inline scripts), `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`.
- Only files inside the output directory are ever served. No directory listings.

## Configuration

`gyeoldaero.toml` in the notebook root, all optional:

```toml
name = "notebook"          # shown at the root of the path bar; default: folder name
[serve]
bind = "127.0.0.1:4321"
[build]
out = "~/.cache/gyeoldaero/notebook-1a2b3c4d"   # default, see below
```

A `build.out` that is the notebook root, an ancestor of it, the home directory or an
ancestor of it is a config error: a synced notebook must not be able to point the build at
the user's files. So is a `build.out` inside the notebook whose folder (or anything below
it) holds an `index.md`: it would hide those notes from the scan. The build writes a marker
file into the output directory it creates and refuses to build into, or delete anything
from, a non-empty directory without it (PRD-002). `gyeoldaero.toml` over 8 MiB is a config error.

The default `out` is `<cache dir>/gyeoldaero/<folder-name>-<hash>`: the cache dir comes
from `dirs::cache_dir()` (`$XDG_CACHE_HOME` or `~/.cache` on Linux, `~/Library/Caches` on
macOS), `<folder-name>` is the notebook folder's name and `<hash>` is 8 hex characters of
a 32-bit FNV-1a hash of the canonical absolute notebook path, so two notebooks with the
same folder name don't share an output dir. `name` is display-only and never part of a
path.

CLI flags override config. Config is parsed by `gyeoldaero-core`. A relative `build.out`
in `gyeoldaero.toml` resolves against the notebook root; a relative `--out` on the command
line resolves against the current directory.

## Security notes

The server may be reachable from other devices on a tailnet, and note content is
user-written but may include pasted material. Treat it as untrusted input to the renderer.

- Escape all text. Raw HTML in Markdown is off. URLs in Markdown links: allow `http`,
  `https`, `mailto` and relative only; drop `javascript:` and `data:`.
- Allowed asset extensions v1: `png jpg jpeg gif webp avif pdf txt`. **No SVG** in v1
  (it can carry scripts).
- No symlink following during scan or copy; canonicalize and verify every output path is
  inside the output root before writing. Files are opened and checked to be the same
  regular file `lstat` saw. Folders are re-checked with `symlink_metadata` before
  `read_dir`, which narrows but doesn't close the race of a folder being swapped for a
  symlink; that must be closed (`openat` with `O_NOFOLLOW`) before PRD-002 renders any
  page.
- The server never serves anything outside the output directory and never serves the
  notebook directly.
- No requests to third parties from rendered pages.

## Testing strategy

| Layer | How |
|---|---|
| Core | Unit tests per rule (ordering, slugs, links, ignore list). Fixture notebooks built in `tempfile` dirs. |
| Render | `insta` snapshot tests of rendered pages for `tests/fixtures/notebook`. Snapshots are reviewed, never blindly accepted. |
| Serve | Integration tests: start a build+watch on a temp notebook, modify files, assert output changes within a timeout. HTTP tests for headers and path traversal. |
| Interaction | Playwright tests in `tests/e2e` against `gyeoldaero build` output of the fixture: expand, enter, back, swipe, flip, keyboard, no-JS fallback. |
| CLI | `assert_cmd`-style tests for exit codes and `new` behavior. |

CI runs fmt, clippy (`-D warnings`), all Rust tests, and the e2e suite.

## Theming (designed for, not built)

All colors, fonts and radii live as CSS custom properties in one block at the top of
`gyeoldaero.css`. Templates are loaded through minijinja from embedded defaults. A later
theming PRD can let users override the token block and/or individual templates from the
notebook without changing this architecture.
