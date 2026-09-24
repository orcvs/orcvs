# 05 — Decide whether `claims_by_cell` keeps its clearing loop

**What to build:** Delete the loop in `claims_by_cell` that clears an earlier claim's ownership of the Cells a later Expression's Span covers, and replace the protection it was meant to give with a test-only guard on the invariant that actually makes it dead: every positioned entry lies inside its own Expression's Span. The issue opened asking for a test of that branch, because `03` required focused tests for overlapping Expression ownership. No reachable Source overlaps (see Comments), so the branch cannot be tested without a seam in shipped code, which the repository contract forbids.

Disjoint Spans alone do not make the clear dead. It changes an outcome only when an earlier Expression's entry lands inside a later Expression's Span, and disjoint Spans do not exclude an entry that escapes its own Span. `expression_spans_are_disjoint_and_name_cells_the_grid_can_answer_for` compares Spans with Spans and never entries with their Span, so today a Parser that recorded an entry outside its Span would change ownership without failing any test. Keeping the loop would not defend against that Parser either: `entry_at` reads only the entries of the Expression whose Span holds the Cell, so `claims_by_cell` and `entry_at` would disagree with or without the clear. The invariant to guard is entry-within-Span, and once it holds, the clear and the "a later Expression owns" wording are both dead.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] `expression_spans_are_disjoint_and_name_cells_the_grid_can_answer_for` (`orcvs/src/source/language_map.rs:2088`) also asserts, for every Expression, that each `expression.positioned()` entry with non-empty `cells` lies within `span.start()..=span.end()`. It fails when the Parser records an entry outside its Expression's Span; confirm this once by widening an entry's range in `lang/src/parser.rs`, then revert.
- [x] The clearing loop in `LanguageMap::claims_by_cell` (`orcvs/src/source/language_map.rs:422-424`) is deleted.
- [x] The docs on `LanguageMap::claims_by_cell`, `LanguageMap::entry_at` and `RenderFrame::expression_at` no longer say a later Expression owns the Cells its Span covers. They say Expression Spans are disjoint and each positioned entry lies inside its own Span, so at most one Expression and at most one claim answer for a Cell, and name the property that guards both.
- [x] `03` gains a comment recording that its "overlapping Expression ownership" criterion describes a case the language does not produce, and pointing here. Its ticked box stays as history rather than being unticked. (Landed on `main` as `03`'s 2026-09-24 correction.)
- [x] `PROPTEST_CASES=32 cargo nextest run --package orcvs --package console --locked`, plus the scoped `cargo fmt` and `cargo clippy` gates for `orcvs` and `console`, pass.

## Answer

**2026-09-24.** The clearing loop in `LanguageMap::claims_by_cell` is deleted, and `expression_spans_are_disjoint_and_name_cells_the_grid_can_answer_for` now also asserts, for every Expression, that each `expression.positioned()` entry with non-empty `cells` lies within `span.start().get()..=span.end().get()`. Together with the existing disjointness check, that is the invariant the clear depended on, so the loop no longer defends anything the property does not already guard.

The docs on `LanguageMap::claims_by_cell`, `LanguageMap::entry_at` and `RenderFrame::expression_at` no longer say a later Expression owns the Cells its Span covers. They say Expression Spans are disjoint and each positioned entry lies inside its own Span, so at most one Expression and at most one claim answer for a Cell, and they name the property that guards both. `entry_at`'s `.rev()` walk and early `return None`, and `expression_at`'s `.rev().find`, are left as they are: with disjoint Spans the order is immaterial and the early return is an exit, not a precedence rule. A search of `orcvs/`, `console/`, `lang/`, `docs/` and `CONTEXT.md` for other prose about a later Expression owning Cells, or overlapping Expression ownership, found none outside these three items and the `.scratch/` history.

Sabotage. With the Function arm of `Parser::take_language_unit` (`lang/src/parser.rs`) recording `cell_start.saturating_sub(1)..self.start + self.consumed()`, starting a Function's entry one Cell before its Expression's Span, `PROPTEST_CASES=32 cargo nextest run --package orcvs --locked -E 'test(expression_spans_are_disjoint)'` failed on the new assertion with `Test failed: "A||~%8169" labelled Cells 2..5 outside its Expression Span 3..=5`, minimal input `(cols, rows, source) = (3, 3, "A||~%8169")`. The parser change was reverted and `git diff lang/` is empty. With the loop deleted and the parser unmodified, `PROPTEST_CASES=32 cargo nextest run --package orcvs --package console --locked` passed all 1067 tests.

## Comments

**2026-09-23 — opened by the audit of the merged pull requests against their issues.** #115 added two `render_frame` tests, neither with overlapping Spans.

**2026-09-23 — no reachable Source overlaps; the clearing line never changes a Cell.** The first
criterion cannot be met through the shipped entry points, so the other two cannot either, and the
issue returns to triage for a decision about the line rather than a test.

Why Spans cannot overlap. `walk_row` hands the Parser the row from `idx` and resumes at
`analysis.cells().end`, so each row's Spans are disjoint and ascending. `Parser::take_language_unit`
records every positioned entry as `cell_start..start + consumed()`, and `consumed()` only grows, so
every entry lies inside its own Expression's Span. A refused Function rewinds only to one Cell past
its start. `rebuild` derives each dirty row the same way and carries clean rows whole. Rows never
share Cells. So when `claims_by_cell` reaches an Expression, every index in its Span is still
`None`, and `by_index[index] = None` overwrites `None` with `None`. The property
`expression_spans_are_disjoint_and_name_cells_the_grid_can_answer_for`
(`orcvs/src/source/language_map.rs:2088`) walks `map.expressions()` and fails when any Cell is given
to two Expressions, so a Parser change that let Spans overlap would fail it. The neighbouring
`deriving_a_language_map_partitions_every_row_at_the_cell_recovery_resumes_from` checks Language
Units, not Expression Spans, and is not the guard.

Probe (local, since removed). Before the clear, `claims_by_cell` asserted that each index was
`None`, and after it that each entry lay inside its Expression's Span. Nothing fired in:

- every 4-Cell row over `" -:!?.*/&#%^+<=>|~bcvx01C4GZ"`, which covers every Function spelling's
  characters: 614,656 rows through `LanguageMap::build` and `claims_by_cell`;
- 200,000 random 16x3 revisions over the same alphabet;
- `cargo nextest run --package orcvs --package console --locked --release` with
  `PROPTEST_CASES=2048`: 962 passed.

Sabotage. With the three clearing lines deleted, `PROPTEST_CASES=32 cargo nextest run --package
orcvs --package console --locked` passed all 960 tests. No Source can make a test fail there,
because deleting the line changes no output.

`entry_at` has the same shape: `return None` after the first Span containing the Cell. Spans are
disjoint, so that is an early exit, not a precedence rule.

Decision needed: delete the clearing loop and reword the "a later Expression owns the Cells its
Span covers" docs on `claims_by_cell`, `entry_at` and `RenderFrame::expression_at`, or keep it as
documented defence against a future Parser whose Expressions may overlap. Deleting it is safe
while `expression_spans_are_disjoint_and_name_cells_the_grid_can_answer_for` holds: a future
overlapping Parser would fail that property before it changed ownership. Either way, `03`'s
"overlapping Expression ownership" criterion describes a case the language does not produce. No
test was added, because the only way to reach the branch is a seam in shipped code, which the
repository contract forbids.

**2026-09-24 — audit of this issue; the decision is to delete the loop and guard entry-within-Span.** The claim that no reachable Source overlaps was checked against the code and holds. `walk_row` resumes at `cells.end` (`orcvs/src/source/language_map.rs:770`), and `analyze()` asserts forward progress. In `take_language_unit`, each entry starts at or after where the previous one ended, and the refused-Function arm rewinds to one Cell past `cell_start` before it records, so no recorded range passes the final `consumed`. `build` and `rebuild` split the Source with `chunks_exact` and derive or carry each row alone. The probe and sabotage figures above were not re-run; they rest on the comment, and the code reading supports them independently.

The earlier comment's case for deletion was incomplete. It said deleting the loop is safe while the disjointness property holds, but that property does not check entries against their Span, which is the condition the clear actually depends on (see What to build). The probe asserted entry-within-Span and was removed, so nothing guards it today. The criteria above replace the test this issue opened asking for with that guard. `03` is `resolved` with its overlap criterion ticked and no pointer here, which the fourth criterion corrects.
