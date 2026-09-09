# 04 — Prove the Expression Span law

**What to build:** One property test over Expression Spans. It must prove that the Spans of one
row do not overlap, do not cross a row edge, and name Cells the Grid can answer for.

**Blocked by:** None.

**Status:** resolved

**Tags:** release/v1

## Why this exists

An Expression Span is the group of Cells that one Expression uses.

The Language Map already has a property test for Language Units. It is
`deriving_a_language_map_partitions_every_row_at_the_cell_recovery_resumes_from`
(`orcvs/src/source/language_map.rs:1364`). It keeps one entry per Cell and proves that each Cell is
named exactly once. That test is complete for units.

No property test looks at Expression Spans. The second property,
`every_expression_and_diagnostic_answers_to_the_revision_that_derived_it` (`language_map.rs:1455`),
reads one Expression at a time. It proves that an Expression's units sit inside its own Span. It
never compares one Expression Span with another.

Expression Spans are proved only by hand-written examples. They use the `expression_spans` helper
(`language_map.rs:795`) and sit at `language_map.rs:1014-1091`.

ADR 0033 makes this gap matter more than before. The row walk used to decide where an Expression
stops. The Parser decides now, and `walk_row` continues at the Cell the Parser reports. So the code
trusts the Parser. A property must prove that the trust is correct.

A unit Span is always two Cells, so a unit test finds errors that an Expression Span test does not.
An Expression Span can hold Cells that make no unit. `a_cell_that_produces_no_language_unit_is_still_its_own_span`
(`language_map.rs:1036`) shows one. So the unit property does not prove the Expression Span law.

## Checklist

- [x] Two Expression Spans in the same row do not overlap. Their start Cells increase.
- [x] An Expression Span does not cross a row edge. Its first and last Cell are in the same row.
- [x] Every Expression Span start and end is a Cell number the Grid can answer for.
      Assert this on the raw Cell numbers. Do not assert it with `positions()`. `Span::indices()`
      removes any Cell number the Grid rejects, so a Span that runs past the end of the Grid passes
      a `positions()` sweep and reports nothing. The Diagnostic arm of the second property already
      makes this mistake impossible for Diagnostics (`language_map.rs:1527-1538`). Copy that shape.
- [x] Every Cell that is not empty answers a Glyph. Only an empty Cell answers `None`. The Language
      Map assigns no Marker Glyph, no Highlight Glyph, and no Space Glyph: the first two come from
      the UI helpers, and Space is the fallback of the render frame. The Map gives `Glyph::Char` to
      any non-empty Cell that no entry claimed (`language_map.rs:322-326` in `build`, and
      `language_map.rs:279-283` in `rebuild`). Today only two examples state this
      (`language_map.rs:1102-1106` and `language_map.rs:1175`).
- [x] The test stays inline in `language_map.rs`. It reads `pub(super)` items.
- [x] Reuse the `revision()` generator (`language_map.rs:1342`). Do not write a second one.

## Coordination

`sequence-values/07` edits the same property block. It adds the Comment to the unit property and
changes the two-Cell assertion at `language_map.rs:1378`. Do both tickets together if you can. One
person then opens the file once.

## Comments

2026-09-09: Audited against the tree. The original ticket asked for the unit partition law from
ADR 0018. Commit `007cee1` (2026-09-06) delivered that law as
`deriving_a_language_map_partitions_every_row_at_the_cell_recovery_resumes_from`, and the ticket was
never updated. Four items were removed:

- *Building the map twice gives the same result.* `a_rebuilt_map_equals_the_map_a_full_build_would_have_made`
  (`language_map.rs:1739`) proves the stronger incremental-versus-full equality. The ticket already
  said to reconcile rather than restate.
- *`prospective_expression_range` drift.* The function was deleted in `28c5a85`.
- *Put `***`, `<<<` and `^^^^` in the generator.* `language_map_partitions_complete_units_left_to_right_without_overlap`
  (`language_map.rs:909`) already asserts the exact ADR 0018 answer for all three by example. A
  generator literal would add a coverage count, not a claim.
- *The units partition each row.* Delivered by the property named above.

The Glyph item was kept but corrected. The ticket said that an unclassified Cell answers `None`, and
the reconciliation in `b0eb0ad` repeated it. The code does the opposite: it gives `Glyph::Char` to
every non-empty Cell that nothing else claimed. Only an empty Cell answers `None`.

The three surviving items are all about Expression Spans, and the file was renamed to say so.
`v1-release/03` lists this ticket as a blocker, so the smaller scope moves the release path.

2026-09-09: Landed as `expression_spans_are_disjoint_and_name_cells_the_grid_can_answer_for`,
a third `#[test]` in the existing `proptest!` block of `mod property` in
`orcvs/src/source/language_map.rs`. It draws from `revision()`; no second generator was written.

What it asserts, per Expression the Map holds: `start <= end`; `end < grid.count()` and
`start / cols == end / cols`, both on the raw Cell numbers the Span carries rather than through
`positions()`, exactly as the Diagnostic arm of the second property does; strictly increasing
start Cells; and no Cell claimed twice, recorded in a `claimed` vector over raw Cell numbers so
non-adjacent Spans are compared too, not only neighbours. A closing sweep over the Grid's
Positions asserts that every Cell whose byte is not the space answers a Glyph.

The Glyph claim was verified against the code before it was written, and the Comments above are
right: `build` and `rebuild` both give `Glyph::Char` to every non-empty Cell nothing else claimed,
so only an empty Cell can answer `None`. The converse is false —
`expression_layout_retains_slots_beyond_invalid_and_missing_source` shows a positioned entry over
empty Cells, which `record_expression` then classifies — so the property states only the direction
that holds, and the doc comment says why.

Three of the four laws were confirmed live by mutating the implementation locally and watching the
property fail, then reverting: resuming the walk at `cells.start + 1` instead of `cells.end`
produced overlapping Spans (`".^" gave Cell 1 to two Expressions`); walking the whole Source as one
row produced a Span across a row edge (`".|" spanned 0..=1 across a row edge`); disabling the
`Glyph::Char` fallback in `build` left a Cell unclassified (`"##" left the non-empty Cell 0
unclassified`). The Grid-bounds law could not be broken the same way: `walk_row` mints a Span's
Cells through `grid.cell_index(..).expect(..)`, so an out-of-Grid Span panics upstream before it
can reach the assertion. The assertion is the second net behind that `expect`, which is the shape
the checklist asked for.

A temporary coverage probe over 256 generated revisions confirmed the property is not vacuous:
234 cases held at least one Expression Span, 156 held a Span of more than one Cell, 177 held two
or more Spans in one row, 224 held Span Cells beyond what the Language Units account for, and
1186 Spans were checked in total. The probe was removed rather than kept —
`generated_revisions_cover_empty_cells_comments_and_complete_expressions` already guards the
generator, and a second fixed-256-case guard would cost derivations for a claim it already makes.

`sequence-values/07` still has to touch this block; it was left alone here.
