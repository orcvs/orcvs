# An Overview of the whole Grid

Status: ready-for-agent

## Problem Statement

A performer working on the Source Grid sees only the part of it the Source View shows. The Grid is always 256 by 256 Cells, far more than any window holds at the Source's own Cell size, so most of a Source is off screen at any moment. The performer cannot see where their work sits on the Grid, where other Functions, Comments or Diagnostics are, or which parts of the Grid Playback is reading and writing this Tick. Getting somewhere distant means Panning by wheel, middle-drag or Alt-drag across unknown territory, or moving the Cursor there a Sector at a time. Orca and its relatives offer no help here: Orca crops the Grid to the window and Orca-c only follows the Cursor.

## Solution

The console can show an **Overview**: a small picture of the whole Grid and its margin, docked beside the Source View, one block of colour per Cell in the Theme's colours. The Source View is drawn on it as an outline, so the performer always sees where they are. Pressing on the Overview Pans the Source View there; pressing inside the outline and dragging carries the Source View along with the pointer as if holding it. The Overview only Pans: it never moves the Cursor or changes a Region. It follows the Source as it changes, including every Tick during Playback, and follows the Theme. It is off by default and shown or hidden from the View menu, beside Diagnostics.

The research behind this spec is `docs/research/egui-minimap.md`.

## User Stories

1. As a performer, I want to see the whole Grid at once, so that I know where everything in my Source is.
2. As a performer, I want to see where the Source View sits on the Grid, so that I know which part I am looking at.
3. As a performer, I want the Source View outline to move as I Pan by wheel, middle-drag or Alt-drag, so that the Overview always agrees with what I see.
4. As a performer, I want the Source View outline to move when Cursor movement Pans the Source View, so that following the Cursor shows on the Overview too.
5. As a performer, I want to press anywhere on the Overview and have the Source View centred there, so that I can jump to a distant part of the Grid in one gesture.
6. As a performer, I want the jump to happen on press, without waiting for me to release or move, so that the Overview feels immediate.
7. As a performer, I want to press inside the Source View outline and drag, so that the Source View moves with my pointer from where I grabbed it, without first jumping to centre on the pointer.
8. As a performer, I want a press outside the outline to centre the Source View and then keep following my drag, so that one gesture both jumps and fine-tunes.
9. As a performer, I want a Pan from the Overview to stop at the same bounds as any other Pan, so that I can never Pan further than the margin past the Grid's edges.
10. As a performer, I want pressing on the Overview never to move the Cursor or change the Region, so that looking around never disturbs where I am editing.
11. As a performer, I want the Source View not to snap back to the Cursor after I Pan from the Overview, so that I can look elsewhere while my Cursor stays put.
12. As a performer, I want the grab and grabbing pointer cursors on the Overview, as on the Source View when Panning, so that I can tell the Overview Pans.
13. As a performer, I want occupied Cells to show in their role's colour, so that I can tell Functions, operands, Comments and Bangs apart at a glance.
14. As a performer, I want empty Cells to show as the Grid background, so that the shape of my work stands out.
15. As a performer, I want Diagnostics to show on the Overview in the Theme's Diagnostic colour, so that I can find errors anywhere on the Grid.
16. As a performer, I want Cells whose Portals Playback read or wrote in the last committed Tick, and Locked Cells, to show in their Portal colours, so that I can see where Playback is active.
17. As a performer, I want the Overview to update every Tick during Playback, so that it shows the Source as it is now.
18. As a performer, I want the Overview to update as soon as I type, delete or paste, so that it never shows a stale Source.
19. As a performer, I want the Cursor's position shown on the Overview, so that I can find my Cursor when it is off screen.
20. As a performer, I want my Region shown on the Overview, so that I can see what I have selected when it extends off screen.
21. As a performer, I want the Overview to draw each Cell as a crisp block at any Zoom and display scale, so that it never looks blurred or uneven.
22. As a performer, I want the Source View outline to shrink on the Overview as I Zoom in and grow as I Zoom out, so that it shows how much of the Grid I can see.
23. As a performer, I want the Source View outline clipped to the Overview on an axis where the whole Source already shows, so that it never draws outside the picture.
24. As a performer, I want no outline drawn when the whole Source already shows on both axes, so that the Overview does not suggest there is somewhere to Pan.
25. As a performer, I want the Overview docked beside the Source View rather than floating over it, so that it never covers Cells I am working on.
26. As a performer, I want to show and hide the Overview from the View menu, so that I can choose between Grid space and orientation.
27. As a performer, I want the Overview hidden by default, so that the console looks as it does today until I ask for it.
28. As a performer, I want showing or hiding the Overview to keep my Cursor in view when it was in view, so that toggling it behaves as toggling Diagnostics does.
29. As a performer, I want the Overview never to take keyboard focus, so that Tab and every typed key still reach the Source.
30. As a performer, I want the Overview to work identically in the browser build, so that the web console offers the same orientation.
31. As a performer, I want the Overview to reflect a Theme change at once, including following the operating system's appearance, so that it always matches the rest of the console.
32. As a Theme author, I want the Overview's Cell colours to come from the colours I already set for the Source Grid, so that I do not have to style the Grid twice.
33. As a Theme author, I want one colour and width for the Source View outline on the Overview, so that I can make it read over my Theme's Cell colours.
34. As a Theme author, I want the new outline keys, `overview.source_view` and `overview.source_view.width`, in the Theme schema and both built-in Themes, so that a custom Theme that leaves them out takes the built-in values, as it does for every style key, and only a width over 2 is refused.
35. As a performer using assistive technology, I want the Overview labelled as a Source overview rather than exposing every Cell, so that it does not flood the accessibility tree with 65,536 nodes.
36. As a performer using assistive technology, I want everything the Overview does to remain possible by keyboard, so that the Overview is a convenience and not the only way to Pan.
37. As a performer on a slow machine, I want the Overview to cost nothing on Render Frames where the Source and Theme have not changed, so that showing it does not slow the console.
38. As a maintainer, I want the Overview's per-revision cost measured by a benchmark with a floor, so that a regression in it fails CI.
39. As a maintainer, I want the Overview's term, gesture and cache recorded in CONTEXT.md and an ADR, so that later work uses the same vocabulary and respects the same constraints.

