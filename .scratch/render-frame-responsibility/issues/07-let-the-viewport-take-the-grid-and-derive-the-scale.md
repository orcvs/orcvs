# 07 — Let the viewport take the Grid, and derive the scale it is handed

**What to build:** the geometry layer stops accepting parameters it can compute from parameters it
already has. Two instances, one ticket, because they are the same fault: a value is unpacked or
recomputed at the call site and then passed alongside the thing it came from, so a caller can pass a
pair that disagrees.

## 1. `SourceShapes::new` takes a scale and the viewport it came from

`show_source` computes the scale from the viewport:

```rust
let scale = grid.cell_size / CELL_SIZE;
```

(`console/src/console.rs:742`.) It then hands both to the shape step:

```rust
let shapes = SourceShapes::new(&paint, &grid, &table, scale, pixels_per_point);
```

(`console/src/console.rs:760`; the signature is `console/src/console.rs:562-568`.) So the function
receives a viewport and a number derived from that viewport, and nothing makes the second agree with
the first. A caller passing `1.0` with a viewport at any other Cell size gets Grid lines and sector
seams a zoom level wide, painted on Cells of a different size, with no error anywhere. The test
helper already writes the same line a second time (`console/src/console.rs:1825`), which is the
duplication the defect invites.

**The trap: `scale` is taken.** `GridViewport::scale(&self, source: Rect) -> f32`
(`console/src/grid_viewport.rs:23-32`) already exists and means something different — the fit from
the Source's own coordinates onto presented points, used by `fit_transform` (`:48-55`) and by the
fitted-zoom computation at `console/src/console.rs:858`. **Do not overload it and do not rename it.**
The new accessor needs its own name for what it answers — the presented Cell side over the Source's
own Cell side, which is what the stroke widths are multiplied by. `cell_scale()` reads honestly;
pick better if there is better.

**The second trap: `CELL_SIZE` lives in `console.rs`.** It is a private const
(`console/src/console.rs:22`), and `grid_viewport.rs` mentions it only in a doc comment (`:335`).
Moving the division into `GridViewport` means moving or sharing the constant. If that reads worse
than it fixes, the acceptable smaller answer is for `SourceShapes::new` to compute
`viewport.cell_size / CELL_SIZE` itself and drop the parameter — the disagreement is closed either
way, and closing it is the point.

## 2. The Grid is unpacked into two loose `usize`s, then re-guarded

`Grid` is `Copy` (`orcvs/src/grid.rs:94`) and `Grid::new` asserts both counts are non-zero
(`orcvs/src/grid.rs:139-140`), so a Grid with no columns is unrepresentable. Every entry point in
`grid_viewport.rs` takes the counts loose anyway:

- `grid_viewport(available: Rect, columns: usize, rows: usize)` (`console/src/grid_viewport.rs:133`)
- `presented_grid(to_global, source, columns: usize, rows: usize, pixels_per_point)` (`:182-188`)
- `GridViewport::cell_at(&self, point, columns: usize, rows: usize)` (`:83-88`)
- and `source_bounds(columns: usize, rows: usize)` next door (`console/src/console.rs:116`)

Having unpacked the guarantee, the module then re-establishes it by hand. `cell_at` opens with:

```rust
if columns == 0 || rows == 0 || !(self.cell_size.is_finite() && self.cell_size > 0.0) {
    return None;
}
```

(`console/src/grid_viewport.rs:89-91`), and `presented_grid` writes `columns.max(1)` (`:197`). Both
defend against a state the type system had already made impossible — and the defence is not free: the
second assertion of `a_viewport_with_no_area_answers_no_cell` passes `0, 0` explicitly
(`console/src/grid_viewport.rs:461-464`), so the impossible state has a test keeping it alive.

Every production call site has the `Grid` in hand when it unpacks it: `console/src/console.rs:827,
835, 891-896` all read `source_grid.columns()` and `source_grid.rows()` off the `Grid` bound at
`:710`, and `console/src/console.rs:774` does it inside the click hit-test. Take the `Grid`.

**What this does not touch:** the finiteness guards on `cell_size` and `pixels_per_point`. Those
defend against real states — a console with no area, a degenerate device scale — and the comments at
`console/src/grid_viewport.rs:191-196` explain why. Only the count guards go.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] `SourceShapes::new` no longer takes a `scale` parameter; it derives it from the viewport, or
      the viewport answers it through an accessor that is **not** named `scale`.
- [ ] `console/src/console.rs:742` and its duplicate in the test helper at `:1825` both go.
- [ ] `grid_viewport`, `presented_grid`, `cell_at` and `source_bounds` take a `Grid`.
- [ ] The `columns == 0 || rows == 0` guard in `cell_at` and the `columns.max(1)` in
      `presented_grid` are deleted, along with the zero-count assertion in
      `a_viewport_with_no_area_answers_no_cell` (`console/src/grid_viewport.rs:461-464`). The test
      itself stays: its first assertion is about a console with no area, which is a real state. A
      comment where the guard was may state that `Grid` makes the other one unrepresentable — one
      line, not a paragraph.
- [ ] The `cell_size` and `pixels_per_point` finiteness guards are untouched, and so are their
      tests.
- [ ] `grid_viewport.rs` keeps testing without a window. If taking a `Grid` forces an `Orcvs` into
      any test in that module, stop and report — the module's whole premise is that the fit is
      "settled by arithmetic a test can ask about without a window"
      (`console/src/grid_viewport.rs:1-7`), and `Grid::new` alone must be enough.
- [ ] Nothing about what is drawn changes.

## Verification

```sh
cargo fmt --all -- --check
cargo clippy --package console --all-targets --locked -- -D warnings
PROPTEST_CASES=32 cargo nextest run --package console --locked
```

Nothing here touches `orcvs` and `console` has no dependents. No doctest run is owed: every item
named is `pub(crate)` or private.

## Comments

`ready-for-agent`. Both changes are mechanical, the two traps that could turn them into a mess — the
taken name `scale` and the `console.rs`-private `CELL_SIZE` — are named above with an explicit
smaller fallback for the second, and the one thing that would signal the change has gone wrong (a
window creeping into `grid_viewport.rs`'s tests) is an acceptance bar rather than something to
discover in review.

Filed as one ticket rather than two. They are the same sentence about the same layer — the geometry
module should take the thing, not a copy of a fact about the thing — and splitting them would put two
signature edits to the same call site in two commits.
