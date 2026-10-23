---
id: PRD-NNN
title: Short title
status: Draft            # Draft | Accepted | In Progress | In Review | Shipped | Deferred
depends_on: []           # e.g. [PRD-001]
branch:                  # filled by prd-start, e.g. prd/NNN-short-slug
---

# PRD-NNN: Short title

## Problem

What's missing or wrong today, from the user's point of view. A few sentences.

## Outcome

What the user can do when this ships. Concrete and observable.

## Scope

What this PRD covers, as a short list.

## Non-goals

What it deliberately does not cover, so nobody builds it by accident.

## Requirements

Numbered, specific, unambiguous. Reference `architecture.md` / `renderer-spec.md`
sections instead of repeating them.

1. …

## Acceptance criteria

Each one testable. Note the kind of test that proves it.

- [ ] AC1 — … *(unit)*
- [ ] AC2 — … *(snapshot)*
- [ ] AC3 — … *(integration / e2e / cli)*

## Open questions

Questions Ellie must answer before this becomes `Accepted`. Delete the section when empty.

## Plan

Filled in by `prd-start`: the implementation steps, in order, with the files each touches.

## Notes

Anything learned during implementation worth keeping (links, decisions, gotchas).
