# 01 — Fix the Grid at 256 by 256

**What to build:** Every Grid is 256 columns by 256 rows, per ADR 0054. A fresh console, File > New, the Function reference and a restored Source all stand on that one shape.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Tags:** release/v1

- [x] `Grid` has one shape; no caller states dimensions, and `DEFAULT_COL_COUNT`/`DEFAULT_ROW_COUNT` give way to the one shape.
- [x] The Function reference loads onto the 256 by 256 Grid rather than a Grid rounded up from its text.
- [x] A stored Source whose Grid is not 256 by 256 is refused through the existing persistence path — set aside under `REFUSED_KEY`, noticed, replaced by an empty Grid — and not migrated.
- [x] A fresh console opens on the Grid's top-left corner; ADR 0047's margin and Pan reach are unchanged.
- [x] A benchmark covers each path that walks every Cell (Language Map derivation, Source snapshot, the stored value) at 256 by 256, so CI's comparison reports what the larger Grid costs.
- [x] Tests that built their own small Grids either keep a test-only constructor beside the shipped one or move to the fixed shape; no shipped function takes a dimension only a test supplies.
- [x] Scoped gates for `orcvs` and `console` pass, and the no-default-features arm of persistence compiles.

## Comments

**Implementation.** `Grid::new()` takes no dimensions and mints the one `COL_COUNT` by `ROW_COUNT` (256 by 256) shape; `DEFAULT_COL_COUNT`/`DEFAULT_ROW_COUNT` and `MAX_COL_COUNT`/`MAX_ROW_COUNT` are gone. The persisted-Grid `TryFrom` refuses every other shape, so 64 by 40 and 128 by 80 autosaves go through the existing refusal path (`REFUSED_KEY`, notice, empty Grid) rather than being migrated; `a_source_stored_at_a_previous_default_grid_is_refused_rather_than_migrated` pins both. The Function reference places its text on the one Grid instead of rounding its size up to the Sector Seam spacing.

**Test-only shapes.** Tests that state a Source as a few short rows keep a smaller Grid through `Grid::with_shape` and `Orcvs::with_shape`, both `#[cfg(any(test, feature = "test-grid-shapes"))]`. `test-grid-shapes` is an `orcvs` feature that only dev-dependencies enable: `orcvs` names itself as a dev-dependency with the feature (the one way a package turns a feature on for its own integration tests, benches and doctests), and `console` names it in its `[dev-dependencies]`. `cargo tree -p console -e normal,features` shows the feature on no normal edge, native or `wasm32-unknown-unknown`, so no shipped build compiles either constructor.

**Benchmarks.** `orcvs/benches/source.rs` group `source_whole_grid`: `language_map_derive`, `snapshot`, `edit_rebuild_valid` at 256 by 256. `console/benches/stored_source.rs` group `stored_source`: `save/256x256`, `restore/256x256` (`required-features = ["persistence"]`, since the Serde impls live behind it).

**Kittest swap.** The 256 by 256 Grid is larger than the default window's console at every Zoom, so the half of `the_pointer_shows_no_grab_hand_where_alt_offers_no_pan` that needs a Grid with nowhere to Pan swaps in `Orcvs::with_shape(64, 40)` before zooming out to `MIN_ZOOM`. The storage tests that corrupted `cols:6` now corrupt `rows:256`, and their 18-Cell counts became 256 by 256.

**Gates.** fmt; clippy `orcvs`, `console`, and `console --no-default-features`; nextest `orcvs` (`PROPTEST_CASES=32`), `console`, and `--workspace --tests --no-default-features`; doctests; `mise run audit_deps`; `mise run check_wasm`. All passed.
