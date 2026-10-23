---
name: adr
description: Record a technical decision for gyeoldaero as an Architecture Decision Record in docs/adr. Use when choosing or swapping a dependency, data format, concurrency approach or anything architecture.md doesn't already settle.
---

# adr

1. List `docs/adr/` and pick the next number `NNNN` (4 digits).
2. Create `docs/adr/NNNN-short-slug.md`:
   ```markdown
   # ADR NNNN: Title

   - Status: Proposed | Accepted | Superseded by ADR XXXX
   - Date: YYYY-MM-DD

   ## Context
   The forces at play, in a few sentences. Link the PRD if there is one.

   ## Decision
   What we're doing, stated plainly.

   ## Consequences
   What gets easier, what gets harder, what we must now keep true.

   ## Alternatives considered
   Each with one line on why not.
   ```
3. Status starts as `Proposed`. Ellie accepts it (in chat or by approving the PR); then set
   `Accepted`.
4. If it changes something in `architecture.md`, update that file in the same PR and link
   the ADR next to the changed section.
5. Superseding: never edit an accepted ADR's decision; write a new ADR and mark the old one
   `Superseded by ADR NNNN`.