## Implementation Decisions

- **Term.** The feature is the **Overview**. CONTEXT.md gains the entry: a reduced picture of the whole Grid and its margin, one mark per Cell in the Theme's colours, beside the Source View; it shows where the Source View is, and a press or drag on it Pans the Source View there; it carries no content, holds no Positions, and moves no Cursor. _Avoid_: minimap, map, thumbnail, navigator, viewport, scrollbar. The outline is "the Source View, as the Overview shows it"; no second term is minted. "Map" is avoided because it collides with Language Map.
- **ADR.** A short ADR records three decisions: the Overview is a further Pan gesture, amending ADR 0045's list of Pan gestures; the Overview's picture is cached in `console` across Render Frames, keyed by the Source's revision and the presented Theme, and not on the Language Map (ADR 0050 was rejected for moving whole-Grid work into revision rebuilds), and a retained texture is consistent with ADR 0040's one-frame GlyphTable because it holds no galleys; and one new Theme key styles the outline.
- **Ownership.** Everything lives in `console`. `orcvs` gains no API: the picture is derived from the Render Frame the console already reads each Render Frame and from the Source's existing revision identifier.
- **Picture.** A console-owned buffer of one colour per Cell, 65,536 of them, and the two-Cell margin around them as Grid background, 260 by 260 colours in all, derived from a Render Frame and a Theme. Each Cell's colour comes from the per-Theme Cell colour table Paint already uses, given the Cell's Source paint and Portal roles, reduced to one colour by a fixed rule: the role's foreground where the Cell is occupied, otherwise the Cell's background where it has one, otherwise the Grid background. The rule is fixed precedence in code, which ADR 0053 allows. No glyph is drawn. The Cursor and the Region are not baked into the buffer.
- **Cache.** The buffer is rebuilt only when the Source's revision identifier or the presented Theme and appearance differ from those it was built from, and is reused, not reallocated. The revision identifier is read before the Render Frame is taken, so a Tick that commits between the two causes at most one extra rebuild and never a stale picture.
- **Texture.** The picture is drawn as one texture of 260 by 260 texels, the Grid and its margin, loaded once with nearest filtering and updated in place, with one whole-image partial update, at most once per Render Frame, only when the buffer was rebuilt. No per-Cell Shapes, mesh, or paint callback.
- **Shape and size.** One texel per Cell, square, drawn at a whole number of physical pixels per texel, recomputed from the current pixels-per-point each Render Frame and snapped to pixels. The Overview's domain includes the two-Cell margin on every side, drawn as Grid background, so the outline can reach every Pan. Squares are chosen over the Source's Cell aspect because a 2:3 picture at one physical pixel per texel cannot keep every row the same height; the outline's position and extent are still exact in Cells.
- **Overlays.** Over the texture, as Shapes: the Region, the Cursor's Cell, then the Source View outline. The outline is the Source View's visible region in the Source's own points, offset by the margin and scaled to the Overview, clipped to the Overview, and omitted when neither axis has anywhere to Pan. It is computed from the existing visible-Source-region answer, not from the visible Positions, which include an extra Cell on each side.
- **Theme.** One new key for the outline's colour and width, added to the Theme's style catalogue, the Theme schema and both built-in Themes, within ADR 0053's width bounds for chrome. Cell colours, the Cursor and the Region reuse their existing keys. Nothing on the Overview animates.
- **Pan request.** `SourceView` gains a single pending-Pan request, expressed as the point of the padded Source the Source View should centre on. The Source scene applies a pending request before its one existing Pan clamp, measured with the snapped Cell, so the Overview obeys the same bound as every other Pan. The Overview never sets the Pan directly.
- **Pointer semantics.** The Overview senses drag only — not click, not focus — so it is out of the Tab order and acts on the press frame without a click-versus-drag delay. On press, if the pointer is outside the outline the grab offset is zero; inside, it is the offset from the pointer to the outline's centre. While the button is held, each Render Frame requests a Pan centring the Source View on the pointer plus that offset. It shows the grab cursor on hover and grabbing while held. It never selects, moves the Cursor, or changes a Region, so the Cursor-follow rule (a Pan alone does not chase the Cursor) holds unchanged.
- **Frame order.** As docked Diagnostics does, the Overview reserves its space before the Source is shown and is filled after, from that Render Frame's geometry; its Pan request is applied on the next Render Frame. One Render Frame of lag during a drag is accepted.
- **Placement and toggle.** A right-docked egui side panel shown before the Source, so it takes width rather than covering Cells. A checkbox labelled "Overview" beside "Diagnostics" in the View menu, not persisted, off on start. Toggling it changes the console's size, which the existing resize rule already handles.
- **Accessibility.** The Overview's response carries the label "Source overview" and exposes no per-Cell nodes.
- **Benchmark.** A Criterion case for deriving the picture from a 256 by 256 Render Frame joins the console paint benchmarks with a floor in the benchmark floors file, reached without widening any shipped item for it. The comparison is CI's.

