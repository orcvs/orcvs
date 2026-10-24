# 02 — Show the Overview

**What to build:** A performer ticks "Overview" in the View menu and a strip docked at the right of the Source View shows the whole Grid and its two-Cell margin, one crisp block per Cell in the Theme's colours. It follows the Source as it is edited and every Tick during Playback, and follows a Theme or appearance change. Unticking it hides it. It is off on start. See `../spec.md` (Picture, Cache, Texture, Shape and size, Placement and toggle, Benchmark).

**Blocked by:** 01 — Record the Overview.

**Status:** resolved

- [x] Built test-first. The picture is derived as a value from a Render Frame and a Theme with no `egui::Context`: an empty Grid is all Grid background; a Function, a Comment, a Bang and an invalid operand take their Theme colours through the existing Cell colour table; Portal roles from a committed Tick take their Portal colours; the margin is Grid background; a different Theme gives a different picture.
- [x] The picture is rebuilt only when the Source's revision identifier or the presented Theme and appearance change, read in that order, into a reused buffer; a Render Frame where neither changed uploads nothing.
- [x] One texture, loaded once with nearest filtering and updated with one whole-image partial update when rebuilt. No per-Cell Shapes.
- [x] It is drawn square at a whole number of physical pixels per texel, snapped, at 1×, 1.5× and 2× — asserted at the Shape level.
- [x] At the minimum window size it fits within the height beside the Source View and half its width, scaled below one physical pixel per texel where it must be, and the Source View keeps the rest.
- [x] A kittest finds the View menu's "Overview" checkbox by label, shows that ticking it docks the Overview beside the Source View without covering it and unticking removes it, and that a Cursor in view stays in view across the toggle.
- [x] The Overview's response is labelled "Source overview" and exposes no per-Cell nodes; Tab never focuses it.
- [x] A Criterion case derives the picture from a 256 by 256 Render Frame, joins the console paint benchmarks, and widens no shipped item for it. The comparison is CI's: the case lands with a provisional floor, which is reset from its first CI figure before the ticket closes.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass; `mise run check_wasm` passes.

## Comments

Shipped in `console/src/console/overview.rs`. `draw_picture` draws the picture as a value from a Render Frame and a Theme: each Cell's Plain colours from `style::CellColours`, reduced to the foreground where the Cell has content, else its background, else the Grid background, and composited over the Grid background. The picture carries the two-Cell margin as Grid background texels, so it is 260 by 260 texels for the 256 by 256 Grid: the margin is in the value the tests read, and the image's rectangle is the padded Source the outline (03) and the Pan (04) map onto. The spec's Picture and Texture bullets and ADR 0080's upload figures now say so.

`Overview` keeps the picture in an `Arc<ColorImage>` redrawn in place through `Arc::make_mut` (the texture update holds the other reference only until the renderer takes it), keyed by the Source's `RevisionId`, then the presented Theme's identity and the appearance. `Console::ui` reads the revision before `render_frame()`. The first draw loads the texture with `TextureOptions::NEAREST`; every later redraw is one `set_partial([0, 0], …)` of the whole image. A right `egui::Panel` shown after the bottom Panel and before the Source holds it, sized to the picture at `floor(pixels_per_point)` (at least one) physical pixels per texel with its corner snapped. Its response senses `Sense::DRAG` alone and is labelled "Source overview". The View menu's "Overview" checkbox sits under Diagnostics, off on start and not saved.

Tests: four picture-as-value tests in `overview::tests`; `console::tests` asserts the image's square, whole-pixel rect at 1×, 1.5× and 2× and that uploads happen only on first show, an edit and an appearance change; `kittest_tests` drives the View menu toggle (docks beside the Source, Cursor kept in view, width restored) and the single labelled, drag-only node that Tab does not focus. The `paint_overview/picture/256x256` bench case measured 408 µs locally; its floor is 1 ms, a starting figure for CI to confirm. It reaches `draw_picture` through the test-only `console::source_shapes` module; `draw_picture` is `pub` inside the crate-private `console::overview`, as `SourceShapes` is inside `console::shapes`, so the crate's public surface is unchanged.
