# 07: A Region beside the Cursor

**What to build:** `Orcvs` holds a Region — an anchor Cell and the Cursor — and the Render Frame carries it. A Region always exists and is the Cursor's one Cell when the anchor sits on the Cursor. The console tints every Cell of a Region larger than one Cell; the Cursor Effect stays on the Cursor's Cell. Nothing yet makes a Region larger than one Cell except a test. See ADR 0046.

**Blocked by:** 06

**Status:** ready-for-agent

- [ ] A Region spans from its anchor to the Cursor, and moving the Cursor past the anchor flips the rectangle.
- [ ] A fresh Orcvs has a Region of the Cursor's one Cell.
- [ ] The Render Frame carries the Region, and the console tints every Cell of a Region larger than one Cell and none of a Region of one Cell.
- [ ] The Cursor Effect is drawn on the Cursor's Cell alone.
- [ ] The Region is not stored with the Source.
