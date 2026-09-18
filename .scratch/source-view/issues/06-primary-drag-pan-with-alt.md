# 06: Primary-drag Pan with Alt

**What to build:** Holding Alt (Option on macOS) and dragging with the primary button Pans the Source View, so a trackpad with no middle button can Pan by dragging. Space was considered and refused: it toggles Playback. A primary drag without Alt selects a Region instead. See ADR 0045 and ADR 0046.

**Blocked by:** 02

**Status:** resolved

- [x] Alt with a primary drag Pans by exactly what the pointer moved, bounded by the Grid's edges.
- [x] A primary click alone still selects a Cell, and a primary drag without Alt does not Pan.
- [x] Alt with a primary drag writes nothing to the Source.
