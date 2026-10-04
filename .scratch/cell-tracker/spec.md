# Cycle through editable note Cells

Status: needs-triage

## Goal

A performer writes a row of notes in Source, drives a selection through those
Cells with a Clock, and hears the selected note on each trigger. Editing one
step changes that step on its next read. Empty steps keep their place in the
cycle and are rests. The Source File contains the complete pattern.

This is the interaction requested from the
[Learn Orca tracker example](https://metasyn.srht.site/learn-orca/sections.html#sequencing).
The target is editable musical data in the Grid. Playing the same pitches from
an expression assembled out of singleton Ranges demonstrates selection and
MIDI, but does not meet this goal.

## Decisions

The language design this effort needed is accepted in four ADRs:

- [ADR 0061](../../docs/adr/0061-a-nested-function-returns-to-the-slot-it-occupies.md):
  a Function's operands are inline input portals, claimed greedily. A nested
  Function hands its two-Cell encoding (its Return) to the portal it occupies,
  decoded by that portal's type, and still writes through its own Output Portal.
- [ADR 0062](../../docs/adr/0062-a-blank-operand-has-no-effect.md): a blank
  operand makes its Function do nothing, without a diagnostic.
- [ADR 0063](../../docs/adr/0063-a-list-is-cells-in-a-claim-not-a-value.md):
  the Sequence value is retired. A List is a horizontal series of two-Cell items
  in one Function's claim with a literal count. Track `@t index count` answers
  item `index % count` every Tick, and a blank item is a rest.
- [ADR 0064](../../docs/adr/0064-a-function-spelling-starts-with-punctuation.md):
  the spelling rule. Its renaming sweep is a separate effort, after this one.

The tracker, in current spellings:

```
~*0101  ~.0108
**    @t0208C4D4E4  G4C5  E4
!~0064E404
```

Delay bangs Timed Play every Tick. Clock writes `00`–`07` into Track's index
operand. Track copies the selected item south into Timed Play's note operand. A
blank item reaches Timed Play as a blank operand, so nothing plays and the
previous note does not repeat.

These decisions settle the questions the first breakdown of this effort left
open: an indexed read, not a region read into a Sequence; items owned by the
Parser as part of Track's claim; rests as blank items; and a footprint fixed by
the Source text, so the dependency graph is built before execution.

## Current evidence

- A nested Function writes no Portal (`PortalAccess::resolve` in
  `orcvs/src/source/portal.rs`), so a nested Increment never advances. ADR 0034
  said a Function "can have both" a nested result and a spatial output; the code
  never did.
- A nested effect Function is refused only during a Tick (`NestedEffectFunction`).
- A blank operand inside a claim is invalid and diagnoses.
- Clock `~.`, Delay `~*` and Timed Play `!~` exist. Select `:?` reads a nested
  Sequence; nothing reads Source Cells by index.
- The local `examples/tracker.orcvs` experiment exercises a constructed Sequence,
  not this interaction. It is not acceptance evidence.

## Delivery order

| Issue | Deliverable | Blocked by |
| --- | --- | --- |
| [01](issues/01-a-nested-function-returns-and-writes.md) | Return, and the nested Output Portal write | None |
| [02](issues/02-a-blank-operand-has-no-effect.md) | Blank operands do nothing | None |
| [03](issues/03-retire-the-sequence-value.md) | Sequence value and its Functions removed | 01 |
| [04](issues/04-track-reads-an-item-from-its-list.md) | Track and Lists | 01, 02 |
| [05](issues/05-verify-live-editing-in-the-console.md) | Real console editing and file workflows | 04 |
| [06](issues/06-deliver-the-cell-tracker-example.md) | Usable Source File, guide and acceptance evidence | 03, 05 |

## Definition of done

- One editable two-Cell Note per occupied step, in Track's own List.
- A Clock cycles through a declared number of steps. Blank steps keep their
  time, and they neither shorten the cycle nor re-trigger the preceding note.
- A complete live edit affects the next Tick whose Source Snapshot has not been
  taken. Intermediate invalid edits have bounded, useful diagnostics and emit
  no spurious notes.
- Open, Save, reopen and a new Playback run reproduce the written pattern.
- The example passes Source/Tick and Playback tests, a console input test, and
  an audible MIDI smoke test with device, channel and BPM recorded.

## Scope and risks

Active pre-release language design, not public compatibility repair. Each
ticket amends `CONTEXT.md`, the Function table and the Function reference in
step with the behaviour it implements.

Follow-ups, not in this effort: the spelling sweep (ADR 0064) and the console
presentation aids that go with it (family colour, hover signatures, typing a
name to get its spelling, and a check of the console font's ligatures);
addressed and vertical Lists; any other List Function.

Risks: Output Portal writes from nested Functions change layouts that stacked
Expressions under nested ones; removing the Sequence value touches the
evaluator, scheduler, Reservation and Function table at once. No dependency,
unsafe code or feature change is prescribed. Any performance claim needs a
reproducible benchmark.

## Verification

Each implementation ticket follows `AGENTS.md` and the skills its changes
trigger: `lang` changes run the scoped gates for `lang` and `orcvs`, `orcvs`
changes for `orcvs` and `console`, and console presentation also uses the egui
skill. Use `PROPTEST_CASES=32` locally. This planning change requires
`node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts` with
output discarded, plus diff review.
