# Test fixtures

`notebook/` is a sample gyeoldaero notebook. It recreates every note from the approved mockup
(`docs/reference/mockup.html`, the `TREE` object) as folders with `index.md`, with bodies
copied verbatim. The children of `cpu-architecture` carry order prefixes (`010-`, `020-`,
`030-`) so sibling ordering is exercised; everything else is unprefixed.

Tests depend on this fixture (model tests, render snapshots, e2e). Changing it changes
their expected output, so do it on purpose and review the resulting snapshot diffs.

`broken/` is a small notebook with known problems for `gyeoldaero check` (PRD-001 AC9):
duplicate slugs (`010-dup`, `dup`), a broken link (`links`), malformed frontmatter
(`bad-frontmatter`), a folder without `index.md` (`no-index`) and two stray `.md` files
(`stray`). Expected: 3 errors, 3 warnings. Change it together with
`crates/gyeoldaero-cli/tests/check.rs`.
