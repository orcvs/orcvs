# 04 — Build a drawn line in one place

**What to build:** One constructor beside `EdgeLine` in `console/src/paint.rs`, `EdgeLine::drawn(rank, width, colour) -> Option<EdgeLine>`, states ADR 0078's rule that a zero width draws no line: `None` at zero width, otherwise the line at that width, unseamed. Every place a line is built uses it: the Grid line and the selection outline in `Paint`, and the Diagnostic and Portal borders in `style::ordinary_border`. What a border composites over reads the Grid line from that same answer instead of repeating `grid_border_width.points() > 0.0`. `PortalChannels.border_width` keeps its `GridWidth` rather than a bare `f32`.

The constructor filters on width only. A transparent line of nonzero width is still drawn and still wins its edge (cell-edges 03), so it must not filter on colour.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] `EdgeLine::drawn` is the only code that decides whether a width draws a line; no `> 0.0` width test is left at a call site.
- [ ] `style` and `Paint` agree on whether the Grid line is drawn from one answer.
- [ ] Tests show: a zero width draws no line for the Grid line, the outline, a Diagnostic and each Portal border; a transparent line of nonzero width still claims its edge; the existing edge and border tests pass unchanged.
- [ ] The empty and Region-spanning Paint benchmarks still exist and still measure the same walk.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass.
