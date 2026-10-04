# Cycle through editable note Cells

Status: needs-triage

## Goal

A performer writes a row of notes in Source, drives a selection through those
Cells with a Clock, and hears the selected note on each trigger. Editing one
step changes that step on its next read. Empty steps keep their place in the
cycle and can be used as rests. The Source File contains the complete pattern.

This is the interaction requested from the
[Learn Orca tracker example](https://metasyn.srht.site/learn-orca/sections.html#sequencing).
The target is editable musical data in the Grid. Playing the same pitches from
an expression assembled out of singleton Ranges demonstrates selection and
MIDI, but does not meet this goal.

## Current evidence

- Clock `~.`, Delay `~*`, Select `:?`, and Timed Play `!~` exist in
  `lang/src/atom.rs`. Select accepts a Sequence; it does not read an arbitrary
  Source region.
- ADR 0017 maps Track to Select and Query to Source Read. It reserves `@<` and
  `@>` without making them parseable Functions. The implementation table has
  neither. ADR 0049 settles Positions as column/row Numbers and describes
  future absolute Address Functions under `&`; issue 01 must reconcile that
  wording with the Source family before choosing spellings.
- Jump can read one aligned Language Unit at a fixed neighboring Portal. It
  does not select a clock-indexed step from a row or transport a Sequence.
- Operand Literals acquire a type from their declared operand position.
  Unclaimed note-like characters do not automatically form Notes. Reading a
  row cannot silently confer a new literal grammar on it.
- ADR 0007 gives Sequences no privileged literal encoding. Absence is not a
  Sequence member, and an absent result preserves old destination characters.
  Therefore a blank step needs an explicit contract for both storage and
  activation; treating a blank as absence and leaving MIDI active can replay
  the preceding note.
- ADRs 0032, 0034 and 0036 establish dependencies before execution and deliver
  current-Tick values. A read whose address or extent depends on a calculation
  must fit that model or explicitly revise it.

The local `examples/tracker.orcvs` experiment and its test exercise a constructed
Sequence, not this interaction. They are evidence for the reusable pieces, not
acceptance evidence for this effort, and are not prerequisites of these tickets.

## Decisions that remain open

1. Read a declared region into a Sequence and apply Select, or read one indexed
   step directly. Prefer evaluating the existing Source Read plus Select
   composition first, but accept it only if it preserves blank step positions
   and supports editing without duplicated note literals.
2. How a region gets its Note interpretation and Parser ownership, including
   whether a declaration makes the data row inert. A read alone must not become
   an independent parser that bypasses the Language Map.
3. Whether empty Cells mean a rest, and what representation preserves its slot.
   Admitting Absence into a Sequence would revise ADR 0007 and the current Atom
   contract; an indexed read or separate activation representation has other
   consequences. No choice is made by this breakdown.
4. How dynamic read geometry enters a dependency graph built before execution.
   Address spelling is settled; read length, orientation, stride, typing,
   timing and validation are not.

Issue 01 records the product and operation contract; issues 02 and 03 settle
the representation and scheduling contracts before implementation begins.
All tickets remain `needs-triage`; dependency links describe order, not an
assertion that their language decisions have been accepted.

## Delivery order

| Issue | Deliverable | Blocked by |
| --- | --- | --- |
| [01](issues/01-settle-the-tracker-contract.md) | Tracker interaction and operation contract | None |
| [02](issues/02-define-note-cell-and-rest-representation.md) | Typed data, ownership, and rest representation | 01 |
| [03](issues/03-settle-read-geometry-and-dependencies.md) | Read footprint and dependency semantics | 01, 02 |
| [04](issues/04-bind-and-read-editable-note-cells.md) | Parser and typed read implementation | 02, 03 |
| [05](issues/05-schedule-source-reads-in-the-current-tick.md) | Current-Tick read integration | 04 |
| [06](issues/06-deliver-notes-and-rests-without-stale-replay.md) | Trigger and rest behavior through Playback | 05 |
| [07](issues/07-verify-live-editing-in-the-console.md) | Real console editing and file workflows | 06 |
| [08](issues/08-deliver-the-cell-tracker-example.md) | Usable Source File, guide and acceptance evidence | 07 |

## Definition of done

- One editable two-Cell Note per occupied step; no singleton Range or duplicate
  endpoint needed for each note. The enclosing declaration is decided in 02.
- A clock cycles through a declared number of steps. Blank steps retain time;
  they do not shorten the cycle or re-trigger the preceding note.
- A complete live edit affects the next eligible Source Snapshot. Intermediate
  invalid edits have bounded, useful diagnostics and emit no spurious notes.
- Current-Tick writes to data are read in dependency order, even when the writer
  appears later in Grid order. Cycles and failed writers follow accepted rules.
- Open, Save, reopen and a new Playback run reproduce the written pattern and
  reset the clock as the existing Playback contract specifies.
- The example passes actual Source/Tick and Playback tests, a console input
  test, and an audible MIDI smoke test with device/channel/BPM recorded.

## Scope and risks

This effort adds language capability. It is active pre-release language design,
not public compatibility repair. An accepted decision must amend the affected
ADRs, `CONTEXT.md`, Function declarations, and Function reference consistently.
No new spelling or signature in these tickets is accepted syntax.

General Source Write, arbitrary Function-valued reads, vertical/multi-track
patterns, a piano roll, built-in audio, song arrangement and pattern chaining
are follow-ups unless the contract demonstrates that one is necessary. A moving
visual playhead is optional presentation; a visible selected index is sufficient.
No release tag is assigned by this planning work.

The main risks are Parser ownership, fixed dependency geometry, rest/activation
interaction, and Source-equivalent live edits. No dependency, unsafe code,
feature or performance change is prescribed. Keep reads bounded to their
declared footprint; any performance claim needs a reproducible benchmark.

## Verification

Every implementation ticket follows `AGENTS.md` and the skills triggered by its
actual changes. `lang` changes run scoped gates for `lang` and `orcvs`; `orcvs`
changes run them for `orcvs` and `console`; console presentation and UI tests
also use the egui skill. Use `PROPTEST_CASES=32` locally. Exercise additional
features or targets only where the repository contract requires them.

This planning change requires `node --test scripts/tests/roadmap.test.ts` and
`node scripts/roadmap.ts` with output discarded, plus diff review. The full
cross-platform, headless-browser, benchmark comparison and 256-case property
passes remain with CI. Track physical MIDI verification separately from tests
that assert commands through an in-memory output adapter.
