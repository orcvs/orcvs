# Let the Source Grid paint answer instead of take

**Status:** open — `01`–`08` are `resolved`; `09` remains `ready-for-agent`. The cull's cost claim is asserted by counting, not by a `console` benchmark. `02`-`06` shipped the two layers; `01` recorded them as [ADR 0040](../../docs/adr/0040-the-console-paints-from-a-value.md).

## Goal

`show_source` paints the Source Grid and answers nothing. It takes `&mut Orcvs` for one `select()` call, so every test of what a Cell looks like builds a running Orcvs, an egui `Context`, a `RawInput` and a `CentralPanel`, then reads the result by filtering a flat `Vec<Shape>`. Six tests do that filtering across 475 lines, up from five and roughly 330 the last time this path changed.

Extract the paint decision as a value. `Paint::derive(&RenderFrame)` answers, per Cell, what the console decided to draw; a separate step turns that into Shapes; `show_source` shrinks to the three calls plus the click it reports. Separately, `RenderFrame` answers the Grid it was derived from, retiring `source_dimensions` and the `expect` and `debug_assert` that recover the Grid's own invariants from `rows[0].len()` twice per Render Frame.

Nothing about what appears on screen changes. Same colours, same geometry, same order.

## Why the test surface is missing — do not re-derive this

It was not a testing decision. ADR 0038 records the chain, and each link forces the next:

1. The console zooms the Source Grid while the chrome stays put. That rules out `ctx.set_zoom_factor`, the cheapest egui-native answer, because it zooms the menu bar and diagnostics too.
2. Zooming only the Grid meant `egui::Scene`, and `Scene::show` sets a layer transform.
3. `GraphicLayers::drain` applies that transform to every shape at end of pass; for a `TextShape` that reaches the galley through `Arc::make_mut`, and epaint's `GalleyCache` keeps an `Arc::clone` of every galley it hands out, so the refcount is never one and the deep clone never elides. A thousand galley clones per Render Frame at the default Grid, on the main thread.
4. No drawing change removes that. Only leaving the transformed layer does — hence ADR 0038.
5. Painting into an untransformed layer leaves the thousand `egui::Button`s buying one thing, "which Position is under the pointer", which `grid_viewport.rs` answers with a division. They went, and with them every widget id, every `Response` and every accesskit node.

Step 5 is where the per-Cell `Response`, the widget ids and the accesskit nodes went. It did not cost a test surface: egui populates no colour on an accesskit node — `fill_accesskit_node_from_widget_info` sets role, label, value and toggled, and the `background_color` and `foreground_color` fields accesskit 0.24.1 defines are never written — so per-Cell widgets would not answer "what colour is this Cell" either. ADR 0038 says the divergence was deliberate and that no surveyed egui terminal renderer leaves the transformed layer. This effort does not reopen any of it.

## What is wrong with the tests, and what is not

The fault is locality, not technique. The six tests are long, and they recover a fact the console already knew by guessing at it afterwards. `background_runs` decides a Shape is a background from `fill.a() > 0 && stroke.width == 0.0`. `background_at` then hit-tests a point against those rectangles to name a colour. The harness subtracts the panel's own fill with a hand-written `retain()` 266 lines away from the code that makes it correct. Each of those steps re-derives something `cell_visuals` stated plainly one function earlier.

**Reading the shape list is not itself the fault.** An earlier draft of this spec said it was — that the flat `Vec<Shape>` was archaeology forced on the console by ADR 0038's removal of the per-Cell widgets. That is wrong, and a review of `egui_kittest` 0.36.1 settles it. Its own shipped regression tests walk `harness.output().shapes` and filter by `Shape::Rect` and `Shape::Text` three times — `tests/regression_tests.rs:405`, `:630-656`, `:795-805` — in code where widgets exist, with neither the `wgpu` nor the `snapshot` feature enabled. `window_fixed_size_is_outer_size` (`:571-657`) does both: the image snapshot sits behind `#[cfg(all(feature = "wgpu", feature = "snapshot"))]` at `:627`, and the shape-list assertion below it runs unconditionally. Upstream treats the shape-list assertion as the test that always runs and the image as the optional extra.

