# Collapse ExpressionMap into a single Expression-extent seam

**Status:** superseded by `language-map` and the decided Expression/runtime invariant.

**Tags:** Improvement

This earlier Expression-extent seam is retained as planning history. The Language Map effort now
owns the single deep Source-derived interface, including Expressions, roots, Language Units,
Footprints, diagnostics, and valid evaluable entries; no competing implementation tickets should
be created from this spec.

## Problem Statement

The Source module answers "what Expression does this Cell belong to" two different ways. `ExpressionMap` maintains that answer incrementally, with each Expression's extent held as an `Arc<RwLock<Range>>` that gets joined and split as Cells are set and unset. But `Source::rebuild_derived_state` never actually edits an existing `ExpressionMap` — it discards the map and rebuilds it from an empty state, in Source order, on every accepted edit. Under that access pattern, the map's own `join_exp` (merging two existing Expressions across a filled gap) can never run — a rebuild always encounters an empty right neighbour — and `split_exp` is reachable only through `ExpressionMap::unset`, a method gated `#[cfg(test)]` and never called from production code at all.

Meanwhile `Source::check_expression_capacity`, which has to answer the same question before an edit is committed (the map isn't updated yet), answers it a second, independent way: a hand-written outward scan over a prospective byte buffer, bounded by a separately-written `indices_share_a_row` check. Nothing enforces that this scan and `ExpressionMap`'s incremental logic agree about where an Expression starts and ends. A future change to Expression-boundary rules — the still-unimplemented `Comment` term in `CONTEXT.md`, for one — would have to be threaded through both algorithms by hand, with no test that would catch the two drifting apart.

## Solution

Replace `ExpressionMap`'s incremental, `Arc<RwLock>`-backed join/split machinery with a single pure row-scan: given a row's bytes, split it into contiguous non-space runs and produce a `Range` per Cell in each run. `ExpressionMap` keeps its name and its `get(idx)` lookup, but its constructor becomes one full-grid pass built from that row-scan, replacing the old empty-then-incrementally-set protocol. `check_expression_capacity` stops re-deriving Expression boundaries by hand and calls the same row-scan primitive on the one row it needs, so there is exactly one algorithm that decides where an Expression starts and ends, invoked at two points in the Source lifecycle (before an edit commits, and after).

## User Stories

1. As a maintainer reading `expression_map.rs`, I want the module's job stated by two small methods (`build`, `get`), so that I can understand what it does without tracing `Arc<RwLock>` mutation through `join_exp`/`split_exp`.
2. As a maintainer changing what delimits an Expression (e.g. adding `Comment` truncation), I want exactly one function that decides an Expression's extent, so that I only have to change the rule in one place.
3. As a maintainer, I want `check_expression_capacity` and `ExpressionMap`'s builder to agree by construction, not by convention, so that a future edit to one can't silently desynchronize from the other.
4. As a maintainer reviewing `Source::set`, I want the capacity pre-check to allocate proportionally to one row rather than the whole Source, so that editing a large Source stays cheap per keystroke.
5. As a maintainer writing a test against `ExpressionMap`, I want to assert on `build`/`get` outputs for a given row's bytes, so that my test describes behaviour and survives the next internal refactor.
6. As a maintainer, I want `Range` to make an inverted extent (`end < start`) unrepresentable, so that a construction mistake fails at the point of construction, not as a confusing panic deep inside `get_exp_src`'s slicing.
7. As a maintainer, I want `Grid` to answer "how many columns" directly, so that `Source`-side code doesn't reconstruct row width by round-tripping through `Position`.
8. As a reviewer of this change, I want the diff to touch only `console/src/source/expression_map.rs`, `console/src/source/model.rs`, and `console/src/grid.rs`, so that I can confirm it doesn't reach across the Source/Playback Engine seam that ADR-0001 and ADR-0002 established.
9. As a future contributor implementing the `Comment` term, I want the Expression-extent scan to be the one place that recognizes what stops a run, so that adding `#`-truncation is a change to one function's rules, not two.
10. As a maintainer running the existing Source test suite, I want every currently-passing behavioural test (joins, splits, prepends, row-edge truncation, diagnostics, Tick Plans) to keep passing unchanged, so that this is verified as an internal refactor with no observable behaviour change.

## Implementation Decisions

- `ExpressionMap` (`console/src/source/expression_map.rs`) becomes `pub struct ExpressionMap { inner: Vec<Option<Range>> }`. `ExpressionRange`, its `Arc<RwLock<Range>>` backing, `join_exp`, `split_exp`, `set_inner`, `set`, and the `#[cfg(test)]`-only `unset` are all removed.
- `ExpressionMap::new(grid)` is removed. The sole constructor is `ExpressionMap::build(grid: Grid, bytes: &[u8]) -> Self`, which uses `grid.cols()` to walk `bytes` one row at a time and fills `inner` from a shared row-scan.
- A row-scan primitive (module-private or `pub(super)`, whichever `check_expression_capacity` needs to call it directly) takes a row's starting absolute index and its bytes, and returns one `Option<Range>` per byte: `None` for a space, `Some(Range)` — using absolute Source indices, not row-local ones — for every Cell in a contiguous non-space run. This is the one place that decides where an Expression starts and ends.
- `ExpressionMap::get(idx) -> Option<Range>` returns a plain copy (`Range` becomes `Copy`), not a clone through a lock.
- `Range` gets private `start`/`end` fields, a `Range::new(start, end)` constructor that asserts `start <= end` (a proven invariant of the scan, not user input — an assertion, not a `Result`), and `start()`/`end()` accessors. Every `range.start`/`range.end` field read in `model.rs` (`get_exp_src`, `plan_tick`, `expression_starts`, diagnostic construction, etc.) becomes a method call.
- `Grid` (`console/src/grid.rs`) gains `pub fn cols(&self) -> usize`, returning the column count it already stores privately.
- `Source::check_expression_capacity` (`model.rs`) stops cloning the whole Source buffer and walking outward by hand. It computes the affected row's start as `(idx / grid.cols()) * grid.cols()`, clones only that row's bytes, overlays the prospective byte, and calls the shared row-scan to get the one `Range` containing `idx` directly — no more hand-written `indices_share_a_row` walk.
- `Source::rebuild_derived_state` and `Source::new` both construct the map with a single `ExpressionMap::build(self.grid, self.inner.as_bytes())` (or the equivalent at construction time), replacing the old two-step "make an empty map, then `set` every occupied index" loop.

## Testing Decisions

- Good tests here assert on `ExpressionMap::build`/`get` outputs for given row inputs, or on `Source`'s existing public behaviour — never on the removed internal join/split mechanics.
- `expression_map.rs`'s existing `#[cfg(test)] mod test` (`test_expression_join`, `_split`, `_prepend`, `_replace`, `_delete_last`, `_edit`) is deleted in full — it tests the incremental protocol this change removes. Replacement tests target `build`/`get` directly: an empty row, one run, multiple runs separated by a gap, and a run that touches the row's edge. Prior art for the assertion style (`assert_range`/`assert_none` helpers, `trace()` setup) already exists in this same module and should be reused where it still fits the new shape.
- Every test in `console/src/source/model.rs`'s `#[cfg(test)] mod test` and `console/src/source/mod.rs`'s `#[cfg(test)] mod tests` must keep passing unchanged — they exercise `Source::set`/`unset`/`execute` at the public seam this change does not move, including `test_set_rejects_an_expression_beyond_parser_capacity_without_mutation` and `rejected_overlong_expression_does_not_poison_source_access`, which are the existing coverage for `check_expression_capacity`.
- No new test seam is introduced. `ExpressionMap` and `check_expression_capacity` are both crate-private already; the deepened interface is tested exactly where the shallow one was.

## Out of Scope

- The `Comment` term from `CONTEXT.md` remains unimplemented; this change only makes it cheaper to add later.
- The MIDI note-table duplication, `App`'s diagnostic-routing duplication, `App::event_handler` test coverage, and the dead-code/boilerplate cleanup identified in the same architecture review are separate candidates and not part of this spec.
- No ADR is warranted: this is an internal, easily-reversible representation change behind an already crate-private seam, not a decision future readers would need explained.
- No change to `Source`'s public behaviour, to the Source/Playback Engine seam (ADR-0001, ADR-0002), or to any Grid method other than the new `cols()` accessor.

## Further Notes

This spec implements Candidate 1 from the 2026-08-29 architecture review (`/mattpocock-skills:improve-codebase-architecture`), combined with Candidate 2 (the `check_expression_capacity` duplication) per the grilling session's decision to fold both into one change rather than land them separately.
