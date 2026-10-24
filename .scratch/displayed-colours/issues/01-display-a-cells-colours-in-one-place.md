# 01 — Display a Cell's colours in one place

**What to build:** `CellColours::displayed` and `CellColours::surface` in `console/src/style.rs`, and a `DisplayedColours` with an opaque background and the Theme's foreground. `contrast::painted` and the Overview's `cell_texel` call it, and the Overview's margin is the surface. The Overview's Region mark stays one rect of `region.background`, and its doc says it is a location mark.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Neither contrast nor the Overview composites a Cell's colours over a surface itself.
- [x] Every Overview texel is opaque under a Theme with a translucent `grid.background` and a translucent foreground, against an expectation worked by hand from the Theme's colours.
- [x] For both built-ins, the contrast `validate` results and every reachable displayed colour are byte-identical before and after.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass, and `mise run check_wasm` passes.

## Comments

- The byte-identity check is a temporary test, not committed. It wrote both built-ins' `validate` results, plus the occupied and empty texel for every Source role, reachable Portal set and `CursorPlacement`, before and after the change, and the two files compared equal.
- `an_empty_cell_a_committed_tick_read_takes_its_portal_background` keeps its `grid_background.blend(input_portal_background)` expectation. It pins the Portal a Tick reads, under a built-in whose Grid background is opaque, where that equals the displayed background.
- `DisplayedColours::foreground` is the Theme's channel rather than the composite, because contrast reports it and matches accepted failures against it.