So the shape list stays a legitimate place to assert from, and ticket `06` keeps one test there on purpose rather than as a concession.

## Screenshot testing was refused — do not re-investigate it

It was never the alternative to this effort, and nobody proposed it. The refusal is recorded here only so the question is not opened later.

`console-testing/spec.md` rules out `egui_kittest`'s `wgpu` and `snapshot` features. That refusal holds:

- `snapshot` and `wgpu` are separate features and neither implies the other, but `snapshot` alone ships no renderer. The default renderer exists only under `#[cfg(feature = "wgpu")]`; without it the harness answers `"No default renderer available. Enable the wgpu feature or set one via HarnessBuilder::renderer"` (`egui_kittest-0.36.1/src/renderer.rs:78-81`). A snapshot-only build therefore compiles and fails when the test runs. `egui_kittest`'s README — not egui's — says to enable both.
- It turns on `eframe/wgpu` in the test build, so eframe would compile with glow and wgpu, undoing a deliberate pin. The feature is the weak form `eframe?/wgpu`, so it fires only because `console-testing` plans `egui_kittest`'s `eframe` feature; without that it would be inert.
- `deny.toml` sets `[graph] all-features = true` with no `exclude-dev` across five targets, so `wgpu` would be audited for `wasm32-unknown-unknown` and `x86_64-pc-windows-msvc`, where no test runs.
- The tests would be platform-sensitive by construction. egui's `kittest.toml` sets a pixel threshold of 0.6 on macOS — "our source of truth", because its CI runs snapshot tests there — and 2.0 elsewhere, with the Bézier Curve demo needing 2.1 on Linux. `egui_kittest`'s own README names MSAA sample placement, texture filtering, WGSL float evaluation and partial derivatives, each implementation-defined per GPU, OS, backend and driver version.
- Linux would need a software Vulkan driver; there is no upstream CI example or documentation for that. Upstream does consider software rasterisation, but on Windows: `egui_kittest`'s manifest enables DX12 "because it always comes with a software rasterizer".

**Not audited, and load-bearing if reused.** Two cost figures in an earlier draft could not be reproduced from source or upstream config: `snapshot` alone at +28 crates and `snapshot` + `wgpu` at +49 against a 112-crate baseline, and egui's own snapshot job at 11m38s warm. Measure them before citing them. What is checkable stands: this workspace's macOS job is at 17m14s against a 20-minute timeout, with `verification-gaps/14` open about it.

Two things that are *not* obstacles, recorded so they are not raised again: the licence allow-list covers wgpu and image, and reference-image size is irrelevant here because the paint tests use 200x200 screens.

## Decisions

Settled in a grilling session. Implement as written. If one is impossible or wrong — not merely awkward — stop and report rather than improvising.

**Three of them were revised by `07`, which re-applies the viewport cull `source-grid-rendering/05` landed as PR #69 and this spec was written without.** A Paint covers the Positions the console draws rather than every Position the Source holds. The revisions are written into the decisions below; `08` carried them into the glossary and ADR 0040. The reason `Paint::derive` takes no `Orcvs` and no `egui::Context` is untouched — a Position range is neither.

### The value layer

