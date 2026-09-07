# Cell-Indexed Parse — the Grid is the data structure

**Status:** ready-for-agent

**Tags:** release/v1

**Supersedes:** an earlier uncommitted spec that addressed a symptom of this — carrying an
accurate anchor-relative measurement through the Parser's contract. Deleted rather than kept, since
handing the Parser the row makes the measurement an address instead of a report. Its rejected
mechanism is recorded under Implementation Decisions.

**Absorbs:** `language-map/08`, whose remaining scope is ticket 07. **Restates:** `language-map/09`. **Reshapes:** `language-map/10`.

## Problem Statement

A person composing on the Grid can lose their music with no diagnostic.

Write `.=0101` above a literal `0102`. The comparison bangs, writes `**` into the Cells beside the
literal, and joins the two into one run. The run no longer parses as the Expression that was there.
Nothing reported it, and editing the comparison so it stopped banging did not clear the `**`, because
the display that replaced the run was no longer a Bang any cleanup could find.

That one is guarded on this branch already, by a search over every Expression run added after review.
The guard is a compensation for the partition, not a fix for it: it exists because a row is divided
at its spaces, and `language-map/10` had already reasoned that a parse boundary makes it dead. It is
stated here because the corruption it prevents must stay prevented after it is deleted.

Write a half-typed `!>00` four columns left of a producer. The producer's `**` lands in a Cell that
one part of the system reads as a typed operand and refuses, and another part dropped from its
operand slots. The Bang is delivered as an activation rather than refused as an operand. The note
plays, or does not, depending on which part answered. Commit `8e7bdce` records the same failure
from the other side: a Play root went silent on every Tick, "with no diagnostic anywhere, because
the half-typed root is terminal and unactivated and takes no turn from which to report anything."

### One root cause, and it is not the scheduler

**A Span is a whitespace run, and the Language Map hands the Parser a copy of one run at a time.**

The row walk splits each row at spaces and at `##`. Each maximal non-space stretch becomes a Span.
The Map then slices the Source to exactly that Span's Cells, copies the bytes into a fresh string,
and hands that to the Parser. Inside the Parser, offset zero is the start of the run, not the start
of the Source. Every position it measures is relative to a copy whose location it cannot know.

Everything downstream is consequence:

- **Positions must be re-based by every consumer.** The Map adds the Span start back to the Parser's
  consumed length to restore a trailing-content verdict. The Tick scheduler adds the anchor to each
  operand offset to get a Cell.
- **Position could not usefully enter the Parser's contract.** When dependency scheduling needed
  operand positions, the change that introduced it added a *reconstruction* — walk the records and
  sum each token's spelling width — rather than extending the Parser's output, because a
  fragment-relative offset is not a fact anyone else can use.
- **The reconstruction contradicts the measurement in three places.** A refused Function spelling
  rewinds one character but the reconstruction claims two Cells. An operand slot the Source ran out
  before is measured as zero and the reconstruction claims the slot's full width. The code carries a
  comment admitting the two "disagree deliberately".
- **A run of standalone Atoms is not one Expression**, so a special path assembles it from the
  partition's units instead of parsing it — a second producer of a consumed width that reports
  `**^^` as one four-Cell Expression where the Parser reports a two-Cell Bang with `^^` still to
  read.
- **Four independent derivations** of which Cells an Expression claims, disagreeing, is what makes
  the two failures above possible.

Every optimisation since has worked *around* this rather than on it — building a standalone run from
the partition rather than from the bytes again, naming an Expression's Cells once per Expression
rather than once per Cell, filtering declared slots down to those abutting the run. The partition
rule itself has not moved since the crate was extracted.

### And the shape of the answer is wrong twice over

The scheduler answers every spatial question — which slots does this output cover, which run does it
abut, does another producer already write here — with sorted vectors of intervals and bounded binary
searches. But every one of those questions is *"what is at this Cell?"*, and the Cell space is dense,
bounded and small: columns times rows, one thousand Cells at the default Grid. The searches run over
structures indexed by search when the space itself could be indexed directly. The win is O(1) reads
and no per-producer allocation, not a smaller structure: the slot collection is roots times arity, a
few hundred entries, which is the same order as the thousand Cells it describes.

