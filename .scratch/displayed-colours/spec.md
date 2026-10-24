# Displayed colours

A refactor from the 2026-10-24 architecture review. contrast and the Overview each composited a Cell's colours over the console's surfaces, and differently. contrast composites the background as window ⊕ grid ⊕ Cell background, with the foreground over that, which is what the console shows. The Overview's texel was grid ⊕ foreground for an occupied Cell and grid ⊕ background for an empty one, which leaves out the window and an occupied Cell's own background. Only `window.background` must be opaque, so a custom Theme with a translucent `grid.background` drew translucent texels. Since fcc9ffce the Overview floats over the Source, so those texels showed the Source beneath them.

## Decisions

1. **`style` answers a Cell's displayed colours.** `CellColours::displayed(paint, portals, placement)` makes the colour decision and composites the background over window ⊕ grid, so the background is opaque. It returns the Theme's foreground beside it, and `displayed_foreground()` composites that over the background. `CellColours::surface()` is the opaque window ⊕ grid surface itself. contrast keeps WCAG measurement, and only the composition moves into `style`. It lives in `style` because the Overview ships on the web and `contrast` does not compile there.
2. **contrast and the Overview both call it.** `contrast::painted` and the Overview's `cell_texel` (the picture and the Cursor mark) collapse to one call each. The Overview's margin is the surface. Paint keeps calling `colours`, because egui composites what Paint draws.
3. **The Overview's Region mark is a location mark.** It is one rect of `region.background` over the picture. It shows where the Region is, not how its Cells look, so it tints Cells the Source leaves untinted inside a Region, such as a Portal's. An opaque `region.background` hides the texels it covers. That is the Theme's choice, and the mark does not cap it.
4. **The shipped colours do not change.** Both built-ins have an opaque `grid.background` and opaque Source foregrounds, so for both of them the contrast report and every displayed colour, at every reachable fact, Portal set and placement, are byte-identical before and after.
5. **No ADR and no new `CONTEXT.md` term.** The style-deepening rule that surface measurement stays in contrast still holds: measurement stays and composition moves. "Displayed colour" is an implementation word.

## Tickets

- `issues/01`: the displayed colours.
