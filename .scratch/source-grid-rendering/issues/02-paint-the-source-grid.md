# 02 — Paint the Source Grid instead of building a Cell button field

**What to build:** Replace the per-Cell `egui::Button` field with one allocated interaction
rectangle and painter drawing, at strict visual parity.

**Blocked by:** 01 — Record the transform ownership decision as an ADR.

**Status:** resolved

- [x] `show_source` allocates one response for the whole Grid and draws every Cell through
      `ui.painter()`. No Cell is a widget.
- [x] A click resolves to a Position by arithmetic through `GridViewport`, not by hit-testing Cells.
      `Orcvs::select` is called with the same Position the button field would have selected, and the
      existing click tests (`console.rs:751`, `:785`) pass unchanged.
- [x] Glyphs are painted from a table of `Arc<Galley>` — one per character of the Source's character
      set, laid out with `Color32::PLACEHOLDER` via `FontsView::layout_delayed_color` — built inside
      a single `ctx.fonts_mut` closure and **rebuilt every Render Frame**. The paint-time colour is
      supplied as `Painter::galley`'s `fallback_color`, which the tessellator substitutes only for
      placeholder vertices.
- [x] The table is per-Render-Frame scratch and is never retained across frames. This is a
      correctness requirement, not a style preference. `RowVisuals::mesh` holds **texel** UVs that
      are normalised at tessellation against the live atlas size
      (`epaint/src/text/text_layout_types.rs:852-855`, `tessellator.rs:2030-2033`), and
      `Fonts::begin_pass` replaces the whole `FontsImpl` — a new atlas at `[max_texture_side, 32]`
      with empty glyph caches — whenever `TextOptions` change or `atlas.fill_ratio() > 0.8`
      (`fonts.rs:734-748`). A galley held across that recreate indexes unrelated texels and paints a
      *different character*. Atlas growth alone is safe, because it only extends the image height and
      every texel keeps its coordinates; recreation is what corrupts, and the two are separate
      events. `font_image_size()` is not a usable signal for either — it can be identical on both
      sides of a recreate.
- [x] Rebuilding per frame is what makes the table correct by construction: epaint's own
      `GalleyCache` is the memo, and it is the only cache in the stack that `begin_pass` invalidates
      alongside the atlas. `egui::cache::FrameCache` and `ctx.data()` evict on last-frame use and know
      nothing about fonts, so either would carry the same corruption.
- [x] The rebuild costs one `Context` write lock for the whole table instead of one per Cell, and one
      `String` per character of the alphabet instead of one per Cell. Those are the wins this issue
      claims; retaining the table across frames is not needed for any of them.
- [x] Shapes are accumulated into a `Vec<Shape>` and submitted with one `Painter::extend`, never
      added one at a time. `Painter::add` goes through `paint_list` to `Context::graphics_mut`, which
      is `Context::write` (`painter.rs:197-199, 213-221`; `context.rs:1038-1040`) — a full Context
      write lock **per shape**. A per-Cell `add` loop would take more locks than the Button field does
      today, turning this issue's headline claim into a regression. `Painter::extend` takes the lock
      once and its own doc says calling it once is faster than calling `add` repeatedly.
- [x] Backgrounds and Glyphs are built as two separate shape sequences and concatenated so every
      background precedes every Glyph. A background belonging to a later Cell must never paint over
      an earlier Cell's Glyph. Both surveyed implementations that coalesce backgrounds hit this and
      left a permanent comment about it; one pins the ordering with a test.
- [x] Cursor and selection strokes are painted after all backgrounds and all Glyphs, so a
      neighbouring Cell's fill cannot paint over the Cursor.
- [x] The alphabet the table covers is stated, and so is the fallback for a character outside it. A
      single uncached `Painter::text` in the Cell loop reintroduces the per-Cell allocation and the
      per-shape lock for that Cell.
- [x] The table has exactly one owner and one construction site. Two callers laying out at different
      sizes would thrash it.
- [x] No per-Cell `String` is allocated and `Painter::text` is not called in the Cell loop — it takes
      `impl ToString` and calls `layout_no_wrap(text.to_string(), ..)`, which costs an allocation and
      a whole-`Context` write lock per call.
- [x] Glyphs are positioned at exact multiples of `CELL_SIZE` and centred in their Cell. A row is
      never laid out as one galley: egui 0.36 shapes through harfrust with `liga`/`calt` enabled and
      `extra_letter_spacing` is not applied within a shaping cluster, so a ligature in a coding font
      would consume two Cells and shift the rest of the row. MonaspaceNeon has ligatures and the
      Source's character set is full of the pairs that trigger them.
- [x] Visual parity holds for every case `cell_visuals` (`style.rs:67-104`) distinguishes: the eight
      Glyph foreground colours, the selection fill and its two stroke states, and all four
      `CursorBloom` fill and line pairs.
- [x] Sector seam lines keep their current geometry and their `sector_line` strength attenuation, and
      are still suppressed on a selected Cell.
- [x] `cell_line_width` is deleted along with `caret_phase_does_not_change_cell_border_geometry`,
      and the property that test held — that caret phase does not move Cell geometry — is asserted
      against the painted geometry instead.
- [x] `glyph_button_fits_the_fixed_cell` is deleted. It asserts that `button_padding` does not
      inflate `add_sized`, about a widget that no longer exists.
- [x] The three `ui.spacing_mut()` lines at `console.rs:294-296` are gone, along with the
      `ui.horizontal` per row.
- [x] The interaction rectangle senses **click only**, never drag. A same-layer child registered
      after the Scene's pan response wins the click tie, and would win the drag too if it sensed
      drag — which would kill the middle-drag pan. `Sense::click()` is `CLICK | FOCUSABLE`; use
      `Sense::CLICK` to keep one rectangle out of the tab order where a thousand Buttons were.
