---
id: PRD-000
title: Repository bootstrap
status: Shipped
depends_on: []
branch: main (bootstrap is the one exception to the no-commits-to-main rule)
---

# PRD-000: Repository bootstrap

## Problem

There is no repository yet. Before any feature work, the project needs its structure,
its specs in place, the Claude Code harness (context file, hooks, skills, agents), CI,
and a sample notebook to test against.

## Outcome

A public GitHub repo `gyeoldaero` whose `main` branch builds, passes CI, contains all specs,
and is ready for PRD-001 to start on a branch using the `prd-start` skill.

## Scope

- Name availability check.
- Git repo, Cargo workspace with four compiling stub crates.
- Docs, harness files and CI copied from the bootstrap package.
- A fixture notebook built from the mockup's sample notes.
- GitHub repo creation and first push.

## Non-goals

- Any real scanning, rendering, watching or serving logic. Stubs only.
- Publishing to crates.io (the name may be reserved later, with Ellie's approval).

## Requirements

1. **Ask Ellie first**, in one message:
   - her GitHub username or org for the repo;
   - the copyright holder name for the MIT license;
   - where to create the local repo (default `~/code/gyeoldaero`);
   - whether to turn on branch protection for `main` after the first push.
2. **Name check**: report whether `gyeoldaero` is already used on crates.io
   (`cargo search gyeoldaero`) and whether `<owner>/gyeoldaero` exists on GitHub
   (`gh repo view <owner>/gyeoldaero`). If the crate name is taken, tell Ellie and continue;
   the binary and repo can still be `gyeoldaero`.
3. **Layout**: create the workspace exactly as in `architecture.md` → *Shape*:
   `crates/gyeoldaero-core`, `crates/gyeoldaero-render`, `crates/gyeoldaero-serve`, `crates/gyeoldaero-cli`
   (binary name `gyeoldaero`). Edition 2024. Shared metadata (`license = "MIT"`, `repository`,
   `rust-version`) in `[workspace.package]`; dependency versions in
   `[workspace.dependencies]`. Only add dependencies the stubs actually use (`clap` for
   the CLI is enough now).
4. **Stubs**: each library crate exposes one documented placeholder item and one passing
   unit test. `gyeoldaero --version` and `gyeoldaero --help` work; subcommands `build`, `serve`,
   `check`, `new` exist and print "not implemented yet (PRD-NNN)" with exit code 2.
5. **Toolchain and lint config**: `rust-toolchain.toml` (stable channel, components
   `rustfmt`, `clippy`), `rustfmt.toml` (`edition = "2024"`), workspace lints in
   `Cargo.toml`: `unsafe_code = "forbid"`, clippy `all = "warn"`, `pedantic` left off for
   now.
6. **Copy from the bootstrap package** (see `START_HERE.md` for the mapping):
   `docs/**` → `docs/`; everything in `scaffold/` → repo root (including dotfiles).
   Make `.claude/hooks/*.sh` executable. Immediately create the empty marker file
   `.claude/bootstrap-in-progress` (it is gitignored). While it exists, the bash guard
   hook allows commits on `main`; delete it as the last step of this PRD.
7. **Fill placeholders**: replace `<OWNER>` and `<COPYRIGHT HOLDER>` and `<YEAR>` in
   `LICENSE`, `README.md`, `Cargo.toml` and `.github/` files.
8. **Fixture notebook** at `tests/fixtures/notebook/`: recreate every note from
   `docs/reference/mockup.html` (the `TREE` object) as folders with `index.md`, bodies
   copied verbatim. Use order prefixes on the children of `cpu-architecture`
   (`010-caches`, `020-pipelining`, `030-memory-ordering`) so ordering is exercised; leave
   the rest unprefixed. Add `gyeoldaero.toml` with `name = "notebook"`. Add a short
   `tests/fixtures/README.md` saying what the fixture is for and that tests depend on it.
9. **Verify the harness**: run each hook script once by hand with a sample JSON payload
   (see `docs/dev-commands.md` → *Testing hooks*) and confirm the expected behavior. Fix
   anything that doesn't work in this environment (e.g. missing `jq`) and tell Ellie.
10. **Verify the build**: `cargo fmt --check`, `cargo clippy --workspace --all-targets --
    -D warnings`, `cargo test --workspace` all pass.
11. **Commit and publish**: one commit on `main`, message
    `chore: bootstrap gyeoldaero repository`. Create the public repo with
    `gh repo create <owner>/gyeoldaero --public --source . --push`, description:
    "find the grain of what you're learning — a terminal notebook where every note is a folder".
    Add topics: `notes`, `zettelkasten`, `neovim`, `yazi`, `rust`, `self-hosted`.
12. **Branch protection** (only if Ellie said yes): require PRs and the CI check on `main`.
    Do this **last**, after step 15, so the final bootstrap push isn't blocked.
13. **Wait for CI** (`gh run watch`) and fix until green.
14. Update `docs/prd/README.md` and this file's front matter: PRD-000 → `Shipped`
    (commit and push while the marker still exists).
15. Delete `.claude/bootstrap-in-progress`. From now on `main` only changes through PRs.

## Acceptance criteria

- [x] AC1 — `cargo fmt --check`, clippy with `-D warnings`, and `cargo test --workspace`
      pass locally. *(cli)*
- [x] AC2 — `gyeoldaero --version` prints the version; each subcommand exits 2 with its
      "not implemented" message. *(cli, by hand)*
- [x] AC3 — Each hook behaves as documented when fed a sample payload. *(by hand)*
- [x] AC4 — The fixture notebook mirrors the mockup tree: same notes, same bodies,
      prefixes as specified. *(by hand; PRD-001 adds an automated test)*
- [x] AC5 — The repo exists on GitHub, CI is green on `main`. *(gh)*
- [x] AC6 — No placeholder strings (`<OWNER>`, `<COPYRIGHT HOLDER>`, `<YEAR>`) remain.
      *(`grep -r`)*

## Plan

1. Ask Ellie the four questions; wait for answers.
2. Name check.
3. `git init`, workspace, crates, toolchain files.
4. Copy docs and scaffold; fill placeholders.
5. Fixture notebook.
6. Test hooks; run checks.
7. Commit, create repo, push, protection, watch CI.
8. Report to Ellie: what was created, anything that needed fixing, next step
   (review and accept PRD-001).
