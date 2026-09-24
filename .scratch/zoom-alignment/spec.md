# Zoom alignment: one zoom, egui's

**Status:** ready-for-agent

**Tags:** release/v1

## Problem Statement

A viewer who presses command `+` in the console expects what every egui app and every browser does: the whole interface grows, including menus, the Panel, the Diagnostics window and the Source Grid. In the console today that chord only changes the Source View's Cell size. The menus, the Panel readouts and the Diagnostics text stay fixed at one size with no way to enlarge them from inside the app. ADR 0045 took egui's zoom chords for the Source View's own Zoom and promised whole-UI zoom would move to command-shift. That promise was never built. It also probably cannot coexist with egui's defaults, because egui matches command `+` logically and command-shift `=` is command `+` on a US layout. So the decision record and the shipped behaviour disagree. The release definition forbids that, and the console carries two zoom concepts where the toolkit expects one.

## Solution

The console uses egui's zoom and no other. Command `+` (or `=`) and command `-` step egui's whole-UI zoom, and command `0` resets it, exactly as egui does by default. The Source Grid grows with everything else because its Cells are sized in points. The View menu offers the same three actions with their chords shown, so the zoom can be found without knowing the keys. The zoom a viewer chose is kept across a restart wherever egui memory is persisted. The Source View's own Zoom, with its stepped Cell sizes, limits and chords, is removed. Pan and the Cursor follow remain, since a zoomed Grid can still be larger than the window.

## User Stories

1. As a viewer, I want command `+` to enlarge the whole console, so that the menus, Panel, Diagnostics and Grid all become easier to read at once.
2. As a viewer, I want command `=` to enlarge the console too, so that I do not have to hold Shift to reach `+` on a US keyboard.
3. As a viewer, I want command `-` to shrink the whole console, so that more of a large Grid fits in the window.
4. As a viewer, I want command `0` to return the console to its normal size, so that I can recover from any zoom in one step.
5. As a viewer, I want zoom to behave as it does in other egui apps and in my browser, so that I do not have to learn a console-specific rule.
6. As a viewer, I want View → Zoom In, Zoom Out and Reset Zoom menu items that show their chords, so that I can discover zoom without knowing the keys.
7. As a viewer who relies on a larger interface, I want the zoom I chose to still be in effect after I restart the console, so that I do not have to re-apply it every launch.
8. As a viewer of a build without persistence, I want the console to start at normal size, so that nothing half-remembered surprises me.
9. As a performer typing into the Source, I want bare `+`, `-`, `=` and `0` to remain Source characters, so that zoom never costs me a Glyph.
10. As a performer, I want a zoom chord never to write to the Source, so that zooming mid-performance cannot change the program.
11. As a viewer, I want the Grid's Cells to stay square and whole-pixel at every zoom, so that Glyphs and borders render crisply.
12. As a viewer, I want Grid lines, Sector Seams and Cursor strokes to keep their Theme-defined display-point widths at every zoom, so that the Theme looks as designed.
13. As a viewer, I want Pan to keep working when a zoomed Grid is larger than the window, so that I can reach every Cell.
14. As a viewer, I want the Cursor follow to keep the Cursor in view after a zoom, so that I never lose my place.
15. As a viewer, I want a zoom that would open a gap past the Grid's edge to settle the view back inside the margin, so that the Source never drifts off-screen.
16. As a viewer of the web build, I want command `+` to change exactly one zoom, egui's or the browser's, never both at once, so that the result is predictable.
17. As a viewer, I want the Diagnostics window to stop reporting a Source View Zoom that no longer exists, so that it describes the console truthfully.
18. As a Theme author, I want Theme widths to keep meaning display points, so that a custom Theme renders the same at every zoom.
19. As a maintainer, I want one zoom concept in the code and the glossary, so that there is no second transform to keep consistent.
20. As a maintainer, I want the decision record to state the zoom the console ships, so that ADR 0045 no longer promises behaviour that does not exist.
21. As a maintainer, I want no shipped code path that only tests can reach, so that removing the Source View's own Zoom leaves no test-only branch behind.
22. As a release reviewer, I want documentation, the glossary and behaviour to agree on zoom, so that the release definition's agreement line holds.

## Implementation Decisions

