# 16 — Construct the highlight and space glyphs

**What to build:** Make the background actually use its three kinds of Cell. The console computes whether a Cell is a marker, a cursor-block highlight, or plain space, and then renders all three as a marker, so the whole background is markers.

**Blocked by:** None (can start immediately).

**Status:** resolved

- [x] A highlight Cell is a highlight, and a space Cell is a space; neither is a marker.
- [x] An empty Source shows markers only where the marker spacing puts them, with plain space between.
- [x] Tests distinguishing the three kinds fail if any two are made equal again.

## Notes

Found while implementing ticket 09, not yet triaged by a human.

`GlyphString::highlight()` and `GlyphString::space()` both construct `Glyph::Marker`. The `Highlight` and `Space` variants exist, and both the styling and the display mapping handle them, but nothing anywhere constructs either one — so the classification `terminator` computes is discarded at the moment it is turned into a glyph.

This also makes `test_terminator` tautological: its three assertions compare values that are all equal, so it passes no matter which branch `terminator` takes. That test needs to be able to fail before it is worth keeping.

## Answer

`GlyphString::highlight()` and `GlyphString::space()` now construct their matching Glyph variants. Pairwise constructor and display assertions distinguish Marker, Highlight, and Space, while an empty-Source regression observes all three through the renderer-facing `App::get(Position)` interface.
