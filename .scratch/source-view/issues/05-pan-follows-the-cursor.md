# 05: Pan follows the Cursor

**What to build:** A Cursor move — by keyboard, by typing, or by a click — or a Zoom that would leave the Cursor outside the Source View Pans just far enough to bring it back. A Pan on its own may still leave the Cursor out of view. See ADR 0045.

**Blocked by:** 02, 04

**Status:** ready-for-agent

- [ ] A Cursor move that would leave the Cursor outside the Source View Pans the least distance that shows it.
- [ ] A Zoom that would leave the Cursor outside the Source View Pans the least distance that shows it.
- [ ] A Pan with no Cursor move and no Zoom is not pulled back to the Cursor.
- [ ] The follow Pan is still bounded by the Grid's edges.
