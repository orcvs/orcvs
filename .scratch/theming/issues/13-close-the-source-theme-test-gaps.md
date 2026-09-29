# 13 — Close the Source Theme test gaps, and stop converting seam colour per Cell

**What to build:** The tests `06` resolved without, and the one per-Cell colour conversion left in the Paint loop.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [x] *(Moved to `17`.)* A Paint-level test shows that an explicit transparent `region.cursor.background` suppresses the fallback to the Cursor fill for the Cursor inside a Region. The code at `console/src/paint.rs:242` (`Some(TRANSPARENT).or(..)`) is right; only the resolver covers the three optional-fill states today.
- [x] *(Moved to `17`.)* A test measures the composited pixel colour of a transparent Grid and of a partial-alpha Grid over the console surface. `console.rs:5643` checks only that `source_panel_frame` passes the fill through.
- [ ] A test holds hit-testing unchanged across Cell border widths from 0 to 1 point, at more than one zoom.
- [ ] `sector_line` (`console/src/style.rs:552`), called per seam Cell from `paint.rs:303` and `:309`, stops un-premultiplying and re-premultiplying `sector.seam` for every Cell. `../schema.md:205` rules out per-Cell colour interpolation. The per-Cell strength is a hash, so this is a once-per-Paint unpack or a premultiplied-space scale, not a table. `paint_derive` stays within `benches/floors.toml`.
- [ ] `contrast.rs:344` and `paint.rs:242` share one derivation of the Region Cursor fill instead of each restating `region_cursor_background.or(cursor_fill)`.
- [ ] `cargo nextest run --package console --locked` and the `--no-default-features` arm pass.

## Comments

**2026-09-23 — opened by the audit of the merged pull requests against their issues.** Residuals of `06`, which is resolved with these lines pointing here.

**2026-09-24 — split.** The two transparent-value tests moved to `17`, which joins `release/v1` and blocks `10`. The remaining lines (hit-testing across border widths, the per-seam colour conversion, the shared Region Cursor fill derivation) stay out of the release as improvements.

**2026-09-29 — audit at `cad296df`.** All three open lines still hold; only their line references drifted and are corrected in place: `sector_line` 562→`console/src/style.rs:552`, its per-seam calls 312/318→`console/src/paint.rs:303`/`:309`, and the duplicated `region_cursor_background.or(cursor_fill)` 397/246→`console/src/contrast.rs:344` and `console/src/paint.rs:242`. No test varies the Cell border width while clicking: the width tests in `console/src/console/tests.rs` check strokes only.
