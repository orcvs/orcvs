# 05 — Verify live editing in the console

Status: resolved
Blocked by: 04, 14

**What to build:**

Use ordinary console input and file workflows to load a tracker, edit a pair
Track selects, clear it and restore it while Playback runs.

## Acceptance criteria

- [x] Typing or pasting a Note into a pair changes the next eligible Tick; a malformed intermediate edit recovers ~~without spurious notes~~ with bounded useful diagnostics, replaying the previous Note for the half-typed Tick as `14` decided.
- [x] Clearing a pair leaves later pairs in place; copy and paste retain empty pairs.
- [x] The pairs east of Track paint as ordinary Source (ADR 0067).
- [x] Open, Save, reopen and a new Playback run reproduce the written pattern and reset the Clock according to the Playback contract.
- [x] Tests drive real console input and existing file workflows under the egui skill; no dedicated tracker editor or extra adapter is introduced.

## Comments

Rewritten for ADR 0067 and ADR 0069, which replaced the List claim, its Items, the Blank Answer and the pending state. Track's pairs are ordinary Source, and an empty pair is copied as a Copy copies empty Cells (ADR 0069).

The console presents no Source or Tick diagnostic yet (ADR 0069), so the presentation criterion needs that path first. Building it settles whether a standalone Operand Literal is diagnosed: the Language Map reports every pair east of Track as an unknown Function, two diagnostics per Cell, because a refused Function advances one Cell. Not diagnosing a Unit that spells a Number or a Note is a change to `Parser::analyze` and the Language Map's row walk, and needs an ADR answering ADR 0067's open question; its cost is that operands left behind by a deleted or replaced Function are no longer diagnosed.

### Decided 2026-10-13 by Toby

Criterion 3 is descoped to painting. The console shows no Source or Tick diagnostic messages
(`docs/adr/0069-an-unwritten-operand-is-invalid-input.md:19`), and no ticket owns presenting them,
so presenting a malformed selected pair's Tick diagnostic is not part of this ticket. Status stays
`ready-for-agent`.

### 2026-10-16

Presenting Source and Tick diagnostics is now owned by `.scratch/source-diagnostics/`, which starts with ADR 0067's open question on standalone Operand Literals. This ticket stays scoped to painting.

### 2026-10-17 — Resolved

`console/src/console/kittest_tests/live_editing.rs` opens a copy of `examples/cell-tracker.orcvs`
(ticket 06) through `open_picked`, edits it with arrow keys, typed text, Backspace, Shift-arrow
Region, Copy and Paste events and `File > Save`, and runs each Tick through
`SourceCommander::execute`, the call Playback makes. Playback stays stopped so the test is the only
thing that runs a Tick; the kittest module forbids waiting on Playback.

- Criterion 1: a typed Note plays on the next Tick that selects it, and a half-typed pair is
  diagnosed once (`&t has partial or invalid input`) and plays its Note the Tick after it is
  finished. That half-typed Tick is not silent: Track is invalid and writes nothing, so the note
  slot keeps the previous step's Note and the Bang replays it, as ADR 0069 decides for a value
  Function that is invalid. The test pins that behaviour and `examples/cell-tracker.md` explains it.
  Whether it should be silent is a language decision, split to `14`.
- Criterion 4: a saved, edited tracker reopened in a new console plays the edited pattern from
  Tick 0. That Playback begins each run at Tick 0 is pinned in `orcvs/src/playback.rs`
  (`beginning_a_run_publishes_tick_zero_playing_and_a_run_origin`,
  `playback_begins_at_the_first_tick_and_advances_one_per_executed_tick`).
- A rest is diagnosed once: by the Tick that empties the note slot, then by the Language Map while
  it stays empty.

### 2026-10-17 — Kept open on `14`

Every criterion but the "without spurious notes" clause of the first is delivered and tested. The
ticket stays open, blocked by `14`, until that decision either makes the half-typed Tick silent or
rewords the criterion to the replay ADR 0069 gives today. `06` and `source-read-write/03` rest only
on the delivered parts: the console tests their own criteria name.

### 2026-10-17 — Resolved after `14`

Toby decided in `14` to keep ADR 0069's replay, so the first criterion is reworded to it and the
ticket is resolved.
