# Source Cell size: Orca-shaped, configurable Cells, and no Cursor Effect

**Status:** ready-for-agent

## Problem Statement

A performer coming from Orca finds the console's Source Grid the wrong shape. Every Cell is a 16 point square, and an 11.5 point monospace Glyph sits inside it with a lot of space either side, so a row reads as spaced-out letters rather than a line of text. Orca draws 2:3 tiles that are taller than wide, so its rows read like a terminal. The square is hard-coded in the console, and ADRs 0038, 0053 and 0058 all describe it, so there is no way to change it, even to try another size.

The Cursor Effect makes the Grid harder to read as well. Around the Cursor it paints an animated glitch: a living green Area and an eroded frame. The performer finds it distracting and wants it gone.

## Solution

The console draws its Source Cells as rectangles, 12 points wide and 18 tall by default. That is Orca's 2:3 shape at about today's size. The width and height can be set in the console's settings file. The Glyph's size follows the Cell's height at Orca's proportion, three quarters of it, so any Cell size the performer picks keeps its letters in proportion. Clicking, dragging, Pan, the Cursor follow, Sector Seams and Zoom work the same on rectangular Cells as they did on square ones. Each axis is snapped to whole physical pixels on its own.

The Cursor Effect is removed. Its replacement is a plain, still outline: the Theme's `cursor.border` around the Cursor's Cell, and the Theme's `region.border` around a Region larger than one Cell. Both are drawn at the Theme's stated widths. The settings and Theme keys that existed only for the effect go away. The effect is turned off by default straight away, and deleted in a later ticket.

## User Stories

1. As a performer coming from Orca, I want Source Cells that are taller than wide, so that a row of the Source reads as a line of text.
2. As a performer, I want the default Cell to be about the size of today's, so that the Grid doesn't suddenly look much bigger or smaller.
3. As a performer, I want to set the Cell's width in my settings file, so that I can make columns tighter or looser.
4. As a performer, I want to set the Cell's height in my settings file, so that I can make rows tighter or looser.
5. As a performer, I want the Glyph to grow and shrink with the Cell's height, so that a Cell size I choose never leaves the letters too small or clipped.
6. As a performer, I want a settings file without Cell keys to give me the default Cell, so that I don't have to configure anything.
7. As a performer, I want a Cell width or height that is out of range or the wrong type to be reported in the usual settings notice and to fall back to its default on its own, so that one typo doesn't break the console or reset my other settings.
8. As a performer, I want a Cell size that doesn't land on whole screen pixels to be snapped quietly, so that rows and columns stay evenly spaced.
9. As a performer on a 1× or 2× display, I want the default Cell to be exactly 12 by 18 points, so that nothing gets snapped away.
10. As a performer, I want every column to be the same width and every row the same height at every Zoom, so that the Grid never looks uneven.
11. As a performer, I want a click anywhere inside a Cell to select that Cell, including near a Sector Seam and at the Grid's edges, so that rectangular Cells don't throw off my clicks.
12. As a performer, I want a Region drag to land on the Cell under the pointer on both axes, so that spanning a Region works as it did before.
13. As a performer, I want a Region drag held past the console's edge to scroll at most one Cell width across or one Cell height down each frame, so that the edge scroll keeps the pace it has today.
14. As a performer, I want the Pan margin past each Grid edge to stay two Cells, measured in Cell widths across and Cell heights down, so that the Grid's edge reads the same on every side.
15. As a performer, I want the Cursor follow to bring the Cursor's whole rectangular Cell into view, so that I never lose the Cursor at an edge.
16. As a performer, I want Zoom to scale the rectangular Cells along with the rest of the console, so that Zoom works as ADR 0058 describes.
17. As a performer, I want Sectors to stay the same number of Cells wide and high, so that Tab and Shift Tab still step by a Sector.
18. As a performer, I want Sector Seams to fall on Cell edges at their usual interval, so that the registration marks still line up with the Grid.
19. As a performer, I want Theme line and border widths to stay in display points whatever the Cell size, so that a Theme looks as it was designed.
20. As a performer, I want the default window to keep its size on screen, so that the narrower Cells show more columns instead of shrinking the window.
21. As a performer, I want the Diagnostics readout to show the Cell's width and height, so that I can see the Cell size actually being drawn.
22. As a performer, I want the Cursor shown as a plain, still outline in the Theme's Cursor border, so that I can find it without an animation pulling at my attention.
23. As a performer, I want a Region larger than one Cell outlined in the Theme's Region border, so that I can see what I've selected.
24. As a performer, I want the outlines drawn at the Theme's Cursor and Region border widths, so that a Theme controls how heavy they look.
25. As a performer, I want nothing around the Cursor to animate, so that a still Source stays still.
26. As a performer with reduced motion turned on, I want the console to look exactly like it does for everyone else, so that there is no second mode to explain.
27. As a performer whose settings file still has the old Glitch keys, I want the console to start and report those keys as unknown, so that I know to remove them.
28. As a Theme author whose Theme still sets `cursor.area`, I want it reported the way any unknown Theme key is, so that my Theme still loads.
29. As a performer on the web, I want the default Cell size, since the web reads no settings file, so that the web console looks like a native one that has no settings.
30. As a maintainer, I want one ADR that records the switch from square Cells to configurable rectangles and marks the square-Cell sentences in ADRs 0038, 0053 and 0058 as amended, so that the decision record agrees with the code.
31. As a maintainer, I want CONTEXT.md's Cursor, Cursor Effect, Sector and Theme entries brought up to date, so that the vocabulary describes what ships.
32. As a maintainer, I want the console's paint to stop requesting repaints that only the animation needed, so that a still console stops waking.

