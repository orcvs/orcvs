# 03 — Let the Render Frame answer its Grid

**What to build:** `RenderFrame` stores the `Grid` it was derived from and answers it, and `source_dimensions` is deleted.

`RenderFrame::derive` already holds `source.grid()` and `Grid` is `Copy`, so the fact is free to keep. Today it is dropped, and `console/src/console.rs:116-128` recovers the column and row counts from `rows[0].len()` behind an `expect("a Render Frame contains at least one row")` and a `debug_assert!` that every row has equal length — two invariants the Grid guarantees, re-derived twice per Render Frame.

**Blocked by:** 02

**Status:** resolved

- [x] `RenderFrame` stores `grid: Grid` and answers it.
- [x] `source_dimensions`, its `expect` and its `debug_assert` are deleted.
- [x] Its three callers ask the Grid instead: `show_source:587`, `show_source_scene:820`, and the test at `console.rs:1687`. The test module's import of it at `console.rs:1131` goes with the function.
- [x] `RenderFrame::rows()` still answers `&[Vec<RenderCell>]` and is not flattened. Two `Orcvs` doctests and dozens of assertions index it as `rows()[i][j]`; that is the right shape for a type whose consumer iterates in row order.
- [x] No change to what is painted.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D warnings` and `PROPTEST_CASES=32 cargo nextest run --package <crate> --locked` for `orcvs` and `console`.
