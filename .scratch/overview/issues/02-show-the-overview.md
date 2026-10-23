# 02 — Show the Overview

**What to build:** A performer ticks "Overview" in the View menu and a strip docked at the right of the Source View shows the whole Grid and its two-Cell margin, one crisp block per Cell in the Theme's colours. It follows the Source as it is edited and every Tick during Playback, and follows a Theme or appearance change. Unticking it hides it. It is off on start. See `../spec.md` (Picture, Cache, Texture, Shape and size, Placement and toggle, Benchmark).

**Blocked by:** 01 — Record the Overview.

**Status:** ready-for-agent

- [ ] Built test-first. The picture is derived as a value from a Render Frame and a Theme with no `egui::Context`: an empty Grid is all Grid background; a Function, a Comment, a Bang and an invalid operand take their Theme colours through the existing Cell colour table; Portal roles from a committed Tick take their Portal colours; the margin is Grid background; a different Theme gives a different picture.
- [ ] The picture is rebuilt only when the Source's revision identifier or the presented Theme and appearance change, read in that order, into a reused buffer; a Render Frame where neither changed uploads nothing.
- [ ] One texture, loaded once with nearest filtering and updated with one whole-image partial update when rebuilt. No per-Cell Shapes.
- [ ] It is drawn square at a whole number of physical pixels per texel, snapped, at 1×, 1.5× and 2× — asserted at the Shape level.
- [ ] A kittest finds the View menu's "Overview" checkbox by label, shows that ticking it docks the Overview beside the Source View without covering it and unticking removes it, and that a Cursor in view stays in view across the toggle.
- [ ] The Overview's response is labelled "Source overview" and exposes no per-Cell nodes; Tab never focuses it.
- [ ] A Criterion case derives the picture from a 256 by 256 Render Frame, joins the console paint benchmarks with a floor, and widens no shipped item for it. The comparison is CI's.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass; `mise run check_wasm` passes.
