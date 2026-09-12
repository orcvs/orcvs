# 03 — Store the Render Frame's Cells flat

**What to build:** `RenderFrame` holds one `Vec<RenderCell>` in row-major order beside the `Grid` it
already carries, and answers `at(Position) -> &RenderCell`. `rows()` goes.

## Two values, one shape, two answers

`RenderFrame` stores `Vec<Vec<RenderCell>>` (`orcvs/src/render_frame.rs:67-71`). `Paint`, derived
from it and covering the same Grid, stores a flat `Vec<CellPaint>` plus the `Grid` and indexes
through `Grid::index` (`console/src/paint.rs:80-84`, `:178-180`). Two values of the same shape,
stored two ways, one derived from the other.

`console/src/paint.rs:16-23` justifies the disagreement:

> This is deliberately unlike `RenderFrame`, which nests a `Vec` per row: a Render Frame's only
> consumer walks it in row order to paint it, and the nesting exists to serve exactly that.

**Check that against the file it is written in.** Eighty-odd lines below, the consumer it describes
does this:

```rust
let cells = frame
    .rows()
    .iter()
    .flatten()
```

(`console/src/paint.rs:105-107`.) It throws the nesting away in the same expression that reads it.
And when the row order *is* needed — `Paint::background_runs`, where a run must not cross a row
boundary — it is not recovered from the nesting at all, because by then there is none:

```rust
for (row, positions) in self.grid.positions_by_row().enumerate() {
```

(`console/src/paint.rs:220`.) The Grid answers it. So the sentence describes a consumer that does
not exist: the row nesting is read once, flattened immediately, and the row structure it was
supposedly serving is obtained from `Grid::positions_by_row()` afterwards.

The only other shipped reader of `rows()` is the click hit-test:

```rust
&& let Some(cell) = frame.rows().get(row).and_then(|row| row.get(column))
```

(`console/src/console.rs:775`.) That is `Grid::position(column, row) -> Option<Position>`
(`orcvs/src/grid.rs:156`) written out by hand, and it does not even need a Cell — it uses the result
only for `cell.position()` (`:776`). The Grid is already in scope three lines above, bound as
`source_grid` at `console/src/console.rs:710`, and used on the line immediately before at `:774`.
Flattening turns that whole `let`-chain arm into `source_grid.position(column, row)`.

## The shape was decided by test churn

`.scratch/source-paint/spec.md` records the reason for keeping the nesting, under the heading "The
Render Frame's shape":

> `RenderFrame` is **not** flattened, and that is not filed as a follow-up. `Vec<Vec<RenderCell>>` is
> right for a type whose only consumer iterates it in row order, and flattening would rewrite two
> `Orcvs` doctests and dozens of assertions to buy an `at(Position)` nobody has asked for.

Quote it honestly, because both halves deserve it. The first clause is the claim this ticket
disputes — the consumer does not iterate it in row order, as shown above. The second clause is
accurate and is the real reason: there are 48 `rows()[..]` indexing sites across the workspace
(`orcvs/src/render_frame.rs` 20, `orcvs/src/app.rs` 14, `console/src/paint.rs` 8,
`console/src/console.rs` 6), three of which are doctest lines in two `Orcvs` doctests
(`orcvs/src/app.rs:96-97` and `:128`).

That is a real cost and nobody should pretend otherwise. But it is a cost of *changing* the shape,
not an argument that the shape is right. **The storage shape of a shipped type was decided by the
churn in its assertions.** Write that down in the commit message, because it is the finding, and the
next reader is entitled to know the shape was not chosen for the type's own sake.

Ticket `06` removes most of the churn independently: `Orcvs::grid()` lets the 13 test sites spelling
`orcvs.render_frame().rows()[2][x + 1].position()` say `grid.position(x + 1, 2)` instead, which is
what they were reaching for all along.

## What else goes with it

`Grid::rows()` answers a count (`usize`, `orcvs/src/grid.rs:350`). `RenderFrame::rows()` answers
slices (`&[Vec<RenderCell>]`, `orcvs/src/render_frame.rs:136`). Both are public, both are reachable
from the same Render Frame, and the collision needed a doc comment to hold it apart:

> This is the count's counterpart, not its rival: [`Grid::rows`] answers how many rows the shape
> has, and this answers what stands in them.

(`orcvs/src/render_frame.rs:129-135`.) A doc comment written to stop two methods of the same name
being confused is evidence that one of them should have a different name — or, here, should not
exist. Flattening deletes the collision and the comment defending it.

**Blocked by:** 02

Real, not bookkeeping. Ticket `02` deletes the Cursor scan and one of the two assertions in
`the_cursor_is_the_selected_position`, both of which are written against `rows()`. Taking `03` first
means rewriting code that `02` then removes, and re-deciding what a deleted test should look like
flattened.

**Status:** needs-triage

- [ ] `RenderFrame` holds `Vec<RenderCell>` in row-major order and answers
      `at(Position) -> &RenderCell`, indexing through `Grid::index` exactly as `Paint::at` does
      (`console/src/paint.rs:178-180`).
- [ ] `RenderFrame::rows()` no longer exists, and neither does the doc comment holding it apart from
      `Grid::rows()`.
- [ ] `Paint::derive` walks `frame.grid().positions_by_row().flatten()` — or the Cells directly —
      and no longer calls `.iter().flatten()` on a nesting.
- [ ] The click hit-test at `console/src/console.rs:772-780` uses `source_grid.position(column, row)`
      and no longer indexes a Cell it does not use.
- [ ] `console/src/paint.rs`'s module doc no longer claims `Paint`'s flatness is deliberately unlike
      `RenderFrame`'s, because it no longer is.
- [ ] Every `rows()[r][c]` site is rewritten — to `at(position)` where the test is about a Cell, and
      to a Grid-minted Position where the test is naming a coordinate. A test that rewrites to
      `at(grid.position(c, r).unwrap())` and reads worse than it did should say so in review rather
      than be forced through.
- [ ] The two `Orcvs` doctests (`orcvs/src/app.rs:96-97`, `:128`) still demonstrate the same thing.
      `cargo test --workspace --doc --locked` passes.
- [ ] Nothing about what is drawn changes.

## Verification

```sh
cargo fmt --all -- --check
cargo clippy --package orcvs --all-targets --locked -- -D warnings
cargo clippy --package console --all-targets --locked -- -D warnings
PROPTEST_CASES=32 cargo nextest run --package orcvs --locked
PROPTEST_CASES=32 cargo nextest run --package console --locked
cargo test --workspace --doc --locked
```

## Comments

`needs-triage`, and it would be dishonest to file it any other way. `.scratch/source-paint/spec.md`
decided the opposite explicitly, and added "that is not filed as a follow-up" — which this ticket is.
The new information is that the stated reason ("a type whose only consumer iterates it in row order")
does not hold against `console/src/paint.rs:105-107` and `:220`, and that the true reason was churn.
A maintainer has to weigh 48 rewritten sites against a shape that matches its consumer, and that
weighing is not an agent's to do.

Worth doing after `06`, whatever the triage outcome: most of the churn is test sites that only want a
coordinate, and `06` gives them a better way to ask.
