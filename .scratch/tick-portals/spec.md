# Tick Portals: the typed record of every Portal access a committed Tick made

Status: resolved

Tickets: `issues/01`–`06`, vertical slices, 01 first.

## Problem Statement

Orca lights each operator's ports as it runs, so the Learn Orca tracker shows a
moving read head down each voice's column. Orcvs paints only static Output
Portals. A dynamic Input Portal is resolved at the Turn in `turn_portal`
(`orcvs/src/source/tick/execution.rs`) and dropped, so nothing outside the Tick
knows which Cells a Function read or where a selected write landed. The tracker
is the first test of the language and console as one experience, and it does not
look like Orca's while it plays.

## Decision

Toby chose design 4 of `/mnt/project-files/investigations/portal-record-design.md`
(a typed record), with design 3's per-Cell paint derived from it, on 2026-10-18,
with six revisions. This spec carries the revisions. The record is authoritative,
the paint projection is derived from it, and recording is unconditional: no
generic recorder and no test-only switch in shipped code.

### 1. What the record holds

`TickPortals` is an immutable, ordered list of `PortalUse` entries. Each entry
carries:

- **Owner.** The anchor `Position` of the Function whose Turn made the access,
  the `Span` its subtree holds (its spelling and every operand, nested
  Functions' included), and the `Function` that Turn executed (after any
  Function Replacement). All three are captured during execution from the
  schedule's node and `Turns`, never recovered from committed Source, which
  the Tick may have rewritten.
