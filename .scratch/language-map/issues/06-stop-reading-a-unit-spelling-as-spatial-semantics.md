# 06 — Stop reading a Language Unit spelling as spatial semantics

**What to build:** Separate the two stages ADR 0018 and ADR 0024 specify, so a `**` that an
Expression consumes as an Atom is no longer treated as a Source-resident Bang by the spatial
behaviour that reads the Language Unit partition.

**Blocked by:** None — can start immediately.

**Status:** ready-for-human

**Tags:** release/v1

- [ ] A `**` an Expression consumes as an Atom activates no root.
- [ ] A `**` an Expression consumes as an Atom plans no expiry and rewrites no Source.
- [ ] A standalone `**` keeps its activation geometry and its one-Tick expiry unchanged.
- [ ] The rule that separates the two is stated once, at the stage ADR 0024 puts it in, rather than
      at each consumer of `LanguageMap::units`.
- [ ] `spatial-tick-planning/03` inherits the separated stages rather than the fused ones.

## The defect

`LanguageUnitKind::Bang` records that two Cells are spelled `**`. ADR 0024 states the limit of that
in one sentence: "A unit's kind records the spelling and nothing else." Two consumers read it as
semantics instead:

- `LanguageMap::is_root_active` (`orcvs/src/source/language_map.rs:234`) filters `units()` for
  `LanguageUnitKind::Bang` and treats every match as a Source-resident Bang with activation
  geometry.
- `tick::turns` (`orcvs/src/source/tick.rs`, added by `spatial-tick-planning/02`) filters the same
  partition and grants every match a producer turn that plans an expiry.

Neither asks whether the `**` is a spatial pulse or an Atom inside an Expression. ADR 0024 puts
that question at the parse stage — "The parse resolves what a Function does with the operands it is
given" — and the partition stage cannot answer it.

## Measured evidence

Row 0 holds `!>00**C4` from column 0, so its `**` is anchored at column 4 and occupies a Raw Play
operand slot. Row 1 holds an unrelated root anchored at column 4, which is that `**`'s south
activation anchor `(x, y+1)`. Grid is 16 wide.

```text
                     on 2f3ed99                 on 84af8bf
 r0  before Tick 0   !>00**C4                   !>00**C4
 r1  before Tick 0       !>007FC4                   !>007FC4

                     plays = 1, diags = 0       plays = 1, diags = 0

 r0  after Tick 0    !>00**C4                   !>00  C4
 r1  after Tick 0        !>007FC4                   !>007FC4
```

An operand fires a Raw Play two rows away that it has no relationship to. The activation half
predates `spatial-tick-planning/02`; that ticket added the expiry half, which additionally destroys
the Expression the operand was spelled in and writes no diagnostic while doing it. One root cause,
two symptoms.

## Why the fused stages made it reachable

ADR 0018 partitions "each row left to right", and then "Expression construction operates on this
partition". ADR 0024 says the same in order: a row-local partition of Language Units, "and then
parses each Expression extent". The partition is stage one and the extent is stage two.

`walk_row` (`orcvs/src/source/language_map.rs:473`) runs both in one loop. It computes a boundary
from a space, a `##`, or the row edge; it partitions Language Units only within that boundary; and
it pushes one Span for the whole group, which becomes exactly one `ExpressionEntry`. So the same
loop decides tokenisation and Expression membership, the dependency between the two stages is
inverted, and there is one list where the ADRs describe a stage that reads another.

That fusion has no name in `CONTEXT.md` or in any ADR. The code and its tests call it a "run"; the
glossary uses the word only as ordinary English inside the **Span** and **Expression** entries, and
defines no such term. Anything asking the partition "where are the Bangs?" therefore gets every
`**` in the Source, with no way to ask the question ADR 0024 reserves for the parse.

## The gap underneath

No ADR states what bounds an Expression extent. `CONTEXT.md`'s **Expression** entry is the only
statement — "a contiguous horizontal run of occupied Cells in one Source row" — and it is prose
using the undefined word. The space rule and the `##` rule live only in `RunBoundary`.

Two questions follow, and they are ADR questions rather than implementation ones:

- What bounds an Expression extent, stated where the other partition rules are stated?
- Is an adjacent standalone `**` its own Expression, or a member of the one it touches? Today
  adjacency means membership, which is why ADR 0006's west `(x-2, y)` and east `(x+2, y)` root
  anchors are unreachable from any Source text. That unreachability is a consequence of the fusion,
  not a decision anybody made.

`spatial-tick-planning/02` recorded both as deferred language design. That framing was wrong and is
corrected here: ADR 0024 already answers the spelling-versus-semantics half, so it is a defect. Only
the extent rule is genuinely open.

## Comments

Found while reviewing `spatial-tick-planning/02` after it was implemented and committed. The probes
above were run on `2f3ed99` and on `84af8bf` and are reproducible from
`SourceUnderTest::new(Grid::new(16, 3))` with the two writes shown.

`lang/bang-expiry-scope-prototype.html` and `spatial-tick-planning/02`'s own Comments both present
the operand case as an open design question with three options. They are wrong on that point and
should be corrected to name this issue instead.

`spatial-tick-planning/03` should not begin before this resolves. Its Portal effect bundles make a
Sequence result encoding a `**` writable into Source, and the same partition read would consume it
on the following Tick.
