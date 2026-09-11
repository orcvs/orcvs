# Draw the Source Grid instead of building it

**Goal:** Replace the field of one `egui::Button` per Cell with a painted Source Grid the console
owns the transform for, at strict visual parity.

## Why

`show_source` (`console/src/console.rs:287-341`) builds one `egui::Button` per Cell inside a
`ui.horizontal` per row. At the default 40x25 that is a thousand interactive widgets per Render
Frame, and the code already spends three lines zeroing a layout engine it does not want —
`item_spacing`, `button_padding` and `interact_size` at `:294-296`.

The interaction those widgets buy is one question — which Position is under the pointer — that
`grid_viewport.rs` already answers arithmetically. A thousand widget ids are hashed and a thousand
interaction rects hit-tested each frame to compute a division.

The cost that actually dominates is not the widgets. `egui::Scene` sets a layer transform
(`egui-0.36.1/src/containers/scene.rs:214`) and `GraphicLayers::drain` applies it to every shape in
that layer at end of frame (`layers.rs:230-240`). `TextShape::transform` reaches that transform
through `Arc::make_mut(galley)` (`epaint-0.36.1/src/shapes/text_shape.rs:107-169`), and epaint's
`GalleyCache` holds an `Arc` to every galley it hands out, so the refcount is always at least two
and `make_mut` always deep-clones — the `Galley` and each row's `Arc<Row>`, including that row's
tessellated vertex buffer. A thousand Cells is a thousand galley deep clones and a thousand vertex
buffer copies per Render Frame, on the main thread.

That cost does not go away by removing the widgets. A painter holding its own cached
`Arc<Galley>` per glyph has the same refcount and takes the same clone. Only leaving the
transformed layer removes it, which is why this effort ends with the console owning its own
transform rather than with a drawing change alone.

Two further things follow from the same layer transform. Glyph quads are scaled while sampling a
fixed-resolution atlas, so text is bilinearly resampled rather than laid out at the zoom it is
shown at — soft above zoom 1.0, chunky below it. And `Glyph::pos` is explicitly not transformed
(there is a `TODO(emilk)` at `text_shape.rs:107-169`), so glyph positions inside a transformed
galley are stale and cannot be trusted for geometry.

## Rules

**Strict visual parity.** `restyle-egui-console` spent five issues settling this palette and
`console-testing/03` exists to pin it to recorded hex. Nothing here changes how the console looks.
Dropping per-Cell borders in favour of Markers is a styling decision and belongs to a styling
effort.

**No performance claim without a number.** `CLAUDE.md` requires a benchmark or profile for any
claim about cost, and `.scratch/benchmarks/spec.md` puts the comparison in CI. This effort claims
structure, not speed: a thousand widgets become one, a thousand per-frame `String` allocations
become zero, a thousand `Context` write locks become none, and cost scales with the viewport
rather than with the Source. Each of those is countable from the diff. No issue here asserts a
frame time, and no new bench harness is added to `console`.

**Parity is asserted against the Render Frame, not against pixels.** `console-testing/spec.md`
already rules image comparison out of scope and already says assertions are made against `Console`
and the Render Frame afterwards. This effort inherits both.

## Prior art

Four egui terminal renderers solve the same problem, and one is on this repository's exact pin.

`peters/horizon` uses `egui` 0.36.1 and draws its grid in `crates/horizon-ui/src/terminal_widget/render.rs`.
No per-cell widgets: one `allocate_exact_size` plus two `ui.interact` rectangles. It coalesces
consecutive same-foreground cells into one text shape and consecutive same-background cells into one
widened rectangle, and caches the whole frame's shape list behind a key over the rectangle, cell
metrics and scroll offset, replaying it when nothing relevant changed.

`horizon` is not a precedent for painting in screen coordinates, and an earlier draft of this spec
wrongly said it was. Its panels are `egui::Area`s that call `ctx.set_transform_layer` as their first
act (`crates/horizon-ui/src/app/panels.rs:604-614`, `app/view.rs:189-193`), and its shape cache
clones each `Shape` into the painter, so the galley refcount is at least two and `Arc::make_mut`
deep-clones unconditionally. It pays exactly the cost this effort exists to remove, and has not
noticed: its profiling spans cover shape building and `tessellate_shapes`, while the clone happens
in `GraphicLayers::drain` inside `end_pass`. That is a stronger argument for issue 03 than a
precedent would have been. No surveyed egui terminal renderer leaves the transformed layer; the ones
that avoid the clone avoid it by never zooming through a transform at all.

`landaire/hxy` (`hxy-view`) lays out one galley per character of a fixed alphabet, caches the table
keyed by font and `pixels_per_point`, and paints each in any colour. It reports the win directly:
around 3,200 `painter.text` calls a frame went from 1,081,977 to 80,031 allocation blocks over 200
frames. It zooms with `ctx.set_zoom_factor` rather than a transform, so its text is always laid out
at native resolution. It also carries the retained-table atlas bug issue 02 avoids by rebuilding
per frame.