- [x] The interaction rectangle is sized to the **Grid**, not to the console area. The Scene's pan
      response covers the whole outer rect, and the letterbox is the only territory where its
      `double_clicked()` still fires — a Cell widget eats the click everywhere it covers, today and
      after this change. A rectangle covering the letterbox would break
      `a_double_click_unpins_the_view_and_hands_it_back_to_the_fit` (`console.rs:1039`), which
      double-clicks at exactly that point.
- [x] With those two constraints `console.rs:995`, `:1020` and `:1039` pass unchanged, and the click
      tests at `:752` and `:786` pass because the Grid rectangle is the click hit.

## Comments

Before adding painting tests, check whether `TexturesDelta` trips its drop assertion
(`epaint/src/textures.rs:335-343`). The existing tests bind `output` from `ctx.run_ui`
(`console.rs:644`, `:911`, `:977`) and call `drop_without_applying_deltas`; changing what the paint
path rasterises may change what is left unapplied.

The spec's open question is settled, and the answer became the two acceptance lines about sense and
size. The rule, from `hit_test.rs:203-429` and the registration order at `ui.rs:297-311, 969-990`:
within one layer, a later-registered child wins the click tie, and wins the drag too if it senses
drag. The Scene's pan response is index 0 because `Ui::new_child` registers it before any content,
so a click-only child takes the clicks and leaves the drag to the Scene. That is exactly what the
thousand Buttons do today, which is why this is parity rather than a new arrangement.

Land this inside `Scene`, unchanged. The galley deep clone this effort's `spec.md` describes is still
paid after this issue, because the cached `Arc` keeps the refcount above one and the layer transform
still reaches every `TextShape`. This issue is not where the cost goes away. It is where the Source
Grid stops being a widget field, which is what makes issue 03 a transform swap rather than a rewrite
of how a thousand widgets are placed by hand.

Do not batch or coalesce here. That is issue 04, and keeping it separate keeps this diff reviewable
against the palette.

Two implementations to read first. `landaire/hxy` (`hxy-view`) builds exactly the glyph table this
issue specifies — one galley per character of a fixed alphabet, cached by font and
`pixels_per_point`, painted in any colour — and its source records that the naive per-cell
`Painter::text` path dominated its allocation profile. `peters/horizon` draws a terminal grid on
`egui` 0.36.1 with no per-cell widgets at all. Neither is a dependency to take: `hxy-view` has
around a hundred downloads and `horizon` is unpublished.

The table cannot be built in `Console::new` regardless: `Context::fonts` panics before the first
`run()`. It has to be reached from inside a pass, which a per-frame rebuild does naturally.

If a retained table is ever revisited — it buys around ninety re-layouts a frame and nothing else —
it must key on `(FontId, pixels_per_point)` *and* validate each frame by re-requesting one probe
character and comparing `Arc::ptr_eq` against the cached entry, rebuilding on mismatch. Every
invalidation event either empties `GalleyCache` or changes the cache key, so the probe detects all
of them and its only failure mode is a spurious rebuild. Do not add that invariant without a reason
to.

egui offers two ways to colour a cached galley at paint time. `layout_delayed_color` lays out with
`Color32::PLACEHOLDER` and `Painter::galley`'s `fallback_color` substitutes for placeholder vertices
only; `Painter::galley_with_override_text_color` replaces glyph vertices unconditionally and leaves
background, underline and strikethrough alone. Both exist on 0.36.1 and both work for single-character
galleys carrying none of those extras. Prefer the placeholder path, which is what egui itself uses.

---

Resolved. The Grid is one `Ui::allocate_exact_size` rectangle sensing `Sense::CLICK`, and every
Cell is a `Shape` in a `Vec` submitted with a single `Painter::extend`: backgrounds, then Glyphs,
then sector seams, then the Cursor and selection strokes. `GlyphTable` lays the alphabet out inside
one `ctx.fonts_mut` closure each Render Frame, and the doc comment on it says why retaining it is
unsound rather than merely wasteful.

Four things the acceptance list did not name, all of them decided in the direction it implies.

The shapes are built with `Shape::galley` and `epaint::RectShape` rather than `Painter::galley` and
`Painter::rect`, because every `Painter` drawing helper ends in `Painter::add` — the per-shape
`Context` write lock this issue exists to stop taking. `Shape::galley` is exactly what
`Painter::galley` wraps, so the `fallback_color` contract is the one this issue asked for.

The Cursor and the selection stroke had to leave the Cell's own `RectShape` to be painted after
every Glyph, so a selected Cell's background carries `Stroke::NONE` and its border is a second
stroke-only `RectShape` in the later sequence. Under the Button field the two were one shape, which
is why a neighbouring Cell's seam could paint over the Cursor; it no longer can.

The out-of-alphabet fallback lays its galley out through `Painter::layout_no_wrap` rather than
`Painter::text`. `Painter::text` would paint immediately and so out of turn, breaking the ordering
the issue requires; laying out and pushing the Shape keeps the sequence intact and costs that one
Cell the same allocation and lock either way.

`caret_phase_does_not_change_cell_border_geometry` could not be re-asserted across both caret
phases. The Cursor's phase turns on a wall-clock delay held inside `orcvs` and nothing the console
can reach flips it, and a seam to set it would be the test-only input into shipped code `CLAUDE.md`
forbids. What replaces it —
`the_caret_reaches_the_paint_of_a_cell_and_never_its_geometry` — asserts the whole of what that
phase could have moved against the painted Render Frame: every Cell, the selected one included,
occupies exactly the rectangle `GridViewport::cell_rect` gives its Position, and the Cursor's own
stroke lands on that same rectangle. `cell_rect` takes a Position and nothing else, so the property
is now structural as well as asserted.