The Map already holds one array of exactly the right shape — the Glyph classification, indexed by
Cell, parallel to the Source. It carries presentation and not semantics, so the scheduler rebuilds
the semantic answer as intervals every Tick.

## Solution

**Parse the row, and store the parse in the shape of the Grid.**

Two changes, and the rest is deletion.

**Partition a row by parse, not by whitespace.** The Parser is handed the row's Cells and the offset
they start at, and it reports where each Expression ends. The walk advances by that. A Span becomes
the Cells one Expression occupies as the Parser found them, rather than a stretch between spaces. A
run of standalone Atoms stops being a shape the Parser cannot take whole, because it is no longer
handed four bytes and told they are everything.

**Stamp the parse into a Cell-indexed array.** The Map gains a second array parallel to the Source,
one entry per Cell, saying what that Cell is: blank, a character claimed by no Expression, part of an
Expression's structural spelling, or a numbered operand slot of a root. The Parser does not *report* a position and leave a consumer to
re-base it. It writes what it found at the Cell it read.

From a prototype of the classification, which encodes the decision more precisely than prose:

```rust
enum CellRole {
    Empty,                                     // no character
    Occupied,                                  // a character, claimed by no Expression
    Structure { expression: ExpressionId },    // a Function, Bang or Activation spelling
    Slot { root: RootId, slot: SlotOrdinal },  // an attachment point of a root
}
```

Every question ADR 0032 requires answered before execution then becomes two array reads for a
two-Cell result written at Cell `n`:

- both entries name the same root and slot, and `n` is that slot's first Cell → a whole-slot
  projection; draw the Data edge
- they name different slots → a partial input projection; reject the Tick
- either is `Structure` → an unsupported structural write; reject the Tick
- neither is claimed → a free write, needing no diagnostic and no guard of its own

**The measurement problem dissolves rather than being fixed.** There is no offset to carry, no anchor
to re-base, and no reconstruction to disagree with. The position is the address written to.

## User Stories

1. As a person composing on the Grid, I want a result written beside an Expression to leave that
   Expression readable, so that my Source is never silently corrupted by a write I cannot see.
2. As a person composing on the Grid, I want every part of the system to agree about which Cells my
   Function occupies, so that a note either plays or is refused for a stated reason.
3. As a person composing on the Grid, I want a Bang landing on a Cell my Function claims to be
   treated as one thing, so that the same Source produces the same music on every Tick.
4. As a person composing on the Grid, I want a stray character to cost me exactly the Cell it
   occupies, so that the Function I typed next is still recognised.
5. As a person composing on the Grid, I want two Language Units written side by side with no space
   between them to be read as two things, so that spacing is a matter of taste rather than of
   meaning.
6. As a person composing on the Grid, I want the Cells the console paints to be the Cells the
   scheduler reserved and the evaluator reads, so that what I see is what runs.
7. As a person composing on the Grid, I want a claimed Cell that holds nothing to be visible as
   ordinary live-edit state, so that I can tell a half-typed Function from a broken one.
8. As the Parser, I want to be handed a row and the Cell it starts at, so that the positions I
   establish are the Grid's positions and not a copy's.
9. As the Parser, I want to report where each Expression ends, so that the walk that calls me
   advances by what I read rather than by where the spaces are.
10. As the Parser, I want to record what I found at each Cell as I read it, so that no later reader
    has to reconstruct a position I already knew.
11. As the Parser, I want to be the only thing that classifies a two-Cell spelling, so that one
    lexer decides what the characters mean.
12. As the row walk, I want to advance by the parse, so that a Span is what one Expression occupies
    rather than what lies between two spaces.
13. As the row walk, I want no special path for a run of standalone Atoms, so that every Span is
    established the same way.
14. As the Language Map, I want to hold one entry per Cell saying what that Cell is, so that every
    spatial question is an index rather than a search.
15. As the Language Map, I want that array built in the same walk that establishes units, Spans and
    diagnostics, so that the rules are stated once.
