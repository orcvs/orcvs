# 03 — Draw every border edge once, highest rank winning

**What to build:** Role, Diagnostic, Output Portal, Input Portal and Locked borders become edges of the same Mesh. Each edge shows the highest-ranked fact either Cell carries on that side, composited once over the facts below it. Two letters of a word share one role line as strong as the word's outline; two Portal Cells share one line; a Portal edge beats a role edge it meets. A border of width zero withdraws from the edge and the next fact shows. The Cursor keeps today's stroke until 04. See `../spec.md` and ADR 0078.

**Blocked by:** 02 — Draw each Grid line once.

**Status:** resolved

- [x] Built test-first at the Paint: an edge answer beside the background runs, coalesced into runs along each row line and column line, carrying colour, width in points and rank, and no geometry. Tests cover equal neighbours, each rank against the next in both orders, width zero falling through, and runs breaking where the winner changes.
- [x] The composition `ordinary_border` performs is kept as the edge colour and is applied once; nothing is painted under an edge.
- [x] No Cell strokes its own border except the Cursor's, which 04 moves.
- [x] Junctions take the colour of the highest-ranked line meeting them and the widest lines' extent.
- [x] With a translucent role border, a word's inner lines and outline measure the same in a 2× screenshot, and crossings show no brighter dot.
- [x] Tests that pinned per-Cell border rectangles are rewritten to the edge rule, not deleted.
- [x] CONTEXT.md's Paint entry says a Paint answers each Cell edge's line.
- [x] `.scratch/theming/schema.md`'s composition steps and the console Theme record state the edge rule and the width-zero rule.
- [x] The console gate and the `--no-default-features` arm pass; `mise run check_wasm` passes.

## Comments

- Role borders are not drawn: the code has no role border and the Theme no key for one, so the Mesh carries Diagnostic, Locked, Input Portal and Output Portal edges over the Grid line. Adding role borders needs Theme keys first, which is a separate ticket. The translucent-border criterion is proved with a Diagnostic border instead (`neighbouring_diagnostic_cells_share_one_line_composited_once`, at 1 point and 2×).
- An edge's colour is the winning Cell's own composited answer from `ordinary_border`, drawn once with nothing under it.
- Of two facts of one rank, the Cell below or to the right draws its edge; their composition can differ where a Portal blends over a Diagnostic on one side only.
- A transparent Portal border of nonzero width still wins its edge, and composites to the colour beneath it, so the Okabe–Ito Portal edges show the Grid line or Diagnostic colour they blend over.
- The single-Cell Cursor still strokes its own border until 04.
