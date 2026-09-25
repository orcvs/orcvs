//! The Source View: the Zoom and Pan the Source is presented under, how a
//! Cursor move or a Region drag moves them, and what the pointer asks of the
//! Region each Render Frame.

use egui::{Color32, CursorIcon, PointerButton, Pos2, Rect, Sense, Vec2, emath::TSTransform};
use orcvs::{
    app::Orcvs,
    grid::{Grid, Position},
    render_frame::RenderFrame,
};

use super::Console;
use super::glyphs::GLYPH_SCALE_STEP;
use super::input::{ZoomCommand, zoom_command};
use super::shapes::{effect_outline, show_source};
use crate::cursor_effects::{CursorEffectMotion, effect_bounds};
use crate::grid_viewport::{CELL_SIZE, GridViewport, presented_grid, snapped_cell_side};
use crate::theme::{Appearance, Theme};

pub(super) const MIN_ZOOM: f32 = 0.25;
pub(super) const MAX_ZOOM: f32 = 2.0;
///
/// How far past the console's edge, in points, a Region drag's pointer has to
/// be for each point the Source View scrolls a frame after it. A pointer four
/// Cells out scrolls one Cell a frame, which is the most a frame scrolls.
///
const EDGE_SCROLL_REACH: f32 = 4.0;

///
/// The margin, in Cells, between the Grid and the console at rest and the
/// distance a Pan can reach past each Grid edge (ADR 0047). It is counted in
/// Cells so it scales with the Zoom, and it is measured in the snapped Cell
/// side so it is always a whole number of physical pixels.
///
pub(super) const SOURCE_MARGIN_CELLS: f32 = 2.0;

///
/// `zoom` after one keyboard Zoom command: stepped by [`GLYPH_SCALE_STEP`] and
/// clamped to [`MIN_ZOOM`]..=[`MAX_ZOOM`].
///
/// Stepped from the nearest multiple of the step rather than by adding it, so
/// a long session stays exactly on the grid [`glyph_scale`](super::glyphs::glyph_scale) quantises to
/// instead of drifting off it through repeated float addition. `Reset`
/// answers 1.0 outright, whatever step `zoom` was on.
///
pub(super) fn stepped_zoom(zoom: f32, command: ZoomCommand) -> f32 {
    if command == ZoomCommand::Reset {
        return 1.0;
    }
    let direction = if command == ZoomCommand::In {
        1.0
    } else {
        -1.0
    };
    let steps = (zoom / GLYPH_SCALE_STEP).round() + direction;
    (steps * GLYPH_SCALE_STEP).clamp(MIN_ZOOM, MAX_ZOOM)
}

pub(super) fn source_bounds(grid: Grid) -> Rect {
    Rect::from_min_size(
        Pos2::ZERO,
        Vec2::new(grid.columns() as f32, grid.rows() as f32) * CELL_SIZE,
    )
}

///
/// The Source View: a Zoom and a Pan, presented as the scale and translation
/// the Cells are drawn under.
///
/// Zoom opens at 1.0 — the Source's own Cell — and is a stated step, never a
/// property of the window. Pan is anchored at the console's top-left and is
/// bounded by the Grid: an axis the whole Source already fills has nowhere to
/// Pan, and a Pan that would open a gap past an edge settles back inside.
///
/// A Cursor move or a Zoom that would leave the Cursor's Cell outside the
/// console Pans the least distance that brings the whole Cell back into view,
/// still bounded by the Grid; a Pan on its own does not chase the Cursor.
/// `previous_cursor` is what tells a Cursor move apart from a frame that
/// merely redrew it — see `docs/adr/0045-the-source-view-is-a-bounded-space.md`.
///
/// `to_global` is derived each frame from Zoom, Pan and the console's origin
/// so `presented_grid` and the diagnostics still read one transform.
///
pub(super) struct SourceView {
    pub(super) zoom: f32,
    pub(super) pan: Vec2,
    /// The Cursor [`show_source_scene`] last saw, so a change from one frame
    /// to the next reads as a Cursor move worth following rather than every
    /// frame answering yes. `None` before the first frame a fresh `SourceView`
    /// presents, so it does not Pan away from wherever the console opened
    /// merely because there was nothing yet to compare the Cursor against.
    previous_cursor: Option<Position>,
    /// The anchor of the primary drag selecting a Region, while one is in
    /// progress. The Cursor follow is paced to the pointer while it is.
    region_drag: Option<Position>,
    /// A Zoom command the View menu asked for this frame, applied by the
    /// next [`show_source_scene`] exactly as the chord it names would be.
    pub(super) requested_zoom: Option<ZoomCommand>,
    pub(super) to_global: TSTransform,
}

