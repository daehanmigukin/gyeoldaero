---
id: PRD-004
title: Watching and incremental rebuilds
status: Draft
depends_on: [PRD-002]
branch:
---

# PRD-004: Watching and incremental rebuilds

## Problem

Notes change constantly, arriving through Syncthing a second after a save in nvim. The
site must keep up on its own, without serving half-written pages.

## Outcome

A watcher in `gyeoldaero-serve` keeps the output directory in sync with the notebook within
about a second of any change, and never leaves the output broken.

## Scope

- `notify` + `notify-debouncer-full` watcher on the notebook root.
- Event filtering using the same ignore rules as scanning.
- Rebuild planning per `architecture.md` → *Build and watch*.
- Serialized rebuilds, last-good output on failure, structured logs.
- A `gyeoldaero build --watch` mode to exercise it before the server exists.

## Non-goals

- HTTP serving (PRD-005). Live reload in browsers.

## Requirements

1. Debounce window 300 ms, configurable via `gyeoldaero.toml` (`[build] debounce_ms`).
2. Body-only change in note N → re-render N and N's parent. Structural or title change →
   full rebuild. Asset change → copy/remove that asset. Changes to `gyeoldaero.toml` → full
   rebuild with the new config.
3. Events from Syncthing temp files and editor artifacts never trigger a rebuild; the
   final rename to `index.md` does.
4. While a rebuild runs, new events queue; at most one follow-up rebuild runs afterwards.
5. On rebuild failure, log the error with the note path and keep the previous output.
6. Log each rebuild at `info`: what changed, pages written, duration.

## Acceptance criteria

- [ ] AC1 — Editing a body updates that page and its parent's page within 2 s; other
      pages' mtimes are unchanged. *(integration)*
- [ ] AC2 — Creating, renaming (including changing an order prefix) and deleting notes
      updates all affected pages. *(integration)*
- [ ] AC3 — Simulated Syncthing write (`.syncthing.index.md.tmp` written, then renamed to
      `index.md`) triggers exactly one rebuild. *(integration)*
- [ ] AC4 — Simulated nvim save (write `4913`, delete it, write `index.md~`, write
      `index.md`) triggers exactly one rebuild. *(integration)*
- [ ] AC5 — A burst of 50 changes in 200 ms results in at most two rebuilds. *(integration)*
- [ ] AC6 — A reader polling a page during a rebuild never sees a truncated file. *(integration)*

## Open questions

- Is a 300 ms default debounce right for Syncthing on Ellie's setup, or should it be
  longer? (Measure during implementation.)