- `Paint` carries no geometry. No `Rect` appears in it. Cell geometry is `GridViewport::cell_rect`'s job and is tested there; a `Rect` in the value layer makes every assertion re-acquire a viewport.
- Stored flat as `Vec<CellPaint>` plus the `Grid` and the drawn ranges. **Revised by `07`:** `at(Position)` indexes by offset within that sub-rectangle — row-major from the drawn range's corner — not through `Grid::index`, which addresses a Cell of the whole Grid where this `Vec` holds only the Positions the console draws. This is deliberately unlike `RenderFrame`'s `Vec<Vec<RenderCell>>`: `Paint`'s primary access is `at(Position)`, and the row nesting exists on `RenderFrame` only to serve painting. Say so in the module doc.
- Background runs are derived, never stored. `Paint::background_runs()` is the coalescing fold — today an inline `Option<(Color32, Rect)>` state machine flushed at two places — given a name, a home, and column ranges instead of rectangles. One stored truth, and the fold is testable with no egui at all.
- `CellPaint` is flat: `background: Option<Color32>`, `border`, `foreground`, two seam colours, `character`. It does not hold a `CellVisuals` alongside the filtered background; holding both would make the skip invariant a property of the struct rather than of the derivation, which is where its subtlety lives. `cell_visuals()` is unchanged and called once inside the derive.
- `Paint::cursor() -> Option<Position>`, not a `bool` on every Cell. `RenderFrame::derive` takes one `selected: Position` and calls `grid.assert_owns(selected)`, so exactly one exists; a per-Cell bool re-opens a state the layer below closed. **Revised by `07`:** the answer is optional because a Paint covers a viewport the Cursor can be outside, never because the Render Frame is vague. Growing `RenderFrame` an accessor for the selected Position to keep the answer total was considered and refused — the shape step reads the Cursor only to place one drawn Cell's border, and a Cursor that is not drawn belongs in no group.
- Seam suppression on the Cursor's Cell is applied by the derive, so the Cursor's `CellPaint` simply has no seams and the shape step never learns the rule.
- `Paint::derive(&RenderFrame, &VisiblePositions)` takes nothing else. No `Orcvs`, no egui `Context`. This is the decision the whole effort turns on: if the derive needs a `Context`, the twenty-two Context-building tests that accept a blank Source to dodge harness cost keep dodging. **Revised by `07`:** the Position range is the second argument, and it is not a `Context` — it is two `Range<usize>` the viewport has already resolved the clip `Rect` into, so a test names one without building anything. The bar this decision sets is unmoved.
- It lives in a new `console/src/paint.rs`. Not in `orcvs`: a background colour is a presentation decision and `orcvs` has no business knowing `Color32` or `PALETTE`.

### The shape step

- Paint order stays here. The order that matters is across kinds, not across Cells, and it exists because of how a painter composites — a fact about egui, not about the Source.
- `shapes(...) -> SourceShapes { backgrounds, borders, glyphs, seams, cursor }`, with `into_shapes()` doing the concatenation. The five locals already exist; promoting them to fields costs nothing and gives the order a name instead of a comment.
- The five fields are built eagerly; `into_shapes()` answers `impl Iterator<Item = Shape>` over the chained owned `Vec`s. Those are two separate decisions and only the first is about laziness. Shape *construction* must not be lazy, because `Painter::extend` runs the iterator inside `ctx.graphics_mut` — a full `Context` write lock — so a lazily built shape is a shape built while holding it. The concatenation is free either way, and materialising it into a sixth `Vec` adds a ~2000-element allocation and a whole re-move per Render Frame that today's `chain` of five `Vec`s does not pay. The ordering test is unaffected: all five fields are complete before the first Shape leaves.

### The seam out

- `show_source` answers `Option<Position>` — the click, and nothing else. It does **not** hand back the `Paint`. Nothing in production would read it, and `Paint::derive` is already independently reachable, so returning it would be a test-only seam cut into a shipped return type — the thing the repository contract forbids and the thing the review used to reject publishing `RenderFrame::derive`.
- `show_source_scene` answers `PresentedSource { viewport: GridViewport, clicked: Option<Position> }`. "Presented" is already the repository's word for this step; `grid_viewport::presented_grid` uses it.
- `Console::ui` calls `orcvs.select(...)`. `show_source` no longer takes `&mut Orcvs`.
- There is no `PaintedSource` and no `SourcePaint`. An earlier draft had one; it existed only to carry the `Paint` out, and with that gone it has no fields worth a type.

### The Render Frame's shape