impl Default for SourceView {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
            previous_cursor: None,
            region_drag: None,
            requested_zoom: None,
            to_global: TSTransform::IDENTITY,
        }
    }
}

///
/// The Pan that keeps the Source inside the console: top-left when the Source
/// is smaller on an axis, and between the two edges when it is larger.
/// [`show_source_scene`] passes the padded Source — the Grid and its margin on
/// both sides — so the edges here are the margin's, not the Grid's.
///
pub(super) fn clamp_pan(pan: Vec2, console: Vec2, source: Vec2) -> Vec2 {
    Vec2::new(
        clamp_pan_axis(pan.x, console.x, source.x),
        clamp_pan_axis(pan.y, console.y, source.y),
    )
}

fn clamp_pan_axis(pan: f32, console: f32, source: f32) -> f32 {
    let slack = console - source;
    if !slack.is_finite() || slack >= 0.0 {
        0.0
    } else {
        pan.clamp(slack, 0.0)
    }
}

///
/// The rectangle a Cursor follow brings into view, in the unpanned points of
/// the padded Source — the Grid with a margin of [`SOURCE_MARGIN_CELLS`] on
/// every side — at the snapped Cell `side` [`show_source_scene`] also bounds
/// [`clamp_pan`] by.
///
/// The Cursor's own Cell, reaching across the margin on each side where the
/// Cursor is on the Grid's first or last Column or Row, so a follow to an edge
/// shows the Grid's edge as an edge rather than flush against the console's
/// (ADR 0047).
///
fn followed_cell(cursor: Position, grid: Grid, side: f32) -> Rect {
    let margin = SOURCE_MARGIN_CELLS * side;
    let cell = Rect::from_min_size(
        Pos2::new(cursor.x() as f32, cursor.y() as f32) * side + Vec2::splat(margin),
        Vec2::splat(side),
    );
    let reach = |at: usize, count: usize| {
        let near = if at == 0 { margin } else { 0.0 };
        let far = if at + 1 == count { margin } else { 0.0 };
        (near, far)
    };
    let (left, right) = reach(cursor.x(), grid.columns());
    let (top, bottom) = reach(cursor.y(), grid.rows());
    Rect::from_min_max(
        cell.min - Vec2::new(left, top),
        cell.max + Vec2::new(right, bottom),
    )
}

///
/// The Pan that brings `cell` — already in the same units as `pan` once
/// translated by it — fully inside a console of `console_size`, moving the
/// least distance along each axis and leaving an axis alone where the Cell
/// already shows in full.
///
/// Not itself bounded by the Grid: [`clamp_pan`] runs after this wherever it
/// is called, so a Cell nearer an edge than the console is wide settles
/// against that edge rather than opening a gap past it.
///
fn follow_cursor(pan: Vec2, console_size: Vec2, cell: Rect) -> Vec2 {
    let shown = cell.translate(pan);
    Vec2::new(
        follow_axis(pan.x, shown.min.x, shown.max.x, console_size.x),
        follow_axis(pan.y, shown.min.y, shown.max.y, console_size.y),
    )
}

///
/// One axis of [`follow_cursor`]: shift `pan` by exactly the overflow past
/// whichever edge the Cell has fallen outside, or leave it be when the Cell
/// already sits between the two.
///
fn follow_axis(pan: f32, min: f32, max: f32, console: f32) -> f32 {
    if min < 0.0 {
        pan - min
    } else if max > console {
        pan - (max - console)
    } else {
        pan
    }
}

