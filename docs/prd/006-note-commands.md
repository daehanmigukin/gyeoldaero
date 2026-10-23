---
id: PRD-006
title: Note commands (`gyeoldaero new`)
status: Draft
depends_on: [PRD-001]
branch:
---

# PRD-006: Note commands (`gyeoldaero new`)

## Problem

In gyeoldaero a note is a folder plus an `index.md`. Making one by hand is two steps and easy
to get wrong (forgetting `index.md`, picking an order prefix). Ellie wants something as
quick as `:touch` or `mkdir` from wherever she is in the tree.

## Outcome

`gyeoldaero new <slug>` creates a note in the current directory in one step, and can open it
in `$EDITOR`. A shell alias makes it a one-letter command.

## Scope

- `gyeoldaero new <slug> [--in DIR] [--after SLUG] [--prefix auto|none] [--title TEXT] [--edit]`.
- Notebook root discovery (walk up to the nearest `gyeoldaero.toml`).
- Docs for a shell alias and an nvim-friendly workflow.

## Non-goals

- A yazi plugin (PRD-008).
- Moving or renaming notes (later PRD if needed).

## Requirements

1. Creates `<dir>/<prefix?><slug>/index.md`. `dir` defaults to the current directory and
   must be inside a notebook (found by walking up to `gyeoldaero.toml`); otherwise exit 1 with
   a clear message.
2. `index.md` is created empty, or with frontmatter containing `title` when `--title` is given.
3. Prefixing:
   - `--prefix auto` (default when any sibling has a prefix): next multiple of 10 after
     the highest sibling prefix, zero-padded to the siblings' width (default 3 digits).
   - `--after SLUG`: the midpoint between that sibling's prefix and the next one; if no
     integer gap remains, exit 1 and suggest renumbering.
   - `--prefix none` or no prefixed siblings: no prefix.
4. Refuses to create a slug that already exists among siblings (with or without prefix).
   Validates the slug against the rules in `architecture.md`.
5. `--edit` runs `$VISUAL`, then `$EDITOR`, then `vi` on the new `index.md`.
6. Prints the created path relative to the notebook root.
7. README documents: `alias t='gyeoldaero new --edit'` and `alias tn='gyeoldaero new'`.

## Acceptance criteria

- [ ] AC1 — Creating a note with no prefixed siblings makes `<slug>/index.md`. *(cli)*
- [ ] AC2 — With prefixed siblings `010-a`, `020-b`, `gyeoldaero new c` makes `030-c`;
      `--after a` makes `015-c`. *(cli)*
- [ ] AC3 — No gap left → exit 1 with a renumbering suggestion. *(cli)*
- [ ] AC4 — Duplicate slug and invalid slug are refused with exit 1. *(cli)*
- [ ] AC5 — Outside a notebook → exit 1 with a message naming `gyeoldaero.toml`. *(cli)*
- [ ] AC6 — `--edit` launches the editor from `$VISUAL`/`$EDITOR`. *(cli with a fake editor script)*
- [ ] AC7 — `gyeoldaero check` on the result reports no new diagnostics. *(cli)*

## Open questions

- Should `gyeoldaero new` default to adding prefixes even when no sibling has one, so ordering
  is explicit from the start? (Bible open question.)
- Should there be `gyeoldaero init` to create `gyeoldaero.toml` in a new notebook? (Default
  proposal: yes, add it to this PRD.)
