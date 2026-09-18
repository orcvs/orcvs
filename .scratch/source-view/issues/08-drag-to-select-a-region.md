# 08: Drag to select a Region

**What to build:** A primary press sets the anchor and the Cursor on the Cell under the pointer; a primary drag moves the Cursor to the Cell under the pointer; release keeps the Region; a click collapses it; Shift with a click extends it from its anchor. A drag past the console's edge scrolls the Source View through the Cursor follow. See ADR 0046.

**Blocked by:** 07

**Status:** ready-for-agent

- [ ] A primary drag spans a Region from the pressed Cell to the Cell under the pointer, and release keeps it.
- [ ] A primary click collapses the Region onto the clicked Cell.
- [ ] Shift with a primary click extends the Region from its anchor to the clicked Cell.
- [ ] A drag past the console's edge scrolls the Source View faster the further the pointer is outside, never more than one Cell a frame, and never past the Grid.
- [ ] Alt with a primary drag, and a middle drag, still Pan and leave the Region as it was.
