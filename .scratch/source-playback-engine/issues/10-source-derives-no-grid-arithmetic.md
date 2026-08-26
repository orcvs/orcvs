# 10 — Source derives no grid arithmetic

**What to build:** Take the last of the grid arithmetic out of the Source. A Tick decides where each Expression's result goes, whether it falls below the bottom row, and whether it fits before the row edge — all of it currently derived by hand from the column count. Those are facts about the Grid, and the Source should ask for them.

**Blocked by:** 09 — Rendering addresses Cells through the Grid.

**Status:** ready-for-agent

- [ ] The Grid answers which row a position is in, and whether a value of a given width fits in that row from that position.
- [ ] A Tick derives no dimensions itself: the result destination, the discard below the bottom row, and the discard at the row edge all go through the Grid.
- [ ] The Source is constructed from a Grid rather than from the whole console options, and so is the commander that owns it.
- [ ] The Source's public interface, its Cell stores, and the change set it returns continue to speak in indices.
- [ ] The empty-Source Tick case is removed: a Source cannot be built without Cells, so a Tick over one is not a case to handle.
- [ ] Tick behaviour tests run on a rectangular Source and cover placement, bottom-row discard, and row-edge discard.
