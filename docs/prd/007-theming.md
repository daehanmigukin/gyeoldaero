---
id: PRD-007
title: Theming
status: Deferred
depends_on: [PRD-003]
branch:
---

# PRD-007: Theming

## Problem

The default look is fixed. Ellie wants to tweak the design later, and other users will
want their own colors and fonts.

## Outcome (sketch)

A notebook can override the design tokens (and optionally individual templates) without
forking gyeoldaero.

## Direction (to be refined before Accepted)

- `gyeoldaero.toml` `[theme]` table with token overrides for light and dark, e.g.
  `accent = "#..."`, `accent_dark = "#..."`, or a path to a `theme.css` containing only a
  token block.
- Optional `theme/templates/` folder in the notebook whose files shadow the embedded
  minijinja templates by name.
- Custom fonts: woff2 files in `theme/fonts/`, referenced by the token block.
- Validation in `gyeoldaero check`: unknown tokens, missing font files, template syntax errors.

## Constraints already in place

`architecture.md` → *Theming* and `renderer-spec.md` §5: all colors and fonts are tokens,
templates load through minijinja. Nothing in PRD-002/003 may hard-code a color or font
outside the token block.

## Open questions

- Tokens only, or templates too, for the first version?
- Ship a few built-in themes?