16. As the Language Map, I want a rebuilt Map to equal the Map a full build would have made, so that
    incremental reparse stays trustworthy.
17. As the Language Map, I want to see which claimed Cells hold nothing, so that the diagnostic a
    half-typed root cannot report from its own turn has a home in a phase that always runs.
18. As the Tick scheduler, I want to classify a write by reading two entries, so that whole-slot
    projections, partial projections and structural writes are told apart without a search.
19. As the Tick scheduler, I want the guard against a result joining two Expressions deleted rather
    than reimplemented, because a boundary set by arity leaves nothing for it to prevent.
20. As the Tick scheduler, I want to answer write contention by the same index, so that two
    producers targeting one Cell are found without a separate map.
21. As the Tick scheduler, I want the operand slots I draw edges against to be the operand slots the
    evaluator reads, so that the two cannot disagree.
22. As the Tick scheduler, I want to keep ordering by dependency with Grid position as the
    tie-break, so that a producer below its consumer still supplies it in the same Tick.
23. As the Tick scheduler, I want to keep diagnosing competing writers, cycles, partial projections
    and structural writes before publishing, so that ADR 0032's guarantees are unchanged.
24. As a maintainer, I want the sorted slot vector, its widest-slot bound and its two preconditions
    deleted, so that an invariant held by assertion becomes one held by construction.
25. As a maintainer, I want the per-Span string copy deleted, so that a Tick stops allocating once
    per Expression on the path under the playback deadline.
26. As a maintainer, I want the second lexer deleted, so that the same characters are not classified
    twice by two functions that can drift.
27. As a maintainer, I want the second producer of a consumed width deleted, so that one Source has
    one answer for where an Expression ends.
28. As a maintainer, I want a benchmark comparison for the Tick path, so that the claim that this is
    faster is measured rather than asserted.
29. As whoever answers the open question about what an incomplete Function claims, I want the choice
    to be whether an unreached slot is stamped at all, so that the decision is one flag at one
    place.
30. As whoever answers that question, I want the regression from `8e7bdce` pinned under every
    candidate answer before the decision lands, so that deferring it does not leave the failure
    unguarded.
31. As whoever answers that question, I want to know that the commit it concerns implemented a third
    rule that the question does not list, so that the options are complete.
32. As whoever rebuilds the dependency scheduler, I want the claim policy to live where arity lives
    rather than as arithmetic in a consumer, so that no consumer holds a policy of its own.

## Implementation Decisions

### The partition

- **A row is partitioned by parse.** The walk hands the Parser the row's remaining Cells together
  with the Cell index they start at, takes the Expression the Parser establishes, and resumes at the
  Cell after it. Spaces stop being a partition rule and become ordinary empty Cells that no Language
  Unit covers. The `##` Comment rule survives — nothing after it on the row is Source — and stays
  stated in one place.
- **The Parser needs one fact from the Grid: the row's width**, so that a two-Cell spelling cannot be
  read across a row edge. `lang` already models a bare column and row, and the boundary between the
  crates is exactly this one number. The Grid's newtypes stay in `orcvs`; they carry provenance, not
  location.
- **A Cell index is a byte offset into the Source.** The Source maps literally onto the Grid,
  row-major, one printable single-byte ASCII character per Cell. Offsets and Cell indices are the
  same number, which is what makes the re-basing unnecessary rather than merely cheaper.
- **The special path for a run of standalone Atoms is deleted**, along with the check that its units
  tile its Span. So is the reconstruction that restores a trailing-content verdict by adding the Span
  start to a consumed length.

### The Cell-indexed array

- **The Map holds one entry per Cell, parallel to the Source**, established in the same walk that
  establishes units, Spans and diagnostics — the walk that already exists so the rules are not
  written twice.
- Each entry says one of three things: the Cell is empty; the Cell is part of a root's structural
  spelling — a Function, Bang or Activation; or the Cell is a numbered operand slot of a root.
