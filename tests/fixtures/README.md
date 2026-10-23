# Test fixtures

`notebook/` is a sample gyeoldaero notebook. It recreates every note from the approved mockup
(`docs/reference/mockup.html`, the `TREE` object) as folders with `index.md`, with bodies
copied verbatim. The children of `cpu-architecture` carry order prefixes (`010-`, `020-`,
`030-`) so sibling ordering is exercised; everything else is unprefixed.

Tests depend on this fixture (model tests, render snapshots, e2e). Changing it changes
their expected output, so do it on purpose and review the resulting snapshot diffs.
