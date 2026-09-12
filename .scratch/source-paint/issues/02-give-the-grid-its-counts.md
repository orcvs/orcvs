# 02 — Give the Grid its counts and rename its iterator

**What to build:** `Grid` answers its column and row counts publicly, and its position iterator takes a name that says what it yields.

`Grid::cols()` is `pub(crate)` and abbreviated. There is no row-count accessor at all: `Grid::rows()` is the iterator of iterators of `Position`, and `source/model.rs:659` already writes `self.grid.rows().count()` to get a count the Grid could simply answer. `CONTEXT.md` defines a Grid as "the fixed rectangular shape a Source occupies: its column and row counts", so the counts take the glossary's own words and the iterator takes the longer name.

Ticket `03` needs this: `RenderFrame` cannot answer its Grid usefully while `console` cannot ask that Grid how wide it is.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] `Grid::columns()` and `Grid::rows()` are public and answer the counts.
- [x] The position iterator is `Grid::positions_by_row()`.
- [x] All fourteen iterator call sites follow, and all nine `cols()` ones. Every one is inside `orcvs`; nothing in `lang` or `console` calls either.

      Iterator (`rows()` → `positions_by_row()`): `app.rs:539`, `grid.rs:379`, `grid.rs:949`, `render_frame.rs:83` (inside `RenderFrame::derive`), `render_frame.rs:412`, `render_frame.rs:442`, `render_frame.rs:447`, `render_frame.rs:466`, `source/language_map.rs:1120`, `source/language_map.rs:1654`, `source/mod.rs:301`, `source/model.rs:640`, `source/model.rs:659`, `source/tick.rs:3029`.

      Count (`cols()` → `columns()`): `source/language_map.rs:201`, `:205`, `:222`, `:224`, `:283`, `:285`, `:305`, `source/tick.rs:144`, `source/tick.rs:1272`.

      The list is a map, not the safety net. Every omission is compile-caught, because the name the iterator gives up is taken by a `usize` that has no `.count()`, `.nth()` or `.flatten()`.
- [x] `source/model.rs:659` asks for the count rather than counting the iterator.
- [x] `Grid::cols()` no longer exists under that name.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D warnings` and `PROPTEST_CASES=32 cargo nextest run --package <crate> --locked` for `orcvs` and `console`.

Also `cargo test --workspace --doc --locked`. `Grid`'s public surface changes and the `Orcvs` doctests in `app.rs` exercise it.