///
/// How far `pointer` is past each edge of `console`, and zero on an axis where
/// it is between the two.
///
fn overshoot(console: Rect, pointer: Pos2) -> Vec2 {
    Vec2::new(
        (console.min.x - pointer.x)
            .max(pointer.x - console.max.x)
            .max(0.0),
        (console.min.y - pointer.y)
            .max(pointer.y - console.max.y)
            .max(0.0),
    )
}

///
/// The part of a follow Pan of `wanted` that a Region drag takes this frame:
/// more the further the pointer is `past` the console's edge, never more than one
/// Cell of `side`, and none on an axis the pointer has not left.
///
fn edge_scroll(wanted: Vec2, past: Vec2, side: f32) -> Vec2 {
    let most = (past / EDGE_SCROLL_REACH).min(Vec2::splat(side));
    Vec2::new(
        wanted.x.clamp(-most.x, most.x),
        wanted.y.clamp(-most.y, most.y),
    )
}

///
/// Whether `to_global` can be presented and inverted.
///
/// `egui::Scene::show` resets a transform that has gone bad
/// (`egui-0.36.2/src/containers/scene.rs:151-152, 168-173`), and the Source is
/// presented without that container, so nothing resets it here. `grid_viewport` answers a Cell
/// size of zero for a console with no area, so the fit it yields has a scaling
/// of zero, and `TSTransform::inverse` divides by the scaling — which
/// `Scene::register_pan_and_zoom` does on every frame the pointer is over the
/// console. An unguarded zero therefore resolves every pointer position to NaN.
///
/// `TSTransform::is_valid` is not enough on its own: it checks only
/// `translation.x` (`emath-0.36.2/src/ts_transform.rs:55-57`) and admits a
/// negative scaling, which would present the Source mirrored.
///
pub(super) fn is_presentable(to_global: TSTransform) -> bool {
    to_global.scaling.is_finite() && to_global.scaling > 0.0 && to_global.translation.is_finite()
}

///
/// What the pointer asked of the Region in one Render Frame.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum PointerSelection {
    /// A click: the Region collapses onto the Cell.
    Select(Position),
    /// Shift with a click: the Cursor moves to the Cell and the anchor stays.
    Extend(Position),
    /// A primary drag: the Region spans from the pressed Cell to the Cell
    /// nearest the pointer.
    Span { anchor: Position, cursor: Position },
}

impl PointerSelection {
    ///
    /// Asks `orcvs` for the Region this selection names. Where the Cursor and
    /// the anchor are is the running Orcvs's business; the pointer only
    /// answers which Cells it asked for.
    ///
    pub(super) fn apply(self, orcvs: &mut Orcvs) {
        match self {
            Self::Select(position) => orcvs.select(position),
            Self::Extend(position) => orcvs.extend(position),
            Self::Span { anchor, cursor } => {
                orcvs.select(anchor);
                orcvs.extend(cursor);
            }
        }
    }
}

///
/// The Source as it was presented for one Render Frame: the geometry it was
/// drawn under, and the Cell a click asked for.
///
/// "Presented" is already the repository's word for this step —
/// `grid_viewport::presented_grid` uses it.
///
pub(super) struct PresentedSource {
    /// The viewport the Cells were drawn at.
    pub(super) viewport: GridViewport,
    /// The Region the viewer's click or drag asked for, for the caller that
    /// owns the Source to select.
    pub(super) selection: Option<PointerSelection>,
}