- `RenderFrame` stores and answers its `Grid`. `source_dimensions`, its `expect` and its `debug_assert` are deleted.
- `Grid::cols()` becomes public and is renamed `columns()`; a `rows()` count is added; the position iterator becomes `positions_by_row()`. The glossary defines a Grid as "its column and row counts", so the counts take the glossary's plainest words and the iterator takes the longer name. Nine call sites, all inside `orcvs`; `source/model.rs:659` already writes `self.grid.rows().count()` for a count the accessor now gives it.
- `RenderFrame` is **not** flattened, and that is not filed as a follow-up. `Vec<Vec<RenderCell>>` is right for a type whose only consumer iterates it in row order, and flattening would rewrite two `Orcvs` doctests and dozens of assertions to buy an `at(Position)` nobody has asked for.

### Vocabulary

- `CONTEXT.md` gains **Paint**: the per-Cell decision of how one Render Frame is drawn — background, border, foreground, seams and the character shown — derived from a Render Frame and carrying no geometry. _Avoid_: shapes, draw list, painter.
- Past tense was rejected for the name. `PaintedCells` claims a painting that has not happened; this is the description a later step turns into shapes.
- **Console** is not written here. `console-testing/02` already owns that entry, is `ready-for-agent`, and specifies it down to its `_Avoid_` list. Nothing in this effort blocks on it: an earlier draft blocked ticket `01` on that entry, but the **Paint** entry landed under ticket `05` in `20d2a8a` and names no Console, and ticket `01` writes an ADR rather than the glossary. See `issues/01` for the full reasoning.
- Types: `Paint`, `CellPaint`, `SourceShapes`, `PresentedSource`.

### Record

- One ADR, number **0040** — the next number above the highest file in `docs/adr/`, which is the rule `docs/adr/README.md` states.
- **Do not take 0027.** That README names 0027 by number as the gap that must not be reused: a branch that has not landed claims it, and reusing it would land the collision the directory has already had once. Everything true of that branch — `revalidate-play-commands-at-the-output-adapter` is 307 commits behind `main`, carries four commits of its own, and had its decision reversed by shipped code, `orcvs::source::model` now exporting `MidiChannel`, `Velocity`, `Controller`, `ControlValue`, `BendLsb` and `BendMsb` — leaves the number no freer. The rule is about the file the branch holds, not about whether its decision survived. Nothing local catches a reuse either: `scripts/check-tooling-contract.sh` fails only on two files in `docs/adr/` sharing a leading number.
- Out of scope: the Evaluator/Interpreter naming drift and the Marker/Highlight glossary drift. The duplicate 0036 is already resolved — `728183c` renumbered the pulse decision to 0039 and `adr-numbering/01` is settled — so this effort touches ADR numbering no further than taking the next number.

### Testing

- Acceptance is named plus behavioural, never a count. A count invites satisfying the number by merging tests, which is how 475 lines became six tests instead of five.
- The behavioural bar: no test asserting what colour a Cell is builds a live `Orcvs` or an egui `Context`.
- One test still reads a flat `Vec<Shape>`: the one asserting `into_shapes()`'s concatenation order. It runs through `console_pass` so it also covers the three-line wiring that nothing else proves. It is not an exception granted reluctantly — paint order is a fact about how a painter composites, the shape list is where that fact is observable, and `egui_kittest`'s own regression tests assert the same way.
- `the_cell_needing_no_background_is_exactly_the_one_filled_with_the_source` moves to the value layer and calls `cell_visuals()` directly for its comparison. It is the one assertion tying the skip decision to the function it must not drift from, which is why the shipped comment says the condition "reads wrong and is right".
- The harness's hand-written `retain()` subtracting the panel's own fill is deleted. The 266-line distance between `visuals.background != PALETTE.source` and the `source_panel_frame()` that makes it correct is a separate locality finding and stays out.

## Scope boundary

Four things were considered and refused, each for a stated reason:

