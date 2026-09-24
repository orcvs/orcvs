# 02 — File > New opens an empty default Grid

**What to build:** `File > New` replaces the environment with a fresh, empty Source on the current
default Grid: the Cursor home, playback stopped, the Source View back at rest. A console that has
been running for hours, holding a stored Source on an older Grid, reaches the same state a first
start would.

**Blocked by:** 01 — Open a Source into the running console.

**Status:** resolved

**Tags:** release/v1

- [x] `File > New` sits in the File menu beside `Load Function reference` and opens an empty Source
      on the current default Grid.
- [x] The Grid is the default the build carries, not the one a stored Source was saved with. A
      console restored onto an older, smaller Grid reaches the current default through this and
      nothing else.
- [x] Every Cell is empty, the Cursor is at the Grid's origin, and the Source View's Pan and Zoom
      are at rest.
- [x] Playback is stopped. If a Tick was playing when New was chosen, it stops, and no engine from
      the replaced environment outlives it.
- [x] Bpm returns to its default, as it already does for `Load Function reference`. Recorded here
      as a decision, not an accident: preserving Bpm across an Open is a later change to `01`'s
      operation if it is wanted.
- [x] Theme, Cursor effects, Source colours and Diagnostics visibility are untouched.
- [x] With the `persistence` feature on, the new empty Source is what the next ordinary save
      stores, and a restart after a New opens the default Grid rather than the Source that was
      stored before it. Storage is not wiped eagerly; the ordinary save path carries it.
- [x] A build without `persistence` behaves the same, minus the storing.
- [x] Tests drive `File > New` through the console — not the operation alone — and assert the Grid,
      the emptiness, the Cursor, the stopped playback and the untouched settings.
- [x] The scoped gates for `console` pass, and `mise run test_persistence` covers the feature-off
      arm.

## Comments

This is the escape hatch `source-view/13` leaves implicit. That ticket decided a stored Source
keeps the Grid it was stored with, which is why a console that has stored anything never sees a
later default. See `spec.md` for why a silent migration is not the answer.

ADR 0054 fixes every Grid at 256 by 256, so "the current default Grid" is that one shape and the older-Grid criterion is met by `menu-structure/01`. New still opens an empty Source and still resets the environment as listed.

`File → New` is `Console::new_source`, one line over `Console::open` with
`persistence::default_source()` — the same empty 256 by 256 Source a console that restores
nothing starts on, now `pub(crate)` so the two cannot name different defaults.

Tests: `kittest_tests::file_new_opens_an_empty_source_on_the_256_by_256_grid` drives the menu item
after writing a Cell, moving the Cursor, zooming, raising Bpm, starting Playback and moving the
Theme and Cursor effects off their defaults, and asserts the 256 by 256 Grid, every Cell empty, the
Cursor at the origin, the Source View at rest, Playback stopped, Bpm back at a fresh console's,
the settings untouched, and the replaced Playback Engine's observation watch closing — its task
holds the only sender, so the watch closing is that engine having ended.
`kittest_tests::file_new_is_what_the_next_save_stores_and_a_restart_opens` (persistence only)
restores a stored Source holding written content, chooses New, checks storage still holds the old
revision, saves, and restarts onto an empty Source. The feature-off arm is the first test alone; it
has no storing to assert.

`mise run test_persistence` is the merge tier's whole pass and is left to CI; the per-change arm
CLAUDE.md names, `cargo nextest run --workspace --tests --no-default-features --locked`, is the
one run locally.
