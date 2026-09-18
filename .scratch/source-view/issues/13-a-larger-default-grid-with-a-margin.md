# 13: A larger default Grid with a margin

**What to build:** The default Grid doubles to 128 by 80 on each axis while the default window keeps its 1024 by 640 point console, so a fresh console opens with room to Pan. The Grid rests two Cells in from the console's top-left, and a Pan reaches two Cells past each of its edges. See ADR 0047.

**Blocked by:** 03, 05

**Status:** resolved

- [x] A console that restores no Source opens a 128 by 80 Grid; the default window is unchanged.
- [x] At rest the Grid sits a margin of two Cells in from the console's top-left, at every Zoom and device scale.
- [x] A Pan, a resize and a Zoom settle no further than two Cells past each Grid edge.
- [x] The Cursor follow and a click after a Pan still land on the Cell presented.
- [x] A stored Source keeps the Grid it was stored with.
