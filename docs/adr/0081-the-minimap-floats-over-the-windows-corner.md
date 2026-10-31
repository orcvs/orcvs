# The Minimap floats over the window's corner

Status: accepted; its corner and its sizing are amended by [ADR 0096](0096-the-minimap-sits-in-the-sources-corner-at-one-pixel-per-cell.md). Amends [ADR 0080](0080-the-minimap-pans-over-a-cached-picture.md)'s placement of the Minimap beside the Source View and its rejection of a floating overlay. Settled in `.scratch/minimap-one-call/`.

ADR 0080 docks the Minimap beside the Source View and rejects a floating overlay because it covers Cells the performer is working on. Docked, the Minimap took 260 points or more of width from the Source.

## Decision

**The Minimap is an egui `Area` pinned so its picture's corner is the window's bottom-right corner.** It floats over the Source, the bottom panel and docked Diagnostics, and takes no width or height from any of them. Opening or closing it leaves the Source's rect unchanged.

**The picture fits in half the window's width and the height below the menu bar**, at whole physical pixels per texel, and is hidden where one pixel per texel does not fit.

**Its layer is the bordered picture's rect.** The `Area` is sized to that rect from its first pass and never moved to fit the window, so it takes no pointer from the Source around it. A press or drag outside the picture reaches the Source as it would with the Minimap closed.

## Rejected alternatives

**Docked beside the Source View**, as ADR 0080 decided. It keeps every Cell uncovered at the cost of the Source's width on every Render Frame the Minimap is open. Floating, the Minimap covers only the window's bottom-right corner, and the View menu closes it.

**Clipping the picture to fit.** A press maps to a Pan only across the whole picture, so a clipped picture leaves part of the Grid out of reach of a press and the outline unable to show those Pans.

**Scaling the picture below one pixel per texel.** Nearest-filtered at a fractional ratio, it drops rows and columns of Cells unevenly. A box-filtered downsample at a whole ratio drops none, but it needs a second picture rebuilt beside the first, and it fades a lone Cell's colour into its neighbours'. It would serve only a window under about 522 points wide, or under about 261 points below the menu bar, on a display at one physical pixel per point, for the 256 by 256 Grid: a quarter of a 1080p screen's width at 100% scaling. At two pixels per point the console's smallest window still holds the picture at one pixel per texel. The case is too rare to carry a second picture for.

## Consequences

- The Cells under the picture are hidden while the Minimap is open.
- A window too small for one physical pixel per texel shows no Minimap. The View menu keeps its choice, and the Minimap returns once the window holds it.
- The Minimap draws over docked Diagnostics as well as the Source, so a readout under the picture is hidden until the Minimap is closed.
- ADR 0080's other decisions stand: the Minimap is a Pan gesture through one pending request before the clamp, and its picture is cached across Render Frames.
