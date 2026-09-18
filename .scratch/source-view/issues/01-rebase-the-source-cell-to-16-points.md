# 01: Rebase the Source's own Cell to 16 points

**What to build:** The Source's own Cell becomes 16 points with an 11.5 point Glyph, so every Zoom step of an eighth is a whole number of points and a Cell is a whole number of physical pixels at 1×, 1.5× and 2×. This is a prefactor: the console still fits the Source to the window exactly as it does today, and no gesture changes. See ADR 0045.

**Blocked by:** None (can start immediately).

**Status:** resolved

- [x] The Source's own Cell is 16 points and the Glyph laid out in it at Zoom 1.0 is 11.5 points.
- [x] Every multiple of an eighth from 0.25 to 2.0 presents a Cell that is a whole number of points.
- [x] The glyph atlas budget still holds fifteen sizes over the Zoom range.
- [x] Tests that stated figures in 25 point Cells are restated in 16 point Cells, and still assert the same behaviour.
