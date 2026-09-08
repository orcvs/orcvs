# 06 — Stop reading a Language Unit spelling as spatial semantics

**What to build:** Make parsing authoritative for Bang meaning: return a Bang Atom when `**` is
valid in context, retaining its Position and Span for Tick execution. Current-result activation and prior-display cleanup follow ADR 0032,
never infer Bang validity independently from the spelling partition. A `**` rejected in
a typed Function operand position remains invalid Source with a syntax diagnostic.

**Blocked by:** tick-execution-order/03.

**Status:** resolved

**Tags:** release/v1

- [x] A `**` rejected in a typed Function operand position activates no root.
- [x] A `**` rejected in a typed Function operand position plans no expiry and rewrites no Source.
- [x] The invalid Expression retains its Source syntax diagnostic; zero Tick Plan diagnostics is not
      evidence that the Source parsed successfully.
- [x] Manual standalone `**` is a no-op; only current-Tick Bang output activates and old display does not replay, following ADR 0032.
- [x] Parsing returns valid Bang Atoms with their Source Positions and Spans available to execution.
- [x] Activation and expiry use parsed Bangs; neither reinterprets `LanguageMap::units` spellings.
- [x] No separate spatial analysis pass reclassifies spellings or decides Bang validity.
- [x] `spatial-tick-planning/03` inherits the parser's authoritative results.

## The defect

`LanguageUnitKind::Bang` records that two Cells are spelled `**`. ADR 0024 states the limit of that
in one sentence: "A unit's kind records the spelling and nothing else." Two consumers read it as
semantics instead:

- `LanguageMap::is_root_active` (`orcvs/src/source/language_map.rs:234`) filters `units()` for
  `LanguageUnitKind::Bang` and treats every match as a Source-resident Bang with activation
  geometry.
- `tick::turns` (`orcvs/src/source/tick.rs`, added by `spatial-tick-planning/02`) filters the same
  partition and grants every match a producer turn that plans an expiry.

Neither asks whether the `**` is a spatial pulse or rejected syntax in a typed operand position.
ADR 0024 puts that question at the parse stage — "The parse resolves what a Function does with the operands it is
given" — and the partition stage cannot answer it.

## Measured evidence

Row 0 holds `!>00**C4` from column 0, so its `**` is anchored at column 4 and occupies a Raw Play
operand slot. Row 1 holds an unrelated root anchored at column 4, which is that `**`'s south
activation anchor `(x, y+1)`. Grid is 16 wide.

```text
                     on 2f3ed99                 on 84af8bf
 r0  before Tick 0   !>00**C4                   !>00**C4
 r1  before Tick 0       !>007FC4                   !>007FC4

                     plays = 1, Tick diags = 0  plays = 1, Tick diags = 0

 r0  after Tick 0    !>00**C4                   !>00  C4
 r1  after Tick 0        !>007FC4                   !>007FC4
```

The spelling in the invalid operand position activates the unrelated Raw Play one row south.
The Source already reports `expected a number, found "**"`; the zero diagnostic counts above are
Tick Plan diagnostics, not Source syntax diagnostics. The parser does not accept a Bang Atom in
this operand position.

The activation half predates `spatial-tick-planning/02`; that ticket added the expiry half, which
also erases the rejected spelling. Current checkout `b4a5e85` reproduces activation but has no Bang
expiry Producer. Expiry is recorded above from `84af8bf`, not from this checkout.

## Architectural contribution

ADR 0018 partitions "each row left to right", and then "Expression construction operates on this
partition". ADR 0024 says the same in order: a row-local partition of Language Units, "and then
parses each Expression extent". The partition is stage one and the extent is stage two.

`walk_row` (`orcvs/src/source/language_map.rs:473`) discovers Language Units and candidate
Expression Spans together. `build` parses those Spans afterward; it does not run the entire parser
inside the row walk. Consolidating the walk removed duplicated space, comment and row-edge logic,
and retained Spans for invalid Cells that form no Language Unit.

Activation's spelling-only reader predates that consolidation. Splitting the walk alone would not
fix it: the parser must establish valid Bangs, and Tick execution must consume those results.
There is no additional spatial analysis pass deciding which spellings count as Bangs. Tick
execution checks neighboring root Positions and plans removal using the parsed Bang's Span.
Preserve shared scanning and invalid-edit diagnostics when establishing that responsibility.

## The gap underneath

No ADR states what bounds an Expression extent. `CONTEXT.md`'s **Expression** entry is the only
statement — "a contiguous horizontal run of occupied Cells in one Source row" — and it is prose
using the undefined word. The space rule and the `##` rule live only in `RunBoundary`.

Two questions follow, and they are ADR questions rather than implementation ones:

- What bounds an Expression extent, stated where the other partition rules are stated?
- Is an adjacent standalone `**` its own Expression, or a member of the one it touches? Today
  adjacency means membership, which is why ADR 0006's west `(x-2, y)` and east `(x+2, y)` root
  anchors are unreachable from any Source text. That unreachability follows the current extent rule;
  separating the row walk alone does not decide a replacement rule.

ADR 0024 distinguishes spelling from contextual meaning, but does not expressly settle every
malformed-Source case. The discussion on 2026-09-06 resolves this example: Orcvs has typed operands,
and `**` rejected in such a position is syntax error content, not a spatial pulse. This is a
deliberate divergence from original Orca, whose operand locks prevent a Bang's own execution without
hiding it from neighboring operations. Other malformed or trailing Source and adjacent Expression
extent rules should not be decided implicitly by a blanket filter.

## Comments