## Testing Decisions

- A good test drives behaviour a performer could observe — what the Overview draws and what a press or drag on it does to the Source View and Cursor — and never inspects the cache, the texture handle, or private fields. Coordinates are derived from live geometry, never written down.
- **Seams, highest first; no new seam is cut into shipped code.**
  1. **The shipped `Console` under `console::kittest_tests`** (existing): the View menu's "Overview" checkbox found by label shows and hides it; a press outside the outline centres the Source View there; a drag from inside the outline moves the Source View by the drag without a jump; neither moves the Cursor nor changes a Region; Tab never focuses it; a Pan from the Overview stops at the margin. Prior art: the docked-Diagnostics View-menu test and the Alt-drag Pan test that shows a Pan reaches the Source as nothing.
  2. **Shape-level `console::tests`** (existing): one textured image at the predicted, pixel-snapped rect; the outline stroke at the predicted rect after a Pan and after a Zoom; the outline clipped on an axis with nowhere to Pan and absent when neither axis can Pan; the Cursor and Region marks at their Cells. Prior art: the existing Source Shape assertions and the visible-Source-region tests.
  3. **The picture as a value** from a Render Frame and a Theme, with no `egui::Context`, because ADR 0040 requires that no test asserting a Cell's colour constructs one and kittest cannot read texture pixels: an empty Grid is all Grid background; a Function, a Comment, a Bang and an invalid operand take their Theme colours; Portal roles from a committed Tick take their Portal colours; a Theme change changes the picture. Prior art: the `CellColours` and Paint derivation tests.
- The pending-Pan request's clamp is covered through seams 1 and 2, not tested in isolation; the arithmetic follows the existing pure `clamp_pan` tests if a case cannot be reached from above.
- No screenshot or snapshot test. Proptest is not needed.
- Gates: the console gate from CLAUDE.md, `mise run check_wasm` for the shared layout, the roadmap tests for the `.scratch/` change, and the benchmark added locally with its comparison deferred to CI.

## Out of Scope

- Zooming the Source View from the Overview (Tiled's wheel-to-zoom); Zoom stays egui's whole-UI Zoom and the wheel over the Overview does nothing.
- Drawing characters on the Overview; it shows blocks only.
- Dirty-rectangle texture updates; the whole image is updated, and a narrower update is a later change only if the benchmark asks for it.
- Persisting whether the Overview is shown across launches.
- A keyboard shortcut for the toggle; every printable key is Source input.
- Resizing the Overview by dragging its edge.

## Further Notes

- Research, with sources for every claim: `docs/research/egui-minimap.md`.
- The cost that matters is one 65,536-Cell pass and a 264 KiB texture upload per revision, which during Playback is every Tick. It is unmeasured until the benchmark lands; no claim about it is made before then.
- The decision to draw square texels rather than the Source's 2:3 Cells is the one presentation choice the research left open. It is decided here for crisp, uniform rows; revisit it only on evidence from use.