- **egui owns zoom.** The console stops turning off egui's keyboard zoom. The chords, step size (0.1), range (0.2 to 5.0) and reset are egui's defaults, unchanged. The console adds no zoom-handling code of its own.
- **The View menu exposes egui's zoom** through egui's own zoom menu buttons, which show the chords while keyboard zoom is on. The View menu keeps its existing mode and Theme pickers.
- **Persistence is egui's.** egui serialises its zoom factor with egui memory, and eframe saves egui memory in a persistence build. The console adds no storage key. A build without persistence starts at 1.0.
- **The Source View's own Zoom is removed:** the Source View's zoom state, the zoom command recogniser and stepped-zoom function, the zoom limits, and the quantised glyph scale derived from them. The Source scene transform keeps translation (Pan) and loses its independent scaling. Glyphs are laid out at the Source's own size, and egui's pixels-per-point does the enlargement.
- **Cell geometry is unchanged in points.** A Source Cell stays 16 points with an 11.5 point Glyph. The existing snapping of Cells to whole physical pixels at any device scale also covers every egui zoom factor, because egui's zoom factor is folded into pixels-per-point.
- **Glyph atlas:** egui keys its font atlas to pixels-per-point, so each zoom level rasterises its own atlas. The fifteen-step budget ADR 0045 reasoned about no longer applies, because only one Glyph size exists per zoom level.
- **Pan, the margin (ADR 0047) and the Cursor follow stay.** Their tests no longer take a Source View Zoom as a parameter. Where they varied zoom, they vary the window or egui's zoom factor instead.
- **Decision record:** a new ADR supersedes ADR 0045's zoom section. It records that zoom is egui's whole-UI zoom and nothing else, that command-shift was withdrawn because it collides with egui's logical match of command `+`, and that the canvas-style pinch and pointer Zoom (already retired by ADR 0045) stay retired. ADR 0038 and ADR 0040 get a pointer where they discuss the Source View's scaling.
- **Glossary:** CONTEXT.md's **Zoom** becomes "egui's whole-UI zoom: a scale of the whole console, Source Grid included, in egui's steps", and keeps its avoid-list.
- **Release scope:** this replaces `source-view/12`. Both tickets block `v1-release/03`. The second is required, not cleanup: after the first, a Source View Zoom other than 1.0 is reachable only from tests, which the repository contract forbids in shipped code.

## Testing Decisions

- **Test through the shipped console, not the zoom arithmetic.** A good test drives the real console with key events and observes what a viewer would see: egui's zoom factor, the Cell under the Cursor, the Source View's Pan and the painted shapes. It never asserts on a private zoom helper.
- **One seam: the kittest harness over the running console** (the `running_console` fixture in the console's kittest tests). It already observes egui's zoom factor, the Source View and the Cell under the Cursor.
  - Command `+`, `=`, `-` and `0` change egui's zoom factor and leave the Source untouched.
  - Bare `+`, `-`, `=` and `0` still write Glyphs.
  - The View menu's zoom items change egui's zoom factor.
  - A zoomed console still paints square whole-pixel Cells, and Theme widths stay at their display-point values.
  - A zoom that would open a gap settles the view back inside the margin.
- **Persistence:** a storage round-trip test sets a zoom factor, saves and restarts, and observes it restored. Prior art is the restored-Theme-reference tests in the console's storage tests, and the theme-preference round-trip `theming/16` plans.
- **Existing tests to flip or remove, not keep alongside:**
  - `command_zoom_chords_change_the_source_view_and_never_the_source` becomes "command chords change egui's zoom and never the Source".
  - The stepped-zoom, zoom-command and glyph-scale unit tests are deleted with the code they test.
  - Tests parameterised over Source View Zoom levels (grid and sector widths, zoomed Sector Seams, Cursor stroke widths, zoomed Cursor follow) are re-expressed over egui zoom factors or device scales, because the property they protect (fixed display-point widths, clipping, the Cursor follow) still matters.
- **Web:** the headless browser suite (merge tier) checks that command `+` reaches egui's zoom factor. Whether the browser also zooms the page is recorded, and the outcome that avoids a double zoom is chosen.
- **Performance:** `paint_derive` must stay within `benches/floors.toml`. Removing the Source transform's scaling should cost nothing; the pull request's benchmark job confirms it.

## Out of Scope

- Pinch-to-zoom and command/ctrl-scroll zoom of the Grid: a canvas convention from the earlier infinite-canvas design, not egui's text-app default. Plain scroll still Pans.
- Sizing the Grid independently of the chrome, such as projecting a large Grid with normal-size menus.
- Font choice or a font-size setting separate from zoom.
- Changing egui's zoom steps, range or chords.

## Further Notes

- `source-view/04` (keyboard Zoom) stays resolved as history. Its behaviour is superseded by this effort's ADR, not reopened.
- `theming/16` (round-trip the theme preference through storage) exercises the same egui-memory persistence path. Whichever lands first sets the restart-test pattern for the other.
