# The Minimap sits in the Source's corner at one pixel per Cell

Status: accepted. Amends [ADR 0081](0081-the-minimap-floats-over-the-windows-corner.md)'s corner and
its sizing. Settled with Toby on 2026-10-31, choosing from captures of the real web console.

ADR 0081 pins the Minimap to the window's bottom-right corner, so it covers the bottom panel and
docked Diagnostics. It draws as many whole physical pixels per texel as the whole pixels per point
allow, so at 2× the picture is 256 points a side. The web console runs at 2×, so the picture took
40% of a 1280-pixel window's width. Over an empty Grid it showed as a large plain square cut into
the Panel, and it looked detached from the console.

## Decision

**The Minimap's picture's corner is the Source's bottom-right corner.** It sits on the bottom
panel's top edge, flush with the window's right edge, and the bottom panel's separator runs along
its top and left edges as before. It floats over the Source only: the bottom panel and docked
Diagnostics are never covered.

**The picture is one physical pixel per texel at every pixels per point.** The 256 by 256 Grid is
256 physical pixels a side: 256 points at 1×, 128 at 2×. It is hidden where it and its border do
not fit in half the Source's width and its whole height.

ADR 0081's other decisions stand: the picture is not clipped or scaled below one pixel per texel,
and the Minimap's layer is the bordered picture's rect.

## Rejected alternatives

The alternatives were captured from the real web console, in both appearances, on desktop and phone.

**Drawing the sector seams into the picture.** It reads as the Grid in miniature, but it repeats
the Source's own lines, so on the Grid it gets lost against them.

**Inset on the Grid, one Cell from the Source's edges.** With seams it gets lost against the Grid.
Matted in Panel background, it stands off the Grid but floats away from the rest of the console.

**A wing of the Panel**, the separator stepping up around a padded picture. It still covers the
Panel's row.

**Hanging from the menu bar** at the top right. It covers Cells at the top of the Source, which the
Source View shows when it opens.

**Two or more physical pixels per texel at 2× and above.** A crisper picture, at four times the
area.

## Consequences

- Each Cell is one physical pixel. At 2× the picture is a quarter of ADR 0081's area, and a ninth
  at 3×.
- A Source too small for the picture hides the Minimap while the View menu keeps it ticked, as ADR
  0081 already does for the window: under 514 points wide or 257 high at 1×, under 258 wide or 129
  high at 2×. The room is the Source's, not the window's, so open docked Diagnostics take height
  from it, and the console's smallest window, 300 by 220 points, never shows the Minimap.
- The bottom panel's row and its status line are never under the Minimap, so ADR 0095's line
  beneath the Panel's row stays for its own reasons.
- A phone-width web console holds the picture at 1 pixel per Cell. Its page settings still open
  it closed (ADR 0091).
