# The Source View has no margin

Status: accepted. Supersedes the margin and Pan reach of [ADR 0047](0047-the-source-view-has-a-margin.md); its default window and pointer cursors stand.

This decision was accepted as ADR 0083 and renumbered to 0084, so history written before the renumbering — commits, pull requests, and issue files that cannot be rewritten — cites it as "ADR 0083" and means this file rather than [ADR 0083](0083-the-console-opens-through-its-own-window-as-a-splash.md), which held the number first and kept it.

ADR 0047 set the Grid two Cells in from the console, and let a Pan reach two Cells past each edge, because a Grid pinned flush to the console's edges could not be told from them and a Cursor on the first or last row or column sat against the chrome. The bars now draw their Source-facing edges in `panel.separator`, a line every built-in Theme keeps apart from the Grid's own lines, so the chrome no longer reads as more Grid. The window's left and right edges have no chrome to confuse with the Grid at all.

## Decision

**The Source View shows the Grid and nothing past it.** At a Pan of zero the Grid's top-left Cell rests on the console's top-left. A Pan stops where the Grid's far edge meets the console's, and a Cursor follow to an edge Cell brings that Cell flush with the console's edge. The extent the Source View and the Overview are measured against is the Grid itself, so the Overview's picture is one texel per Cell of the Grid. On an axis where the Grid is smaller than the console, it rests at the top-left and has nowhere to Pan, as before.

## Rejected alternatives

**Keeping the margin at the far edges only.** The Grid would rest flush at the top-left while a Pan could still open a two-Cell gap at the bottom and right, so the two ends of one axis would disagree.

## Consequences

Everything the Source View shows is a Cell of the Grid: nothing [ADR 0043](0043-the-grid-is-bounded-by-definition.md) places outside the Grid is ever on screen. A Cursor on Cell 0,0 draws its outline against the window's left edge and the top bar's separator.
