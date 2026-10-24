# An Overview (minimap) of the Source Grid in egui

Status: decided in [ADR 0080](../adr/0080-the-overview-pans-over-a-cached-picture.md) and `CONTEXT.md`,
which settle the term and the design below; this note keeps the research behind them. "Minimap" is
the request's word. §6 proposed **Overview** as the domain term, because "map" collides with
Language Map and the glossary avoids "viewport", "scroll" and "drag". Citations are `path:line` in this repository, or in the egui `0.36.2` crates
(`egui-0.36.2/…`, `epaint-0.36.2/…`, `egui_glow-0.36.2/…`, `egui-wgpu-0.36.2/…` from the cargo
registry; `egui_demo_lib/…` from `emilk/egui` tag `0.36.2`, commit `49682f8b`). Prior art cites the
upstream repository and file.

## Recommendation

Draw the Overview as **one 256×256 texture, one texel per Cell, filtered `NEAREST`**, in a docked
strip beside the Source View, with the Source View drawn over it as one Theme-coloured stroke.

1. **Pixels.** A console-owned `Vec<Color32>` of 65,536 entries, filled from the Render Frame's
   per-Cell facts through the existing `style::CellColours` table (`console/src/style.rs:123-170`),
   reduced to one colour per Cell by a fixed rule (role foreground if occupied, else the Cell's
   background, else `grid_background`). No second classifier, no glyphs.
2. **Cache.** Rebuild the buffer only when the Source's `RevisionId` or the presented Theme and
   appearance change. Read `RevisionId` (`orcvs/src/app.rs:324`) before `render_frame()` so a Tick
   that commits in between costs one extra rebuild, never a stale image. Keep the cache in
   `console`, not on the Language Map.
3. **Texture.** `Context::load_texture` once with `TextureOptions::NEAREST`, then
   `TextureHandle::set_partial([0, 0], whole_image, NEAREST)` at most once per Render Frame when
   the key changed. Upload the whole image; dirty-rectangle patches only if a benchmark says so.
4. **Draw.** `Painter::image` into a rect sized to whole physical pixels and snapped with
   `round_to_pixels`, then the Source View rectangle as one `rect_stroke`, then the Cursor and
   Region as small Shapes so a Cursor move costs no rebuild.
5. **Interaction.** Sense `Sense::DRAG` only. A press outside the rectangle centres the Source View
   on the pointer; a press inside keeps the grab offset; either then follows the pointer. The
   gesture is a Pan: it goes through one new pending-Pan request on `SourceView`, applied before the
   existing clamp, and never moves the Cursor.
6. **Placement.** A docked `egui::Panel::right` shown before the Source, toggled from the View menu
   beside Diagnostics, off by default.

This needs a short ADR (it adds a Pan gesture, a cross-Render-Frame cache and a Theme key) and a
`.scratch/overview/` spec with issues, per `docs/agents/issue-tracker.md`.

## 1. Texture versus shapes

egui 0.36.2 ships no minimap or overview widget, and its demo has none
(`egui_demo_lib/src/demo/painting.rs` and `paint_bezier.rs` are the closest pointer-mapping
patterns). The console ships the glow renderer on native and WASM; wgpu is reached only through the
test-only `release-capture` feature (`console/Cargo.toml:92,113-118`).

| | Texture, whole image via `set_partial` | One `Mesh`, 65,536 quads |
|---|---|---|
| CPU per change | fill 65,536 `Color32` (256 KiB) | build 262,144 vertices + 393,216 indices |
| CPU per Render Frame, unchanged | one quad | copy of ~6.5 MiB: no tessellation cache |
| GPU upload per change | 256 KiB (`texSubImage2D` / `write_texture`) | none separately |
| GPU upload per Render Frame | 4 vertices | ~6.5 MiB `STREAM_DRAW` |
| At 30 Ticks a second | ~7.5 MiB/s, only when the Source changed | ≥195 MiB/s at 30 fps, more at 60 |

Why:

- **The mesh is re-sent every frame.** `Mesh::add_colored_rect` is 4 vertices of 20 bytes and 6
  `u32` indices per quad (`epaint-0.36.2/src/mesh.rs:12-24,199-237`). egui deliberately keeps no
  tessellation across frames (`egui-0.36.2/src/context.rs:2866-2868`), and even an `Arc<Mesh>` is
  appended into the output each frame (`epaint-0.36.2/src/tessellator.rs:1436-1451`) and uploaded
  per paint (`egui_glow-0.36.2/src/painter.rs:471-497`).
