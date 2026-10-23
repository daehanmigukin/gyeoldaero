# gyeoldaero — Development Workflow

How work happens in this repo. Written for both Ellie and Claude Code. `CLAUDE.md` summarizes
it; this file is the full version.

---

## 1. Source-of-truth order

```
docs/bible.md            (what and why — Ellie only)
  └─ docs/architecture.md, docs/renderer-spec.md   (how, at system level)
       └─ docs/prd/NNN-*.md                         (one unit of work)
            └─ code and tests
```

A lower level never contradicts a higher one. When implementation reveals that a spec is
wrong or incomplete, **stop and propose the spec change** to Ellie (in chat or as a PR
comment). Do not quietly implement something different from the spec.

Significant technical decisions that the specs don't settle (a dependency swap, a data
format, a concurrency model) are recorded as ADRs in `docs/adr/` with the `adr` skill.

## 2. PRD lifecycle

| Status | Meaning | Who sets it |
|---|---|---|
| `Draft` | Written, open questions may remain. Not ready to build. | Claude or Ellie |
| `Accepted` | Ellie approved the scope and acceptance criteria. Ready to build. | **Ellie only** |
| `In Progress` | A branch exists and work has started. | Claude (`prd-start`) |
| `In Review` | PR open, checks green, reviewers run. | Claude (`prd-ship`) |
| `Shipped` | Merged to `main`. | Claude, after Ellie merges |
| `Deferred` | Parked. | Ellie |

The status lives in the PRD's front matter and in `docs/prd/README.md`. Claude never starts
implementing a PRD that is not `Accepted`, and never marks one `Accepted`.

## 3. Branches, commits, PRs

- `main` is protected and always green. After the bootstrap commit, nothing is committed
  to `main` directly.
- One PRD → one branch → one PR. Branch name: `prd/NNN-short-slug`.
  Small fixes outside a PRD: `fix/short-slug`, docs-only: `docs/short-slug`.
- Conventional Commits: `feat(render): …`, `fix(core): …`, `test(serve): …`,
  `docs(prd): …`, `chore: …`, `ci: …`. Reference the PRD in the body: `Refs: PRD-002`.
- Small commits that each build and pass tests.
- PRs use `.github/pull_request_template.md`. Ellie merges. Squash-merge is the default.

## 4. Building a PRD (the loop)

1. **Start** (`prd-start` skill): confirm `Accepted`, create the branch, set `In Progress`,
   write an implementation plan into the PRD's *Plan* section, create a task list.
2. **Tests first**: use the `test-author` agent to turn the acceptance criteria into
   failing tests (unit, snapshot, integration or e2e as appropriate). Commit them.
3. **Implement** in small steps until the tests pass. The hooks format code on every edit
   and run checks before Claude finishes a turn.
4. **Review**: run the `code-reviewer` agent against the diff and the PRD. For anything
   touching parsing, rendering, file paths or the server, also run `security-reviewer`.
   Fix findings or record why not.
5. **Ship** (`prd-ship` skill): full checks, tick the acceptance checklist, update docs
   (`dev-commands.md`, README if user-facing), set `In Review`, push, open the PR.
6. After Ellie merges: set `Shipped`, update the PRD index.

## 5. Definition of done

A PRD is done when all of these are true:

- [ ] Every acceptance criterion has a test that fails without the change and passes with it.
- [ ] `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
      `cargo test --workspace` pass. The e2e suite passes when the PRD touches the renderer.
- [ ] No new `unwrap()`/`expect()` in library code paths that handle user input.
- [ ] Public items in library crates have doc comments.
- [ ] Snapshot changes were reviewed one by one.
- [ ] Reviewer findings are fixed or answered in the PR.
- [ ] Docs updated; the PRD's checklist is ticked and its status is `In Review`.

## 6. Writing a PRD

Use the `prd-write` skill. A good PRD:

- States the problem and the user-visible outcome in a few sentences.
- Lists explicit **non-goals**.
- Has acceptance criteria that are each testable by a specific kind of test.
- Names open questions instead of guessing. Ellie answers them before `Accepted`.
- Runs past the `spec-reviewer` agent before being handed to Ellie.

## 7. Working with Ellie

- Ask before anything hard to undo: deleting files outside the repo, force pushes,
  changing GitHub settings, publishing a crate, changing the license.
- When a choice is genuinely Ellie's (product behavior, naming, scope), ask. When it's an
  engineering default with a clear best answer, choose it and say so in the PR.
- Keep chat updates short. The PRD, the task list and the PR are the record.
