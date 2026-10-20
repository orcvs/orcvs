# 03 — Draw every border edge once, highest rank winning

**What to build:** Role, Diagnostic, Output Portal, Input Portal and Locked borders become edges of the same Mesh. Each edge shows the highest-ranked fact either Cell carries on that side, composited once over the facts below it. Two letters of a word share one role line as strong as the word's outline; two Portal Cells share one line; a Portal edge beats a role edge it meets. A border of width zero withdraws from the edge and the next fact shows. The Cursor keeps today's stroke until 04. See `../spec.md` and ADR 0078.

**Blocked by:** 02 — Draw each Grid line once.

**Status:** ready-for-agent

- [ ] Built test-first at the Paint: an edge answer beside the background runs, coalesced into runs along each row line and column line, carrying colour, width in points and rank, and no geometry. Tests cover equal neighbours, each rank against the next in both orders, width zero falling through, and runs breaking where the winner changes.
- [ ] The composition `ordinary_border` performs is kept as the edge colour and is applied once; nothing is painted under an edge.
- [ ] No Cell strokes its own border except the Cursor's, which 04 moves.
- [ ] Junctions take the colour of the highest-ranked line meeting them and the widest lines' extent.
- [ ] With a translucent role border, a word's inner lines and outline measure the same in a 2× screenshot, and crossings show no brighter dot.
- [ ] Tests that pinned per-Cell border rectangles are rewritten to the edge rule, not deleted.
- [ ] CONTEXT.md's Paint entry says a Paint answers each Cell edge's line.
- [ ] `.scratch/theming/schema.md`'s composition steps and the console Theme record state the edge rule and the width-zero rule.
- [ ] The console gate and the `--no-default-features` arm pass; `mise run check_wasm` passes.
