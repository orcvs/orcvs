# 01 — Order effects by Language Map Position

**What to build:** Replace expression-only planning with ADR 0020's one row-major pass over all
actionable Language Units and Expression roots.

**Blocked by:** language-map/03 — Move Source consumers behind the Language Map.

**Status:** resolved

**Tags:** release/v1

- [x] Turns are ordered by row-major anchor Position from one Source Snapshot.
- [x] Each producer emits effects in a stable local order.
- [x] Writes, activations, locks, diagnostics, and terminal commands share the same ordering model.
- [x] Later effects win Cell conflicts independently after complete-write validation.
- [x] Planned writes gain no same-Tick turn and generated Functions wait for the next Snapshot.
- [x] A root whose turn passed is never revisited.
- [x] Existing Tick Plan atomicity and Source/Playback seam remain intact.

## Downstream integration

- [ ] The `order_by_anchor` guard cannot be tested end to end from this ticket, and issue 02 now
      carries the checkbox that pins it. See "What did not get pinned" below.
- [ ] Cell-conflict resolution over genuinely overlapping bundles is exercised by
      `issues/03-add-directional-bang-movement.md` and `issues/04-add-directional-jump-chains.md`.
- [ ] The activation and lock Effect variants arrive with `issues/02-...` and `issues/05-...`.

## Comments

Resolved as the ordering framework. `orcvs/src/source/tick.rs` holds ADR 0020's model: `turns`
collects the producers one Source Snapshot grants and orders them by row-major anchor, each producer
emits an ordered sequence of `Effect`s, and `resolve` folds those effects in producer-then-emission
order into the Tick Plan. `plan_tick` is now that pass and nothing else. `Turn`, `Producer`, and
`Effect` are recorded in `CONTEXT.md`, since ADR 0020 used the words and the glossary did not carry
them.

The Expression root is the only producer with a turn. Issues 02 to 05 each add a `Producer` variant,
an `emit` arm, and where needed an `Effect` variant with its `resolve` arm; none adds a second
ordering pass. No `Activate` or `Lock` variant exists yet, because no producer emits one and a dead
variant would claim a seam it does not hold.

### What did not get pinned

Two acceptance boxes are satisfied by construction and at the seam, but cannot be driven from Source
text yet. Both are recorded here rather than left to be discovered.

The row-major guard is not load-bearing today. `LanguageMap::expressions()` already yields Expression
roots in row-major order — extents are collected in index order and never overlap — so deleting the
`order_by_anchor` call from `turns` leaves all 164 tests green. That was confirmed by mutation, not
assumed. The guard exists for the second producer kind rather than the first, so issue 02, which adds
a producer over the Language Unit partition, now carries the checkbox that pins it end to end.

Cell conflicts are inexpressible in Source today. Every result is one two-Cell Atom written below its
root, distinct roots in a row sit at least three columns apart, and roots in different rows target
different rows, so no two producers can target overlapping Cells. Conflict resolution is therefore
pinned as a unit test against `resolve`, and issues 03 and 04 bring the overlapping bundles that make
it reachable from a Tick-by-Tick Source Grid.

Value-producing roots still evaluate on every Tick without activation. No box here asks to gate them
and ADR 0020 does not require it. ADR 0020's first paragraph, ordering scheduled expiries ahead of a
Tick's new Play Commands, is the Playback Engine's obligation; `orcvs/src/playback.rs` has no
scheduled-expiry concept yet, and this ticket's only playback obligation — that the seam stays
intact — holds.
