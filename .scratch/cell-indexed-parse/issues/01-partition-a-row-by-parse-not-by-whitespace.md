# 01: Partition a row by parse, not by whitespace

**What to build:** A row is divided into Expressions by the Parser rather than by its spaces. The
Parser is handed the row's remaining Cells together with the Cell index they start at, and reports
where the Expression it read ends; the walk resumes at the Cell after it. A space becomes an
ordinary empty Cell that no Language Unit covers, rather than a partition rule.

Two Language Units written flush against each other are two units. A run of standalone Atoms such
as `**^^` is as many Expressions as the Parser finds, so the assembly path that built one Expression
from the partition's units — and the check that those units tile the Span end to end — has nothing
left to do and is deleted with it. The reconstruction that restored a trailing-content verdict by
adding a Span's start back to a consumed length goes too, because there is no longer a fragment
whose offsets need re-basing.

The Parser needs exactly one fact from the Grid: how wide a row is, so that a two-Cell spelling
cannot be read across a row edge. The Comment rule survives unchanged — nothing after `##` on a row
is Source — and stays stated in one place.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Tags:** release/v1

## Pre-delivery audit at `593613c` — 2026-09-08

The implementation statements in this audit describe commit `593613c`, before
live-typed-execution delivery. For the completed transferred work and current
implementation, see [delivery evidence](../../live-typed-execution/evidence.md).

Implemented in `593613c`, not in the planning-only commit `64291cc`. The current
`walk_row` calls `Parser::at` on the remaining row and advances to `analysis.cells().end`;
`source_end` bounds it at the Comment. Parser tests
`an_analysis_reports_the_cells_it_read_from_the_cell_it_was_told_it_began_at` and
`an_unrecognized_function_consumes_one_cell_and_records_one_invalid_slot`, Map tests
`adjacent_standalone_bangs_have_distinct_parsed_spans` and
`a_rebuilt_map_equals_the_map_a_full_build_would_have_made`, and the public
`truncated_operand_owns_the_available_row_tail` / `operand_claims_stop_before_comments`
regressions cover the delivered boundary. ADR 0033 records it. The remaining silent
half-typed-write diagnostic recorded in the original comments below belongs to live-typed-execution delivery;
it does not reopen partitioning.

The original checklist below is retained as historical scope; this audit records
its disposition at `593613c`. The linked delivery evidence records subsequent delivery.

- [x] The walk advances by what the Parser read, and spaces are no longer a partition rule.
- [x] The Parser receives the row's Cells and the Cell index they start at, and no Source bytes are
      copied into a fresh string per Expression.
- [x] Two Language Units written with no space between them are established as two units.
- [x] `**^^` establishes the Expressions the Parser finds, not one assembled from units.
- [x] The standalone-run assembly path, its tiling check, and the caller-asserted consumed width it
      used are deleted.
- [x] The trailing-content reconstruction is deleted, and the verdict it restored is unchanged for
      every Source that had one.
- [x] A two-Cell spelling is never read across a row edge.
- [x] The rebuild-equivalence property still holds: a rebuilt Map equals the Map a full build would
      have made.
- [x] The parse boundary rule is recorded in an ADR beside the Comment rule.

## Comments

This absorbs `language-map/08`, which describes the same change. That ticket should be closed as
absorbed rather than worked separately.

Recorded as ADR 0033. Three deletions went further than the checkboxes asked, because keeping them
meant keeping a rule to explain them:

- `AnalysisStatus::Incomplete` and `Record::Incomplete` are gone. They only ever meant "the fragment
  ended", and a row does not end mid-Expression: a spelling or a slot the row edge cuts short is
  refused like any other. Two states, not three.
- `SourceAnalysis::consumed()` is gone and `cells()` replaces it. A width has to be re-based against
  an anchor; an address does not.
- `analyze` is total. The capacity bound is a fact about the Expression rather than about where it
  ends, so it is reported as an invalid Expression of the Cells that were read, and the row walk
  needs no second rule for how far to advance. `try_parse` still refuses.

`root_layout`'s two whitespace-era guards went with them: the `span_width > layout_width` check that
produced "trailing Source makes this Expression structurally unstable", and the
`offset <= span_width` filter `8e7bdce` added. Both compared an arity claim against a whitespace
run, and there is no whitespace run.

**What that costs, and who owes it.** With the slot filter gone, a half-typed `!>00` claims all eight
Cells its arity declares, so a `**` written into Cells 6-7 is its Note operand rather than an
activation and the Play root beneath takes no turn.
`a_half_typed_function_claims_every_cell_its_arity_declares` states this. The claim is right — one
derivation, and both readings now agree the Cell is a slot — but the Tick is silent about it, and
ADR 0032 requires that verdict before publishing. `cell-indexed-parse/03` draws it; until then this
Source is refused without saying so.