2026-09-06: Confirmed parser ownership. The parser returns Bang types only when valid in context;
Tick execution determines their effects. "Spatial activation" means activation of neighboring
roots by Position, not another analysis pass. This replaces the earlier proposal for a separate
spatial-eligibility decision.

Found while reviewing `spatial-tick-planning/02` after it was implemented and committed. The probes
above were run on `2f3ed99` and on `84af8bf` and are reproducible from
`SourceUnderTest::new(Grid::new(16, 3))` with the two writes shown.

`lang/bang-expiry-scope-prototype.html` and `spatial-tick-planning/02`'s own Comments both present
the operand case as an open design question with three options. They are wrong on that point and
should be corrected to name this issue instead.

`spatial-tick-planning/03` should not begin before this resolves. Its Portal effect bundles make a
Sequence result encoding a `**` writable into Source, and the same partition read would consume it
on the following Tick.

2026-09-06: User confirmed that Bangs embedded in typed Function operands like this example are
invalid syntax in Orcvs. Original Orca's character-based behaviour is regarded as a quirk, not a
compatibility requirement for this case. See [upstream research](../orca-bang-reference.md).
The earlier description of an Expression consuming this spelling as an Atom was incorrect and has
been corrected above. Standalone Bang timing and broader Expression extent rules are not changed by
this clarification.


2026-09-06: Further clarification supersedes the exploratory standalone Bang Function proposal.
Functions can return Bang values; `**` is the transient Source representation of that output, whose
presence activates neighboring roots. Bang does not need its own Function call or operand stack.
Parsing remains authoritative for the validity of that representation. Output timing is still a
separate decision. Original Orca permits manual `*` entry through its keyboard handler, so an
output-only authoring restriction must not be inferred from upstream behaviour.


2026-09-06: User established a same-Tick execution constraint: when Functions output Note C4 into
a MIDI Play call and Bang adjacent to that call during Tick T, C4 must play during T. This exposes
a broader issue than recognizing valid Bangs. The current fixed-Snapshot operand evaluation and
deferred Source writes fail even when both producers precede the MIDI root in row-major order.
A temporary public-Source probe (`/tmp/orcvs-issue06-probe/src/bin/same_tick.rs`) starts with
`r0: .=0101`, `r1:       .^3C`, `r2: !>007FD4` on a 16 × 4 Grid. Tick 0 commits `**` at (0, 1)
and C4 at (6, 2), but returns zero Play Commands and zero Tick diagnostics. Its assertion requiring
one command fails. Merely adding same-Tick activation would still leave pre-parsed operands stale.
Whether the requirement also holds when either producer is later than the MIDI call in Source
order remains to be settled; no replacement execution algorithm has been selected.


2026-09-06: Proceeding with the Orca-style ordering fallback in
[ADR 0031](../../../docs/adr/0031-evaluate-turns-against-working-source.md). Earlier admitted
writes must be visible to later original roots in the same Tick, including writes that complete
previously invalid operands. Current parsing supplies both operands and valid Bangs. Bang remains a
value. Original Bangs clean up at their own ordered turns only while still valid and untouched across
their complete Span, without erasing newer writes; generated
Bangs have no cleanup turn until the next Tick. An upward Portal does not rewind an earlier root.
This supersedes the prior fixed-Snapshot timing assumption and the earlier acceptance of unchanged
Bang expiry. Implementation and verification are in progress; human acceptance remains pending.

Additional acceptance evidence required:

- [x] Note C4 and Bang produced before MIDI's turn emit C4 during that same Tick.
- [x] A producer chain reads earlier same-Tick values; completed original operands can execute.
- [x] Original Function identities receive at most one turn; overwritten/generated Functions wait.
- [x] Cleanup respects original Bang ownership and current parsed validity.
- [x] Backward delivery does not rerun an earlier root; ordering is explicit in tests and docs.

2026-09-06: Broader operation-ordering exploration is recorded in the
[executable walkthrough](../operation-ordering-prototype.html) and
[implementation findings](../operation-ordering-exploration.md). The current Source-parseable
scalar Functions admit a bounded dependency-scheduling prototype. A fixed upward Portal is an
ordinary producer-to-consumer edge; it does not require reevaluating earlier roots. Parser-owned
operand layouts and output footprints are the missing production information. The earlier
row-major work remains a provisional fallback, not the settled scheduling design.

2026-09-06: User confirmed Bang lifetime: `**` is a visual representation of a Function's Bang
output; manually entering `**` is a no-op. Only a Bang produced during T activates during T;
its displayed encoding does not become another event on T+1. This supersedes the cleanup-turn
and potential replay semantics of the provisional ADR 0031 implementation. The glossary and
prototype now reflect the confirmed rule; the production fallback still requires replacement.
Original Orca's raw-glyph quirks do not override this Orcvs rule.

2026-09-06: Design approved. ADR 0032 now supersedes ADR 0031 and conflicting earlier timing
clauses. Implementation is split into `tick-execution-order/01` (parser-owned inputs), `/02`
(dependency execution and Bang lifetime), and `/03` (integration and verification). This issue's
current acceptance criteria follow ADR 0032; earlier comments remain history. Implementation is
paused with parser and test-fixture drafts recorded on the new tickets, not marked complete.

2026-09-06: Resolved by the ADR 0032 scheduler. `LanguageMap::is_root_active` and spelling-derived
cleanup turns are removed. Parser-owned Bang Atoms and Spans identify display cleanup, while only a
fresh Function result creates an activation event. Invalid embedded `**`, manual no-op, same-Tick
Note plus Bang delivery, and no replay are covered through Source execution. The accepted HTML
walkthrough is preserved separately on branch `prototype-operation-ordering`, commit `841f696`.