- **`rect_filled` per Cell is worse.** Feathering doubles the vertices
  (`tessellator.rs:767-810`), rects ≤2 physical pixels take the line-segment path
  (`tessellator.rs:1810-1824`), and each `Painter::add` takes the graphics lock — the reason the
  console already uses one `Painter::extend` (`.agents/skills/egui/references/guide.md:105-111`).
- **A `PaintCallback` is backend-specific.** glow accepts only `egui_glow::CallbackFn`
  (`egui_glow-0.36.2/src/painter.rs:443-447`) and wgpu only `egui_wgpu::Callback`
  (`egui-wgpu-0.36.2/src/renderer.rs:952-955`), so it means two implementations and `unsafe` GL in
  shipped code to reinvent a texture upload.
- **`set_partial` over `set`.** Same bytes, but a full delta reallocates: glow calls
  `tex_image_2d` where a partial calls `tex_sub_image_2d` (`egui_glow-0.36.2/src/painter.rs:599-629`),
  and wgpu recreates the texture and bind group where a partial writes into it
  (`egui-wgpu-0.36.2/src/renderer.rs:663-737`). Partial deltas queued in one frame all upload, while
  full ones collapse to the last (`epaint-0.36.2/src/textures.rs:301-308`), so upload once per frame.
- **`NEAREST`, not the default.** `TextureOptions::default()` is `LINEAR`
  (`textures.rs:182-188`) and blurs Cell edges; `NEAREST` is `textures.rs:191-196`. `load_texture`'s
  own docs say to call it once per image, not in the UI code (`context.rs:2360-2361`).

Prior art agrees. VS Code draws into a CPU `ImageData` and re-rasterises only changed lines
(`microsoft/vscode` `src/vs/editor/browser/viewParts/minimap/minimap.ts:513-528,1963-2029`). Tiled
renders the map to a cached image and redraws only the rectangle on a pan
(`mapeditor/tiled` `src/tiled/minimap.cpp:80-81,158-194`). The one egui minimap found, egui-xyflow,
paints a shape per node (`src/render/minimap.rs:93-118`), which is fine for a node graph and the
wrong scale for 65,536 Cells.

## 2. Colour: reuse the Paint decision, block per Cell

At one texel per Cell a glyph is illegible; VS Code falls back to blocks for exactly this
(`editorOptions.ts:2805,3555`). So each texel is a category colour, and the categories already
exist:

- Per Cell, `RenderCell::source_paint()` gives `Unclaimed`, `Function`, `Bang`, `Comment` or
  `Operand{token, state}` (`orcvs/src/source/mod.rs:51-66`, `orcvs/src/render_frame.rs:107`), and
  `portal_roles()` gives Output, Input or Locked from the last committed Tick
  (`render_frame.rs:30-72,175,259-300`). Portal roles are the only "active this Tick" fact.
- `style::CellColours::colours(source_paint, portal_roles, placement)` already turns those into
  Theme colours, lazily filled per Theme, so a whole Grid is one indexed load per Cell
  (`console/src/style.rs:100-170`). A fixed reduction of its `CellVisuals` to one colour is
  precedence in code, which ADR 0053 allows (`docs/adr/0053-…:55`).
- `Paint::derive_with_theme(FramePaint::whole(frame), theme)` would also work
  (`console/src/paint.rs:251-258`; ADR 0040 foresaw a whole-Grid consumer at `docs/adr/0040-…:39`),
  but it builds seams, outlines and characters the Overview discards, at ~0.7–1.0 ms per call on CI
  (`benches/floors.toml:109-117`).

**Theme.** Cell colours need no new field. The Source View rectangle has none; borrowing
`cursor_border` or `region_border` would make one channel show two facts. Add one key (for example
`overview.view.border` and its width) through `style_catalogue!` (`console/src/theme.rs:127-200`),
`.scratch/theming/schema.md`, and both built-ins, within ADR 0053's width bounds
(`docs/adr/0053-…:33,47-49`). Nothing animates: "a Theme decides how presentation looks, never how
much it moves" (`CONTEXT.md:333`).

## 3. Cache and cost

- Today nothing is cached per Render Frame: `Console::ui` calls `render_frame()` every frame
  (`console/src/console.rs:352`), which walks all 65,536 Cells, and Paint covers the visible
  Positions only (`console/src/console/shapes.rs:642-660`).
