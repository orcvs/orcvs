# The Overview floats over the window's corner

Status: accepted. Amends [ADR 0080](0080-the-overview-pans-over-a-cached-picture.md)'s placement of the Overview beside the Source View and its rejection of a floating overlay. Settled in `.scratch/overview-one-call/`.

ADR 0080 docks the Overview beside the Source View and rejects a floating overlay because it covers Cells the performer is working on. Docked, the Overview took 260 points or more of width from the Source.

## Decision

**The Overview is an egui `Area` pinned so its picture's corner is the window's bottom-right corner.** It floats over the Source, the bottom panel and docked Diagnostics, and takes no width or height from any of them. Opening or closing it leaves the Source's rect unchanged.

**The picture fits in half the window's width and the height below the menu bar**, at whole physical pixels per texel, and is hidden where one pixel per texel does not fit.

**Its layer is the bordered picture's rect.** The `Area` is sized to that rect from its first pass and never moved to fit the window, so it takes no pointer from the Source around it. A press or drag outside the picture reaches the Source as it would with the Overview closed.

## Rejected alternatives

**Docked beside the Source View**, as ADR 0080 decided. It keeps every Cell uncovered at the cost of the Source's width on every Render Frame the Overview is open. Floating, the Overview covers only the window's bottom-right corner, and the View menu closes it.

## Consequences

- The Cells under the picture are hidden while the Overview is open.
- The Overview draws over docked Diagnostics as well as the Source, so a readout under the picture is hidden until the Overview is closed.
- ADR 0080's other decisions stand: the Overview is a Pan gesture through one pending request before the clamp, and its picture is cached across Render Frames.
