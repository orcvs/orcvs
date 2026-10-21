# 01 — Measure the shapes step

**What to build:** A benchmark of the step that turns a Paint into Shapes, at the same Grid sizes and viewport cases `paint_derive` uses (fitted, culled, single, empty), so the edge work after it can be compared against today's per-Cell borders. `paint_derive` measures the derivation only. See `../spec.md` and ADR 0078.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] A Criterion benchmark builds the Source Shapes from a derived Paint at each existing benchmark size and viewport case, and joins the CI benchmark series and its floors as the existing ones do (`.scratch/benchmarks/spec.md`).
- [x] It reaches the shapes step without a shipped item that only the benchmark uses: no `pub` widened for it, no parameter or branch only it populates. If that is not possible from the bench crate, the ticket records why and what it measures instead.
- [x] Tessellation is not measured; the ticket's notes say so.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass. The comparison itself is CI's.

## Comments

**2026-10-20 — what shipped.** `paint_shapes` in `console/benches/paint.rs` times `SourceShapes::geometry` on an already-derived Paint, at a Grid presented from the origin, for `single`, `empty`, and `fitted`/`culled` at every `FRAME_SIZES` entry at 1×, and `fitted@1.5x` at every size, because a fractional device scale snaps Cell sides and edges through different rounding. The names sit in the same series as `paint_derive` and `paint_background_runs`.

- **Visibility.** The shapes step, `GridViewport` and `presented_grid` are private to the console crate, and its only public entry point, `Console::start`, cannot build a Grid at the benchmark sizes. A test-only feature, `test-source-shapes`, exposes `SourceShapes` and `presented_grid` through `console::source_shapes`. As with `orcvs`'s `test-grid-shapes`, only the console's own dev-dependency on itself enables it, so no shipped build carries the module and the crate's public surface is unchanged. Inside the crate, `SourceShapes`, `SourceShapes::geometry`, `GridViewport` and `presented_grid` are declared `pub` so the module can re-export them; their modules stay private. No parameter or branch was added.
- **Not measured.** Glyph placement is left out because it needs a `GlyphTable`, and so an `egui::Context`. Glyphs are not part of the edge work. Tessellation is left out because it is egui's, after `Painter::extend`.
- **Floor.** `benches/floors.toml` guards `paint_shapes/fitted/256x256` with a starting figure of 900 µs, about twice the local figure, in the way `execute_function`'s floor was set. The 256×256 case varied by most of its own figure between local samples, so 128×128 carries the guard. Ticket 02 deliberately moves this number and revises the figure.

**2026-10-21 — evidence.**

- `cargo fmt --all -- --check` — passed
- `cargo clippy --package console --all-targets --locked -- -D warnings` — passed
- `cargo clippy --workspace --all-targets --no-default-features --locked -- -D warnings` — passed
- `cargo clippy --workspace --all-targets --target wasm32-unknown-unknown --locked -- -D warnings` — passed (stand-in for `mise run check_wasm`, which needs `trunk`)
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked` — passed
- `cargo nextest run --package console --locked` — passed, 559 tests
- `cargo nextest run --workspace --tests --no-default-features --locked` — passed, 1793 tests
- `cargo tree --workspace --all-features -e features --locked` — resolves (the feature half of `mise run audit_deps`; `cargo deny` is not installed here)
- `cargo bench --package console --bench paint --locked -- paint_shapes --output-format bencher --warm-up-time 0.5 --measurement-time 1 --sample-size 10 --nresamples 1000` — run once, to set the starting floor; `fitted/128x128` 416 µs, `fitted@1.5x/128x128` 400 µs. The comparison is CI's.
