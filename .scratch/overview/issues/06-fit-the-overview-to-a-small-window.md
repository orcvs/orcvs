# 06 — Fit the Overview to a small window

**What to build:** A window too small for the Overview's picture at its usual size shows it smaller, still square and at a whole number of physical pixels per texel, or not at all. The picture takes at most half the window's width, so the Source keeps the rest, and the height beside the docked panels. It is never clipped or scaled to fit, because a press on it maps to a Pan only across the whole picture. See `../spec.md` (Shape and size).

**Blocked by:** 02 — Show the Overview.

**Status:** resolved

- [x] Built test-first at the Shape level: in a window too narrow for the picture at 2×, it is drawn at one physical pixel per texel; where one pixel per texel does not fit, in width or in height, no picture is drawn; in each case the View menu's choice is kept and the picture returns at its usual size in the default window.
- [x] The picture never takes more than half the window's width and never leaves the window.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass.

## Comments

Raised in review of #279 and #280: the panel was sized from the texture alone, so at the smallest window (300 by 220 points) it left the Source about 25 points of width and its picture was taller than the room beside the docked panels, clipping the lower Grid out of reach of a press.

Shipped in `console/src/console/overview.rs`. `show_overview` reads the room the root `Ui` has left after the docked panels, takes half its width and all its height less the panel frame's margins, and converts it to physical pixels per texel of the padded Grid. The picture is drawn at the smaller of that, rounded down, and the usual `floor(pixels_per_point)`, at least one; below one it answers `None` before the texture is touched, so `Console::ui` neither outlines nor Pans from it that Render Frame. `overview_open` is not changed, so the View menu stays ticked and the Overview returns once the window holds it. The half-width rule is the fallback chosen for this ticket; hiding rather than clipping keeps the outline able to reach every Pan.

Tests: `the_overview_shrinks_to_whole_pixels_per_texel_that_fit_and_hides_below_one` in `console::tests` drives the shipped `Console` in a 500-point-wide window at 2× (one pixel per texel) and 1.5× (one pixel per texel, as usual) and at 1× (hidden), in a 300-point-tall window at 1× (hidden), and at the smallest window at 1× and 2× (hidden), then in the default window, where each returns at its usual size. `the_overview_outline_shrinks_as_the_performer_zooms_in` runs in a window twice the default, because zoomed in to 2× the default window no longer holds the picture beside docked Diagnostics.