///
/// Shows the Source in the console area at the Source View's Zoom and Pan, and
/// answers the geometry it was presented under along with the Region a click
/// or a drag asked for.
///
/// The console owns the scale and translation the Source is presented under —
/// `view.to_global` — and `grid_viewport::presented_grid` is the one place that
/// scale is applied, so a Cell's two axes still cannot part company: one
/// `scaling` serves both. Every Cell, and so every click that lands on one,
/// goes through that one arithmetic. Nothing here sets a layer transform.
/// See `docs/adr/0038-the-console-owns-the-source-grid-transform.md` and
/// `docs/adr/0045-the-source-view-is-a-bounded-space.md`.
///
/// Pan is by wheel or two-finger scroll, by middle-drag, and by Alt (Option)
/// held with a primary drag, bounded by the Grid's edges plus a margin of
/// [`SOURCE_MARGIN_CELLS`] (ADR 0047). Where the Grid and its margins leave
/// somewhere to Pan, the pointer shows a grab hand while Alt is held over the
/// console outside a Region drag, and a grabbing hand during a drag Pan. A primary
/// click selects a Cell — [`show_source`]'s own click-sensing rect answers it,
/// with Shift extending the Region rather than collapsing it — and a primary
/// drag without Alt selects a Region from the pressed Cell to the Cell nearest
/// the pointer; neither Pans. A drag past the console's edge scrolls after the
/// Cursor at the pointer's pace, at most one Cell a frame — see
/// `docs/adr/0046-the-primary-drag-selects-a-region.md`. Pinch and command-wheel do not
/// Zoom: Zoom is a command `=`, `+`, `-` or `0` chord from the keyboard
/// alone, stepped by [`GLYPH_SCALE_STEP`] and clamped to
/// [`MIN_ZOOM`]..=[`MAX_ZOOM`]. A Zoom that would open a gap past an edge
/// settles back inside through the same `clamp_pan` a Pan does.
///
/// A Cursor move or a Zoom that would leave the Cursor's Cell outside the
/// console Pans just far enough to bring it back, before that same
/// `clamp_pan` settles the result inside the Grid; a Pan with neither is not
/// pulled back to the Cursor. `frame` already carries a keyboard Cursor move
/// from this same Render Frame — `Console::ui` reads it after
/// `Orcvs::event_handler` runs — so that case is caught the frame it happens.
/// A click's or a drag's Cursor move reaches the Source only after this call
/// returns (`Console::show_source_panel` applies the [`PointerSelection`]
/// next), so it is followed on the frame after, not this one.
///
pub(super) fn show_source_scene(
    ui: &mut egui::Ui,
    frame: &RenderFrame,
    font_family: &egui::FontFamily,
    view: &mut SourceView,
    cursor_effect: CursorEffectMotion,
    theme: &Theme,
) -> PresentedSource {
    let source_grid = frame.grid();
    let source = source_bounds(source_grid);
    // `Sense::CLICK | Sense::DRAG` rather than `Sense::click_and_drag()`,
    // which adds `FOCUSABLE` (`egui-0.36.2/src/sense.rs:81-83`) and would let
    // Tab focus the console area, where a focused widget keeps every key from
    // the Source.
    let (console, mut pan) =
        ui.allocate_exact_size(ui.available_size_before_wrap(), Sense::CLICK | Sense::DRAG);

    if !view.zoom.is_finite() || view.zoom <= 0.0 {
        view.zoom = 1.0;
    }
    view.zoom = view.zoom.clamp(MIN_ZOOM, MAX_ZOOM);

    let zoom_before_command = view.zoom;
    // A View menu item and a chord are the same command: the menu's is
    // taken first, since a click that closed the menu carries no chord.
    let command = view
        .requested_zoom
        .take()
        .or_else(|| ui.input(|i| i.events.iter().find_map(zoom_command)));
    if let Some(command) = command {
        view.zoom = stepped_zoom(view.zoom, command);
    }
    let zoomed = view.zoom != zoom_before_command;

    // Middle-drag Pans outright; a primary drag Pans only with Alt (Option)
    // held, so a trackpad with no middle button still has a way to Pan by
    // dragging. Without Alt this branch is skipped: a primary gesture that
    // stayed inside the click threshold resolves to `show_source`'s own click
    // on release, and one that moved past it selects a Region below.
    //
    // A primary drag that began selecting a Region stays one if Alt is
    // pressed partway through it.
    let alt = ui.input(|i| i.modifiers.alt);
    let drag_panning = pan.dragged_by(PointerButton::Middle)
        || (pan.dragged_by(PointerButton::Primary) && view.region_drag.is_none() && alt);
    if drag_panning {
        view.pan += pan.drag_delta();
        pan.mark_changed();
    }

    if pan.contains_pointer() {
        let pan_delta = ui.input(|i| i.smooth_scroll_delta());
        if pan_delta != Vec2::ZERO {
            view.pan += pan_delta;
            pan.mark_changed();
        }
    }

    // A Cursor move is a change from the Cursor `previous_cursor` last saw,
    // not every frame the Cursor happens to be drawn — otherwise an ordinary
    // Pan with the Cursor already out of view would be pulled straight back
    // to it. `None` on a fresh `SourceView`'s first frame answers no move, so
    // the console does not Pan away from where it opened before anything has
    // moved the Cursor at all.
    let cursor = frame.cursor();
    let cursor_moved = view
        .previous_cursor
        .is_some_and(|previous| previous != cursor);
    view.previous_cursor = Some(cursor);

    // The Cell side `presented_grid` will draw at, snapped to whole physical
    // pixels, so the follow and the bounds below are measured against the
    // Grid as drawn rather than the unsnapped extent the Zoom asked for.
    let side = snapped_cell_side(CELL_SIZE * view.zoom, ui.ctx().pixels_per_point());

    // `view.pan` places the padded Source — the Grid with a margin on every
    // side — so a Pan of zero rests the Grid one margin in from the console's
    // top-left, and the clamp lets a Pan reach one margin past each far edge.
    let margin = Vec2::splat(SOURCE_MARGIN_CELLS * side);
    let cursor_at = followed_cell(cursor, source_grid, side);

    // A Region drag follows the Cursor at the pointer's pace rather than in
    // one jump, so the Source View scrolls after a pointer held past the edge
    // — the Cursor is the Cell nearest the pointer, one past the edge — and
    // keeps scrolling each frame the pointer stays there, which is why this
    // frame asks for the next.
    let pointer = ui.input(|i| i.pointer.interact_pos());
    // Still paced on the frame the button comes up — `region_drag` is cleared
    // only once this frame has answered it — so a release with the Cursor
    // many Cells past the edge does not jump the Source View to it.
    let dragging_region = view.region_drag.is_some();
    if zoomed || (cursor_moved && !dragging_region) {
        view.pan = follow_cursor(view.pan, console.size(), cursor_at);
    } else if dragging_region && let Some(pointer) = pointer {
        let past = overshoot(console, pointer);
        let wanted = follow_cursor(view.pan, console.size(), cursor_at) - view.pan;
        view.pan += edge_scroll(wanted, past, side);
        if past != Vec2::ZERO {
            ui.ctx().request_repaint();
        }
    }

    let source_size = Vec2::new(source_grid.columns() as f32, source_grid.rows() as f32) * side;
    let padded_size = source_size + 2.0 * margin;
    view.pan = clamp_pan(view.pan, console.size(), padded_size);

    // The pointer offers a Pan only where there is one: not on a Grid that
    // with its margins fits the console on both axes, and not for Alt
    // pressed partway through a Region drag, which stays one. Alt is the one
    // Pan gesture the pointer can announce before it starts: a middle-drag
    // has no hover state to show, and a wheel Pan is not a grab.
    let pannable = padded_size.x > console.width() || padded_size.y > console.height();
    if pannable && drag_panning {
        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
    } else if pannable && alt && view.region_drag.is_none() && pan.contains_pointer() {
        ui.ctx().set_cursor_icon(CursorIcon::Grab);
    }

    let to_global = TSTransform::new(console.min.to_vec2() + view.pan + margin, view.zoom);
    view.to_global = if is_presentable(to_global) {
        to_global
    } else {
        TSTransform::IDENTITY
    };

    let grid = presented_grid(
        view.to_global,
        source,
        source_grid,
        ui.ctx().pixels_per_point(),
    );
    let clicked = show_source(ui, frame, font_family, grid, console, cursor_effect, theme);

    // A primary drag without Alt selects a Region: its anchor is the Cell the
    // press landed on, and a press off the Grid selects nothing. The drag is
    // decided once the pointer has moved past egui's click distance, so the
    // anchor is read from where the press began rather than where the
    // pointer is now.
    if pan.drag_started_by(PointerButton::Primary) && !ui.input(|i| i.modifiers.alt) {
        view.region_drag = ui
            .input(|i| i.pointer.press_origin())
            .and_then(|origin| grid.cell_at(origin, source_grid))
            .and_then(|(column, row)| source_grid.position(column, row));
    }
    let spanned = view
        .region_drag
        .filter(|_| pan.dragged_by(PointerButton::Primary))
        .zip(
            pointer
                .and_then(|pointer| grid.nearest_cell(pointer, source_grid))
                .and_then(|(column, row)| source_grid.position(column, row)),
        )
        .map(|(anchor, cursor)| PointerSelection::Span { anchor, cursor });
    // Answered on every frame of the drag, not only when the pointer moves:
    // a pointer held past the edge moves no further while the Source View
    // scrolls under it, so the Cell nearest it changes without an event.
    // Asking for the same Region again is idempotent, so a repeated frame
    // changes nothing.
    //
    // Release keeps the Region the drag last spanned, and spans no further:
    // a Cursor moved on the release frame would be followed in full on the
    // next, once the drag no longer paces it.
    if !pan.dragged_by(PointerButton::Primary) {
        view.region_drag = None;
    }

    PresentedSource {
        viewport: grid,
        selection: spanned.or(clicked),
    }
}