Neither hand-builds an `epaint::Mesh`, manages a texture atlas, or uses a paint callback for glyphs.
Every implementation goes through `Shape::text` or `Shape::galley`. Issue 04 reaches for no mesh at
all: it coalesces runs of same-coloured Cell backgrounds through the ordinary painter, because
coalescing is what the working implementations actually do. Grid lines are not in it — per-Cell
stroked borders stay, and issue 04 records why a spanning line cannot reproduce today's one under
this effort's strict-parity rule.

These are references to read, not dependencies to take. `hxy-view` has a hundred-odd downloads and
`horizon` is not published at all.

## Deliberately not in scope

**No frame-level shape cache and no dirty tracking.** `horizon` caches a whole frame's `Vec<Shape>`
behind a geometry key and reports 39.6us to 4.4us on its grid render — and its history carries four
shipped bugs from it: theme absent from the key, mid-frame palette mutation, a stale selection
highlight replayed after copy, and hover state absent from the key. Three of those four are state
this console has. The Cursor blink is the sharpest: `console.rs:554` already schedules repaints whose
only difference is blink phase, so a geometry-keyed cache would replay the wrong phase. This effort
rebuilds every Render Frame on purpose. If a frame cache is ever wanted, its key must include Cursor
phase, selection and palette, and it belongs to its own effort with its own tests.

## What this unblocks

`ADR 0005` defers Source addressing for an infinite canvas. A renderer whose cost scales with the
total Cell count cannot serve one; a renderer that iterates the visible Position range can. Culling
is issue 05 here and it is the reason this effort matters beyond tidiness. It does not reopen
ADR 0005 and no issue here assumes an infinite Source — the Grid stays fixed.

## What this deletes

`console-testing/spec.md` carries a rule written to work around the widget grid: *"The Source Grid
is a field of `egui::Button`s whose labels are single characters and mostly blank, so AccessKit
queries cannot address them unambiguously."* After this effort there are no Cell widgets to
address, so the rule describes nothing and the workaround it justifies is no longer needed.

`cell_line_width` (`console.rs:398-400`) takes `_selected` and `_cursor_visible`, uses neither, and
has exactly one caller and one test. `CLAUDE.md` forbids a shipped function taking a parameter only
a test populates. The function goes with the buttons.

`glyph_button_fits_the_fixed_cell` (`console.rs:960`) asserts that `add_sized` is not inflated by
`button_padding`. It tests a widget that will not exist. `caret_phase_does_not_change_cell_border_geometry`
(`console.rs:1059`) asserts that the caret phase does not move Cell geometry; under a painter that is
structural, and the assertion moves to the geometry that replaces it rather than being dropped.

## Vocabulary

No new domain terms. `CONTEXT.md` already has **Render Frame**, **Glyph**, **Marker**, **Cursor**,
**Grid**, **Position** and **Cell**, and this effort changes only how a Render Frame reaches the
screen. "Painter", "galley", "mesh" and "transform" are toolkit words and stay out of the glossary.

## Sequencing

`console-testing/01` renames `shell` to `console` and moves every path this effort touches. It is
`ready-for-agent` and blocked by nothing. It lands first, so this effort's diffs are diffs rather
than a rename tangled with a rewrite. Every issue here cites paths under `console/`.

## Settled: how one interaction rectangle coexists with the Scene

Within one layer, a later-registered child wins the click tie, and wins the drag too if it senses
drag (`hit_test.rs:203-429`; registration order at `ui.rs:297-311, 969-990`). The Scene's pan
response is registered first, before any content. So issue 02's rectangle must sense **click only**,
leaving the drag — and the pan — to the Scene, and must be sized to the **Grid** rather than the
console area, because the letterbox is the only place the Scene's own `double_clicked()` still
fires. Both constraints hold today, discharged by the Cell widgets; issue 02 inherits them
explicitly.

## What issue 03 does not cost

`Scene::register_pan_and_zoom` (`egui-0.36.1/src/containers/scene.rs:229`) is public and takes a
`&mut TSTransform` the caller owns. It applies the drag-pan for the configured buttons, the
zoom-at-pointer transform, the `zoom_range` clamping and `Response::mark_changed`, and it does not
touch the layer transform — that is `Scene::show`'s separate `set_transform_layer` call. So retiring
the Scene container does not mean reimplementing pan and zoom input. It means keeping that helper and
giving up only the layer transform, which is the part that clones the galleys and resamples the
glyphs. What genuinely becomes the console's is applying the transform when it computes Cell
rectangles, and the fit-to-viewport reset, which is already console code at `console.rs:389-393`.