- **The array is the answer to every spatial question the scheduler asks.** It replaces the flat
  sorted slot vector across all roots, the widest-slot bound, both bounded searches, the windowed
  scan over Expression Spans, and the map of Cells already targeted this Tick.
- **Per-root records stay.** The array answers "what is at this Cell". Evaluation still needs each
  root's operands in order, and diagnostics still need a Span to highlight. The array replaces the
  search structures, not the per-root data.
- **The Glyph array is the precedent and the sibling.** The Map already holds a Cell-indexed array
  for presentation; this is the semantic one beside it. Whether the two later merge is out of scope.
- Memory is roughly four kilobytes per revision at the default Grid, which holds only because the
  identifiers are narrow: sixteen bits for an Expression or root and eight for a slot ordinal keep an
  entry to four bytes. At pointer width, which is what the scheduler uses for the analogous field
  today, the same array is twenty-four kilobytes. Incremental reparse gets
  simpler, not harder: rewriting the affected rows' slice of the array replaces re-deriving a
  partition and re-pointing every Expression's range into it.

### What the scheduler keeps

ADR 0032 is unchanged and this spec implements it more directly rather than amending it.

- **Ordering stays dependency-first with Grid position as the tie-break.** A producer below its
  consumer must still supply it in the same Tick; a fixed upward Portal is an ordinary dependency.
  The topological sort and its cycle diagnostic are untouched.
- **The four graph errors are still diagnosed before publishing** — competing writers, same-Tick
  dependency cycles, partial input projections, unsupported structural writes. They become array
  reads rather than searches. This is why operand-slot granularity is required: an Expression-level
  interval cannot tell a whole-slot projection from a write straddling two slots, and that verdict is
  owed before any root takes a turn.
- **Portal destination resolution is untouched.** Where a result goes is a separate question from
  where Source is, and it stays in `orcvs`.
- **Nested Functions stay inside their containing Expression** and use the existing Evaluator. The
  dependency graph is not a syntax tree; its edges are spatial.

### The open question, unchanged in substance and simplified in form

- Whether an operand slot the Parser never reached is claimed becomes: **is that Cell stamped, or
  left empty?** One flag at one place, rather than an arithmetic filter in one consumer.
- **There are three candidate rules, not the two the open question lists.** The commit that the
  question concerns filtered declared slots with a comparison that *kept the first missing slot*,
  deliberately, so that a write abutting the run could complete the Function. That is neither
  "claim every declared Cell" nor "claim only Cells that hold characters". Whoever answers should be
  told this; the third rule needs a name so the behaviour is not reconstructible only from a
  comparison operator.
- **This spec does not answer it.** There is no default; every construction names a rule. When the
  answer lands, the losing rules are deleted and no consumer changes.

### Explicitly rejected

- **Ordering Functions by output position.** It gives a correct order for every program that exists
  today, because the current Portal puts a result directly below its root and consumption therefore
  flows downward. It fails on the case ADR 0032 exists for — a producer below its consumer, reached
  by a fixed upward Portal — where positional order runs the graph backwards.
- **Treating the dependency graph as a syntax tree.** Its edges come from Grid adjacency and Portal
  resolution, not from grammar; it is a DAG whose cycles must be diagnosed; and it varies between
  Ticks without the Source varying.
- **Keeping the interval search and only fixing the measurement.** That was the superseded spec. It
  made the Parser report a position accurately, which is worth doing but leaves the fragment, the
  re-basing and the second lexer in place.
- **Re-parsing each root at its turn** instead of holding operand positions. The graph must be built
  before any turn runs, so the positions are needed earlier than a turn can supply them.

## Testing Decisions

**What makes a good test here.** Assert what a Source revision means and what a Tick does, through
the seams that already carry those questions. The Cell array is observable — it decides which Cells
a write lands on and therefore whether a note sounds — so its contents are fair game; the fact that
it is stored as one array is not. A test that must change when the storage changes is testing past
the seam.

**Seams. No new ones.**

- **Language Map construction** is the primary seam, and it already exists. It carries the partition
  change and the array.
- **Tick planning** is the secondary seam, and it already exists. It carries the scheduling
  behaviour.
