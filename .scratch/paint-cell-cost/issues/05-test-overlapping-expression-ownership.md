# 05 — Test overlapping Expression ownership in the Claim index

**What to build:** A test for the branch where a later Span clears an earlier claim's ownership of a Cell. `03` required focused tests for overlapping Expression ownership, and none exercises it.

**Blocked by:** None — can start immediately.

**Status:** needs-triage

- [ ] A Source whose Expressions overlap reaches `by_index[index] = None` (`orcvs/src/source/language_map.rs:422-424`). The test asserts which Cells keep which Claim, and what their written state reads through `RenderCell`.
- [ ] The test fails when that clearing line is removed. Confirm this by breaking it once, then revert.
- [ ] `cargo nextest run --package orcvs --locked` and `--package console` pass.

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
`None`, and `by_index[index] = None` overwrites `None` with `None`. No test guards that
disjointness. The property
`deriving_a_language_map_partitions_every_row_at_the_cell_recovery_resumes_from` asserts that no
Cell belongs to two Language Units, but it walks `map.units()` and never reads
`map.expressions()`, and a Function's Expression Span covers several Language Units. Overlapping
Expression Spans would pass it.

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
documented defence against a future Parser whose Expressions may overlap. Deleting it should come
with a property over `map.expressions()` asserting their Spans are disjoint, so a Parser change
that breaks the argument above fails a test instead of silently changing ownership. Either way, `03`'s
"overlapping Expression ownership" criterion describes a case the language does not produce. No
test was added, because the only way to reach the branch is a seam in shipped code, which the
repository contract forbids.
