# 02: Stamp the parse into a Cell-indexed array

**What to build:** The Language Map holds one entry per Cell, parallel to the Source, saying what
that Cell is. It is filled during the same walk that establishes Language Units, Spans and
diagnostics, so the rules are stated once rather than twice.

The Parser stops reporting a position for a consumer to re-base. It writes what it found at the Cell
it read. A Cell index is a byte offset into the Source, because the Source maps literally onto the
Grid, so there is no conversion step and no anchor arithmetic.

Four states, from a prototype, because the set of states is the decision:

```rust
enum CellRole {
    Empty,                                     // no character
    Occupied,                                  // a character, claimed by no Expression
    Structure { expression: ExpressionId },    // a Function, Bang or Activation spelling
    Slot { root: RootId, slot: SlotOrdinal },  // an attachment point of a root
}
```

Three of those need saying, because earlier drafts got each wrong:

**`Occupied` exists because a character can belong to no Expression.** Under partition-by-parse a
bare `0102` is invalid Source — the Parser tries a Function spelling, fails, and rewinds one Cell —
so its Cells are claimed by nothing. Stamping them `Empty` would make them indistinguishable from
blank Source; stamping them `Structure` would make a write near them a structural write. They are
data at rest, and the only thing that currently turns on their being recognisable is presentation.

**`Structure` is keyed by Expression, not by root.** A standalone Bang or Activation Expression
holds no Function candidate and therefore has no root, so there is no root identifier to write. It
still occupies Cells, and those Cells must be distinguishable from `Occupied`.

**Bang display must be representable, because a Tick clears it before any root takes a turn.** A
comparison rewrites its own `**` over the previous Tick's `**` every Tick, and that write must be
planned without a diagnostic. Today the scheduler subtracts the Bang Spans the plan clears before it
computes anything spatial. Either the array is stamped as the Tick leaves it — cleared Bang display
reading `Empty` — or the clearing step survives explicitly. State which; do not leave it to the
consumer.

**Widths are part of the decision.** `ExpressionId` and `RootId` are at most sixteen bits and
`SlotOrdinal` at most eight, which keeps an entry to four bytes and the array to roughly four
kilobytes at the default Grid. At pointer width the same array is twenty-four kilobytes. The
scheduler's analogous field is a bare `usize` today, so this is a narrowing and not an assumption.

**The ordinal is enough, with no start marker.** Reading two adjacent entries: both naming the same
root and slot means those two Cells are that slot, because an operand spelling is two Cells wide, so
the write covers it exactly. Naming different slots means the write straddles two operands. A
one-Cell operand can never appear twice in a row, so it falls to the straddle case correctly. This
is the property that lets the whole classification be two array reads.

The Map already holds a Cell-indexed array for Glyph classification. This is the semantic one beside
it; whether they later merge is a separate question.

**Blocked by:** 01

**Status:** wontfix

**Tags:** release/v1

## Pre-delivery audit at `593613c` — 2026-09-08

The implementation statements in this audit describe commit `593613c`, before
live-typed-execution delivery. For the completed transferred work and current
implementation, see [delivery evidence](../../live-typed-execution/evidence.md).

Not implemented: `LanguageMap` contains units, expressions, Glyphs and lexical
diagnostics, but no semantic CellRole array. `64291cc` created the seven planning tickets;
it did not implement this array. ADR 0034 deliberately leaves storage representation open.
The mandatory four-byte entry, identifier widths, CellRole states and same-walk array
stamping are abandoned. Parser-owned positions, distinction between claimed and unclaimed
Cells, row-local rebuild equivalence, and correct cleared Bang display remain requirements
of [live-typed-execution](../../live-typed-execution/spec.md), implemented through the chosen
representation rather than this mandatory array. No unchecked item below is a prerequisite.

The original checklist below is retained as historical scope; this audit records
its disposition at `593613c`. The linked delivery evidence records subsequent delivery.

- [ ] Every Cell an Expression occupies is stamped, and no Cell outside one is `Structure` or `Slot`.
- [ ] The array and each Expression's Span agree about which Cells that Expression occupies.
- [ ] A character belonging to no Expression is distinguishable from blank Source and from a
      structural Cell.
- [ ] A standalone Bang or Activation Expression's Cells are stamped without requiring a root.
- [ ] A structural Cell is distinguishable from an operand slot Cell, and a slot Cell names which
      slot of which root it is.
- [ ] Bang display cleared before the first turn is represented, and the case where a producer
      rewrites its own display over the previous Tick's is stated and tested.
- [ ] The array is established in the existing row walk, not in a second pass.
- [ ] A refused Function spelling occupies exactly one Cell.
- [ ] An operand slot the Source ran out before is stamped or not according to the rule named at one
      place, and that place is named.
- [ ] Entry widths are declared and an entry is four bytes.
- [ ] The rebuild-equivalence property holds for the array as well as for the Map's other products.
- [ ] Incremental reparse rewrites only the affected rows' entries.