- `Parser::analyze` remains a seam and drops to a lower level: what one parse of a row reports.

**At the Language Map seam**

- A row partitions by parse. Two Language Units written flush against each other are two units, and
  a run of standalone Atoms is as many Expressions as the Parser finds. Prior art: the existing
  partition and unit-kind tests.
- Every Cell of an Expression is stamped, and no Cell outside one is. The array and the Expression's
  Span agree.
- **The rebuild-equivalence property is the pinning test and carries over unchanged**: a rebuilt Map
  equals the Map a full build would have made. It must pass at every step, and it is what makes the
  partition change safe.
- The three contradiction sites become assertions about what a Cell holds rather than about a
  reported width: a refused Function spelling occupies one Cell, and an operand slot the Source ran
  out before occupies none.

**At the Tick planning seam**

- The reproduced silent failure — a producer's `**` landing on a Cell one reader treats as an operand
  and another dropped — asserts that both now give the same answer.
- The Source the join guard was written for — a result flush against a neighbouring Expression —
  asserts that the neighbour is still read as the same Expression after the write, with the guard
  deleted rather than merely passing.
- Each of ADR 0032's four graph errors keeps its existing test, and each must still be diagnosed
  before publishing. Prior art: the existing competing-writer, cycle, partial-cover and structural
  write tests.
- The `8e7bdce` regression is asserted **under every candidate rule**, in one file, before the open
  question is answered. Nothing in it depends on which rule wins.
- Ordering is unchanged and its existing tests must pass untouched, including a producer below its
  consumer supplying it in the same Tick.

**Performance**

The Tick benchmark already exists on this branch. This spec claims the Tick path gets cheaper —
allocations per Expression removed, searches replaced by indexing — so that claim is settled by the
benchmark workflow's comparison, not by a local number.

## Out of Scope

- **Answering what an incomplete Function claims.** The spec makes every candidate rule a one-place
  flag and names the third one the open question omits. It decides none of them.
- **Merging the semantic array with the Glyph array.** They are siblings; whether presentation and
  semantics share one array is a separate question.
- **Portal destination resolution**, computed destinations, and variable-width Sequence projection.
  ADR 0032 defers these and so does this.
- **Changing the ordering model.** Dependency order with position as tie-break is ADR 0032's and
  stays.
- **Same-Tick structural mutation**, delayed feedback, and the directional movement, Jump and Halt
  designs. Those must adopt the dependency model before adding effects, and this does not change
  that.
- **The scheduler's parallel per-root vectors.** Replacing them with one table of per-root state is a
  real locality improvement and unrelated to the partition.

## Further Notes

- **Why the effort is separate.** `measured-claims` addressed the reconstruction's inaccuracy. This
  addresses why a reconstruction exists. The earlier spec is superseded rather than amended, because
  its central mechanism — carrying an anchor-relative measurement through the Parser's contract — is
  unnecessary once the Parser is handed the row.
- **Relationship to the existing language-map tickets.** `language-map/08` is this spec's first
  slice and should be closed as absorbed rather than worked separately. `language-map/09` is
  restated, not answered. `language-map/10` is reshaped: rebuilding the scheduler on the fixed-width
  partition becomes indexing an array the Map already holds.
- **This is active language design, not public-API breakage.** Neither workspace crate is
  publishable and the Orcvs language is pre-release with no compatibility contract. The Parser's
  input, the Map's interface and the Expression's accessors all change.
- **Vocabulary.** CONTEXT.md lists "footprint" and "extent" on Span's Avoid list, so neither is used.
  "Claim" is the repository's own word for what an Expression occupies. CONTEXT.md gains an entry for
  the Cell-indexed classification, and the ADR that the partition change writes records the parse
  boundary rule beside the Comment rule.
- **Provenance.** Reached by working backwards from two reproduced Tick failures through four
  disagreeing derivations, a fragment-relative parse, and a whitespace partition that has not moved
  since the crate was extracted. Four parallel interface designs were produced for the symptom before
  the cause was identified; their surviving contribution is the requirement that whatever enumerates
  an Expression's Cells is also what reads them.
