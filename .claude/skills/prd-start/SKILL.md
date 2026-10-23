---
name: prd-start
description: Start implementing an Accepted gyeoldaero PRD — create its branch, set status to In Progress, write the implementation plan and task list. Use when Ellie says to start, pick up or work on a PRD.
---

# prd-start

## Preconditions

- The PRD's front matter says `status: Accepted`. If not, stop and tell Ellie what status
  it has and what's missing (usually open questions). Never change it to `Accepted`.
- Every PRD in `depends_on` is `Shipped` (or Ellie explicitly says to proceed anyway).
- The working tree is clean. If not, ask Ellie what to do with the changes.

## Steps

1. `git switch main && git pull --ff-only`.
2. `git switch -c prd/NNN-slug` (slug from the PRD file name).
3. Re-read the PRD and every spec section it references.
4. Write the **Plan** section of the PRD: ordered steps, each naming the files or modules
   it touches and which acceptance criteria it serves. Include the test-first step.
   If planning reveals a gap or conflict in the specs, stop and ask Ellie before going on.
5. Update front matter: `status: In Progress`, `branch: prd/NNN-slug`. Update the row in
   `docs/prd/README.md`.
6. Commit: `docs(prd): start PRD-NNN` with `Refs: PRD-NNN`.
7. Create a task list mirroring the plan, ending with "Review (code-reviewer,
   security-reviewer if applicable)" and "Ship (prd-ship)".
8. Next: use the `test-author` agent to write failing tests for the acceptance criteria,
   then implement.

Tell Ellie in two or three lines: branch name, number of plan steps, and anything in the
plan she should look at.