///
/// The frame the Source Grid is painted on.
///
/// The fill is load-bearing rather than decorative. `cell_visuals` answers
/// `None` for a Cell's background wherever the panel has already painted
/// `background`, on the grounds that this frame has already painted exactly
/// that colour across the whole console and clips every Shape to it. An ordinary
/// Cell therefore has no rectangle of its own.
///
/// `background` is the resolved Theme's live `grid_background`, not a
/// constant: a loaded Theme has to repaint this panel on the very next frame
/// for the Cell it stands in for to still agree with it
/// (`.scratch/theming/issues/06`/`07`).
///
/// It is a function rather than a literal at the panel so the painting tests
/// render on the same ground production does, and so
/// `the_omitted_background_is_the_colour_the_panel_is_filled_with` has one
/// value to pin instead of a comment to trust.
///
pub(super) fn source_panel_frame(background: Color32) -> egui::Frame {
    egui::Frame::new().fill(background)
}

///
/// What the Source panel presented in one Render Frame, for the frame's
/// repaint schedule and the Diagnostics window.
///
pub(super) struct ShownSource {
    /// The console area the Source was presented in.
    pub(super) console: Rect,
    /// The Cell side the Source was drawn at.
    pub(super) cell_size: f32,
    /// Whether any of the Cursor Effect reaches the console area, so that its
    /// next change needs a Render Frame.
    pub(super) shows_cursor_effect: bool,
}

