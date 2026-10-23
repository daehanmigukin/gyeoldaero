---
name: snapshot-review
description: Review changed insta snapshot files for gyeoldaero's rendered HTML one by one before accepting them. Use whenever cargo insta reports new or changed snapshots.
---

# snapshot-review

Snapshots are the renderer's contract. A bad snapshot accepted once becomes the "correct"
output forever, so every change gets read.

1. `cargo insta test --workspace` to collect pending snapshots.
2. List pending files (`cargo insta pending-snapshots` or `find . -name '*.snap.new'`).
3. For each one, show the diff and decide:
   - **Expected**: the change follows from the current PRD's requirements. Name the
     requirement or acceptance criterion it serves.
   - **Unexpected**: anything else (changed classes the JS depends on, lost escaping,
     reordered children, changed asset hashes without asset changes). Do not accept; find
     the cause.
4. Accept only expected ones individually: `cargo insta accept --snapshot <path>`
   (or `cargo insta review` and step through). Never `cargo insta accept` for everything
   in one go.
5. In the PR description, list accepted snapshot changes with a one-line reason each.

Check in every snapshot diff that user content is still escaped and that markup classes
match `docs/renderer-spec.md` §3.