- The one revision-keyed cache in `console` is `OpenSourceFile::unsaved`, keyed on `RevisionId`
  (`console/src/source_file.rs:55-121`). A `RevisionId` is minted on every write and every Tick
  commit (`orcvs/src/source/model.rs:522-532`), so during Playback the Overview rebuilds once per
  Tick: one 65,536-Cell pass plus a 256 KiB upload. That is the cost to benchmark.
- The one-frame `GlyphTable` rule (ADR 0040; `console/src/console/glyphs.rs:29-53`) forbids
  retaining galleys because the font atlas is rebuilt each pass. An owned `TextureHandle` is not
  atlas-relative, so retaining it is consistent, and the egui skill asks not to rebuild a reusable
  rendering resource every frame (`.agents/skills/egui/SKILL.md:51-52`).
- Do not cache on the Language Map or in `orcvs`: ADR 0050 was rejected for moving whole-Grid
  Paint into revision rebuilds (`docs/adr/0050-…:3`, `docs/adr/0052-…:15`). A console buffer
  rebuilt lazily on draw adds nothing to a revision.
- Add an `overview` case with a floor to `console/benches/paint.rs` and `benches/floors.toml`, as
  ADR 0079 did (`docs/adr/0079-…:21`); the comparison is deferred to CI. Reuse the buffer so a
  rebuild allocates nothing.

## 4. Geometry: padded Source, snapped Cell, Zoom

- The Overview's domain is the padded Source, the Grid plus two Cells of margin on each side, since
  that is everything a Pan reaches (`console/src/console/source_view.rs:30,416-417`; ADR 0047).
  Measure with the snapped Cell (`source_view.rs:332`, `console/src/grid_viewport.rs:290-305`).
- The rectangle is the visible region in Source points, which
  `diagnostics_panel::visible_source_region` already answers
  (`console/src/console/diagnostics_panel.rs:55-57`), shifted by the margin and scaled. Do not draw
  it from `VisiblePositions`, which adds a Cell on each side (`grid_viewport.rs:45-48`). Clip it on
  an axis with nowhere to Pan, and hide it when neither axis can Pan (VS Code's `sliderNeeded`,
  `minimap.ts:313,1526`; `CONTEXT.md:341-342`).
- Crisp texels need a whole number of physical pixels per texel. Size the image `256·k /
  pixels_per_point` points for whole `k`, recomputed each frame, and snap it
  (`emath-0.36.2/src/gui_rounding.rs:36-43`; upstream does the same in
  `egui_demo_lib/src/rendering_test.rs:497-529`). `Painter::image` is neither snapped nor feathered;
  the `Image` widget is snapped but feathers its edge by about a pixel
  (`tessellator.rs:694-700,1966-1977`), so prefer `Painter::image` with a rounded rect.
- Zoom is egui's whole-UI zoom (ADR 0058), folded into `pixels_per_point`
  (`context.rs:2283-2290`). Zooming in shrinks the console in points while the Source's points stay
  fixed, so the rectangle covers less of the Overview.

**Aspect is the one real trade-off.** Cells are 12×18 by default and configurable (ADR 0079), so
the padded Source is 2:3. A square 256×256-texel image at whole-pixel scale is crisp but draws Cells
square, so the rectangle's shape differs from the screen's while its position stays exact. Drawing
it 2:3 keeps the shape but makes rows alternate 1 and 2 pixels at `k = 1`, or doubles the size at
`k = 2`. I would start square and crisp, and revisit only if it reads wrong in use.

## 5. Interaction

- **Sense `Sense::DRAG` alone.** `click_and_drag()` is `CLICK | FOCUSABLE | DRAG`
  (`egui-0.36.2/src/sense.rs:81-83`): `FOCUSABLE` puts it in the Tab order, which the console forbids
  for anything in the Source area (ADR 0048; `source_view.rs:322-327`), and sensing click and drag
  together waits to tell them apart (`sense.rs:72-79`). A drag-only widget is dragged at once
  (`egui-0.36.2/src/interaction.rs:196-220`), and `is_pointer_button_down_on()` is true from the
  press frame (`response.rs:588-597`), so a press acts with no delay. Map the pointer with
  `RectTransform`, as `egui_demo_lib/src/demo/painting.rs:34-55` does.
- **Press outside centres, press inside keeps the grab offset.** This is the convergent behaviour of
  Tiled (`minimap.cpp:224-245`), Zed (`crates/editor/src/element.rs:6788-6806,6853-6876`) and VS
  Code's fit modes (`minimap.ts:1250-1257,1344-1345`). Re-centring on every frame, as egui-xyflow
  does (`src/render/canvas.rs:938-958`), jumps on the first move.
