# Console requirements

> Snapshot: incorporates ADRs through 0110 and ADR changes dated through 2026-11-15.
> A newer ADR binds where it differs until this document is updated; the
> [reading order](README.md#reading-order) says which ADRs are newer.
> Source ADRs: 0022, 0034 (Render Frame amendment), 0038, 0040, 0042, 0043 (presentation clause), 0044,
> 0045, 0046, 0047, 0048, 0051, 0052, 0053, 0054 (console clauses), 0055 (console clause), 0058,
> 0059 (console clause), 0061 (paint clause), 0063 (no Sequence value), 0069 (paint clause),
> 0076 (Panel clauses), 0077 (paint clauses), 0078, 0079, 0080, 0081, 0082, 0083, 0084,
> 0085 (paint clause), 0086 (Small Console shows the Panel), 0088 (console Theme key), 0089,
> 0091 (settings keys), 0092, 0095, 0096, 0098, 0100, 0102, 0105 (paint clause).
> Reviewed, not applicable or wholly superseded: 0050.

This document owns the console: the boundary between `orcvs` and the UI toolkit, how the Source
Grid is drawn, the Source View, Pan and Zoom, the Region and keyboard routing, Themes, the window,
the Minimap, the Panel, its Toasts and Info, and touch input. It does not own what the language
means ([language.md](language.md)), how a Tick runs ([evaluation.md](evaluation.md)), Playback and
outputs ([playback.md](playback.md)), or the site and its page settings ([site.md](site.md)). Terms
are defined in [CONTEXT.md](../../CONTEXT.md). The decisions and the arguments for them are in
[the ADRs](../adr/).

## Crate boundary

### CONS-010 `orcvs` does not depend on the UI toolkit

`orcvs` never depends on `egui`, `eframe` or `winit`. No module moves into `orcvs` until it is free
of them. `lang` parses and interprets. `orcvs` owns the Source, the Grid, the Language Map, the
Cursor, the Region, the Render Frame and the Playback Engine. The console owns the window, input
translation, style and presentation. The running instance is named `Orcvs`, not `App`.

Reason: the compiler enforces this cut, and domain tests stop paying for a UI compile.
Sources: [ADR 0022](../adr/0022-keep-the-core-crate-free-of-the-ui-toolkit.md).
Evidence: `orcvs/Cargo.toml` (no toolkit dependency); enforced by the compiler.

### CONS-011 Facts about a Source live in `orcvs`; how the console shows them lives in the console

Being free of the toolkit is necessary but not sufficient for code to live in `orcvs`. A fact about
a Source (for example, which Language Unit a Cell belongs to) belongs to `orcvs`. A decision about
how the console shows a Source (alphas, font sizes, glow bands) belongs to the console, even when it
can be written as plain numbers.

Built differently; see [Accepted but not built](#accepted-but-not-built-or-built-differently).

Reason: the compiler enforces only the dependency half, so review enforces this half.
Sources: [ADR 0042](../adr/0042-responsibility-is-not-freedom-from-the-toolkit.md).
Evidence: enforced by review; no mechanical gate.

### CONS-012 Choosing an output device is console configuration

The console selects MIDI devices and network destinations. A running Orcvs does not own the
choice of output.

Reason: an output port is a user's configuration, not part of a running Orcvs.
Sources: [ADR 0022](../adr/0022-keep-the-core-crate-free-of-the-ui-toolkit.md); CONTEXT "Orcvs (running instance)".
Evidence: `console/src/console_midi.rs`, `console/src/midi.rs`, `console/src/web_midi.rs`.

## Drawing the Source

### CONS-020 The console holds the Source's translation and draws in an untransformed layer

The console holds the translation the Source is presented under. It applies that translation when
it computes each Cell's rectangle. No layer carrying the Source Grid has a layer transform, and the
console uses no `egui::Scene`. The Source View holds a translation and no scale ([CONS-040](#cons-040-the-console-has-one-zoom-eguis-whole-ui-zoom)).

Reason: a layer transform deep-clones every Cell's galley on every Render Frame, resamples Glyphs, and leaves glyph positions stale.
Not: drawing glyph quads into a hand-built Mesh under a transform.
Sources: [ADR 0038](../adr/0038-the-console-owns-the-source-grid-transform.md), [ADR 0058](../adr/0058-zoom-is-eguis-whole-ui-zoom.md).
Evidence: `console/src/console/source_view.rs` `show_source_scene`, `console/src/grid_viewport.rs`; test `console/src/console/tests.rs` `no_layer_carrying_the_source_grid_is_transformed`.

### CONS-021 Each Cell lays out its own Glyph at the size it is drawn

Each Cell's Glyph is laid out on its own, at the size it is drawn in points, so each zoom factor
rasterises one Glyph size. The console never coalesces a run of Cells into one galley.

Reason: a Position has to land where the Grid says. A ligature-bearing shaper merges pairs such as `->` and shifts every column after them.
Sources: [ADR 0038](../adr/0038-the-console-owns-the-source-grid-transform.md), [ADR 0058](../adr/0058-zoom-is-eguis-whole-ui-zoom.md).
Evidence: `console/src/console/glyphs.rs`; test `console/src/console/tests.rs` `a_glyph_is_laid_out_at_its_cells_own_size_at_every_device_scale`.

### CONS-022 A Paint is a value with no geometry, derived without a running console

The console decides how drawn Cells look as a value. `Paint::derive_with_theme` takes a
`FramePaint` (a Render Frame paired with drawn ranges minted for that Frame's Grid) and a Theme, and
nothing else: no `Orcvs` and no `egui::Context`. A Paint carries no `Rect`. Where a Cell sits
belongs to `GridViewport`, and a separate step builds the Shapes. `RenderFrame::derive` stays
crate-private, and drawing the Source answers only the Cell it clicked: no test-only seam is cut
into a shipped function. A test that asserts a Cell's colour builds no `Context`. A cost claim on
this path is asserted by counting Cells and Shapes, not by timing.

Reason: the console paints the Grid as one surface with no per-Cell widgets, so a colour can be tested only as a value.
Not: screenshot tests, which are platform-sensitive.
Sources: [ADR 0040](../adr/0040-the-console-paints-from-a-value.md).
Evidence: `console/src/paint.rs` `Paint`, `FramePaint`; test `console/src/console/tests.rs` `the_cursor_reaches_the_paint_of_a_cell_and_never_its_geometry`.

### CONS-023 A Paint covers only the Positions the console draws

A Paint covers the `VisiblePositions` the Source View answers: two column and row ranges clamped to the
Grid. Each Cell's paint reads only its own Cell, so the cull saves work and never changes an
answer. A Paint answers the Cursor as `Option<Position>`, and `None` means the Cursor is not drawn.

Reason: the cost of drawing a Render Frame follows the Source View, not the 256 by 256 Grid.
Sources: [ADR 0040](../adr/0040-the-console-paints-from-a-value.md).
Evidence: `console/src/grid_viewport.rs` `visible_positions`; test `console/src/console/tests.rs` `the_draw_loop_paints_the_visible_range_rather_than_the_whole_source`.

### CONS-024 Cell backgrounds coalesce into runs drawn as ordinary rectangles

Adjacent Cells with the same background in a row coalesce into a `BackgroundRun` over a column
range, and each run is drawn as an epaint rectangle. Backgrounds are never one hand-built Mesh.

Reason: this keeps pixel snapping and per-rectangle anti-aliasing with epaint.
Sources: [ADR 0040](../adr/0040-the-console-paints-from-a-value.md).
Evidence: `console/src/paint.rs` `BackgroundRun`; test `console/src/console/tests.rs` `a_background_run_covers_exactly_the_cells_it_replaces`.

### CONS-025 A Render Frame Cell carries the parser's Claim, shared, not projected

A Render Frame Cell carries the parser's Claim for the slot that covers it: the Cells it claims,
the declared Token, the bound Atom or none, and `written`. `None` means no entry claims the Cell.
One Claim is shared by every Cell it covers. The Expression-local `parent` does not cross into the
console. A Render Frame carries `Option<Token>`, never a separate per-Cell classification.

Reason: a scalar projection of the Claim loses the difference between a blank operand and a wrong one, and each reader that needs the Claim would rebuild it.
Sources: [ADR 0044](../adr/0044-paint-reads-the-parsers-claim.md), [ADR 0052](../adr/0052-a-claim-answers-whether-its-slot-is-written.md), [ADR 0034](../adr/0034-execute-against-live-typed-expressions.md) (Render Frame amendment).
Evidence: `orcvs/src/render_frame.rs` `RenderCell::claim`, `RenderCell::source_paint`; test `orcvs/src/render_frame.rs` `a_valid_function_with_unbound_operands_shares_each_parsers_claim`.

### CONS-026 A Claim answers whether its slot is written, once, when it is built

`written` is true when any Cell the Claim covers holds content in the revision it was read
against. A space counts as blank. The answer is computed once per Claim, while the Render Frame
builds its Claims from that revision's bytes; no revision rebuild computes it. The Language Map
holds only parser Claims: no Source bytes and no whole-Grid Paint view. Paint makes no per-Cell
Span walk and no pointer-keyed lookup.

Reason: the Claim's construction is the one place that holds both its Cell range and the revision's bytes. Every later place pays for a map or a side table.
Not: caching finished Paint facts on the Language Map, which puts Paint work into every revision rebuild.
Sources: [ADR 0052](../adr/0052-a-claim-answers-whether-its-slot-is-written.md).
Evidence: `orcvs/src/source/language_map.rs` `LanguageMap::claims_by_cell`, `Claim::written`; test `orcvs/src/render_frame.rs` `every_cell_of_a_claim_reads_the_written_answer_its_claim_was_built_with`.

### CONS-027 The console decides how each language fact is shown

`RenderCell::source_paint` combines a Claim's Token, Atom and `written` into Function, Pending
Operand, Valid Operand, Invalid Operand, Bang, Comment and Unclaimed. The console chooses the
colours and their precedence for those facts. The paint reads them from the Source alone, while
the Source is being typed:

- A Pending operand shows its declared Token's colour, not a fault.
- An empty claimed operand Cell shows its declared Token's background and no character.
- A Function the Parser refuses to nest shows as invalid.
- Interval Set and Voicing operands each have a type of their own.
- A Payload is drawn as text.

A Cell shows its own character, or a space.

Reason: choosing how a fact looks is the console's decision. Fixing the fact is the language's.
Sources: [ADR 0044](../adr/0044-paint-reads-the-parsers-claim.md), [ADR 0042](../adr/0042-responsibility-is-not-freedom-from-the-toolkit.md), [ADR 0034](../adr/0034-execute-against-live-typed-expressions.md), [ADR 0061](../adr/0061-a-nested-function-returns-to-the-slot-it-occupies.md), [ADR 0069](../adr/0069-an-unwritten-operand-is-invalid-input.md), [ADR 0085](../adr/0085-udp-and-osc-send-the-cells-up-to-their-closer.md), [ADR 0105](../adr/0105-interval-set-and-voicing-are-literal-types.md).
Evidence: `console/src/style.rs`, `console/src/paint.rs`; test `console/src/style.rs` `a_pending_operand_keeps_its_declared_colour_rather_than_diagnostic`.

### CONS-028 The static Output Portal highlight is each value answer's Output Portal Reservation

Before any Tick runs, the Source revision answers which Cells draw as an Output Portal: every Cell
of an Output Portal Reservation. Each Expression whose leading Cells parse as a Function reserves,
whether or not its operands bind. Its root reserves, and so does every nested Function that answers
a value. An effect Function, a Bang Function, a Terminal Output Function, Halt and a Source Bundle
reserve nothing, and the row edge clips a Reservation. Which Cells a Reservation covers is the rule
the schedule reserves an answer write by ([evaluation.md](evaluation.md)). The highlight is never
part of a Claim, and it paints beside the Tick Portal roles ([CONS-029](#cons-029-a-cells-tick-portal-roles-combine-and-one-is-painted)).

Reason: a Function shows where its answer lands from the Source alone, so a Function whose Turn wrote nothing still shows its own.
Sources: [ADR 0052](../adr/0052-a-claim-answers-whether-its-slot-is-written.md), [ADR 0077](../adr/0077-a-committed-tick-records-its-portal-accesses.md), [ADR 0063](../adr/0063-a-list-is-cells-in-a-claim-not-a-value.md).
Evidence: `orcvs/src/source/language_map.rs` `LanguageMap::output_portal_highlight`, `output_portal_reservations`; `orcvs/src/source/mod.rs` `SourceRevision::output_portal_highlight`; test `orcvs/src/source/language_map.rs` `a_scalar_root_covers_the_cell_pair_south_of_its_output_portal`.

### CONS-029 A Cell's Tick Portal roles combine, and one is painted

The Render Frame gives each Cell three roles:

- Input Portal, for each Cell a resolved read covers.
- Output Portal, for each Cell an admitted write covers.
- Locked, for the Function Cells of each root a lock withheld.

The console paints Output Portal over Input Portal over Locked. Output and Input Portal leave a
bound Function's paint alone. Locked paints a Function and a Bang. A `Current` record paints every
resolved read and admitted write. A `Retained` record paints a read or a write only while all of
these hold:

- The displayed revision's Language Map still claims the entry's anchor for a Function that begins
  there and parsed as the entry's Function.
- No edit since the commit has written a Cell of that owner's subtree. Writing back what stood
  there still counts as an edit.
- For a `Selected` entry, no resolved read from a strict descendant of the owner has read a Cell
  an edit has written since.

A lock stays marked while the Halt stands at its anchor and neither its own Cells nor the locked
root's spelling are edited, and only where the displayed revision still has an Expression with a
Function beginning at the locked anchor. Whether the Halt would still be banged is not asked. Unresolved reads, refused writes, locks that withheld nothing and Source Bundle
parts paint nothing.

Reason: a Cell both read and written shows the write, as Orca draws a port over a locked Cell.
Sources: [ADR 0077](../adr/0077-a-committed-tick-records-its-portal-accesses.md).
Evidence: `orcvs/src/render_frame.rs`; test `console/src/style.rs` `output_portal_wins_over_input_portal_on_every_channel`.

## Cell geometry and edges

### CONS-120 Source Cells are configurable rectangles

A Source Cell is 12 points wide and 18 tall by default. Its width and height are set under `[cell]`
in the config file, within 8 to 64 points each, and each key falls back on its own. Each axis snaps
to whole physical pixels on its own, at least one pixel. A Cell's rectangle is its column times the
snapped width and its row times the snapped height, from the presented Grid's corner. The Glyph's
size is the smaller of three quarters of the snapped height and the snapped width divided by 0.62
(Monaspace Neon's advance). The default window of 1024 by 640 points shows about 85 by 35 Cells.
A Sector is a block of Cells, not a square.

Reason: Orca's 2:3 shape reads as a row of text, and 12 by 18 stays whole at 1×, 1.5× and 2×. The Glyph's width term stops a narrow Cell overlapping its neighbours.
Sources: [ADR 0079](../adr/0079-source-cells-are-configurable-rectangles.md).
Evidence: `console/src/grid_viewport.rs` `CELL_SIZE`, `glyph_size`; test `console/src/grid_viewport.rs` `the_source_cell_is_twelve_by_eighteen_points`.

### CONS-121 The Cursor and Region show a still outline

The Cursor and a Region show a still outline in `cursor.border` and `region.border` at their
widths. Nothing a Theme styles is animated. `[cursor_effects]` keys get the unknown-key notice, and a Theme that
sets `cursor.area` is refused.

Reason: motion around the Cursor distracts from reading the Grid.
Sources: [ADR 0079](../adr/0079-source-cells-are-configurable-rectangles.md).
Evidence: test `console/src/config.rs` `the_removed_glitch_keys_are_unknown`.

### CONS-122 Each Cell edge shows one line, the highest-ranked fact either side

An edge between two Positions, or between a Position and the Grid's outside, shows exactly one
line. The facts rank, highest first:

1. The Cursor or Region outline: the Cursor's Cell when no Region spans more than one Cell, or a spanning
   Region's perimeter.
2. The Cursor's own Cell inside a spanning Region.
3. A Bang's role border.
4. The Portal and Locked borders.
5. Diagnostic.
6. A claimed Cell's role border.
7. The Grid line.

Among the Portal and Locked borders, the order is ADR 0077's: Output Portal over Input Portal over
Locked ([CONS-029](#cons-029-a-cells-tick-portal-roles-combine-and-one-is-painted)). A Bang ranks
above the Portals, so it is framed on all four sides.

A width of zero withdraws a fact from the edge rather than hiding the edge. The winning line's
colour is composited once over the facts below it. A Sector Seam composites over its edge's line,
except on the Cursor or Region outline. The Paint decides each edge's fact, colour and width, and
coalesces them into runs.

Built differently; see [Accepted but not built](#accepted-but-not-built-or-built-differently).

Reason: Cells that each stroke their own border draw every shared edge twice, at twice the Theme's width, and composite a translucent edge twice.
Not: a lattice of full-length lines with outlines painted over it.
Sources: [ADR 0078](../adr/0078-each-cell-edge-is-drawn-once.md).
Evidence: `console/src/paint.rs` `EdgeRank`, `EdgeLine`, `Edges`; test `console/src/console/tests.rs` `an_empty_grid_draws_each_grid_line_once`.

### CONS-123 Edge lines sit on whole physical pixels as one Mesh

A line w physical pixels wide on boundary pixel b covers b − ⌊w/2⌋ to b − ⌊w/2⌋ + w. A line
thinner than one pixel covers one pixel, at its alpha scaled by its width. Junctions are their own
pieces: as wide as the widest vertical line there and as tall as the widest horizontal one, with
the highest-ranked lines keeping their own widths through the junction. No two pieces overlap. All
the pieces are one `Shape::Mesh` of axis-aligned quads. The shapes step has a benchmark,
`paint_shapes`.

Reason: epaint rounds thin rectangles and segment ends inward, which leaves holes at crossings and notches at corners. Whole-pixel quads have no edge to anti-alias.
Sources: [ADR 0078](../adr/0078-each-cell-edge-is-drawn-once.md).
Evidence: `console/src/console/shapes.rs`, `console/benches/paint.rs`; test `console/src/console/tests.rs` `a_one_pixel_crossing_is_drawn`.

## Source View, Pan and Zoom

### CONS-030 The Source View shows the Source at its own Cell size

The Source View opens on the Grid's top-left Cell. The Source's Cells are sized in points.
Resizing the window changes how much of the Source shows, never the Cell size. The view is never
fitted to the window, letterboxed, centred or scrolled.

Reason: a spatial language is moved through, not fitted. A Grid fitted to the window read as an infinite canvas the language does not have.
Not: an `egui::ScrollArea` with scrollbars.
Sources: [ADR 0045](../adr/0045-the-source-view-is-a-bounded-space.md), [ADR 0058](../adr/0058-zoom-is-eguis-whole-ui-zoom.md), [ADR 0054](../adr/0054-a-grid-is-always-256-by-256.md), [ADR 0043](../adr/0043-the-grid-is-bounded-by-definition.md).
Evidence: `console/src/console/source_view.rs`; test `console/src/console/tests.rs` `a_resize_keeps_the_cell_size_and_shows_more_or_less_of_the_source`.

### CONS-031 A Pan stops at the Grid's edges, and nothing past the Grid is shown

At a Pan of zero, the Grid's top-left Cell rests on the console's top-left. A Pan stops where the
Grid's far edge meets the console's edge. On an axis where the Grid is smaller than the console,
the Grid rests at the top-left and has nowhere to Pan. A resize or a zoom that would open a gap
past an edge settles back inside. The Source View and the Minimap are measured against the Grid
alone.

Reason: the Grid is bounded, so its presentation is too; anything outside it is not a Position.
Sources: [ADR 0045](../adr/0045-the-source-view-is-a-bounded-space.md), [ADR 0084](../adr/0084-the-source-view-has-no-margin.md), [ADR 0058](../adr/0058-zoom-is-eguis-whole-ui-zoom.md).
Evidence: `console/src/console/source_view.rs`; test `console/src/console/tests.rs` `clamp_pan_pins_a_smaller_source_at_the_origin_and_a_larger_one_to_its_edges`.

### CONS-032 The Pan gestures, none of which moves the Cursor or the Region

These gestures Pan:

- The wheel or a two-finger scroll.
- A middle-drag.
- Alt (Option) held with a primary drag.
- A press or drag on the Minimap ([CONS-090](#cons-090-the-minimap-is-a-pan-gesture)).
- A finger's drag ([CONS-110](#cons-110-a-finger-pans-taps-and-long-presses)).

A primary click or drag with no modifier belongs to making a Region. No Pan moves the Cursor or
changes the Region.

Reason: Space toggles Playback, so it cannot be the drag modifier. The plain primary drag is the one gesture a Region has.
Sources: [ADR 0045](../adr/0045-the-source-view-is-a-bounded-space.md), [ADR 0046](../adr/0046-the-primary-drag-selects-a-region.md), [ADR 0080](../adr/0080-the-minimap-pans-over-a-cached-picture.md), [ADR 0102](../adr/0102-a-finger-pans-the-source-and-asks-for-the-keyboard.md).
Evidence: `console/src/console/source_view.rs`; test `console/src/console/tests.rs` `wheel_pans_a_larger_source_to_its_edges_and_a_smaller_one_nowhere`.

### CONS-033 The Source View follows the Cursor just far enough

A Cursor move that would leave the Cursor out of view Pans just far enough to show its whole Cell.
A follow to an edge Cell brings that Cell flush with the console's edge. A resize or zoom that
would take a shown Cursor out of view does the same. A Pan alone is never pulled back to the
Cursor, and a resize or zoom after the viewer has Panned the Cursor out of view keeps that Pan.

Reason: the next keystroke writes at the Cursor, so the viewer must be able to see it, without taking away a deliberate Pan.
Sources: [ADR 0045](../adr/0045-the-source-view-is-a-bounded-space.md), [ADR 0058](../adr/0058-zoom-is-eguis-whole-ui-zoom.md), [ADR 0084](../adr/0084-the-source-view-has-no-margin.md).
Evidence: `console/src/console/source_view.rs`; test `console/src/console/source_view.rs` `a_cursor_move_that_would_leave_it_outside_the_view_pans_to_show_it`.

### CONS-034 The pointer shows a drag Pan

The pointer is a grab hand while Alt is held over the console, and a grabbing hand while an
Alt-drag or a middle-drag is Panning. A middle-drag shows no hover state before it starts, and a
wheel Pan shows no hand.

Reason: a modifier gesture has to announce itself before the drag begins.
Sources: [ADR 0047](../adr/0047-the-source-view-has-a-margin.md), as amended by [ADR 0079](../adr/0079-source-cells-are-configurable-rectangles.md).
Evidence: test `console/src/console/kittest_tests.rs` `the_pointer_shows_a_grab_hand_for_alt_and_a_grabbing_hand_while_a_drag_pans`.

### CONS-035 The default window is fixed in points

The default window gives the Source 1024 by 640 points.

Reason: the window's size is stated in points, never as a count of Cells.
Sources: [ADR 0047](../adr/0047-the-source-view-has-a-margin.md), as amended by [ADR 0079](../adr/0079-source-cells-are-configurable-rectangles.md).
Evidence: `console/src/main.rs`; test `console/src/console/tests.rs` `the_default_window_is_a_fixed_size_in_points`.

### CONS-040 The console has one zoom, egui's whole-UI zoom

On native, command `+` (or `=`) and command `-` step egui's zoom factor by egui's step of a tenth,
within egui's range of 0.2 to 5.0. Command `0` resets it to 1.0. The View menu offers egui's own
Zoom In, Zoom Out and Reset Zoom. The menus, the Panel, the Diagnostics and the Source Grid grow
together. Theme widths stay in display points at every zoom.

Reason: two zooms would mean two transforms, two meanings of one word, and chords that work differently from every other egui app.
Sources: [ADR 0058](../adr/0058-zoom-is-eguis-whole-ui-zoom.md).
Evidence: `console/src/console/menu_bar.rs`; test `console/src/console/kittest_tests.rs` `command_zoom_chords_change_egui_zoom_and_never_the_source`.

### CONS-041 Zoom persists in egui's memory

eframe saves the zoom factor with egui's memory in a persistence build. The console stores no zoom
key of its own. A build without persistence starts at 1.0.

Reason: egui already owns and serialises the factor.
Sources: [ADR 0058](../adr/0058-zoom-is-eguis-whole-ui-zoom.md).
Evidence: test `console/src/console/kittest_tests.rs` `a_zoom_survives_a_save_and_a_restart`.

### CONS-042 On the web, the browser's page zoom is the one zoom

The web console leaves egui's keyboard zoom off, as eframe sets it, and leaves the browser's
command `+`, `-` and `0` alone. Its View menu has no zoom items.

Reason: zooming egui's factor as well would zoom twice.
Sources: [ADR 0058](../adr/0058-zoom-is-eguis-whole-ui-zoom.md).
Evidence: `console/src/console/menu_bar.rs` (wasm branch); test `console/src/console/tests.rs` `a_browser_zoom_that_would_leave_the_cursor_outside_the_view_pans_to_show_it`.

### CONS-043 Zoom never takes a Source character, and pinch never zooms

Bare `+`, `-`, `=` and `0` are Source characters. Command-shift chords are not zoom. Pinch and
command-wheel do not zoom; plain scroll Pans.

Reason: these characters are Source. egui matches command `+` regardless of Shift.
Sources: [ADR 0058](../adr/0058-zoom-is-eguis-whole-ui-zoom.md).
Evidence: test `console/src/console/tests.rs` `pinch_and_command_wheel_do_not_zoom`.

## Region and keys

### CONS-050 A Region always exists, and the Cursor stays one Cell

A Region spans from an anchor Cell to a live end. When the two are the same Cell, the Region is
that one Cell. The Cursor sits at the live end, except after command `A`.

Reason: every rule written about the Cursor's one Cell keeps holding.
Not: a Cursor with an extent.
Sources: [ADR 0046](../adr/0046-the-primary-drag-selects-a-region.md).
Evidence: `orcvs/src/region.rs`, `orcvs/src/app.rs`; test `orcvs/src/app.rs` `a_fresh_orcvs_has_a_region_of_the_cursors_one_cell`.

### CONS-051 The pointer selects a Region

- A primary press sets the anchor and the Cursor on the Cell under the pointer.
- Dragging moves the Cursor to the Cell under the pointer, and release keeps the Region.
- A click collapses the Region, and Shift with a click extends it from its anchor.
- A drag past the console's edge moves the Cursor past that edge, and the follow scrolls after
  it: faster the further out the pointer is, never more than one Cell a frame, and never past the
  Grid.

Reason: this is how spreadsheets and Tiled split the primary drag.
Sources: [ADR 0046](../adr/0046-the-primary-drag-selects-a-region.md).
Evidence: test `console/src/console/tests.rs` `a_primary_drag_spans_a_region_that_release_keeps`.

### CONS-052 The keyboard edits a Region as a spreadsheet does

- Shift with an arrow moves the live end, and the Cursor moves with it.
- A bare arrow collapses the Region and moves.
- Command `A` spans the whole Grid and leaves the Cursor and the view where they were. The live end
  becomes the Grid's last Cell.
- Escape collapses the Region onto the Cursor.

Moving the live end past the anchor flips the rectangle.

Reason: every spreadsheet leaves the active Cell, and the view with it, where it was.
Sources: [ADR 0046](../adr/0046-the-primary-drag-selects-a-region.md).
Evidence: test `orcvs/src/app.rs` `shift_arrows_extend_the_region_and_a_bare_arrow_collapses_it`.

### CONS-053 A Region is written as a block

- Typing writes at the Cursor, steps right and collapses the Region.
- Command Enter followed by a character fills every Cell of the Region with it.
- Backspace and Delete empty every Cell of a Region larger than one Cell and keep the Region.
- Copy puts the Region's rows on the clipboard, joined by newlines, with trailing spaces kept.
  Cut copies, then empties.
- Paste writes from the Region's top-left, spaces included, clipped at the Grid's edges. The
  Region becomes what landed. A character that cannot be a Cell lands as an empty Cell, and `\r\n`
  is one row break.

Reason: Orcvs has no undo, so the fill sits behind an explicit chord.
Not: typing fills the Region.
Sources: [ADR 0046](../adr/0046-the-primary-drag-selects-a-region.md).
Evidence: test `orcvs/src/app.rs` `command_enter_then_a_character_fills_the_region`.

### CONS-054 A Region is not stored with the Source

The Region lives beside the Cursor in `Orcvs` and reaches the console on the Render Frame. It is
never stored with the Source. A Region larger than one Cell is drawn with a background tint.

Reason: a Region is editing state, not content.
Sources: [ADR 0046](../adr/0046-the-primary-drag-selects-a-region.md).
Evidence: `orcvs/src/render_frame.rs` `RenderFrame::region`; test `orcvs/src/render_frame.rs` `render_frame_carries_the_region_it_was_derived_for`.

### CONS-055 The Source takes every key that no control holds

The Source gets every key while no control holds egui's keyboard focus and no popup (a menu, a
ComboBox list) is open. It gets none while one does. The answer is latched once every widget has
run, and read on the next frame. No control is named.

Reason: a named list of controls leaks keys to every control added later. Latching keeps the Escape that leaves a field from also collapsing the Region.
Not: the Grid as a focusable widget, which would leave keys going nowhere at startup.
Sources: [ADR 0048](../adr/0048-the-source-takes-the-keys-no-control-holds.md).
Evidence: `console/src/console/input.rs` `latch_keyboard_owner`; test `console/src/console/kittest_tests.rs` `a_focused_menu_item_keeps_region_and_clipboard_commands_from_the_source`.

### CONS-056 Keys a control took disarm a fill

When command Enter has armed a fill and the next key goes to a control, the fill is disarmed.

Reason: a key a control takes is the next event, even though the Source never sees it.
Sources: [ADR 0048](../adr/0048-the-source-takes-the-keys-no-control-holds.md).
Evidence: test `console/src/console/tests.rs` `keys_the_bpm_field_took_disarm_a_fill_armed_before_it`.

### CONS-057 The console area never takes focus, and Tab steps by Sector

Nothing in the area the Source is shown in is focusable, so Tab never lands there. While the
Source holds the keys, Tab and Shift Tab step the Cursor a Sector along its row. One press never
both moves egui's focus and steps the Cursor.

Reason: a focusable rectangle would count as a control holding the keys and lock the Source out.
Sources: [ADR 0048](../adr/0048-the-source-takes-the-keys-no-control-holds.md); Sector navigation ruled, no ADR (`.scratch/sector-navigation/issues/01-tab-steps-the-cursor-by-sector.md`); CONTEXT "Sector".
Evidence: test `console/src/console/kittest_tests.rs` `tab_never_focuses_the_console_area_the_source_is_shown_in`.

## Themes

### CONS-060 A Cell's facts reach its channels through Theme colours alone

A Cell carries facts. The console draws them through channels: glyph colour, fill, border and the
lines it shows. No code chooses a fact's channel in control flow. A Theme puts a fact on a channel
by giving that fact's key for that channel a colour, and keeps it off by making the colour
transparent. Diagnostic and Output Portal take a key per channel. A fact's channel composites over
the Token's colour on that channel, so a transparent channel leaves the Token showing.

Reason: a mapping written as control flow cannot be restated by a Theme.
Sources: [ADR 0051](../adr/0051-a-theme-maps-facts-to-channels.md), [ADR 0053](../adr/0053-one-theme-styles-the-whole-console.md).
Evidence: `console/src/style.rs`; test `console/src/style.rs` `an_invalid_operand_draws_diagnostic_but_keeps_its_declared_tint`.

### CONS-061 One Theme styles the whole console

The same resolved Theme styles the Source Grid and every part of the console around it: the page,
panels, widgets, selection, Grid lines and Sector Seams. egui's `Visuals` are built from the
Theme. Selection at startup, a change in the OS appearance and the mode toggle each apply one
Theme to both at once.

Reason: a separate Source scheme and chrome theme would need a contrast review for every pairing.
Sources: [ADR 0053](../adr/0053-one-theme-styles-the-whole-console.md).
Evidence: `console/src/style.rs` `style`; test `console/src/console/kittest_tests.rs` `the_mode_switches_source_and_chrome_together`.

### CONS-062 Settings select a dark and a light Theme by identity, and hold no Theme values

On native, the `[theme]` table of `~/.orcvs/config.toml` names a `dark` and a `light` Theme by
identity. The console reads the file once, at startup, and never writes either selection. An absent
key selects `grayscale-dark` or `grayscale-light`. No menu offers a Theme setting or picker, and
the console has no Theme editor.

Reason: a Theme is its own document, and a stored value would stop a moved default from reaching an install.
Sources: [ADR 0053](../adr/0053-one-theme-styles-the-whole-console.md) (amendments 2026-10-10 and 2026-10-25).
Evidence: `console/src/config.rs`, `console/src/theme_selection.rs`; test `console/src/theme_selection.rs` `a_default_selection_presents_each_appearances_built_in_and_raises_no_notice`.

### CONS-063 The appearance mode is one toggle

One icon toggle at the right of the top bar shows the appearance the console shows, `☾` or `☼`. A
press holds the other appearance as egui's `ThemePreference`, which egui's memory stores. Until
the first press the preference is `System`, so the console follows the OS. No control returns it to
`System`. The toggle names itself for accessibility.

Reason: one toggle that shows the current appearance is the plainest mode control; three buttons confuse.
Sources: [ADR 0053](../adr/0053-one-theme-styles-the-whole-console.md) (amendment 2026-10-31).
Evidence: `console/src/console/menu_bar.rs`; test `console/src/console/kittest_tests.rs` `the_mode_toggle_flips_the_appearance`.

### CONS-064 A custom Theme inherits exactly one built-in and states only its changes

Every Theme is dark or light. A custom Theme names one built-in in `inherits` and lists only the
properties it changes. Omitted properties keep the parent's values, and nothing is recalculated. It
never inherits from another custom Theme. Its appearance is its parent's, and a conflicting
declaration is refused. Only a dark Theme fills the dark selection. Settings never override a
Theme's values. Built-in and custom Themes are the same type.

Reason: one answer to what a colour is. An override would be a second answer.
Sources: [ADR 0053](../adr/0053-one-theme-styles-the-whole-console.md).
Evidence: `console/src/theme_document.rs`; test `console/src/theme_document.rs` `a_custom_theme_cannot_chain_from_another_custom_theme`.

### CONS-065 Native Theme files are read from one directory at startup

On native, every startup scans `~/.orcvs/themes/` and reads its Theme files. No other directory
is searched. Edits take effect on the next launch: there is no watcher and no reload action.

Reason: one location needs no precedence rules, and the files stay authoritative.
Sources: [ADR 0053](../adr/0053-one-theme-styles-the-whole-console.md).
Evidence: `console/src/theme_registry.rs`; test `console/src/theme_registry.rs` `only_direct_children_with_a_theme_suffix_load_and_other_files_are_ignored`.

### CONS-066 A Theme file's identity is its filename stem

The filename without its extension is the identity that settings and `inherits` reference. A
`name` in the document is only a display label. Built-in identities are reserved, and a file that
uses one is refused and reported. When several files share a stem, every one of them is refused and
reported by path, whatever the enumeration order. The stem keeps its case, and the `.toml`
extension matches case-insensitively.

Reason: directory order never chooses a winner.
Sources: [ADR 0053](../adr/0053-one-theme-styles-the-whole-console.md).
Evidence: test `console/src/theme_registry.rs` `duplicate_identities_conflict_whatever_the_enumeration_order`.

### CONS-067 An unavailable selected Theme falls back without losing the selection

A selected Theme that is missing, malformed, conflicted, retired or of the wrong appearance does
not stop startup. The console shows its appearance's default built-in and a notice naming the Theme
and the problem. The fallback is only what is shown: the saved selection is kept, so repairing the
file restores it on the next launch.

Reason: a broken Theme file must never cost a performer the console or the choice they made.
Sources: [ADR 0053](../adr/0053-one-theme-styles-the-whole-console.md).
Evidence: test `console/src/theme_registry.rs` `a_refused_selected_file_falls_back_and_is_restored_by_its_repair`.

### CONS-068 A Theme document is strict TOML

A Theme document is a `.toml` file with `format = "orcvs-theme"`, `version = 1`, `name`, `inherits`
and a `style` table of named properties. No other extension is a Theme file. It is decoded directly
into a strict type:

- An unknown key, a repeated key, a malformed colour, an out-of-range width, a missing or
  unsupported marker, or an unknown appearance refuses the whole document. Nothing is ignored or
  clamped.
- An error names the file, the key and its line. Its message is capped at 1024 bytes.
- Contrast warnings alone never refuse a document.

The complete property catalogue is `.scratch/theming/schema.md`, held to the code by
`theme_document.rs`'s catalogue tests.

Reason: TOML is typed, always has a table root and refuses repeated keys, so the derived type is strict on its own.
Not: JSON or YAML documents.
Sources: [ADR 0053](../adr/0053-one-theme-styles-the-whole-console.md).
Evidence: `console/src/theme_document.rs`; test `console/src/theme_document.rs` `the_format_marker_must_be_orcvs_theme`.

### CONS-069 Theme widths are bounded display points

A Theme controls colours (hex RGBA, with alpha as the only opacity) and stroke widths:

- Grid line, Cell border and Sector Seam widths accept 0 to 1 point. Chrome border widths accept
  0 to 2 points.
- Widths are display points and keep their visible thickness at every zoom and Cell size.
- Width zero hides a stroke and changes nothing else.

Fonts, sizes, spacing, corner radii, shadows and gradients are outside a Theme.

Reason: a line has to read the same at every zoom. A separate opacity key would be a second answer.
Sources: [ADR 0053](../adr/0053-one-theme-styles-the-whole-console.md), [ADR 0058](../adr/0058-zoom-is-eguis-whole-ui-zoom.md).
Evidence: test `console/src/theme_document.rs` `widths_accept_integers_and_floats_within_their_bounds`.

### CONS-070 Theme layers composite over an opaque backdrop

The window backdrop is opaque, and a Theme whose backdrop has alpha is refused. The Grid
background, panels and Cells above it may be translucent and reveal the console beneath, never the
desktop. The layers stack in this order:

1. The Grid background.
2. `cell.background`: one uniform base fill for every Cell, transparent by default, never counted
   as a fact fill for the Region fallback.
3. Named role backgrounds per Token, and the Diagnostic and Portal channels.
4. The Cursor and Region fills.

Changing a role's foreground never recalculates its background. Individual Cells store no styling.

Reason: painting and contrast validation have to use the same composite.
Sources: [ADR 0053](../adr/0053-one-theme-styles-the-whole-console.md).
Evidence: test `console/src/theme_document.rs` `resolve_rejects_a_nonopaque_window_background`.

### CONS-071 The Cursor's colours can be omitted, cleared or transparent

Four properties take one of three states: `cursor.background`, `region.cursor.background`,
`cursor.foreground` and `region.cursor.foreground`.

- Omission inherits the parent's value.
- `none` clears the colour. A cleared Region-Cursor colour falls back to the Cursor's colour. A
  cleared `cursor.background` leaves no Cursor fill. A cleared `cursor.foreground` leaves the
  glyph in its role colour.
- An explicit transparent colour is a supplied value: it hides what it colours and triggers no
  fallback.

No other key takes `none`.

Reason: absence and transparency mean different things for these four properties alone.
Sources: [ADR 0053](../adr/0053-one-theme-styles-the-whole-console.md) (amendment 2026-10-20).
Evidence: test `console/src/theme_document.rs` `optional_colours_are_omitted_cleared_or_a_supplied_transparent_colour`.

### CONS-072 Painting precedence is fixed, never a Theme property

A Theme chooses colours and cannot reorder facts:

- A bound Function keeps its own paint inside an Output or Input Portal, and so does a Bang.
- Otherwise Portal paint takes precedence over the claim's, Diagnostic included ([CONS-029](#cons-029-a-cells-tick-portal-roles-combine-and-one-is-painted)).
- The Cursor's own fill takes precedence over role backgrounds.
- In a Region larger than one Cell, the Cursor uses the Region's cursor fill, falling back to the
  Cursor fill. Other Cells take the Region fill only where they have no fact fill.

Reason: this keeps the decided appearance while channels gain alpha compositing.
Sources: [ADR 0053](../adr/0053-one-theme-styles-the-whole-console.md) (amendment 2026-10-22).
Evidence: test `console/src/style.rs` `a_bang_keeps_its_own_paint_inside_every_portal_role`.

### CONS-073 Every painted text state clears 4.5:1, except Comment

The contrast check measures each resolved Theme's effective foreground against its actual
composited background. It covers every reachable painted text state (roles, Portals, Cursor,
Region, chrome text on panels and inputs) and reports each by role and state. A transparent fact
foreground is measured as the Token it reveals, and a fully transparent glyph is not measured.
Every role except Comment is held to 4.5:1; Comment is measured and reported, and never fails. No
Theme has an accepted-exception list. Every built-in passes the gate. A loaded Theme below the
floor still loads, and the console shows its report as a notice.

Reason: a faint Comment is the point of the role, and per-Theme exceptions would hide failures. WCAG 1.4.3 does not exempt de-emphasised text.
Sources: [ADR 0053](../adr/0053-one-theme-styles-the-whole-console.md) (amendments 2026-10-01, 2026-10-23, 2026-10-25).
Evidence: `console/src/contrast.rs`; test `shipped_theme_gate` in the same file.

### CONS-074 The built-ins pass a colour-vision gate

A test-only gate simulates dichromacy (Viénot, Brettel and Mollon) and measures every pair of
Source glyph channels by CIEDE2000, at their displayed colours in each reachable painted state.
Protanopia and deuteranopia fail a pair below a ΔE00 of 5.0, unless both colours are neutral greys.
Two glyph roles closer than that on the Grid background must differ in fill or in a drawn border.
Tritanopia is measured and not gated. Loaded Themes are not measured.

Reason: text contrast cannot tell whether two Token colours are distinguishable.
Sources: [ADR 0053](../adr/0053-one-theme-styles-the-whole-console.md) (amendments 2026-10-10, 2026-10-25).
Evidence: `console/src/contrast/colour_vision.rs`; test `shipped_theme_colour_vision_gate` in the same file.

### CONS-079 The console's Theme format has no Sound Module key

The console draws no Sound Module, so its Theme format has no `sound.magnitude` key. A console
Theme document that sets it is refused as an unknown key, as [CONS-068](#cons-068-a-theme-document-is-strict-toml) states. The colour
itself belongs to the site's Sound Module ([SITE-038](site.md#site-038-the-sound-module-takes-one-theme-colour-soundmagnitude)).

Reason: the console's Theme format gains the key only when the console draws the Sound Module.
Sources: [ADR 0088](../adr/0088-the-sound-module-shows-the-synths-channels-and-effects-as-cells.md).
Evidence: `console/src/theme_document.rs` (no `sound.*` key); test `console/src/theme_document.rs` `an_unknown_or_wrong_case_style_property_is_refused`.

### CONS-075 Two built-in Themes ship

The built-ins are `grayscale-dark` and `grayscale-light`, and each is its appearance's default and
fallback. An identity the registry does not hold is missing: a selection naming one falls back and
is kept, and a document inheriting one is refused with a notice. `console/src/theme.md` records
each built-in's values.

Reason: the grayscale pair is the chosen default appearance, and nothing depends on a particular Theme, so no other built-in is needed.
Sources: [ADR 0053](../adr/0053-one-theme-styles-the-whole-console.md) (amendment 2026-10-25).
Evidence: `console/src/theme_registry.rs`; test `console/src/theme_selection.rs` `a_selection_of_a_retired_built_in_falls_back_to_the_default_and_is_kept`.

### CONS-076 A Theme decides how things look, never how much they move

No Theme property controls motion. A Theme styles the Cursor's and the Region's colours and the
role backgrounds.

Reason: how something looks and how much it moves are separate concerns.
Sources: [ADR 0053](../adr/0053-one-theme-styles-the-whole-console.md).
Evidence: test `console/src/config.rs` `the_removed_glitch_keys_are_unknown`.

### CONS-077 The web console runs on the built-in Themes

The web reads no Theme directory, and Theme import is off, so the web console uses the built-ins
alone.

Reason: the web has no settings file that could select an imported Theme.
Sources: [ADR 0053](../adr/0053-one-theme-styles-the-whole-console.md) (Status); `.scratch/menu-structure/issues/04`.
Evidence: `console/src/web_startup.rs`.

### CONS-078 No code, test or bench depends on a particular Theme

Shipped code reaches a Theme only through the registry: by selection, or by an appearance's default
and fallback. Only the built-in definitions and the registry's list name a built-in.

- A test that needs a Theme uses `ThemeRegistry::built_in().fallback(appearance)`. It overrides
  only the fields it needs, and says why.
- A test that needs an identity uses `ThemeIdentity::default_for(appearance)`.
- Expected colours come from the Theme's fields, never from copied hex.
- Gates about the shipped Themes iterate every built-in the registry holds.

Reason: retiring or retuning a built-in edits its definition and record, and nothing else.
Not: a lint or grep gate (role, not spelling, separates allowed uses).
Sources: [ADR 0082](../adr/0082-nothing-depends-on-a-particular-theme.md).
Evidence: enforced by review; gates `console/src/contrast.rs` `shipped_theme_gate`, `console/src/style.rs` `every_built_in_chrome_matches_its_record`.

## Window, settings and files

### CONS-080 The native splash is the console's own window

`main` opens the root window borderless, splash-sized and centred, showing the Orcvs logo. When
the splash ends (by itself, or on a key or click, which reaches nothing else), Source View commands
give the same window its decorations and size. The logo then fades over the running Grid.

Reason: eframe has no splash API, and a hidden root window may get no frames on some platforms.
Sources: [ADR 0083](../adr/0083-the-console-opens-through-its-own-window-as-a-splash.md).
Evidence: `console/src/console/splash.rs`; test `console/src/console/kittest_tests/splash.rs` `the_hold_ends_by_itself_and_grows_the_window_to_the_default_console`.

### CONS-081 The console stores and restores its own window geometry

eframe's `persist_window` is off. The console saves the content size, frame position, and maximised
and fullscreen state under its own key, in egui points. The size is clamped to the monitor the
window opens on. The splash's end returns the window to that geometry, or to the default size
centred. A save during the splash stores the geometry the console will return to, never the
splash's.

Reason: eframe restoring the window would show a console-sized frame before every splash.
Sources: [ADR 0083](../adr/0083-the-console-opens-through-its-own-window-as-a-splash.md).
Evidence: `console/src/window_geometry.rs`, `console/src/main.rs`; test `console/src/console/kittest_tests/splash.rs` `a_save_during_the_hold_stores_the_consoles_geometry`.

### CONS-130 Settings are read once at startup, with no settings UI

On native the console reads `~/.orcvs/config.toml` once, at startup. On the web it reads the same
text from its page instead ([site.md](site.md)). No menu offers a setting, nothing reloads the
file, and the console never writes it. The network destination is not a setting: the console
stores it itself, and the Output list edits it (PLAY-350).

Reason: a setting has one owner, the file, and the console never stores a second answer.
Sources: [ADR 0053](../adr/0053-one-theme-styles-the-whole-console.md) (amendment 2026-10-10), [ADR 0091](../adr/0091-the-web-console-reads-its-settings-from-its-page.md).
Evidence: `console/src/config.rs`; test `console/src/console/kittest_tests.rs` `no_menu_offers_a_setting`.

### CONS-131 The settings open views and time Toasts

These settings exist on native and the web:

- `[cell] width` and `height` ([CONS-120](#cons-120-source-cells-are-configurable-rectangles)).
- `[minimap] open`: default open.
- `[diagnostics] open`: default closed.
- `[info] open`: default shown.
- `[toasts] seconds`.
- The Theme selection ([CONS-062](#cons-062-settings-select-a-dark-and-a-light-theme-by-identity-and-hold-no-theme-values)).

The View menu toggles the Minimap, Diagnostics and Info, and a menu change is not saved.
`console/src/config.rs` owns the key list, the defaults and the ranges. An unusable Theme selection
falls back as [CONS-067](#cons-067-an-unavailable-selected-theme-falls-back-without-losing-the-selection)
states. How other faulty keys fall back is an [open decision](#open-decisions).

Reason: the views a console opens with, including on a phone, are configuration rather than code.
Sources: [ADR 0091](../adr/0091-the-web-console-reads-its-settings-from-its-page.md), [ADR 0100](../adr/0100-failures-are-toasts-and-the-panel-keeps-a-line-for-the-cursor.md), [ADR 0098](../adr/0098-the-overview-is-called-the-minimap.md).
Evidence: test `console/src/config.rs` `the_minimap_and_the_diagnostics_open_as_the_file_says`.

### CONS-132 A stored Source of another shape is discarded, not migrated

A stored Source that does not decode to the 256 by 256 Grid is reported, and the console starts an
empty Grid, which the next save writes over it. File > New replaces the environment with an empty
Source on the one Grid.

Reason: Orcvs is pre-release, and every stored Source is an autosave.
Sources: [ADR 0054](../adr/0054-a-grid-is-always-256-by-256.md).
Evidence: `console/src/persistence.rs`; test `console/src/persistence.rs` `a_source_stored_at_a_previous_default_grid_is_discarded_rather_than_migrated`.

### CONS-133 The Help menu reaches the site's Learn and Reference pages

The Help menu offers the site's Learn and Reference pages, the Function reference, the shipped
examples, and About.

Reason: the site's documentation is reached from the console.
Sources: ruled 2026-10-26, no ADR ([orcvs-site/13](../../.scratch/orcvs-site/issues/13-link-learn-and-reference-from-the-help-menu.md)).
Evidence: test `console/src/console/kittest_tests.rs` `the_help_menu_offers_the_sites_learn_and_reference_pages`.

## Minimap

### CONS-090 The Minimap is a Pan gesture

A press on the Minimap centres the Source View there. A drag carries the Source View with the
pointer. A press inside the outline keeps its grab offset rather than jumping. The Minimap reaches
the Source View through one pending Pan applied before the clamp, so it stops at the Grid's edges
as every Pan does. It never moves the Cursor or changes the Region. It shows a grab hand, and a
grabbing hand while held.

Reason: a 256 by 256 Grid needs a way to jump across it.
Sources: [ADR 0080](../adr/0080-the-minimap-pans-over-a-cached-picture.md), [ADR 0045](../adr/0045-the-source-view-is-a-bounded-space.md) (amendment 2026-10-23).
Evidence: `console/src/console/minimap.rs`; test `console/src/console/kittest_tests.rs` `a_press_outside_the_outline_centres_the_source_view_there_and_it_stays`.

### CONS-091 The Minimap's picture is a console cache, rebuilt only when its key changes

The picture is one colour per Cell, decided as a value from a Render Frame and a Theme. It is held
in a console-owned buffer keyed by the Source's revision and the presented Theme and appearance, and
drawn as one nearest-filtered texture updated in place. The revision is read before the Render
Frame, so a racing Tick costs a rebuild, never a stale picture. Nothing about the Minimap lives in
`orcvs` or on the Language Map. A console benchmark with a floor measures the rebuild.

Reason: per-Cell Shapes would upload 65,536 quads every frame, and a Language Map cache would put whole-Grid work into every revision.
Not: a paint callback (backend-specific `unsafe` GL).
Sources: [ADR 0080](../adr/0080-the-minimap-pans-over-a-cached-picture.md).
Evidence: `console/src/console/minimap.rs`, `console/benches/minimap.rs` `paint_minimap`; test `console/src/console/tests.rs` `the_minimap_uploads_its_picture_only_when_the_source_or_appearance_changes`.

### CONS-092 The Minimap draws the Source View as an outline in its own Theme key

The Minimap draws the Source View as an outline in `minimap.source_view`, whose width
`minimap.source_view.width` is within the chrome bounds. Cell colours, the Cursor and the Region
reuse their own keys.

Reason: borrowing the Cursor's or the Region's border would make one channel show two facts.
Sources: [ADR 0080](../adr/0080-the-minimap-pans-over-a-cached-picture.md), [ADR 0098](../adr/0098-the-overview-is-called-the-minimap.md).
Evidence: test `console/src/console/tests.rs` `the_minimap_outlines_the_source_view_where_a_pan_leaves_it`.

### CONS-093 The Minimap floats in the Source's bottom-right corner

The Minimap is an `Area`. The corner of its picture is the Source's bottom-right corner, on the
bottom panel's top edge and flush with the window's right edge. It floats over the Source only,
never over the bottom panel or docked Diagnostics, and opening it leaves the Source's rect
unchanged. Its layer is exactly the bordered picture, so a press outside the picture reaches the
Source. It is one labelled node, "Source minimap", that Tab never focuses.

Reason: docked, it would take the Source's width; in the window's corner, it would cover the Panel.
Sources: [ADR 0081](../adr/0081-the-minimap-floats-over-the-windows-corner.md), [ADR 0096](../adr/0096-the-minimap-sits-in-the-sources-corner-at-one-pixel-per-cell.md), [ADR 0098](../adr/0098-the-overview-is-called-the-minimap.md).
Evidence: test `console/src/console/kittest_tests.rs` `the_minimap_starts_pinned_in_the_sources_bottom_right_corner_and_the_view_menu_toggles_it`.

### CONS-094 The Minimap is one physical pixel per Cell, or hidden

The picture is one physical pixel per Cell at every pixels-per-point: 256 physical pixels a side.
It is never clipped or scaled. Where the picture and its border do not fit in half the Source's
width and its whole height, the Minimap is hidden. The View menu keeps its choice, and the Minimap
returns once it fits.

Reason: clipping would leave Cells out of reach of a press, and fractional scaling would drop rows and columns unevenly.
Sources: [ADR 0081](../adr/0081-the-minimap-floats-over-the-windows-corner.md), [ADR 0096](../adr/0096-the-minimap-sits-in-the-sources-corner-at-one-pixel-per-cell.md).
Evidence: test `console/src/console/minimap.rs` `the_minimap_hides_where_one_pixel_per_texel_does_not_fit_in_the_source`.

### CONS-095 The console uses the Minimap's names alone

The View item, the accessible label, the Theme keys, the `[minimap]` table and the
`paint_minimap/picture/256x256` benchmark all say Minimap. A Theme that sets `overview.*` is refused
as an unknown key, and a settings file with `[overview]` gets the unknown-key notice.

Reason: the language is pre-release, with no compatibility contract.
Sources: [ADR 0098](../adr/0098-the-overview-is-called-the-minimap.md).
Evidence: test `console/src/theme_document.rs` `the_overview_keys_are_refused`.

## Panel, Toasts and Info

### CONS-099 The Panel's Readouts show the run

The Panel is static. It shows Tick at **T**, the Run Clock at **C**, a beat marker and BPM, each
as a Readout in the accessibility tree. A stopped Playback shows the rest marker, the held Tick and
the held Run Clock. After a Rewind it shows Tick `0` and a Run Clock of zero.

Reason: the Panel is the performer's telemetry; what the Run Clock counts belongs to Playback.
Sources: [ADR 0076](../adr/0076-stopping-playback-holds-its-tick.md); CONTEXT "Panel", "Readout", "BPM", "Run Clock" (see [playback.md](playback.md)).
Evidence: `console/src/console/panel.rs`; test `console/src/console/kittest_tests/panel_readouts.rs` `the_tick_run_clock_and_beat_marker_readouts_are_in_the_accessibility_tree`.

### CONS-100 The Panel opens with Play and Rewind Cells

The Panel's row opens with two Cells, `P` and then `R`, one Cell apart and before **B**. Each is one
Source Cell on the window's Cell columns. Both draw in the Theme's `button` role, and `P` draws in
`button.pressed` while Playback plays. `P` does what Space does, and `R` rewinds. On native,
Command Shift R (Ctrl outside macOS) also rewinds, and a held chord rewinds once. The web binds no
Rewind chord, and there is no Playback menu. Every console shows the Panel, the Small Console
included.

Reason: a phone has no Space key, and the browser reserves Command Shift R. A Cell is as easy to tap as to type into.
Sources: [ADR 0089](../adr/0089-the-panel-plays-and-rewinds.md), [ADR 0076](../adr/0076-stopping-playback-holds-its-tick.md), [ADR 0086](../adr/0086-a-small-console-takes-its-source-by-message.md).
Evidence: test `console/src/console/kittest_tests.rs` `play_and_rewind_are_source_cells_one_cell_apart`.

### CONS-101 One Playback status routes every failure

One console Playback status takes each frame's drained diagnostics and the Playback observation, and
alone decides where each failure belongs. MIDI selection keeps only its own messages: a refused
discovery, a refused connect, and the browser's "Opening …" until a destination is installed. The
network setting keeps only the value it refused. An Overrun or an Omitted summary is no failure and
raises no message. A MIDI output failure stands until the observation reports a delivery succeeded
or a destination was installed.

Reason: one place decides where a failure belongs, so an engine failure never reads as a MIDI failure.
Sources: [ADR 0092](../adr/0092-the-panel-routes-playback-failures-in-one-place.md), [ADR 0055](../adr/0055-playback-diagnostics-are-retained-within-a-bound.md), [ADR 0059](../adr/0059-the-browser-answers-midi-discovery-from-access-it-already-holds.md).
Evidence: `console/src/playback_status.rs`; test `each_output_failure_goes_to_its_own_destination` in the same file.

### CONS-102 An engine failure stands until the next run begins

Clock, start and retune failures are engine failures, with a status of their own. They are events:
the latest stands until the next run begins, and a newer one replaces it. A Rewind while playing
begins a new run.

Reason: a clock failure is not a MIDI fault.
Sources: [ADR 0092](../adr/0092-the-panel-routes-playback-failures-in-one-place.md).
Evidence: test `console/src/playback_status.rs` `an_engine_failure_is_shown_until_the_next_run_begins`.

### CONS-103 No build branches on whether it has MIDI

The status routing is the same in every build. Network failures are shown and also reported to the
developer console. MIDI and engine failures are shown.

Reason: a build without a MIDI backend installs no connection, so it can have no MIDI failure to route differently.
Sources: [ADR 0092](../adr/0092-the-panel-routes-playback-failures-in-one-place.md).
Evidence: `console/src/playback_status.rs` (`report::error!` for network failures).

### CONS-104 The Output list has one labelled section per output type

The Panel's **O** opens the Output list. Its **MIDI** section holds Scan and the destinations, and
its **UDP/OSC** section holds the host and the two ports, with a separator between them. A section
the build cannot use is not shown: the web has no UDP/OSC section, and a build without a MIDI
backend has no MIDI section.

Reason: **O** is Output, not MIDI.
Sources: [ADR 0095](../adr/0095-the-output-list-shows-every-output-type.md).
Evidence: `console/src/console/panel.rs`; test `console/src/console/kittest_tests/panel_readouts.rs` `the_output_list_has_a_section_for_each_output_type`.

### CONS-105 The closed Output field names every live output

The closed field lists the selected MIDI destination, then the network host, joined by ` · `. With
nothing live it reads `None`. A failure never appears in the field.

Reason: the field shows what is live, and failures have Toasts.
Sources: [ADR 0095](../adr/0095-the-output-list-shows-every-output-type.md).
Evidence: test `console/src/console/kittest_tests/panel_readouts.rs` `the_output_list_has_a_section_for_each_output_type`.

### CONS-106 Each standing failure is a Toast

Each standing message floats as its own Toast in the window's top-right corner, beneath the menu bar
and clear of the Minimap. The standing messages are:

- The MIDI selection.
- A MIDI output failure.
- A refused UDP or OSC host or port.
- A failed UDP or OSC send.
- An engine failure.

A Toast names its output (`MIDI`, `UDP/OSC` or `ENGINE`), carries the message and a dismiss
button, and fades after `[toasts] seconds`. A dismissed Toast returns only after its reporter has
cleared and failed again. Toasts are `egui::Area`s, with no added dependency.

Reason: a status line that grows and shrinks with its messages would move the Source's bottom edge.
Sources: [ADR 0100](../adr/0100-failures-are-toasts-and-the-panel-keeps-a-line-for-the-cursor.md).
Evidence: `console/src/console/toasts.rs`; test `console/src/console/kittest_tests/toasts.rs` `a_failure_floats_as_a_toast_in_the_top_right_corner`.

### CONS-107 O shows a failure that outlives its Toast

While any message stands, the Panel draws its **O** Readout in the error colour. The Panel gains no
line for it.

Reason: a faded or dismissed Toast must never leave a broken output unsignalled.
Sources: [ADR 0100](../adr/0100-failures-are-toasts-and-the-panel-keeps-a-line-for-the-cursor.md).
Evidence: test `console/src/console/kittest_tests/toasts.rs` `a_toast_fades_after_its_seconds_and_o_keeps_the_failure`.

### CONS-108 Info describes what the Cursor's Cell belongs to

Info follows the Cursor, never the pointer. It has two lines:

- The first shows the Cells of each part the Cursor's Cell belongs to, painted as the Source paints
  them, with the Cursor's outline on its Cell.
- The second shows, beneath each part, the Function's name, a Role, or what its answer is, or the
  diagnostic that covers the Cell.

Each entry is as wide as the wider of its two lines, and the next entry starts two Cells past it on
a Cell column. The answer's Position is not shown.

Reason: editing is driven by the keyboard and the Cursor, and painting the Cells ties Info to the Cursor.
Not: a floating tooltip.
Sources: [ADR 0100](../adr/0100-failures-are-toasts-and-the-panel-keeps-a-line-for-the-cursor.md) (amendment 2026-11-12).
Evidence: `console/src/info.rs`; test `console/src/console/kittest_tests/info.rs` `info_describes_the_function_under_the_cursor`.

### CONS-109 The Panel is laid on Cell rows, and Info keeps its rows while shown

The Panel is laid on the Cell the Source presents:

1. A Cell row of room beneath its top edge.
2. The Readouts.
3. While Info is shown, a Cell row of room, Info's Cells and its labels.
4. One Cell row before the window's edge.

Info is a View toggle, shown by default, and bound to ⌘I on native, as Diagnostics is to ⌘D.
Shown, Info keeps its rows whether or not it has anything to say. Hidden, it gives its three Cell
rows back to the Source. The Panel's height never depends on the messages. The Diagnostics
Readouts add their own height when opened.

Reason: the Source's bottom edge moves only when the viewer chooses.
Sources: [ADR 0100](../adr/0100-failures-are-toasts-and-the-panel-keeps-a-line-for-the-cursor.md) (amendment 2026-11-12).
Evidence: test `console/src/console/kittest_tests/info.rs` `the_panel_lays_its_rows_on_the_grids_cells`.

## Touch

### CONS-110 A finger Pans, taps and long-presses

A drag that a finger starts Pans by what the finger moved, under the same clamp as every Pan, and
leaves the Cursor and the Region alone. A tap selects the Cell under it. A finger held still for
egui's long press (`InputOptions::max_click_duration`) selects a Region from the Cell it pressed to
the Cell nearest the finger, for as long as it stays down. A mouse and a pen keep the gestures of
[CONS-032](#cons-032-the-pan-gestures-none-of-which-moves-the-cursor-or-the-region) and [CONS-051](#cons-051-the-pointer-selects-a-region).

Reason: a phone's viewer tries one finger first.
Not: two fingers to Pan with one finger selecting.
Sources: [ADR 0102](../adr/0102-a-finger-pans-the-source-and-asks-for-the-keyboard.md).
Evidence: `console/src/console/source_view.rs` `touch_pressed`; test `console/src/console/tests.rs` `a_finger_drag_pans_the_source_and_leaves_the_region`.

### CONS-111 A finger's press asks for the on-screen keyboard

While the latest primary press was a finger's and the Source holds the keys, each frame reports the
Cursor's Cell as where text is edited, so the phone opens its keyboard. A mouse press withdraws the
request, so the desktop's canvas keeps the keys. A control that holds the keys reports its own
position, or none.

Reason: asking on every platform would route desktop keys through eframe's text input.
Sources: [ADR 0102](../adr/0102-a-finger-pans-the-source-and-asks-for-the-keyboard.md).
Evidence: `console/src/console/input.rs` `ask_for_touch_keyboard`; test `console/src/console/tests.rs` `a_finger_tap_asks_for_the_keyboard_and_a_mouse_click_does_not`.

### CONS-112 A phone keyboard's text and IME events are Source input

Each character of committed text is one Cell's text. A `DeleteSurrounding` of n is n Backspaces,
and a composition still in progress writes nothing. eframe's text input has autocorrect,
autocomplete and spellcheck off. The page asks the browser to shrink the page for the keyboard
(`interactive-widget=resizes-content`).

Reason: a keyboard must not rewrite Cells already written.
Sources: [ADR 0102](../adr/0102-a-finger-pans-the-source-and-asks-for-the-keyboard.md).
Evidence: `console/src/console/input.rs`, `console/src/web_startup.rs`, `console/index.html`; test `console/src/console/tests.rs` `a_phone_keyboard_writes_a_cell_per_character_and_deletes_by_backspace`.

## Accepted but not built, or built differently

- CONS-011: `orcvs` still carries presentation configuration with no reader:
  `orcvs/src/opts.rs` `CursorBloomRadius` and `RenderFrame::cursor_bloom_radius`. No tracking
  issue found.
- CONS-122: the Theme key `cell.selection.border.width` is still accepted and does nothing,
  although a Cell's border width no longer hides anything. Tracked in
  [cell-edges/07](../../.scratch/cell-edges/issues/07-retire-the-cell-selection-border-width.md).

## Open decisions

None of these is binding.

- The native document model and per-key settings fallback. `.scratch/menu-structure/spec.md`
  (undated, with no recorded maintainer ruling) proposes the following, and the code does it
  (`console/src/console/files.rs`, `console/src/config.rs`):
  - The window title names the open Source File and marks unsaved changes.
  - New, Open, the Function reference, an example, Quit and closing the window ask before
    discarding unsaved changes.
  - Autosave is session recovery, and Save writes the file. The web has no Open or Save.
  - A malformed file, an unknown key, or a mistyped or out-of-range value raises a notice, and
    only the affected keys take their defaults.

  Recording these as binding needs an ADR or a dated ruling (run decision D3).

- Web Theme import. ADR 0053 keeps "Web Themes are imported as files" as the design for when
  import returns, and v1 disables it ([CONS-077](#cons-077-the-web-console-runs-on-the-built-in-themes)).
  Tracked in `.scratch/menu-structure/`.
- Source undo with Command Z and Command Shift Z. Proposed in
  [source-undo](../../.scratch/source-undo/spec.md) (needs-triage). Until it is decided, Orcvs has
  no undo, which [CONS-053](#cons-053-a-region-is-written-as-a-block) relies on.
- ADR 0053's Status line does not name its 2026-10-31 amendment, which ends the follow-the-OS
  choice once the toggle is pressed. The snapshot follows the amendment. A Status-line edit is
  for the ADR process (run decision D7).
