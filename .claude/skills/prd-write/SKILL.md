---
name: prd-write
description: Draft a new PRD for gyeoldaero from Ellie's idea, using the template, with testable acceptance criteria and open questions. Use when Ellie describes a feature, change or problem to be specced. Never implements anything.
---

# prd-write

Turn an idea into a `Draft` PRD that Ellie can accept or change. Writing a PRD is the
whole job here: do not create branches or touch code.

## Steps

1. Read `docs/bible.md`, `docs/architecture.md`, `docs/renderer-spec.md` (if the idea
   touches pages) and `docs/prd/README.md`. Check the idea doesn't contradict the bible's
   principles or non-goals. If it does, say so before writing anything.
2. If the idea is vague in a way that changes scope, ask Ellie up to three short questions
   first. Otherwise write the draft and list remaining questions inside it.
3. Pick the next free number `NNN` and a short slug. Copy `docs/prd/TEMPLATE.md` to
   `docs/prd/NNN-slug.md`.
4. Fill it in:
   - **Problem / Outcome**: user-visible, a few sentences each.
   - **Non-goals**: at least two, chosen to prevent likely scope creep.
   - **Requirements**: numbered, specific; reference spec sections instead of copying them.
   - **Acceptance criteria**: each testable, each tagged with the test kind
     (unit, snapshot, integration, e2e, cli, by hand). "By hand" only when unavoidable.
   - **Open questions**: real decisions for Ellie, each with a default proposal.
   - Leave **Plan** empty.
5. Add a row to `docs/prd/README.md` with status `Draft`.
6. Run the `spec-reviewer` agent on the new file. Apply its fixes that are clearly right;
   turn the rest into open questions.
7. Tell Ellie in a few lines: the PRD number, the one-sentence outcome, and the open
   questions she needs to answer. Remind her that she sets `Accepted`.

If a PRD would change `bible.md`, write the proposed bible change in the PRD's *Notes*
section and flag it; never edit the bible.