- **One Pan entry point.** Today `pan` changes only inline in `show_source_scene`, by drag
  (`source_view.rs:363-368`), wheel (`:370-376`) and follow (`:405-414`), all clamped once at
  `:418`. Add a pending-Pan request on `SourceView` that `show_source_scene` applies before that
  clamp, so the bound and the snapped-Cell measurement stay in one place. `ScrollArea::scroll_to_rect`
  does not apply; the Source is not in a `ScrollArea` (ADR 0038).
- **Pan, never select.** The Cursor follow fires only on a Cursor move or a resize
  (`source_view.rs:347-352,384-387`), pinned by
  `a_pan_with_no_cursor_move_and_no_zoom_is_not_pulled_back_to_the_cursor` (`tests.rs:5804`). If an
  Overview click selected a Cell, the next frame would follow the Cursor and fight the Pan.
- **Frame order.** Docked Diagnostics reserves its space before the Source and fills it after
  (`console.rs:362-376`, `panel.rs:262-283`). The Overview can do the same: reserve the strip
  before, then read this frame's geometry and post its Pan request after, which lands next frame —
  one frame of lag during a drag. Removing it means applying the request before
  `show_source_panel` from last frame's geometry; `request_discard` reruns the whole pass and is
  meant for sizing glitches (`context.rs:1969-1995`). Accept the frame unless it shows in use.
- Mirror the grab and grabbing cursor icons the Pan already sets (`source_view.rs:425-430`).

## 6. Placement, vocabulary, tests

- **Docked, not floating.** "Diagnostics occupy their own space rather than covering the Source
  View" (`.scratch/docked-diagnostics/issues/01-…:3`) is the standing rule; an `Area` at
  `Order::Foreground` (`egui-0.36.2/src/containers/area.rs`) would cover Cells and compete with the
  Grid's hit-testing. A right `egui::Panel` (0.36's name for `SidePanel`,
  `containers/panel.rs:206-276`) shown before the Source takes width instead. Toggle it with a
  checkbox beside Diagnostics in the View menu (`console/src/console/menu_bar.rs:199-208`), not
  persisted, as `diagnostics_open` is (`console.rs:78,217`). Off by default, as Zed and egui-xyflow
  ship theirs.
- **Term.** Nothing in `CONTEXT.md`, `docs/` or `.scratch/` uses "overview" or "minimap".
  Proposed entry — **Overview**: a reduced picture of the whole Grid and its margin, one mark per
  Cell in the Theme's colours, beside the Source View; it shows where the Source View is, and a press
  or drag on it Pans the Source View there; it carries no content, holds no Positions, and moves no
  Cursor. _Avoid_: minimap, map, thumbnail, navigator, viewport, scrollbar. The rectangle needs no
  second term: "the Source View, as the Overview shows it".
- **Accessibility.** Every action it offers is a Pan reachable by keyboard and wheel, so label the
  response ("Source overview") rather than exposing Cells to AccessKit, as VS Code marks its minimap
  presentational (`minimap.ts:1196-1197`).
- **Tests.**
  - The mapping (Overview point ↔ padded point ↔ `pan`, margin edges, snapped Cell at 1×, 1.5×,
    2×, an axis with nowhere to Pan) as pure arithmetic, like `grid_viewport::tests` and
    `clamp_pan` (`tests.rs:5336`).
  - The colour buffer as a value from a Render Frame and a Theme with no `Context`
    (`docs/adr/0040-…:35`); kittest cannot read texture pixels.
  - `console::tests` Shape assertions for the image rect and the rectangle stroke after a Pan.
  - A kittest that finds the View toggle by label and shows a press or drag Pans and leaves the
    Cursor where it was, like `alt_held_with_a_primary_drag_pans_and_reaches_the_source_as_nothing`
    (`kittest_tests.rs:2287`), with coordinates from live geometry.

## Risks

- **Performance:** one whole-Grid pass and a 256 KiB upload per revision, which is every Tick during
  Playback. Unmeasured; it needs the benchmark and floor before any claim.
- **Public API, unsafe, dependencies, features:** none if it stays in `console`. A
  `RenderFrame::revision()` would grow `orcvs` API for one consumer, which ADR 0040 warns against
  (`docs/adr/0040-…:9`).
- **Concurrency:** the `RevisionId` read and `render_frame()` are not atomic; reading the id first
  bounds the error to one extra rebuild.

## Not checked

egui_node_graph (clone failed), Aseprite and LDtk navigators, game-engine tile minimaps, and
Sublime Text beyond its community docs (sublimetext.com was unreachable).
