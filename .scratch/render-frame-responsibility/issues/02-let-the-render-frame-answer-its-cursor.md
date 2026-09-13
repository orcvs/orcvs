# 02 — Let the Render Frame answer its Cursor

**What to build:** `RenderFrame::cursor() -> Position`, and the deletion of the scan that recovers
it.

## The round trip

`RenderFrame::derive` is **handed** the Cursor. Its signature takes `selected: Position`
(`orcvs/src/render_frame.rs:75-80`), and the first thing it does with it is assert it belongs to the
Grid:

```rust
let grid = source.grid();
grid.assert_owns(selected);
```

(`orcvs/src/render_frame.rs:81-82`.) It then throws the Position away. Every Cell computes
`let is_selected = position == selected;` (`:85`) and stores two bools, `selected: is_selected` and
`cursor_visible: is_selected && cursor_visible` (`:102-103`). A `Position` that was one value on the
way in leaves as a thousand bools on the default 40 by 25 Grid (`orcvs/src/grid.rs:81-82`), of which
999 are `false`.

`Paint::derive` then buys it back. It walks every Cell of the flattened rows
(`console/src/paint.rs:105-107`) and accumulates the answer from inside a `.map()` closure:

```rust
let mut cursor = None;
...
.map(|cell| {
    if cell.selected() {
        cursor = Some(cell.position());
    }
```

(`console/src/paint.rs:99`, `:109-111`.) A side effect inside a `map` — the closure's return value
is a `CellPaint`, and the thing the loop is really computing sneaks out through a captured `mut`.
Then it unwraps, defended by a nine-line cross-crate proof:

```rust
// Total, and the whole chain is in `orcvs::render_frame`:
// `RenderFrame`'s fields are private and `RenderFrame::derive` is
// its only constructor, that function calls
// `Grid::assert_owns(selected)` before building a single Cell, and
// it then derives one Cell per Position of that same Grid. So the
// selected Position is a Position of the Grid being walked, and
// exactly one Cell compares equal to it.
cursor: cursor.expect("a Render Frame selects one of its Cells"),
```

(`console/src/paint.rs:164-172`.)

And it is not the only site. `console/src/console.rs:1478-1488` is a second scan with its own
`.find(|cell| cell.selected())` and its own `.expect("the Cursor is on a Cell")`, in a test helper
named `selected_cell` that exists to answer the question `RenderFrame` was handed the answer to.

## This is a totality defect, not a panic risk

State this plainly in the commit message, because the obvious reading is wrong.

**The invariant is total today.** The proof comment at `console/src/paint.rs:164-171` is correct in
every step: `RenderFrame`'s fields are private, `derive` is its only constructor, `assert_owns` runs
before a Cell is built, and one Cell is built per Position of that same Grid. Nobody is going to see
that `expect` fire, and this ticket is not filed because anyone might.

It is filed because **a total function was made partial and then re-totalised by an assertion whose
proof lives in another crate.** `derive` received a `Position` — total. It published bools —
partial, since nothing in the *type* `&[Vec<RenderCell>]` says exactly one is `true`. `Paint::derive`
then restores totality with a runtime `expect` whose justification cannot be checked without opening
`orcvs/src/render_frame.rs`. The cost is paid in review attention and in the nine lines of comment
that are the interest on it, not in crashes.

`Paint`'s own documentation makes the argument against itself. `Paint::cursor` is doc-commented:

> One Position for the whole Paint rather than a flag on every Cell: `RenderFrame::derive` takes one
> selected Position and asserts the Grid owns it, so exactly one exists, and a per-Cell bool would
> re-open a state the layer below has closed.

(`console/src/paint.rs:194-201`.) The layer below is exactly what opened it. `Paint` closes a state
its own input opened one function earlier, and the right fix is for the input not to open it.

## The fix

`Position` is `Copy` (`orcvs/src/grid.rs:21`) and is already a parameter of `derive`. Store it,
answer it.

Deleted by that one accessor: the `let mut cursor = None`, the side-effecting `map`, the `Option`,
the `expect`, the nine-line proof comment, the `selected_cell` helper in `console.rs`, and the half
of `the_cursor_is_the_selected_position` (`console/src/paint.rs:482-501`) that asserts the Render
Frame selects exactly one Cell — an assertion that only means anything while the scan exists.

**Out of scope:** removing `RenderCell::selected` and `RenderCell::cursor_visible` themselves. They
have readers in `orcvs`'s own tests and doctests (`orcvs/src/app.rs:128, 437, 448-449`) and in
`console/src/paint.rs:152-157`, and the Cell shrinking to content and Glyph is ticket `04`'s change.
This ticket adds the accessor and deletes the recovery; it leaves the bools where they are.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] `RenderFrame::cursor() -> Position` exists, documented as the Position `derive` was given and
      asserted, not as a Position found by looking.
- [x] `Paint::derive` reads `frame.cursor()`. No `Option`, no `expect`, no assignment inside the
      `map` closure, and the `map` closure captures nothing mutable.
- [x] The nine-line proof comment at `console/src/paint.rs:164-171` is gone rather than moved. The
      fact it proves is now stated by the type.
- [x] `console/src/console.rs`'s `selected_cell` test helper reads `cursor()` or is deleted; no
      `.find(|cell| cell.selected())` survives in either crate.
- [x] `the_cursor_is_the_selected_position` either goes or keeps only what is still worth asserting
      — that the Paint's Cursor is the Position that was selected. Its second assertion, that the
      Render Frame's Cells select exactly that one, is the scan's own correctness and goes with it.
- [x] A new test in `orcvs` asserts `RenderFrame::cursor()` is the Position `derive` was given,
      including after `Orcvs::select` moves it.
- [x] Nothing about what is drawn changes. Same colours, same geometry, same order.

## Verification

```sh
cargo fmt --all -- --check
cargo clippy --package orcvs --all-targets --locked -- -D warnings
cargo clippy --package console --all-targets --locked -- -D warnings
PROPTEST_CASES=32 cargo nextest run --package orcvs --locked
PROPTEST_CASES=32 cargo nextest run --package console --locked
cargo test --workspace --doc --locked
```

The doctest run is owed because `RenderFrame` gains a public method and `Orcvs::render_frame`'s
doctests (`orcvs/src/app.rs:96-97, 128`) read the Render Frame directly.

## Comments

`ready-for-agent`. Nothing here needs the seam criterion settled, nothing here is aesthetic, and the
fix is one field and one accessor. It is the cheapest ticket in the effort and the one most clearly
right on its own: even if ticket `01` is refused outright, a `Position` handed in should not have to
be found again.

Landed on top of `cull-source-paint`: `Paint::cursor` remains `Option<Position>` because a Paint
covers a viewport and the Cursor can sit outside it (`source-paint/07`). The scan/`expect` path is
gone; `derive` reads `frame.cursor()` and keeps `Some` only where the drawn ranges cover that
Position. The checklist's "No Option" referred to the pre-cull recovery `Option`+`expect`, not the
viewport coverage answer.
