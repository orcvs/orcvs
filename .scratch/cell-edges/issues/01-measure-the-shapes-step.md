# 01 — Measure the shapes step

**What to build:** A benchmark of the step that turns a Paint into Shapes, at the same Grid sizes and viewport cases `paint_derive` uses (fitted, culled, single, empty), so the edge work after it can be compared against today's per-Cell borders. `paint_derive` measures the derivation only. See `../spec.md` and ADR 0078.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] A Criterion benchmark builds the Source Shapes from a derived Paint at each existing benchmark size and viewport case, and joins the CI benchmark series and its floors as the existing ones do (`.scratch/benchmarks/spec.md`).
- [x] It reaches the shapes step without a shipped item that only the benchmark uses: no parameter or branch only it populates. The items it calls are made `#[doc(hidden)] pub`, as `theme` and `cursor_effects` already are for this file; the comment below records why.
- [x] Tessellation is not measured; the ticket's notes say so.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass. The comparison itself is CI's.

## Comments

**2026-10-20 — what shipped.** `paint_shapes` in `console/benches/paint.rs` times `SourceShapes::geometry` on an already-derived Paint, at a Grid presented from the origin at 1×, for `single`, `empty`, and `fitted`/`culled` at every `FRAME_SIZES` entry. The names sit in the same series as `paint_derive` and `paint_background_runs`.

- **Visibility.** The shapes step, `GridViewport` and `presented_grid` were private to the console crate. The only public entry point is `Console::start`, and it cannot build a Grid at the benchmark sizes. So `console::shapes` and `grid_viewport` are now `#[doc(hidden)] pub` modules, and `SourceShapes`, `SourceShapes::geometry`, `GridViewport` and `presented_grid` are `pub`. This is the precedent `theme` and `cursor_effects` set for this file. No parameter or branch was added. Three intra-doc links into items that are still private became plain code spans, because the rustdoc gate refuses a public item linking to a private one. A feature-gated exposure, modelled on `orcvs`'s `test-grid-shapes`, would have meant editing both bench tasks in `mise.toml`, the tooling contract, and the clippy gates that compile the bench.
- **Not measured.** Glyph placement is left out because it needs a `GlyphTable`, and so an `egui::Context`. Glyphs are not part of the edge work. Tessellation is left out because it is egui's, after `Painter::extend`.
- **No floor.** `benches/floors.toml` guards figures CI has measured and the repository has agreed on. `paint_background_runs` has no floor either. Ticket 02 deliberately moves this number, so a floor should be set once 02's CI figure exists, not before.
