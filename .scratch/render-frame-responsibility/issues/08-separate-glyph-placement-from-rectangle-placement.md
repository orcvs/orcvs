# 08 — Separate glyph placement from rectangle placement

**What to build:** split the one loop in `SourceShapes::new` so that the shapes needing no font atlas
are built without one, and the galleys are placed in a step that takes the `GlyphTable`.

## The weld

`SourceShapes::new` walks the Grid once (`console/src/console.rs:608-651`) and builds four kinds of
shape in that single pass: the Cell border, the Cursor's border, the two sector seams, and the
Glyph. The first three are arithmetic on a `Rect` and a `Color32`. The fourth is:

```rust
if cell.character != ' ' {
    let galley = table.glyph(cell.character);
    glyphs.push(Shape::galley(
        rect.center() - galley.size() / 2.0,
        galley,
        cell.foreground,
    ));
}
```

(`console/src/console.rs:643-651`.) That needs a `GlyphTable`, which is laid out from an
`egui::Context` (`console/src/console.rs:748-754`), which needs a font atlas. Because it shares the
loop with the other three, **every** shape in the value has to be built with a live `Context` in
hand.

## What that costs the tests

Seven tests go through the `source_shapes` helper (`console/src/console.rs:1824-1846`), which builds
an `egui::Context`, runs a UI pass, lays out a `GlyphTable`, and throws the pass away
(`output.drop_without_applying_deltas()`). Five of them assert nothing that involves a galley:

| Test | Line | What it asserts |
| --- | --- | --- |
| `the_cursor_reaches_the_paint_of_a_cell_and_never_its_geometry` | `:1884` | border rects, the Cursor's rect, background heights |
| `a_cell_border_is_one_grid_line_wide_whatever_the_cell_is_doing` | `:2202` | stroke widths |
| `a_background_run_covers_exactly_the_cells_it_replaces` | `:2447` | run rects against the Cells they replace |
| `a_background_run_is_the_rectangle_its_columns_span` | `:2554` | run tiling |
| `a_sector_seam_is_drawn_on_the_cell_edge_the_paint_asks_for` | `:2743` | seam endpoints |

Each of those is pure geometry over a `GridViewport`, and each builds a window's worth of machinery
to get at it. The helper's own doc comment states the reason without noticing it is a finding:

> An `egui::Context` is built here for one reason: a galley needs a font atlas, and a Glyph is a
> galley.

(`console/src/console.rs:1816-1818`.) One reason — which is served by one of the four things the
function builds.

## The claim this ticket does **not** make

The reviews put the payoff as "the font atlas is then needed only by the one test that asserts a
galley". **That does not survive checking, and the ticket says so rather than inheriting it.**

`a_glyph_is_painted_for_every_cell_that_shows_one_and_no_other` (`console/src/console.rs:2262`) is
indeed the only test that reads a galley's metrics — it asserts
`text.pos == rect.center() - text.galley.size() / 2.0` at `:2306-2310`. But
`every_background_is_painted_before_every_glyph_and_the_cursor_after_both`
(`console/src/console.rs:1950`) also needs the glyph group populated: it asserts
`!shapes.glyphs.is_empty()` (`:1976`) and that each is a `Shape::Text` (`:1977-1982`), because the
whole point of that test is the order of the kinds *relative to each other*. A test about glyphs
being painted after backgrounds cannot be written without glyphs.

So the honest payoff is **five of seven**, with two tests still building a `Context`. That is still
worth doing; it is not the clean sweep the reviews described.

## Shape of the split

Glyph *placement* genuinely needs the atlas — the centring at `:646` divides `galley.size()`, which
does not exist until the galley does — so the split is not "geometry here, colours there". It is:

- one pass over the Grid producing borders, cursor, seams (and the backgrounds already built
  separately at `:582-607`), taking a `&Paint` and a `&GridViewport` and no table;
- a second step taking the `GlyphTable` and producing the `glyphs` field.

`SourceShapes` keeps its five fields and `into_shapes` keeps its chain
(`console/src/console.rs:671-678`), because paint order across the kinds is the thing that value
exists to state and `source-paint` settled it. What changes is how the fields are filled.

**Do not make the glyph step lazy.** `source-paint/spec.md` settles this: `Painter::extend` runs the
iterator inside `ctx.graphics_mut`, a full `Context` write lock, so a lazily built shape is a shape
built while holding it. All five fields must be complete before the first `Shape` leaves.

**Blocked by:** 07

Both rewrite `SourceShapes::new`'s parameter list and its loop; `07` is two parameters and this is a
restructure. Taking them in the other order writes the same signature twice and resolves the same
conflict twice.

**Status:** ready-for-agent

- [ ] Borders, the Cursor's border and the sector seams are built by a function that takes no
      `GlyphTable` and no `egui::Context`.
- [ ] The five geometry tests listed above build no `egui::Context`, directly or through a helper.
- [ ] `SourceShapes` still has five fields and `into_shapes` still chains them in the same order.
      Paint order is unchanged and `the_shape_groups_reach_the_painter_in_the_order_into_shapes_chains_them`
      still passes.
- [ ] No shape construction becomes lazy.
- [ ] The two tests that still need the atlas —
      `a_glyph_is_painted_for_every_cell_that_shows_one_and_no_other` and
      `every_background_is_painted_before_every_glyph_and_the_cursor_after_both` — keep a helper that
      says in one line why they need it. The old helper's "for one reason" comment is now accurate
      and should stay.
- [ ] Nothing about what is drawn changes. Same shapes, same order, same positions.

## Verification

```sh
cargo fmt --all -- --check
cargo clippy --package console --all-targets --locked -- -D warnings
PROPTEST_CASES=32 cargo nextest run --package console --locked
```

No `orcvs` run and no doctest run: everything named is `pub(crate)` or private to `console`.

## Comments

`needs-triage`, and the reason is in the section above. This is a restructure of a shipped loop whose
payoff was overstated in the reviews — five tests stop building a `Context`, not six of seven — and
five tests is a real but partial win against a change that touches the one function every painted
shape goes through. A maintainer should decide whether that trade is worth taking, and whether it is
worth taking *before* ticket `04`, which will add the bloom and seam derivation to this same area of
`console`.

The smaller alternative, if the trade is refused: leave the loop alone and give `source_shapes` a
sibling helper that builds only the non-glyph groups, so the five geometry tests can call that. It
buys the same test property without restructuring shipped code, at the cost of a second construction
path that has to stay in step with the first — which is the kind of second truth this effort is
otherwise removing.

> *This was generated by AI during triage.*

**Triage 2026-09-12 — accepted → `ready-for-agent`.**

The loop split is accepted at the honest payoff (five of seven). The sibling-helper alternative is
refused — it would leave a second construction path. Sequencing: blocked on `07` only; does **not**
wait on `04`.
