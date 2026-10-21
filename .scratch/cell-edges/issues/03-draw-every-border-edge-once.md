# 03 — Draw every border edge once, highest rank winning

**What to build:** Role, Diagnostic, Output Portal, Input Portal and Locked borders become edges of the same Mesh. Each edge shows the highest-ranked fact either Cell carries on that side, composited once over the facts below it. Two letters of a word share one role line as strong as the word's outline; two Portal Cells share one line; a Portal edge beats a role edge it meets. A border of width zero withdraws from the edge and the next fact shows. The Cursor keeps today's stroke until 04. See `../spec.md` and ADR 0078.

**Blocked by:** 02 — Draw each Grid line once.

**Status:** resolved

- [x] Built test-first at the Paint: an edge answer beside the background runs, coalesced into runs along each row line and column line, carrying colour, width in points and rank, and no geometry. Tests cover equal neighbours, each rank against the next in both orders, width zero falling through, and runs breaking where the winner changes.
- [x] The composition `ordinary_border` performs is kept as the edge colour and is applied once; nothing is painted under an edge.
- [x] No Cell strokes its own border except the Cursor's, which 04 moves.
- [x] Junctions take the widest lines' extent; inside it the highest-ranked lines keep their own widths and the highest-ranked of the rest fills only what the other lines reach.
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

**2026-10-21 — cost.** Local runs of `paint_shapes` with the bench task's settings, on a 4-core 2.8 GHz Xeon cloud container: `fitted/128x128` measures 704 µs on 02 and 2.34 ms here, and `fitted/16x16` 11 µs and 36 µs. A callgrind profile of `fitted/64x64` puts about a third of the step in `Paint::edges` (a `winner` per edge, coalesced into runs) and most of the rest in `edge_mesh`, which expands the runs back into a dense line per edge and builds a junction per crossing. Junctions are computed once per crossing and the Mesh is reserved. The floor on `paint_shapes/fitted/128x128` is raised to 5 ms, about twice this figure. Making the step cheaper, for instance by having the Paint answer the dense per-edge lines `edge_mesh` reads instead of runs, is not part of this ticket.

**2026-10-21 — junction widths.** A narrower high-ranked line meeting wider Grid lines took the whole junction, so a 1 pixel border turned its corners as a 2 by 2 block (review of #264). Toby chose to amend ADR 0078: the top lines keep their own bands inside the junction and the next line fills the rest, as up to three quads per band of rows where it differs. `a_narrow_border_keeps_its_width_through_a_wider_grid_lines_junction` pins it pixel by pixel. A line that wins a crossing along one axis still runs through the whole junction, so a faded Portal line still crosses a wider Grid line at its own fade. Only a junction where lines of more than one rank meet is split; the table keeps one small entry per crossing. Locally `fitted/128x128` measures 2.9–3.1 ms against 2.36 ms before in the same session, and `fitted/16x16` 42 µs against 35 µs, inside the 5 ms floor.

**2026-10-21 — corner inside the box.** The review on #265 found a Grid pixel inside a narrow border's box at its bottom-right corner: a 1 pixel line lies right of or below its boundary while a 2 pixel line straddles it, so the junction's leftover pixel fell inside the box there and nowhere else. Every line meeting a mixed junction now has an arm from its own side to the far edge of the top lines' crossing, and the rest colour fills only pieces a lower arm reaches. Both corner tests now expect that pixel empty. One `Junction` per crossing carries the four lines meeting it, and `edge_mesh` keeps two rows of them rather than a table for the whole range. Locally `fitted/128x128` measures about 2.9 ms against 3.6 ms before the change in the same session.