- **Keep the capture, name the helpers** — leave `show_source` and the `run_ui` shape capture alone, and fix only what is wrong with the three filtering helpers: give them names that say what they match, and move the panel-fill `retain()` beside the `source_panel_frame()` that makes it correct. This is the honest competitor, because it treats the same locality fault this effort treats, and it is what upstream egui does. It is refused on reach, not on technique: every colour assertion still builds a live `Orcvs`, an egui `Context`, a `RawInput` and a `CentralPanel` to read one `Color32`, so the bar in `06` — no test asserting a Cell's colour constructs either — stays unmet.
- **Fold-only** — extract just the coalescing state machine and leave `show_source` intact. One ticket, and it leaves all six long tests in place.
- **Shape-half-only** — take tickets `02` and `03` and drop the paint inversion. Two small tickets, no new types, no divergence.
- **Making the components egui-native** — a `Widget` impl answers a `Response`, which carries hover, click, rect and id, not what colour a Cell is; restoring per-Cell widgets gives accesskit nodes carrying the *label*, which makes one test native and leaves every colour assertion where it is.

## Traps

- **Worktree discipline.** Run everything from `.worktrees/source-paint`. Never `cd` to the primary checkout or another worktree. Never use bare `git stash` — the stash stack is shared across checkouts and other sessions are live; prefer a temporary WIP commit.
- **`mise trust`.** The first `cargo` invocation in a fresh worktree fails until `mise trust <worktree>/mise.toml` runs.
- **`06` cannot be split from its test migration.** The moment `show_source`'s signature changes, every test that captures a shape list through it stops compiling, and each commit is expected to pass its gates.

## Verification

For each ticket, on the crates it edits and their dependents — `orcvs` means `orcvs` and `console`; `console` has no dependents:

```sh
cargo fmt --all -- --check
cargo clippy --package <crate> --all-targets --locked -- -D warnings
PROPTEST_CASES=32 cargo nextest run --package <crate> --locked
```

Ticket `02` also owes `cargo test --workspace --doc --locked`, because `Grid`'s public surface changes. Tickets touching `.scratch/` owe `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null`.

Deferred to CI, and named on the `Not run` line rather than left off: `mise run check_wasm` — nothing here is platform-conditional, there is no `cfg` and no new dependency, and the merge tier compiles the WASM target anyway. The persistence arm is not owed at all; no ticket touches the feature.

No benchmark is owed through `06`, and no claim about cost is made in either direction there. The
shape *sequence* handed to `painter.extend` is unchanged — the same Shapes, in the same order — but
the work that builds it is not. One pass over the Grid inside `show_source`'s loop becomes three:
`Paint::derive` walks the Render Frame, `Paint::background_runs()` folds over the derived
backgrounds, and `SourceShapes::new` walks the Grid again to place the geometry. Two heap
allocations per Render Frame are new with them — the `Vec<CellPaint>` the derive collects, around a
thousand entries on the default 40x25 Grid, and the `Vec<BackgroundRun>` the fold answers. Both are
what the layering buys.

There is no regression against `main` on the Shape vectors themselves: the review pass restored
`Vec::with_capacity` for `glyphs` beside the one `borders` already had, after it caught a densely
written Source regrowing it by reallocation on every Render Frame.

**`07` cuts all three of those walks to the viewport**, which is what `source-grid-rendering/05`
already did to the single loop they replaced. The `Vec<CellPaint>` is sized to the drawn Positions
rather than to the Grid, so on a zoomed console it is a fraction of a thousand entries rather than
a thousand, and on a Grid that resize has grown it does not follow the Grid at all.

That claim is asserted by counting rather than by timing, which is what `05` did and all its surface
allowed: `paint.rs` counts the `CellPaint`s a culled derivation builds, with no `Context` at all,
and `console.rs` counts one stroked Cell rectangle per drawn Position and a shape total that falls
with the zoom. It is still **unmeasured** in the sense `CLAUDE.md` means — there is no `console`
benchmark, `[[bench]]` appears only in `orcvs/Cargo.toml` and `lang/Cargo.toml`, and no frame time
is asserted anywhere. `09` is where that benchmark is filed; running the comparison is CI's, per
`.scratch/benchmarks/spec.md`.
