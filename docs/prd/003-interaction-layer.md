---
id: PRD-003
title: Interaction layer (`gyeoldaero.js`) and e2e tests
status: Draft
depends_on: [PRD-002]
branch:
---

# PRD-003: Interaction layer (`gyeoldaero.js`) and e2e tests

## Problem

Static pages work, but they don't feel like walking directories. The approved mockup
defines swipes, animated transitions, remembered state and yazi-style keys.

## Outcome

The built site behaves like `docs/reference/mockup.html` on a phone and a laptop, while
remaining a set of static pages that work without JavaScript.

## Scope

- `gyeoldaero.js`: router with `pushState`/`popstate`, prefetching, transitions, gestures,
  flip swipe, animated expand/collapse, session state, came-from highlight, keyboard,
  hint bar.
- Playwright e2e suite in `tests/e2e` run against `gyeoldaero build` of the fixture notebook.
- CI job for the e2e suite.

## Non-goals

- Live reload when notes change (separate PRD if wanted).
- Search, backlinks UI.

## Requirements

1. Implement `renderer-spec.md` §4 in full. Port logic from the mockup's script where it
   applies, adapted to multi-page navigation (fetch + extract `.view`).
2. `gyeoldaero.js` is plain ES2020, no dependencies, no build step, under 15 KB unminified.
   It must not throw if `localStorage`/`sessionStorage` are unavailable.
3. All markup hooks are the classes and `data-path` attributes from §3.
4. e2e tooling lives entirely in `tests/e2e/` (`package.json`, `playwright.config.ts`).
   The suite builds the fixture with the release binary, serves the output with a static
   server (or `gyeoldaero serve` once PRD-005 exists), and runs on Chromium and WebKit, with
   a mobile viewport project (e.g. iPhone 13) using touch.
5. Document how to run it in `docs/dev-commands.md`.

## Acceptance criteria

Map one-to-one to `renderer-spec.md` §7:

- [ ] AC1 — Expand/collapse animates and persists per page in the session. *(e2e)*
- [ ] AC2 — Open via control, row swipe (past 70 px) and `l`; short swipe springs back. *(e2e)*
- [ ] AC3 — Back via button, page swipe (with parent peek), `h`, path segment and browser
      back; URL updates each time and reload shows the same page. *(e2e)*
- [ ] AC4 — Flip swipe reverses gestures, persists across reloads, updates hint text and
      `aria-pressed`. *(e2e)*
- [ ] AC5 — Returning to a parent restores scroll position and highlights the came-from row. *(e2e)*
- [ ] AC6 — `prefers-reduced-motion: reduce` swaps views with no animation. *(e2e with emulated media)*
- [ ] AC7 — With JavaScript disabled, the static behavior from PRD-002 still works. *(e2e)*
- [ ] AC8 — No console errors on any fixture page; script size under budget. *(e2e + check)*
- [ ] AC9 — Ellie has tried the built fixture on her phone over the tailnet and approved
      the feel. *(by hand — record in the PR)*

## Open questions

- Should vertical scroll position be kept when opening a child that was already visited
  in this session, or always start at the top? (Default proposal: start at the top,
  matching the mockup.)
