# Zoom is egui's whole-UI zoom

Status: accepted. Supersedes the zoom section of [ADR 0045](0045-the-source-view-is-a-bounded-space.md) — its keyboard Zoom of the Source View's Cell size and its promise of whole-UI zoom on command-shift. ADR 0045's bounded Pan, its 16 point Cell and its retirement of the pointer Zoom stand, as does [ADR 0047](0047-the-source-view-has-a-margin.md)'s margin.

**The console has one zoom, and it is egui's.** Command `+` (or `=`) and command `-` step egui's zoom factor, and command `0` returns it to 1.0, with egui's own step of a tenth and egui's own range of 0.2 to 5.0. The console adds no zoom handling of its own: it leaves `Options::zoom_with_keyboard` on, as egui ships it, and the View menu offers egui's own Zoom In, Zoom Out and Reset Zoom buttons, which show the chords while keyboard zoom is on. A viewer who presses command `+` gets what every egui app and every browser gives them: the whole interface grows — the menus, the Panel, the Diagnostics and the Source Grid together.

**The Source Grid grows with everything else because its Cells are sized in points.** egui folds its zoom factor into `pixels_per_point`, so a Source Cell stays 16 points with an 11.5 point Glyph and becomes more physical pixels. The console already snaps a Cell's side and the Grid's corner to whole physical pixels at any `pixels_per_point`, so every zoom factor paints square, whole-pixel Cells with nothing new to snap. Theme widths are display points and stay so: a Grid line, a Sector Seam and a Cursor stroke keep the width the Theme states in points at every zoom. egui keys its font atlas to `pixels_per_point`, so each zoom factor rasterises one Glyph size; the fifteen-size budget ADR 0045 reasoned about does not arise.

**Zoom changes the console's size in points, so the Source View's bounds and Cursor follow answer it as they answer a resize.** A zoom that would open a gap past the Grid's margin settles back inside through the same Pan clamp, and a zoom that would leave the Cursor out of view Pans just far enough to bring it back.

**The zoom persists where egui memory does.** egui serialises its zoom factor with its memory, and eframe saves that memory in a persistence build and restores it before the console is built. The console stores no key of its own. A build without persistence starts at 1.0.

**On the web the browser's page zoom is the one zoom.** eframe's web runner turns egui's keyboard zoom off and sets the zoom factor to 1.0 before it builds the app, and leaves the browser's default action for command `+`, `-` and `0` alone, so the browser zooms the page and eframe follows the device pixel ratio as the native `pixels_per_point`. The console leaves that as eframe sets it. Turning egui's keyboard zoom on there would zoom twice: the browser's page and egui's factor on top of it.

**Command-shift is withdrawn.** ADR 0045 moved whole-UI zoom to command-shift `=` and command-shift `-`. That was never built, and it cannot coexist with egui's defaults: egui matches command `+` logically and ignores an extra Shift, and on a US layout command-shift `=` is command `+`. Bare `+`, `-`, `=` and `0` remain Source characters.

**The canvas conventions stay retired.** Pinch and command-wheel zoom the Grid in the infinite-canvas design ADR 0045 replaced. They are not egui's text-app default and do not return; plain scroll still Pans.

## Rejected alternatives

**Keep a Source View Zoom beside egui's.** Two zooms mean two transforms to keep consistent, a glossary with two meanings for one word, and chords that do different things in this console from every other egui app. The Source View's own Zoom stays at 1.0 and responds to no input; `.scratch/zoom-alignment/issues/02` removes it.

**Size the Grid independently of the chrome.** Projecting a large Grid under normal-size menus is a real use, but it is a separate setting with its own design, not a second meaning of the zoom chords.

## Consequences

`CONTEXT.md`'s **Zoom** is egui's whole-UI zoom. ADR 0045's Status line points here.

While `.scratch/zoom-alignment/issues/02` is open, the Source View still carries a Zoom field, its limits and the Glyph scale quantised from them, reachable only by a test that sets the field.
