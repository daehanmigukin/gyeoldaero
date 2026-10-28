---
id: PRD-002
title: Static renderer and `gyeoldaero build`
status: Draft
depends_on: [PRD-001]
branch:
---

# PRD-002: Static renderer and `gyeoldaero build`

## Problem

The model exists but there is nothing to look at. The first visible deliverable is a set
of static pages that already work in any browser, without JavaScript.

## Outcome

`gyeoldaero build [PATH] [--out DIR]` writes a complete static site for a notebook. Opening it
in a browser shows the mockup's design and lets you expand notes, open notes and go back,
all without JavaScript.

## Scope

- Markdown rendering (comrak, GFM, raw HTML escaped, safe link schemes).
- Templates (minijinja, embedded) producing the page anatomy and static markup in
  `renderer-spec.md` §2–3.
- `gyeoldaero.css` with the full token block and component styles from the mockup (§5).
- Bundled fonts with license files; asset hashing for cache busting.
- Copying allowed assets; atomic writes; output path safety.
- `gyeoldaero build` subcommand.

## Non-goals

- `gyeoldaero.js` and all enhanced behavior (PRD-003). The hint bar is rendered but hidden.
- Incremental rebuilds (PRD-004).
- Theme overrides (PRD-007).

## Requirements

1. Follow `renderer-spec.md` §2, §3, §5 and §6, and `architecture.md` → *Rendering* and
   *Security notes*.
2. The page template set: `base.html` (document shell), `page.html` (note view),
   `row.html` (child row macro). Embedded with `include_str!` and loaded through a
   minijinja `Environment` built once per build.
3. Wiki links render as relative `href`s to the target note's page; broken links render as
   specified in §4.7 (static part).
4. Fonts: vendor Atkinson Hyperlegible Next (400, 600, 700) and JetBrains Mono (400, 500,
   600) as woff2 under `crates/gyeoldaero-render/assets/fonts/` with `OFL.txt` for each, from
   the official upstream repositories. Record source URLs and versions in
   `assets/fonts/SOURCES.md`.
5. The build removes stale files from previous builds in the output directory (pages for
   deleted notes) but never deletes anything outside it, and refuses to use an output
   directory that contains the notebook or is the notebook.
   **Marker file** (Ellie, 2026-10-23): when the build creates the output directory it
   writes a gyeoldaero marker file into it. It refuses to build into, or delete anything
   from, a non-empty directory without that marker. This covers a `build.out` pointing
   outside the notebook (e.g. `~/Documents` from a synced `gyeoldaero.toml`), which
   PRD-001's config checks can't catch.
6. `gyeoldaero build` prints `Built N pages in X ms → <out>` and exits 0; on errors in the
   notebook it still builds and prints diagnostics, exiting 1 only on hard errors.
7. Before any page is rendered, the scanner's folder-swap race is closed (PRD-001 Notes →
   *Must do before PRD-002 renders any page*): folders and files are opened relative to
   their parent handle with `O_NOFOLLOW` through a safe crate, chosen in an ADR that weighs
   `cap-std` against `rustix`. Otherwise a folder swapped for a symlink could leak files
   from outside the notebook into served pages.

## Acceptance criteria

- [ ] AC1 — Snapshot tests (`insta`) of the root page, a mid-level page
      (`cpu-architecture/caches`) and a leaf page for the fixture notebook. *(snapshot)*
- [ ] AC2 — Every fixture note has a page at `<path>/index.html`; relative links between
      pages all resolve to existing files. *(integration: crawl the output)*
- [ ] AC3 — Raw HTML in Markdown is escaped; `javascript:` and `data:` links are dropped.
      *(unit)*
- [ ] AC4 — An asset with an allowed extension is copied; an `.svg` is not. *(integration)*
- [ ] AC5 — Rebuilding after deleting a note removes its page; an output dir that
      contains the notebook is refused; a new output dir gets the marker file; a
      non-empty output dir without the marker is refused and nothing in it is changed or
      deleted. *(integration)*
- [ ] AC9 — A folder swapped for a symlink between listing and reading is not followed
      (handle-relative opening). *(unit)*
- [ ] AC6 — Rendered pages make no external requests and contain no inline `<script>` or
      `style=` attributes. *(integration: scan output)*
- [ ] AC7 — Without JS, in a real browser: `<details>` expand, `.enter` links and back
      links work. *(e2e with JS disabled; may land with PRD-003's harness if Playwright
      isn't set up yet, but then PRD-003 must include it)*
- [ ] AC8 — Visual check: light and dark screenshots of the three snapshot pages are
      attached to the PR next to the mockup for comparison. *(by hand)*

## Open questions

- Marker trust: a marker inside the notebook could be planted by a synced folder. PRD-001
  already refuses a `build.out` inside the notebook that holds notes; should the marker
  also carry a token recorded outside the notebook (e.g. under the cache dir)? (Security
  review suggestion; default proposal: yes.)

- Should the root heading show the notebook `name` from config or the root folder slug?
  (Default proposal: `name`.)

- Duplicate slugs: PRD-001 keeps every clashing note in the tree with the same `NotePath`,
  so two pages would render to the same output file. Should the build refuse to run while
  any slug clash exists? (Default proposal: yes, refuse to build, using the clash errors
  as the message.)
