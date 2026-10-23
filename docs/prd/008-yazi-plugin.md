---
id: PRD-008
title: yazi plugin
status: Deferred
depends_on: [PRD-006]
branch:
---

# PRD-008: yazi plugin

## Problem

In yazi, a note is a folder, so hovering it shows a directory listing instead of the
note's text, and creating a note means leaving yazi or typing a shell command.

## Outcome (sketch)

Inside a notebook, yazi previews a folder's `index.md` when you hover it, and a keybinding
creates a new note in the current directory (`gyeoldaero new`) and opens it in nvim.

## Direction (to be refined before Accepted)

- A yazi previewer (Lua) that, for directories containing `index.md`, renders the file
  (via the built-in Markdown previewer or `glow`) above the child listing.
- A plugin action bound to a key (e.g. `t`) that prompts for a slug and runs
  `gyeoldaero new --edit`.
- Lives in `contrib/yazi/gyeoldaero.yazi/` in this repo, installable with `ya pack`.

## Open questions

- Check yazi's current plugin and previewer API at the time this is picked up; it has
  changed between releases.