- **What happened**, one of:
  - `Read { origin, site }`. The Input Portal handed to the Interpreter for a
    Turn that got past the writer wait. `site` is `Ok(InputSpan)` when the
    selected Portal resolved inside the Grid, and `Err(PortalError)` when it
    did not. A resolved `InputSpan` says a site was read from. It does not say
    the Cells held a valid value: whether the operand bound is the Turn's
    diagnostic, not the record's.
  - `Write { origin, outcome }`. One attempted delivery at one Output Portal
    site in `deliver_output`. `outcome` is `Ok(OutputSpan)` for an admitted
    write, or `Err(WriteRefusal)`, which covers every refusal there: a
    geometric `Portal(PortalError)` with the attempted `Position` where one
    exists, `Replacement(ReplacementChange)`, and `OrderingDefect`.
  - `Lock { site, verdict }`. A Halt's lock, with the verdict
    `dependencies::lock_verdict` answered.
  - `Bundle { part, outcome }`. One part of a Source Bundle, in the order the
    parts ran: `Vacate` (the producer's own Span), `Place` (the destination),
    `LeftBang` (a refused move leaving `**`), and `Fire` (a Bang Function's
    clear). A refused destination is `Err(BundleRefusal)`: `Portal(PortalError)`
    with the attempted site where one exists, or `Occupied(OutputSpan)` when
    the destination resolved but was not vacant.
- `origin` is `Declared` for a Portal the Function declares at a fixed offset,
  and `Selected` for one its operands chose this Turn.

`PortalError` is promoted to `pub` and reused, not mirrored.

### 2. Which events belong

The record holds accesses the Tick attempted, not Portals that merely resolved:

- A read is recorded once the Turn passes the writer wait and its Input Portal
  is handed to the Interpreter. A Turn that waits records nothing until it is
  retaken, so a retried Turn leaves exactly one read.
- A write is recorded for each site `deliver_output` is asked to deliver to,
  admitted or refused. A destination that resolved for a Turn whose
  interpretation then failed was never delivered to, so it is absent.
- A Turn the schedule refused before interpretation (syntax, arity, suppressed,
  operand decode) records nothing; its diagnostic stands as today.

Order is the order execution made the accesses: Turns in settled order, and
within a Turn its read, then its writes in site order, then bundle parts in the
order they ran.

### 3. Types that carry provenance

`InputSpan` and `OutputSpan` wrap a `Span` with private constructors visible
only to Tick execution, and expose a read-only `span()`. An `InputSpan` is built
only from the Input Portal a Turn resolved, and an `OutputSpan` only from an
admitted `SpanWrite::span()`, so neither can be minted from an arbitrary Span and
neither converts into the other.

### 4. Commit: one path, atomic with the Tick Plan

`tick::plan` returns one `ExecutedTick`, which only execution builds and which
holds the Tick Plan and its `TickPortals` together. Both `PlannedTick` (the
optimistic path) and `Source::execute` (the locked fallback,
`orcvs/src/source/model.rs`) hand that one value to `Source::commit_tick`, the
only thing commit takes, which stores the record with the revision the commit
leaves the Source at and clears the set of Cells edited since. A refused
`PlannedTick` is dropped whole and publishes neither. A committed Tick that
accessed no Portal replaces the previous record with an empty one. The Playback
Engine and `TickPlan` are unchanged.

### 5. Freshness

The Source keeps the last committed record, carrying its Tick and the
`RevisionId` the commit left, and the Cells an edit has written since that
commit, one bit per Cell; a Tick commit's own writes mark none. A
`SourceRevision` captures the record and those edited Cells as they were when it
was read, sharing both rather than copying them, so an older `SourceRevision`
keeps its own.
`SourceRevision::tick_portals()` answers the record with its freshness:

- `Current` when the revision is the one the commit left.
- `Retained` when an ordinary edit has landed since. Positions are still valid,
  because Orcvs edits never move Cells, but the Functions that made each access
  may have changed.

Opening or loading a Source builds a new Source, which has no record until its
first Tick. Stopping keeps the record, so a paused tracker still shows where
each voice is.

### 6. Paint

`RenderFrame::derive` folds the record into each `RenderCell` as it builds the
Cell vector, replacing the `output_portal: bool` field with a small set of
role bits. It adds no allocation per frame, which an allocation assertion pins,
and a benchmark measures the derive with a record present.

- One new theme role, **Input Portal**, in both built-in themes and in the
  contrast checks. Orca styles every input port alike, so the dynamic read (the
  playhead) and a declared read share it.
- Admitted writes, declared or selected, paint with the existing Output Portal
  channel. Before any Tick, the static Output Portal highlight paints as today.
- A lock that withheld a root (`LockVerdict::Locked`) marks the two Cells of
  that root's Function with a third role, **Locked**, which has its own Theme
  channels (`locked.*`) in both built-ins and in the contrast checks. Orca
  colours the Cells an operator locks; the Cells a Halt locks are the root's
  own spelling. A missed lock, an ordering defect, a site holding no root and
  a lock with nothing to lock mark nothing (issue `07`).
- Refused writes, refused bundle parts and off-Grid sites stay in the record
  and do not paint.
- A `Current` record paints every admitted read and write. A `Retained` record
  paints only entries whose owner Function still stands at its anchor in the
  displayed revision, and whose recorded subtree holds no Cell an edit has
  written since the commit; an edit elsewhere in the Expression leaves them
  lit. A `Selected` entry also needs every resolved read recorded by a strict
  descendant of its owner, an anchor in the owner's subtree other than its
  own, to have read no Cell an edit has written since: a nested read answers
  an operand with Cells outside the subtree. A `Declared` entry's
  place cannot move, so a nested read does not put it out (ADR 0077). A
  Halt's lock site is a fixed offset, so a lock is retained as a `Declared`
  entry that also needs no edit to have written the Cells it marks: it stays
  marked until an edit reaches the Halt's own Cells or the locked root's own
  spelling, and an edit to that root's operands leaves it marked.
- Precedence is the existing schema's (`.scratch/theming/schema.md`,
  composition steps 4 and 5): Portal channels over every other non-Function
  fact, Diagnostic included, as `console/src/style.rs` already pins. Between
  Portal channels, Output Portal wins over Input Portal. A bound Function keeps
  all its own paint inside either Portal. A Bang keeps its glyph foreground and
  takes the Portal background. Locked paints below both Portals, so a locked
  Cell a Function reads or writes paints as that read or write would, as Orca
  draws a port over a locked Cell, and a locked Function's spelling there keeps
  its own paint and shows no lock; Locked paints a bound Function and a Bang, the only Cells a lock
  marks. Cursor and Region keep their existing precedence over every Portal
  channel and over Locked.

### 7. Vocabulary

`CONTEXT.md` defines **Tick Portals** as an observation of a committed Tick, not
a Portal, so ADR 0009's "a Portal does not survive its Tick" stays true. ADR 0077
records the record, its event meaning and its freshness contract.

## Acceptance tests

- A Turn that waits for a writer and is retaken leaves one read entry.
- A refused optimistic plan publishes no record, and the locked fallback does.
- An empty committed Tick replaces the previous record; an edit makes the record
  `Retained`; an older `SourceRevision` keeps its own; a newly opened Source has
  none.
- Refused writes (geometric, replacement, ordering) and refused bundle parts
  keep their reasons and do not paint.
- Entry order and owners stay correct when the Tick rewrites the Source the
  owners stood in (a mover, a Function Replacement).
- Overlapping read and write roles, empty read Cells, and the existing
  precedence paint as section 6 states.
- The tracker example lights each voice's `&v` read as it steps.
- A Halt's locked root is marked Locked on a `Current` record, stays marked
  after an edit outside the Halt, and is put out by an edit to the Halt until
  the next Tick.

## Out of scope

- A lane (a Read's whole selectable range). Orca highlights the port Cells only.
- An empty-Cell glyph (`.scratch/empty-cell-glyph`); an empty read Cell shows
  only the Input Portal background.
- A hover inspector or arrows. The record supports them; no caller asks yet.