## Implementation Decisions

- **Order.** The Cursor Effect is turned off by default first: Glitch amount and frequency default to zero, which already draws the still outline and schedules no repaint. Then the Cell becomes rectangular and configurable. Deleting the effect is a later, separate ticket that nothing else waits on, so until then the prefactor carries the effect's geometry over to a width and a height.
- **The Cell is a width and a height.** The console's single Cell side becomes a 2-D size everywhere it is read: the viewport geometry, the Source View's Pan bounds, margin, Cursor follow and edge scroll, the shapes step, the Diagnostics readout, the Diagnostics source reserve (which reads the height) and the default window size.
- **The viewport geometry stays the single expression of the Cell fit.** A Cell's rectangle is its column times the width and its row times the height, from the presented Grid's corner. The Cell a point falls in is found by dividing by each axis's own side, with the existing exact-inverse correction applied per axis. The snap floors each axis on its own to whole physical pixels, at least one pixel per axis. A Cell that isn't positive and finite on both axes is no Cell at all.
- **`presented_grid` takes the Cell size** instead of reading a constant. The console owns the configured size and passes it in, so the Pan bounds and the drawn Grid read the same snapped Cell.
- **Glyph size is derived.** It is three quarters of the configured Cell height in points (13.5 at the default 18), and replaces the shared default-font-size constant wherever the Source's Glyphs are laid out. egui still rasterises one Glyph size per Zoom factor, as ADR 0058 says.
- **Settings.** A new `[cell]` section in `~/.orcvs/config.toml` takes `width` and `height`, in points, both optional. It is read with the existing settings reader's rules: an absent key is the default, and an out-of-range or mistyped value is a notice that falls back on its own. The bounds are 4 to 64 points per axis. The web reads no file and uses the default.
- **The Cursor outline.** The Cursor's Cell is stroked inside its rectangle in `cursor.border` at `cursor.border.width`. A Region larger than one Cell is stroked around the whole Region in `region.border` at `region.border.width`, and the Cursor inside it is stroked as well. A width of zero draws no stroke, as Theme widths already do.
- **Removed with the effect:** the Cursor Effect module and its settings type, the `[cursor_effects]` settings section with its Glitch amount and frequency, the `cursor.area` Theme key, the reduced-motion preference read only for the effect, the effect's sampling and repaint scheduling, and its benchmark cases. A file that still has the removed keys gets the existing unknown-key notice. There is no compatibility shim, because the language and console are pre-release.
- **Decision records.** One new ADR (next number) records rectangular, configurable Cells, the derived Glyph, the per-axis snap and the removal of the Cursor Effect. It names the sentences it amends in 0038 ("square-Cell fit"), 0053 ("Existing square Cells … remain unchanged" and the Cursor Effect motion settings) and 0058 ("square, whole-pixel Cells", "16 points with an 11.5 point Glyph"). CONTEXT.md's Cursor, Cursor Effect (removed), Sector ("square block" becomes a block of Cells) and Theme (the Cursor Effect sentence) entries are updated to match.

## Testing Decisions

- A good test pins behaviour a viewer could notice, through the highest seam that answers it: which Cell a point resolves to, which rectangle a Cell is drawn at, what a settings file yields, and which Shapes a Render Frame paints. Tests don't reach into private helpers.
- **The viewport geometry is the primary seam.** Its existing tests carry over, now with Cells whose width and height differ: a Cell's own corner and centre resolve to that Cell across a sweep where width and height step independently (so reading one axis with the other's side fails), the far edge resolves to the last Cell, every Cell is a whole number of physical pixels on both axes at every Zoom step, and a degenerate width or height answers no Cell. The test-only fitted viewport fits Cells of the default shape, not squares.
- **Settings reading** is tested at the settings reader as it is today: the default when there's no `[cell]` section, both keys read, one bad key falling back on its own while the other is kept, and the removed Glitch keys reported as unknown.
- **Console behaviour** is tested through the existing headless console and kittest suites. Clicks on Cells near Sector Seams and the Grid edge resolve correctly. Pan bounds and the margin use the Cell's width and height. The Cursor follow and the edge scroll are capped per axis. The Diagnostics readout shows width by height. The Cursor and Region outlines are drawn in the Theme's colours and widths, and no Area Shapes are painted.
- Prior art: the viewport geometry's exact-inverse sweep, the settings reader's fall-back-alone tests, the console tests that read the painted Shapes of one Render Frame, and the kittest tests that check whole-pixel Cells at a Zoom.
- The second slice is built test-first at the viewport seam, one behaviour at a time, before any caller changes.

## Out of Scope

- A settings UI, menu or slider for the Cell size. A later ticket can add one.
- Reloading settings while running.
- Changing the Cell size from Zoom chords the way Orca's zoom keys do. Zoom stays egui's whole-UI zoom.
- Choosing a font or Glyph face. Only the size is derived.
- Any Cursor animation, blink or motion setting to replace the effect.
- The theme workbench page that lives outside the repository. Its Cell drawing is updated by hand afterwards and never moved or renamed.

## Further Notes

- At the default 12 by 18, a Cell is whole on both axes at 1×, 1.5× and 2×. At Windows' 1.25× the height is 22.5 pixels, so it snaps to 22. Most egui Zoom steps produce fractional sizes for any Cell, today's 16 point square included, so the per-axis floor is what keeps rows and columns identical there.
- The settings reader already treats unknown keys and sections as notices, so removing the Glitch keys can't stop a console from starting.
- Check the real console as well as the tests: compare against a screenshot from before the change, and click Cells near Sector Seams and the Grid edge.
