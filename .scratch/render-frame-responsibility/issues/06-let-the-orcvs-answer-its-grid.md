# 06 — Let the Orcvs answer its Grid, and retire what the tests bent around

**What to build:** `Orcvs::grid() -> Grid`; the retirement of `Orcvs::index()`; and a rename for a
test whose title has been false since `RenderFrame::grid()` was made public.

Three small facts about one crate's public surface. They are filed together because they are one
finding — the shape of `Orcvs`'s API was settled by what its callers could reach rather than by what
they were asking for — and separately they are each two lines.

## 1. There is no `Orcvs::grid()`, so tests derive a Render Frame to name a coordinate

`grid` is a private field of `Orcvs` (`orcvs/src/app.rs:66`) with no accessor. Inside the crate that
is fine; `orcvs`'s own tests write `let grid = app.grid;` and then `grid.position(x, y)`
(`orcvs/src/app.rs:664-665`). Outside it, a caller that wants to name a coordinate has to build a
whole Render Frame and index into it:

```rust
orcvs.select(orcvs.render_frame().rows()[2][x + 1].position());
```

(`console/src/paint.rs:367`.) There are 13 sites spelled that way in `console`
(`console/src/paint.rs:367, 370, 441, 484, 521, 572, 575, 641` and
`console/src/console.rs:1957, 2038, 2266, 2269, 2748`). Each one derives every Cell of the Grid,
allocates a `Vec` per row, computes a bloom band and two seam strengths for every Position, and then
reads one `Position` out of the result — for a value that is a function of two integers.

**`Orcvs::grid()` closes nothing.** `Grid` is `Copy` (`orcvs/src/grid.rs:94`), it mints only valid
Positions, and it is already publicly reachable from the same object by
`orcvs.render_frame().grid()` — the accessor `source-paint/03` added. The only thing the private
field buys is that the reachable route is the expensive one.

There is a second cost, in the type's own documentation. `Orcvs`'s doctest at
`orcvs/src/app.rs:47-61` demonstrates that "only a Grid mints" a Position — by constructing a
**different** Grid:

```rust
let orcvs = Orcvs::new(16, 16);
let grid = Grid::new(16, 16);
```

Those two Grids have different `GridId`s, so every Position the doctest mints is one the `Orcvs`
beside it would refuse (`Grid::owns`, `orcvs/src/grid.rs:222`; the behaviour is pinned by
`test_grid_refuses_a_position_minted_by_another_grid`, `:483`). The doctest cannot call `select` with
what it built, and does not. A doctest demonstrating Position provenance with a lookalike Grid is the
clearest statement that the real one should be reachable.

## 2. `Orcvs::index()` is public with no non-test caller

`pub fn index(&self, position: Position) -> usize` (`orcvs/src/app.rs:213-215`) is a one-line
forward to `self.grid.index(position).get()`. Its only callers in the workspace are
`orcvs/src/app.rs:510` and `:514`, both inside `app.rs`'s own `#[cfg(test)]` module, in
`test_to_idx`. Nothing in `console`, nothing in `lang`, nothing in `shell`.

It also un-mints what `Grid` was careful to mint: `Grid::index` answers a `CellIndex`
(`orcvs/src/grid.rs:186`), a type that exists so an index cannot be confused with an arbitrary
`usize`, and this method strips it with `.get()` before handing it out. A public method that exists
only to make a checked type unchecked, for tests.

Retire it. `test_to_idx` is testing `Grid::index`, which has its own tests
(`test_grid_converts_positions_to_indices_in_row_order`, `orcvs/src/grid.rs:464`), so it can call
`app.grid.index(position).get()` directly from inside the crate, or go.

## 3. A test whose title is false

`app_exposes_a_render_frame_without_leaking_its_grid_or_cursor` (`orcvs/src/app.rs:423-438`) asserts
what the Render Frame contains, and its title has been false since `source-paint/03` made
`RenderFrame::grid()` public (`orcvs/src/render_frame.rs:125-127`). A Render Frame now answers its
Grid by design — that was the point of the ticket — and it answers the Cursor's Cell too, through
`RenderCell::selected()` (`:47-49`), which this very test asserts at `:437`.

The test itself is fine; it checks that `render_frame` reflects a write and the selection. Only the
name is wrong, and a name asserting the opposite of what the code does is worse than no name. Rename
it for what it checks, or fold it into the test beside it. Do not "fix" it by hiding `grid()`.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] `Orcvs::grid(&self) -> Grid` exists and is documented: the Grid this running Orcvs's Source
      occupies, and the only thing that mints a Position `select` will accept.
- [x] The 13 `orcvs.render_frame().rows()[r][c].position()` sites in `console` name their coordinate
      through `Orcvs::grid()` instead. A site that genuinely wants a Cell, not a coordinate, keeps
      its Render Frame and says so.
- [x] `Orcvs`'s doctest at `orcvs/src/app.rs:47-61` uses `orcvs.grid()` rather than a second
      `Grid::new(16, 16)`, and therefore can — and does — hand the Position it mints to `select`.
      That is the point the doctest was making and could not previously demonstrate.
- [x] `Orcvs::index` no longer exists. `test_to_idx` either goes through `Grid` from inside the
      crate or is deleted as covered by `orcvs/src/grid.rs:464`.
- [x] `app_exposes_a_render_frame_without_leaking_its_grid_or_cursor` is renamed to what it asserts,
      or merged into a neighbour. Nothing is made private to rescue the old name.
- [x] No behaviour change anywhere. This ticket adds one accessor, removes one, and renames a test.

## Verification

```sh
cargo fmt --all -- --check
cargo clippy --package orcvs --all-targets --locked -- -D warnings
cargo clippy --package console --all-targets --locked -- -D warnings
PROPTEST_CASES=32 cargo nextest run --package orcvs --locked
PROPTEST_CASES=32 cargo nextest run --package console --locked
cargo test --workspace --doc --locked
```

The doctest run is owed twice over: `orcvs`'s public surface changes in both directions, and the
`Orcvs` doctest is itself edited.

## Comments

`ready-for-agent`. Each of the three is independently checkable against a named line, none of them
needs the seam criterion settled, and none changes what is drawn or computed.

Worth doing early whatever happens to the rest of the effort: ticket `03` has to rewrite 48
`rows()[..]` sites, and most of the `console` ones are here because there was no other way to ask for
a coordinate. Landing this first turns a large part of `03`'s churn into a deletion.

Sixteen `render_frame().rows()[r][c].position()` call sites in console (ten paint, six console)
became `grid().position(c, r)` — the ticket's thirteen was written against an earlier paint
surface; every coordinate site went. One assertion that already held a Frame now compares
`paint.cursor()` to `frame.cursor()` rather than indexing the origin Cell.
