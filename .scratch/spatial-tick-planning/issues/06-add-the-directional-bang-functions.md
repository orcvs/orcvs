# 06 — Add the Directional Bang Functions

**What to build:** Implement `*^`, `*v`, `*<`, and `*>`. Each is inert until Bang activation. Each
activation writes the matching Self-Banging Function into the two Cells immediately outside its own
two-Cell Span, in the direction its Portal offset declares.

**Stage:** 2 of 2. Issue 03 builds every mechanism this needs except the activation source. These
four differ from the Self-Banging Functions in exactly one declared property: they take ordinary Bang
activation where those take intrinsic. ADR 0029 calls that asymmetry "the whole reason both forms
exist" and refuses to collapse it.

**Blocked by:** 03 — Add the Self-Banging Functions.

**Status:** resolved

**Tags:** release/v1

**Sources of truth:** `CONTEXT.md` defines Directional Bang Function; ADR 0006 states the emission
geometry as Cell offsets; ADR 0029 states the activation asymmetry; ADRs 0004 and 0009 define
validated Portal effect bundles; `activation-representation/01` (Revised answer only) is the design
of record.

- [x] `*^`, `*v`, `*<`, and `*>` are rows in `define_functions!`. Each declares no operand, an effect
      kind, ordinary Bang activation, and its Portal offset: `*^` declares `(0, -1)`, `*v` declares
      `(0, 1)` — the default — `*<` declares `(-2, 0)`, and `*>` declares `(2, 0)`.
- [x] The horizontal offsets are two columns and not one. A Directional Bang Function emits outside
      its own Span, while a Self-Banging Function moves one Cell. A direction name would have hidden
      that difference; the offsets state it.
- [x] An active Directional Bang Function writes its matching Self-Banging Function at that offset.
      It reuses issue 03's write, including the precondition that the destination Cells are empty and
      inside the Grid.
- [x] A refused destination diagnoses and emits nothing. This is where the two Function groups
      differ in what a refusal costs: a Self-Banging Function replaces its own Span with Bang.
- [x] A generated Self-Banging Function first receives a turn from the next Source Snapshot. It does
      not move during the Tick that wrote it.
- [x] These four Functions answer an effect and not a value, and ADR 0025's single construction point
      refuses each by its declared kind.
- [x] A Bang result activates a zero-operand root at its west anchor. Inherited from issue 03, which
      could not drive it: the `column - 2` lookup in `bang_roots` went unreachable while every
      Function in `define_functions!` declared an operand, and issue 03 made it reachable without
      making it decide anything — its four roots are intrinsically active, and both callers of that
      arm act on a root only where the root is not. A Directional Bang Function is the first root
      that declares no operand *and* waits for activation, so it is the first Source that can tell
      the arm apart from silence. The `bang_roots` doc comment near `orcvs/src/source/tick.rs:534`
      names this ticket as the owner of the test that drives it.
- [x] Tick-by-Tick Source Grid tests cover emission in all four directions, Grid edges, and the
      refused destination. One test drives a full cycle: a Bang activates `*>`, `*>` writes `>>`, and
      `>>` moves on the following Tick.

## Comments

2026-09-11: Built. Every item is delivered. Five notes.

**`SourceEffect` grew the bundle rather than a second variant of `Interpretation`.** Issue 03 left
the type saying "Every producer today is a Self-Banging Function" and naming this ticket as the one
that would widen it. It is widened with `SourceBundle`, which is `Advance` or `Emit`: the first
plans two Portals — spaces over the producer's own Span, then the spelling at the displaced Span —
and the second plans one. The variant also fixes what a refusal costs, because the two are one fact:
a producer that was leaving its Cells reports in them, and a producer that is staying has nowhere of
its own to report.

**The spelling is read from the Function it emits.** `function_kind!` writes
`Function::SelfBangingNorth.spelling()` for both `*^` and `^^`, so `^^` has one home whichever row
names it, and a fifth pair is two table rows with no third place to keep in step. The two arms sit
beside each other in that macro on purpose: read as a pair they are ADR 0029's asymmetry as two
lines — the same spelling, the same kind of Portal, a different bundle and a different activation
column.

**The horizontal offsets are the difference, stated.** `*<` declares `(-2, 0)` where `<<` declares
`(-1, 0)`. `every_source_writing_function_declares_the_effect_its_spelling_names` writes all eight
out, so a row copied from its pair fails there rather than emitting one Cell inside its own Span.

**Three scheduling rules had to stop asking `source_write().is_some()`.** Whether the bundle
reserves the producer's own Span, whether ordering the producer after itself is a defect or the
design, and whether a refused destination delivers activation are all questions about the Span the
producer stands in, and a Directional Bang Function earns none of them. They ask `advances` now.

**Issue 03's west-anchor item is closed here.** `*^` is the first zero-operand root that waits for a
Bang, so the `column - 2` arm of `bang_roots` finally decides something. Deleting the arm leaves the
north and west halves of `an_active_directional_bang_function_emits_its_self_banging_function`
emitting nothing.

**Issue 03 stays open on one item, and it is not one of its own.** The west-anchor item is closed
here, but the ticket inherited an acceptance line from the file it renamed: ADR 0009's refusal of a
Portal for a Terminal Output Function is still raised only in the `#[cfg(test)]` `carry` helper.
Neither Function group gives input a way to name a destination for such a Function — both state
theirs from a declaration, and the `performs_terminal_output()` gate in `computations` still comes
first — so the refusal has the same one raiser it had before this effort started.
