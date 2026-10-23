---
name: prd-ship
description: Finish a gyeoldaero PRD — run all checks and reviews, tick the acceptance checklist, update docs, set In Review, push the branch and open the PR. Use when implementation of a PRD is complete.
---

# prd-ship

## Steps

1. Confirm you're on the PRD's branch and the tree is committed.
2. Run the full checks:
   ```sh
   cargo fmt --all --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```
   plus the e2e suite if the PRD touches `gyeoldaero-render` assets or templates
   (see `docs/dev-commands.md`). Fix failures; don't skip.
3. Walk the acceptance criteria one by one. For each, name the test that proves it (file
   and test name). Tick it only if that test exists and passes. Untickable items: stop and
   tell Ellie.
4. Reviews, if not already done on the final diff:
   - `code-reviewer` agent on `git diff main...HEAD`.
   - `security-reviewer` agent if the diff touches parsing, rendering, file paths,
     watching or the server.
   Fix findings or write a one-line reason for each one you don't fix.
5. Docs: update `docs/dev-commands.md` for new commands, `README.md` for user-facing
   changes, and the PRD's *Notes* with anything worth keeping.
6. Set the PRD to `status: In Review` (front matter and index). Commit:
   `docs(prd): PRD-NNN ready for review`.
7. Push the branch (`git push -u origin prd/NNN-slug`) and open the PR with
   `gh pr create`, filling `.github/pull_request_template.md`:
   - Title: `PRD-NNN: <title>`.
   - Summary, acceptance criteria with their tests, reviewer findings and responses,
     anything that needs Ellie's eyes (screenshots for UI work).
8. `gh pr checks --watch`; fix anything red.
9. Give Ellie the PR link and a three-line summary.

After Ellie merges: set the PRD to `Shipped` on `main` via a small `docs/prd-NNN-shipped`
branch and PR, or include it in the next PRD's first commit.
