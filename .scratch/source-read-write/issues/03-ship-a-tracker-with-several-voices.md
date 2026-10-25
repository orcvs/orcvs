# 03 — Ship a tracker with several voices

Status: resolved
Blocked by: 02 — Read a pair in a direction; 07 — Accept the Read, Write and Copy families; cell-tracker/05 — Verify live editing in the console

**What to build:**

Port the Orca TRACKER patch: one Clock shared by several voices, each offsetting it with `.+` and triggering its own Timed Play. Each voice's `&v` sits at the head of its own column of a pattern grid and reads down it. `&v 00` is the Read's own operand, so each voice offsets its row index by one, for example Clock → `.+ 01` → `&v`, and its column begins on the row directly below the operand, beside the Read's Output Portal. Ship the Source File and a guide, loadable through the console's File > Open.

## Acceptance criteria

- [x] The shipped Source File has at least four voices, each with its `&v` at the head of its own column, with some deliberately empty steps.
- [x] The guide maps each Orca operator in the patch to its Orcvs Function, explains that `&v` counts rows from its own operand and why each voice adds `01`, and gives the BPM, MIDI channels, velocity and length, with exact columns for spatial alignment.
- [x] A test loads the exact shipped Source File and checks two complete loops of every voice through Source/Tick and Playback, including empty steps and Note Off timing, asserting each Tick's diagnostics exactly.
- [x] The example works through the console editing and file workflows verified by `cell-tracker/05`: edit a step, clear and restore it, Save and reopen.
- ~~Record an audible MIDI smoke test with device, channels and BPM; do not mark delivery complete on automated evidence alone.~~ Moved to `source-read-write/14`; delivery is complete only when it is.

## Comments

### 2026-10-17 — Resolved

Shipped `examples/tracker.orcvs` and `examples/tracker.md`. One Clock `~.0208` at (0, 0) writes the
step at (0, 1). Each of four voices, 20 columns apart, reads it with `.+01&$0001` and writes the sum
into the `n` slot of a Read `&v` nested in its Timed Play's note slot, `!~0v64&v  04`; a Delay
`~*0102` bangs each Play. Nesting the Read keeps the Play's length operand off the step column, which
starts on the row below the `n` operand beside the Read's Output Portal. Orca has no absolute read;
the guide says `&$` is how the four voices share one Clock.

`orcvs/src/playback/tests/shipped_trackers.rs` runs the exact file for 32 Ticks through a
hand-driven Playback run and asserts every voice's delivery, Note Off timing and every Tick's
diagnostics exactly; `a_half_typed_step_silences_only_its_own_voice` covers a partial step.
`console::kittest_tests::live_editing::the_four_voice_tracker_edits_saves_and_reopens_a_voices_steps`
edits, clears, restores, saves and reopens a step through the console.

The last criterion needs a person with a MIDI device and moves to `source-read-write/14`, as `cell-tracker/13` did
for the single-voice example.

### 2026-10-18 — Replaced by the Learn Orca tracker

Toby asked for the tracker to work as Learn Orca's does. `examples/tracker.orcvs` is now that port:
each of eight voices has its own Clock `~.0110` nested in an Add whose offset points its nested
`&v` Read at the shared sixteen-row pattern, and a Delay `~*0101` bangs its Timed Play every Tick.
The voices stand in Orca's two staircases of four. The tests above are rewritten for it as
`the_orca_tracker_plays_two_bars_of_its_one_note`,
`every_orca_tracker_voice_plays_its_own_column_on_its_own_channel` and
`console::kittest_tests::live_editing::the_orca_tracker_edits_saves_and_reopens_a_voices_steps`.
