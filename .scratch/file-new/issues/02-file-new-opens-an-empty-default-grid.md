# 02 — File > New opens an empty default Grid

**What to build:** `File > New` replaces the environment with a fresh, empty Source on the current
default Grid: the Cursor home, playback stopped, the Source View back at rest. A console that has
been running for hours, holding a stored Source on an older Grid, reaches the same state a first
start would.

**Blocked by:** 01 — Open a Source into the running console.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] `File > New` sits in the File menu beside `Load Function reference` and opens an empty Source
      on the current default Grid.
- [ ] The Grid is the default the build carries, not the one a stored Source was saved with. A
      console restored onto an older, smaller Grid reaches the current default through this and
      nothing else.
- [ ] Every Cell is empty, the Cursor is at the Grid's origin, and the Source View's Pan and Zoom
      are at rest.
- [ ] Playback is stopped. If a Tick was playing when New was chosen, it stops, and no engine from
      the replaced environment outlives it.
- [ ] Bpm returns to its default, as it already does for `Load Function reference`. Recorded here
      as a decision, not an accident: preserving Bpm across an Open is a later change to `01`'s
      operation if it is wanted.
- [ ] Theme, Cursor effects, Source colours and Diagnostics visibility are untouched.
- [ ] With the `persistence` feature on, the new empty Source is what the next ordinary save
      stores, and a restart after a New opens the default Grid rather than the Source that was
      stored before it. Storage is not wiped eagerly; the ordinary save path carries it.
- [ ] A build without `persistence` behaves the same, minus the storing.
- [ ] Tests drive `File > New` through the console — not the operation alone — and assert the Grid,
      the emptiness, the Cursor, the stopped playback and the untouched settings.
- [ ] The scoped gates for `console` pass, and `mise run test_persistence` covers the feature-off
      arm.

## Comments

This is the escape hatch `source-view/13` leaves implicit. That ticket decided a stored Source
keeps the Grid it was stored with, which is why a console that has stored anything never sees a
later default. See `spec.md` for why a silent migration is not the answer.

ADR 0054 fixes every Grid at 256 by 256, so "the current default Grid" is that one shape and the older-Grid criterion is met by `menu-structure/01`. New still opens an empty Source and still resets the environment as listed.