impl Console {
    ///
    /// Shows `frame` in the central panel at the Source View's Zoom and Pan,
    /// in the Theme `appearance` presents, and selects the Region the pointer
    /// asked for.
    ///
    /// The Theme is borrowed from the selection for the panel; the panel reads
    /// the font, the Source View and the running Orcvs as fields beside it.
    ///
    pub(super) fn show_source_panel(
        &mut self,
        root: &mut egui::Ui,
        frame: &RenderFrame,
        appearance: Appearance,
        cursor_effect: CursorEffectMotion,
    ) -> ShownSource {
        let theme = self.themes.presented(appearance);
        egui::CentralPanel::default()
            .frame(source_panel_frame(theme.grid_background))
            .show(root, |ui| {
                let console = ui.available_rect_before_wrap();
                let presented = show_source_scene(
                    ui,
                    frame,
                    &self.font_family,
                    &mut self.source_view,
                    cursor_effect,
                    theme,
                );
                // The Source Grid answers which Cells the pointer asked for;
                // moving the Cursor and the anchor there is the Source's own
                // business, and this is where the running Orcvs is owned.
                if let Some(selection) = presented.selection {
                    selection.apply(&mut self.orcvs);
                }

                let viewport = presented.viewport;
                let cursor_rect = viewport.cell_rect(frame.cursor().x(), frame.cursor().y());
                let outline = effect_outline(frame, &viewport);
                ShownSource {
                    console,
                    cell_size: viewport.cell_size,
                    shows_cursor_effect: effect_bounds(cursor_rect, viewport.cell_size)
                        .union(outline.expand(viewport.cell_size))
                        .intersects(console),
                }
            })
            .inner
    }
}
