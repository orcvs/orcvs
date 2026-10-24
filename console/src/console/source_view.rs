//! The Source View: the Pan the Source is presented under, how a Cursor move,
//! a Region drag or a change of the console's size in points moves it, and
//! what the pointer asks of the Region each Render Frame.

use egui::{Color32, CursorIcon, PointerButton, Pos2, Rect, Sense, Vec2};
use orcvs::{
    app::Orcvs,
    grid::{Grid, Position},
    render_frame::RenderFrame,
};

use super::Console;
use super::shapes::show_source;
use crate::grid_viewport::{
    GridViewport, ScreenMap, SourceCellPoint, SourceCellRect, presented_grid, snapped_cell,
};
use crate::theme::{Appearance, Theme};

///
/// How far past the console's edge, in points, a Region drag's pointer has to
/// be for each point the Source View scrolls a frame after it. A pointer four
/// Cells out scrolls one Cell a frame, which is the most a frame scrolls.
///
const EDGE_SCROLL_REACH: f32 = 4.0;

///
/// The margin, in Cells, between the Grid and the console at rest and the
/// distance a Pan can reach past each Grid edge. It is counted in Cells and
/// measured in the snapped Cell's width across and its height down, so it is
/// always a whole number of physical pixels.
///
const SOURCE_MARGIN_CELLS: f32 = 2.0;

///
/// The Source View: a Pan, presented as the [`ShownSourceView`] the Cells are
/// drawn under.
///
/// The Source is drawn at its `cell` in points. egui's whole-UI zoom
/// is what enlarges it with the rest of the console, through
/// `pixels_per_point`, so the Source View holds no scale of its own. Pan is
/// anchored at the console's top-left and is bounded by the Grid: an axis the
/// whole Source already fills has nowhere to Pan, and a Pan that would open a
/// gap past an edge settles back inside.
///
/// A Cursor move that would leave the Cursor's Cell outside the console Pans
/// the least distance that brings the whole Cell back into view, still bounded
/// by the Grid; a Pan on its own does not chase the Cursor. A zoom or resize
/// that changes the console's size in points, or the snapped Cell, does
/// the same only for a Cursor the view showed in full before it, so a Pan the
/// viewer chose away from the Cursor survives it. `previous_cursor` and
/// `previous_console` are what tell those changes apart from a frame that
/// merely redrew the Cursor.
///
/// `shown` is derived each frame from the Pan, the margin and the console's
/// top-left, so Paint, the Overview and Diagnostics read one presentation.
///
pub(super) struct SourceView {
    pub(super) pan: Vec2,
    /// The Cell the Source is drawn at, width by height in points, before
    /// [`show_source_scene`] snaps it to whole physical pixels. It is the Cell
    /// the console started with and nothing changes it while the console runs,
    /// so opening another Source carries it into the new view.
    pub(super) cell: Vec2,
    /// The Cursor [`show_source_scene`] last saw, so a change from one frame
    /// to the next reads as a Cursor move worth following rather than every
    /// frame answering yes. `None` before the first frame a fresh `SourceView`
    /// presents, so it does not Pan away from wherever the console opened
    /// merely because there was nothing yet to compare the Cursor against.
    previous_cursor: Option<Position>,
    /// The anchor of the primary drag selecting a Region, while one is in
    /// progress. The Cursor follow is paced to the pointer while it is.
    region_drag: Option<Position>,
    /// The console's size in points and the snapped Cell on the last
    /// frame [`show_source_scene`] presented, so a change of either reads as
    /// a zoom or resize worth following, and so whether the Cursor showed
    /// under them can be asked. `None` before the first frame, as
    /// `previous_cursor` is.
    previous_console: Option<(Vec2, Vec2)>,
    /// How the Source View was presented on the last frame.
    shown: ShownSourceView,
    /// The place in Source Cells the next Render Frame centres the console
    /// on, when one has been asked for. Counted in Cells so
    /// [`show_source_scene`] measures it with the Cell it snaps that Render
    /// Frame, and bounds it by the same clamp as every Pan.
    requested_centre: Option<SourceCellPoint>,
}

impl SourceView {
    /// A view of a fresh Source, unpanned, drawn at `cell`.
    pub(super) fn new(cell: Vec2) -> Self {
        Self {
            pan: Vec2::ZERO,
            cell,
            previous_cursor: None,
            region_drag: None,
            previous_console: None,
            shown: ShownSourceView::none(),
            requested_centre: None,
        }
    }

    ///
    /// Asks the next Render Frame to Pan the console's centre onto `point`. A
    /// later request before that frame replaces this one. It is a Pan alone,
    /// so it moves no Cursor and is not pulled back to it.
    ///
    pub(super) fn centre_on(&mut self, point: SourceCellPoint) {
        self.requested_centre = Some(point);
    }

    ///
    /// How the Source View was presented on the last frame, or before the
    /// first a presentation with no Cell, which has nowhere to Pan.
    ///
    pub(super) fn shown(&self) -> ShownSourceView {
        self.shown
    }

    ///
    /// Settles this frame's Pan from `input`: the drag and wheel deltas, then
    /// a Cursor follow or a Region drag's edge scroll, then a requested
    /// centre, then the clamp, in that order. See [`show_source_scene`] for
    /// what each gesture does.
    ///
    pub(super) fn settle(&mut self, input: PanInput) -> Settled {
        let PanInput {
            middle_drag,
            primary_drag,
            alt,
            contains_pointer,
            scroll,
            pointer,
            console,
            cell,
            grid,
            cursor,
        } = input;

        // The console's size in points rather than egui's zoom factor: on the
        // web the browser's page zoom leaves that factor at 1.0, and eframe may
        // report the new device pixel ratio a frame before the canvas's new
        // size in points. egui's zoom, the browser's and a resize all change
        // the size in points, and a new scale can move the snapped Cell on its
        // own.
        //
        // Only a Cursor whose Cell showed in full under the previous size and
        // Cell, at the Pan the last frame settled, is followed: one the viewer
        // Panned out of view stays out. A web zoom that reaches egui over two
        // frames is followed on the second, because the first leaves the
        // Cursor showing.
        let console_size = console.size();
        let resize_to_follow = self.previous_console.is_some_and(|(size, previous_cell)| {
            let was_shown = Rect::from_min_size(Pos2::ZERO, size)
                .contains_rect(cursor_cell(cursor, previous_cell).translate(self.pan));
            (size, previous_cell) != (console_size, cell) && was_shown
        });
        self.previous_console = Some((console_size, cell));

        // Middle-drag Pans outright; a primary drag Pans only with Alt (Option)
        // held, so a trackpad with no middle button still has a way to Pan by
        // dragging. A primary drag that began selecting a Region stays one if
        // Alt is pressed partway through it.
        //
        // Still paced on the frame the button comes up: the caller clears
        // `region_drag` only once this frame has answered it, so a release
        // with the Cursor many Cells past the edge does not jump the Source
        // View to it.
        let dragging_region = self.region_drag.is_some();
        let drag = middle_drag.or(primary_drag.filter(|_| alt && !dragging_region));
        if let Some(delta) = drag {
            self.pan += delta;
        }
        if contains_pointer {
            self.pan += scroll;
        }

        // A Cursor move is a change from the Cursor `previous_cursor` last saw,
        // not every frame the Cursor happens to be drawn — otherwise an
        // ordinary Pan with the Cursor already out of view would be pulled
        // straight back to it. `None` on a fresh `SourceView`'s first frame
        // answers no move, so the console does not Pan away from where it
        // opened before anything has moved the Cursor at all.
        let cursor_moved = self
            .previous_cursor
            .is_some_and(|previous| previous != cursor);
        self.previous_cursor = Some(cursor);

        // `pan` places the extent — the Grid with a margin on every side — so
        // a Pan of zero rests the Grid one margin in from the console's
        // top-left, and the clamp lets a Pan reach one margin past each far
        // edge.
        let cursor_at = at_rest(cell).rect_to_screen(followed_cells(cursor, grid));

        // A Region drag follows the Cursor at the pointer's pace rather than in
        // one jump, so the Source View scrolls after a pointer held past the
        // edge — the Cursor is the Cell nearest the pointer, one past the edge
        // — and keeps scrolling each frame the pointer stays there, which is
        // why this frame asks for the next.
        let mut repaint = false;
        if resize_to_follow || (cursor_moved && !dragging_region) {
            self.pan = follow_cursor(self.pan, console_size, cursor_at);
        } else if dragging_region && let Some(pointer) = pointer {
            let past = overshoot(console, pointer);
            let wanted = follow_cursor(self.pan, console_size, cursor_at) - self.pan;
            self.pan += edge_scroll(wanted, past, cell);
            repaint = past != Vec2::ZERO;
        }

        // A requested centre places the extent outright, after any follow of
        // this frame, so the clamp below bounds it as it bounds every Pan.
        if let Some(centre) = self.requested_centre.take() {
            self.pan = console_size / 2.0 - at_rest(cell).to_screen(centre).to_vec2();
        }

        let extent = extent(grid).size() * cell;
        self.pan = clamp_pan(self.pan, console_size, extent);

        Settled {
            pan: self.pan,
            repaint,
            can_pan: pannable(console_size, extent),
            drag_panned: drag.is_some(),
        }
    }
}

///
/// What [`SourceView::settle`] reads of one Render Frame: raw input, the
/// console and the Source, with no Pan rule applied yet.
///
pub(super) struct PanInput {
    /// The drag delta while the middle button drags the console.
    pub(super) middle_drag: Option<Vec2>,
    /// The drag delta while the primary button drags the console.
    pub(super) primary_drag: Option<Vec2>,
    /// Whether Alt (Option) is held.
    pub(super) alt: bool,
    /// Whether the console contains the pointer.
    pub(super) contains_pointer: bool,
    /// egui's smoothed wheel or two-finger scroll delta.
    pub(super) scroll: Vec2,
    /// Where the pointer is, if anywhere.
    pub(super) pointer: Option<Pos2>,
    /// The console area the Source is presented in.
    pub(super) console: Rect,
    /// The Cell snapped to whole physical pixels.
    pub(super) cell: Vec2,
    pub(super) grid: Grid,
    pub(super) cursor: Position,
}

///
/// The Pan [`SourceView::settle`] settled on, and what the caller asks of
/// egui for it.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Settled {
    /// The Pan, inside the clamp.
    pub(super) pan: Vec2,
    /// Whether a Region drag's pointer is past the console's edge, so the
    /// next frame scrolls further.
    pub(super) repaint: bool,
    /// Whether the extent leaves anywhere to Pan.
    pub(super) can_pan: bool,
    /// Whether a drag Panned this frame.
    pub(super) drag_panned: bool,
}

///
/// The region of Source Cells the Source View can show: `grid` with a margin
/// of [`SOURCE_MARGIN_CELLS`] past each edge.
///
pub(super) fn extent(grid: Grid) -> SourceCellRect {
    let margin = SOURCE_MARGIN_CELLS;
    SourceCellRect::from_min_max(
        SourceCellPoint::new(-margin, -margin),
        SourceCellPoint::new(grid.columns() as f32 + margin, grid.rows() as f32 + margin),
    )
}

///
/// The height in points of one Row of `cell`-high Cells between the margins
/// the extent of `grid` has above and below it: the least a console shows of
/// the Source.
///
pub(super) fn one_row_between_margins(grid: Grid, cell: f32) -> f32 {
    (extent(grid).size().y - grid.rows() as f32 + 1.0) * cell
}

///
/// The Source View at a Pan of zero, in points from the console's top-left:
/// the extent's corner on the console's.
///
fn at_rest(cell: Vec2) -> ScreenMap {
    ScreenMap::new(Pos2::ZERO + SOURCE_MARGIN_CELLS * cell, cell)
}

///
/// How far a console `console` points long falls short of an extent `extent`
/// points long, on an axis where it does: the furthest a Pan reaches back
/// along it. `None` where the console shows the whole extent, so there is
/// nowhere to Pan.
///
/// The clamp and [`pannable`] both ask this, so the Overview outlines the
/// Source View exactly where it can Pan.
///
fn slack(console: f32, extent: f32) -> Option<f32> {
    let slack = console - extent;
    (slack.is_finite() && slack < 0.0).then_some(slack)
}

///
/// Whether a console of `console` points leaves anywhere to Pan an extent of
/// `extent` points over, on either axis.
///
fn pannable(console: Vec2, extent: Vec2) -> bool {
    slack(console.x, extent.x).is_some() || slack(console.y, extent.y).is_some()
}

///
/// The Pan that keeps the extent inside the console: top-left when the extent
/// of `extent` points is smaller on an axis, and between the two edges when it
/// is larger.
///
pub(super) fn clamp_pan(pan: Vec2, console: Vec2, extent: Vec2) -> Vec2 {
    let axis = |pan: f32, console: f32, extent: f32| {
        slack(console, extent).map_or(0.0, |slack| pan.clamp(slack, 0.0))
    };
    Vec2::new(
        axis(pan.x, console.x, extent.x),
        axis(pan.y, console.y, extent.y),
    )
}

///
/// The region a Cursor follow brings into view: the Cursor's own Cell,
/// reaching to the extent's edge on each side where the Cursor is on the
/// Grid's first or last Column or Row, so a follow to an edge shows the Grid's
/// edge as an edge rather than flush against the console's.
///
fn followed_cells(cursor: Position, grid: Grid) -> SourceCellRect {
    let extent = extent(grid);
    let reach = |at: usize, count: usize, start: f32, end: f32| {
        let near = if at == 0 { start } else { at as f32 };
        let far = if at + 1 == count {
            end
        } else {
            at as f32 + 1.0
        };
        (near, far)
    };
    let (left, right) = reach(
        cursor.x(),
        grid.columns(),
        extent.min().x(),
        extent.max().x(),
    );
    let (top, bottom) = reach(cursor.y(), grid.rows(), extent.min().y(), extent.max().y());
    SourceCellRect::from_min_max(
        SourceCellPoint::new(left, top),
        SourceCellPoint::new(right, bottom),
    )
}

///
/// The Cursor's own Cell at a Pan of zero, at the snapped `cell`
/// [`show_source_scene`] also bounds [`clamp_pan`] by.
///
fn cursor_cell(cursor: Position, cell: Vec2) -> Rect {
    at_rest(cell).rect_to_screen(SourceCellRect::cell(cursor))
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
/// `cell` width across or height down, and none on an axis the pointer has not
/// left.
///
fn edge_scroll(wanted: Vec2, past: Vec2, cell: Vec2) -> Vec2 {
    let most = (past / EDGE_SCROLL_REACH).min(cell);
    Vec2::new(
        wanted.x.clamp(-most.x, most.x),
        wanted.y.clamp(-most.y, most.y),
    )
}

///
/// Whether `origin` can be presented: finite on both axes.
///
/// The origin is the console's corner plus a Pan built from pointer input, and
/// nothing else resets one that has gone bad. A non-finite origin would put
/// every Cell, every click and Diagnostics' visible Source region
/// at NaN.
///
pub(super) fn is_presentable(origin: Pos2) -> bool {
    origin.is_finite()
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
/// Shows the Source in the console area at the Source View's Pan, keeps how
/// it was presented as `view`'s [`SourceView::shown`], and answers the Region
/// a click or a drag asked for, for the caller that owns the Source to
/// select.
///
/// The console owns the origin the Source is presented at, and
/// `grid_viewport::presented_grid` is the one place a Cell's rectangle is
/// derived from it. Every Cell, and so every click that lands on one, goes
/// through that one arithmetic. Nothing here sets a layer transform.
/// See `docs/adr/0038-the-console-owns-the-source-grid-transform.md`.
///
/// Pan is by wheel or two-finger scroll, by middle-drag, and by Alt (Option)
/// held with a primary drag, bounded by the Grid's edges plus a margin of
/// [`SOURCE_MARGIN_CELLS`]. Where the Grid and its margins leave
/// somewhere to Pan, the pointer shows a grab hand while Alt is held over the
/// console outside a Region drag, and a grabbing hand during a drag Pan. A primary
/// click selects a Cell — [`show_source`]'s own click-sensing rect answers it,
/// with Shift extending the Region rather than collapsing it — and a primary
/// drag without Alt selects a Region from the pressed Cell to the Cell nearest
/// the pointer; neither Pans. A drag past the console's edge scrolls after the
/// Cursor at the pointer's pace, at most one Cell a frame. Pinch and command-wheel do not
/// zoom. Zoom is egui's whole-UI zoom, which changes the console's size in
/// points and never the Source's; a zoom that would open a gap past an edge
/// settles back inside through the same `clamp_pan` a Pan does.
///
/// A Cursor move that would leave the Cursor's Cell outside the console Pans
/// just far enough to bring it back, before that same `clamp_pan` settles the
/// result inside the Grid. A zoom (egui's or the browser's) or a resize does
/// the same for a Cursor whose Cell the view showed in full the frame before;
/// one the viewer had Panned out of view, even in part, stays where the Pan
/// put it. A Pan with none of them is not pulled back to the Cursor. `frame`
/// already carries a keyboard Cursor move from this same Render Frame — `Console::ui`
/// reads it after `Orcvs::event_handler` runs — so that case is caught the
/// frame it happens.
/// A click's or a drag's Cursor move reaches the Source only after this call
/// returns (`Console::show_source_panel` applies the [`PointerSelection`]
/// next), so it is followed on the frame after, not this one.
///
pub(super) fn show_source_scene(
    ui: &mut egui::Ui,
    frame: &RenderFrame,
    font_family: &egui::FontFamily,
    view: &mut SourceView,
    theme: &Theme,
) -> Option<PointerSelection> {
    let source_grid = frame.grid();
    // `Sense::CLICK | Sense::DRAG` rather than `Sense::click_and_drag()`,
    // which adds `FOCUSABLE` and would let
    // Tab focus the console area, where a focused widget keeps every key from
    // the Source.
    let (console, pan) =
        ui.allocate_exact_size(ui.available_size_before_wrap(), Sense::CLICK | Sense::DRAG);

    // The Cell snapped to whole physical pixels once, for the follow, the
    // bounds and `presented_grid` below alike, so the Pan is measured against
    // the Grid as drawn rather than the unsnapped extent the Source asks for.
    let cell = snapped_cell(view.cell, ui.ctx().pixels_per_point());

    let alt = ui.input(|i| i.modifiers.alt);
    let pointer = ui.input(|i| i.pointer.interact_pos());
    let dragged = |button| pan.dragged_by(button).then(|| pan.drag_delta());
    let settled = view.settle(PanInput {
        middle_drag: dragged(PointerButton::Middle),
        primary_drag: dragged(PointerButton::Primary),
        alt,
        contains_pointer: pan.contains_pointer(),
        scroll: ui.input(|i| i.smooth_scroll_delta()),
        pointer,
        console,
        cell,
        grid: source_grid,
        cursor: frame.cursor(),
    });
    if settled.repaint {
        ui.ctx().request_repaint();
    }

    let origin = console.min
        + settled.pan
        + at_rest(cell)
            .to_screen(SourceCellPoint::new(0.0, 0.0))
            .to_vec2();
    let origin = if is_presentable(origin) {
        origin
    } else {
        Pos2::ZERO
    };
    let grid = presented_grid(origin, source_grid, cell, ui.ctx().pixels_per_point());
    view.shown = ShownSourceView::new(grid, console, source_grid);

    // The pointer offers a Pan only where there is one: not on a Grid that
    // with its margins fits the console on both axes, and not for Alt
    // pressed partway through a Region drag, which stays one. Alt is the one
    // Pan gesture the pointer can announce before it starts: a middle-drag
    // has no hover state to show, and a wheel Pan is not a grab.
    if settled.can_pan && settled.drag_panned {
        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
    } else if settled.can_pan && alt && view.region_drag.is_none() && pan.contains_pointer() {
        ui.ctx().set_cursor_icon(CursorIcon::Grab);
    }

    let clicked = show_source(ui, frame, font_family, grid, console, theme);

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

    spanned.or(clicked)
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
/// for the Cell it stands in for to still agree with it.
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
/// The Source View as one Render Frame presented it, for the Overview and the
/// Diagnostics to read: the map Paint drew the Source under, the console area
/// it was drawn in and the extent it Pans over.
///
#[derive(Clone, Copy, Debug)]
pub(super) struct ShownSourceView {
    viewport: GridViewport,
    console: Rect,
    extent: SourceCellRect,
}

impl ShownSourceView {
    ///
    /// The Source View presented as `viewport` in `console`, over the extent
    /// of `grid`.
    ///
    pub(super) fn new(viewport: GridViewport, console: Rect, grid: Grid) -> Self {
        Self {
            viewport,
            console,
            extent: extent(grid),
        }
    }

    ///
    /// The presentation before the first frame: no Cell, no console and no
    /// extent, so nothing to outline.
    ///
    fn none() -> Self {
        let nowhere = SourceCellPoint::new(0.0, 0.0);
        Self {
            viewport: GridViewport {
                cell_size: Vec2::ZERO,
                rect: Rect::ZERO,
            },
            console: Rect::ZERO,
            extent: SourceCellRect::from_min_max(nowhere, nowhere),
        }
    }

    /// The map from Source Cells to the screen points Paint drew them at.
    pub(super) fn screen(&self) -> ScreenMap {
        self.viewport.screen()
    }

    /// The region of Source Cells the console shows, unclipped.
    pub(super) fn shown(&self) -> SourceCellRect {
        self.screen().rect_to_cells(self.console())
    }

    ///
    /// The region the console shows clipped to the extent, or `None` where
    /// the console shows the whole extent and so has nowhere to Pan.
    ///
    pub(super) fn outline(&self) -> Option<SourceCellRect> {
        pannable(
            self.console().size(),
            self.extent.size() * self.screen().cell(),
        )
        .then(|| self.shown().intersect(self.extent))
    }

    /// The Grid as Paint drew it, for tests to read.
    #[cfg(test)]
    pub(super) fn viewport(&self) -> GridViewport {
        self.viewport
    }

    /// The console area the Source was drawn in.
    pub(super) fn console(&self) -> Rect {
        self.console
    }

    /// The region of Source Cells the Source View can show.
    pub(super) fn extent(&self) -> SourceCellRect {
        self.extent
    }
}

impl Console {
    ///
    /// Shows `frame` in the central panel at the Source View's Pan,
    /// in the Theme `appearance` presents, and selects the Region the pointer
    /// asked for. How it was presented is the Source View's
    /// [`SourceView::shown`].
    ///
    /// The Theme is borrowed from the selection for the panel; the panel reads
    /// the font, the Source View and the running Orcvs as fields beside it.
    ///
    pub(super) fn show_source_panel(
        &mut self,
        root: &mut egui::Ui,
        frame: &RenderFrame,
        appearance: Appearance,
    ) {
        let theme = self.themes.presented(appearance);
        egui::CentralPanel::default()
            .frame(source_panel_frame(theme.grid_background))
            .show(root, |ui| {
                let selection =
                    show_source_scene(ui, frame, &self.font_family, &mut self.source_view, theme);
                // The Source Grid answers which Cells the pointer asked for;
                // moving the Cursor and the anchor there is the Source's own
                // business, and this is where the running Orcvs is owned.
                if let Some(selection) = selection {
                    selection.apply(&mut self.orcvs);
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use egui::{Pos2, Rect, Vec2};
    use orcvs::grid::Grid;

    use super::{PanInput, SourceView, edge_scroll};
    use crate::grid_viewport::{CELL_SIZE, SourceCellPoint, snapped_cell};

    /// A console 200 points square, the size most of these settle under.
    const CONSOLE: Vec2 = Vec2::new(200.0, 200.0);

    ///
    /// A frame with no gesture: a console of `size` points from the screen's
    /// top-left over `grid` at the Source's own Cell, the Cursor at `cursor`.
    ///
    fn still(size: Vec2, grid: Grid, cursor: (usize, usize)) -> PanInput {
        PanInput {
            middle_drag: None,
            primary_drag: None,
            alt: false,
            contains_pointer: false,
            scroll: Vec2::ZERO,
            pointer: None,
            console: Rect::from_min_size(Pos2::ZERO, size),
            cell: CELL_SIZE,
            grid,
            cursor: grid.position(cursor.0, cursor.1).expect("inside the grid"),
        }
    }

    ///
    /// A fresh `SourceView`'s first frame does not Pan to the Cursor, however
    /// far a fixture puts it from an unpanned top-left origin. The Cursor
    /// follow needs a previous Cursor to compare against, and there is none
    /// yet on the very first frame — the same reason a Source reload would
    /// not surprise a viewer either.
    ///
    #[test]
    fn a_fresh_source_view_does_not_pan_to_the_cursor_on_its_first_frame() {
        let grid = Grid::with_shape(32, 32);
        let mut view = SourceView::new(CELL_SIZE);
        // Column 30 at the Source's own Cell size is far outside a 200 point console.
        view.settle(still(CONSOLE, grid, (30, 30)));

        assert_eq!(
            view.pan,
            Vec2::ZERO,
            "the first frame panned to a Cursor it had no previous position for"
        );
    }

    ///
    /// A Cursor move that would leave the Cursor outside the Source View Pans
    /// the least distance that shows the whole Cursor Cell, still bounded by
    /// the Grid's edges.
    ///
    #[test]
    fn a_cursor_move_that_would_leave_it_outside_the_view_pans_to_show_it() {
        let grid = Grid::with_shape(32, 32);
        let mut view = SourceView::new(CELL_SIZE);

        view.settle(still(CONSOLE, grid, (0, 0)));
        assert_eq!(view.pan, Vec2::ZERO, "the fixture did not open unpanned");

        // Column and row 30 end 33 Cells along once the two Cells of margin are
        // counted, entirely past a 200 point console on both axes.
        view.settle(still(CONSOLE, grid, (30, 30)));

        assert_eq!(
            view.pan,
            Vec2::splat(200.0) - 33.0 * CELL_SIZE,
            "the Cursor move did not Pan the least distance that shows it: {:?}",
            view.pan
        );
    }

    ///
    /// A Cursor move onto the Grid's last or first Column and Row brings the
    /// margin past that edge into view with it, so the Grid's edge shows as
    /// an edge rather than flush against the console's.
    ///
    #[test]
    fn a_cursor_follow_to_an_edge_cell_shows_the_margin_past_it() {
        let grid = Grid::with_shape(32, 32);
        let mut view = SourceView::new(CELL_SIZE);

        view.settle(still(CONSOLE, grid, (0, 0)));
        assert_eq!(view.pan, Vec2::ZERO, "the fixture did not open unpanned");

        // 32 Cells and a margin of two either side is 36 Cells, so the far
        // margin's edge meets a 200 point console 36 Cells less 200 points in.
        view.settle(still(CONSOLE, grid, (31, 31)));
        assert_eq!(
            view.pan,
            Vec2::splat(200.0) - 36.0 * CELL_SIZE,
            "the follow to the last Column and Row left the far margin off screen: {:?}",
            view.pan
        );

        view.settle(still(CONSOLE, grid, (0, 0)));
        assert_eq!(
            view.pan,
            Vec2::ZERO,
            "the follow back to the first Column and Row left the near margin off screen: {:?}",
            view.pan
        );
    }

    ///
    /// A Pan with no Cursor move and no zoom is not pulled back to the
    /// Cursor, even while the Cursor sits outside the Source View.
    ///
    #[test]
    fn a_pan_with_no_cursor_move_and_no_zoom_is_not_pulled_back_to_the_cursor() {
        let grid = Grid::with_shape(32, 32);
        let mut view = SourceView::new(CELL_SIZE);

        // First frame: no previous Cursor to compare against, so no follow.
        view.settle(still(CONSOLE, grid, (30, 30)));
        assert_eq!(view.pan, Vec2::ZERO);

        // A wheel Pan all the way to the far edge, with the Cursor still
        // unmoved at (30, 30) and no zoom. Left to the follow this would land
        // 33 Cells less 200 points in instead — see
        // `a_cursor_move_that_would_leave_it_outside_the_view_pans_to_show_it`
        // — so landing on the margin's far edge, 36 Cells less 200 points in,
        // is what proves a Pan alone is not chasing the Cursor.
        view.settle(PanInput {
            contains_pointer: true,
            scroll: Vec2::new(-1_000.0, -1_000.0),
            ..still(CONSOLE, grid, (30, 30))
        });

        assert_eq!(
            view.pan,
            Vec2::splat(200.0) - 36.0 * CELL_SIZE,
            "a Pan alone was pulled back toward the Cursor: {:?}",
            view.pan
        );
    }

    ///
    /// The follow Pan is itself naive — it only asks whether the Cursor's
    /// Cell already shows inside the console — so an already out-of-bounds
    /// Pan it leaves untouched still has to settle back inside the Grid
    /// through `clamp_pan`, the same as an ordinary Pan or a zoom does.
    ///
    #[test]
    fn the_follow_pan_is_still_bounded_by_the_grids_edges() {
        let grid = Grid::with_shape(32, 32);
        let mut view = SourceView::new(CELL_SIZE);

        view.settle(still(CONSOLE, grid, (0, 0)));
        assert_eq!(view.pan, Vec2::ZERO, "the fixture did not open unpanned");

        // A Pan past the Grid's near edge, which nothing but `clamp_pan` can
        // answer: the Cursor's new Cell (5, 0) sits at 84..96 across, already
        // inside a 200 point console once this Pan is applied, so the follow
        // itself has nothing to add.
        view.pan = Vec2::new(50.0, 50.0);
        view.settle(still(CONSOLE, grid, (5, 0)));

        assert_eq!(
            view.pan,
            Vec2::ZERO,
            "clamp_pan did not settle the follow's own Pan back inside the Grid: {:?}",
            view.pan
        );
    }

    ///
    /// A Pan that would open a gap past an edge after a resize settles back
    /// inside the Grid.
    ///
    /// The Pan has already taken the Cursor, at (0, 0), out of view, so the
    /// resize does not follow it: what settles the gap is `clamp_pan`.
    ///
    #[test]
    fn a_resize_that_would_open_a_gap_settles_the_source_view_back_inside() {
        let grid = Grid::with_shape(32, 32);
        let mut view = SourceView::new(CELL_SIZE);

        // 32 Cells and a margin of two either side is 36 Cells, so the far edge
        // of a console is 36 Cells less its size in.
        let far_edge = |console: f32| Vec2::splat(console) - 36.0 * CELL_SIZE;
        view.pan = far_edge(200.0);
        view.settle(still(CONSOLE, grid, (0, 0)));
        assert_eq!(view.pan, far_edge(200.0));

        view.settle(still(Vec2::splat(400.0), grid, (0, 0)));
        assert_eq!(
            view.pan,
            far_edge(400.0),
            "the resize left a gap past the Grid: {:?}",
            view.pan
        );
    }

    ///
    /// A window resize that would leave the Cursor outside the Source View Pans
    /// to show it, as a zoom does: both change the console's size in points.
    ///
    #[test]
    fn a_resize_that_would_leave_the_cursor_outside_the_view_pans_to_show_it() {
        let grid = Grid::with_shape(32, 32);
        let mut view = SourceView::new(CELL_SIZE);
        // Column 13 and row 8 end 16 and 11 Cells along, margin counted:
        // inside a 200 point console and past a 160 point one.
        view.settle(still(CONSOLE, grid, (13, 8)));
        assert_eq!(
            view.pan,
            Vec2::ZERO,
            "the fixture's first frame already Panned"
        );

        let shrunk = Vec2::splat(160.0);
        view.settle(still(shrunk, grid, (13, 8)));
        assert_eq!(
            view.pan,
            Vec2::splat(160.0) - Vec2::new(16.0, 11.0) * CELL_SIZE,
            "the resize did not Pan the least distance that shows the Cursor: {:?}",
            view.pan
        );

        // A frame at the size it already had is not a resize, so a Pan away from
        // the Cursor stays where it was put.
        view.pan = Vec2::ZERO;
        view.settle(still(shrunk, grid, (13, 8)));
        assert_eq!(view.pan, Vec2::ZERO, "an unchanged size chased the Cursor");
    }

    ///
    /// A resize leaves a Pan the viewer chose away from the Cursor where it was:
    /// only a Cursor the view already showed is followed.
    ///
    #[test]
    fn a_resize_after_a_pan_away_from_the_cursor_keeps_the_pan() {
        let grid = Grid::with_shape(32, 32);
        let mut view = SourceView::new(CELL_SIZE);
        // Column 9 and row 5 start 11 and 7 Cells along, margin counted: 132 and
        // 126 points, inside the 200 point console before the Pan.
        view.settle(still(CONSOLE, grid, (9, 5)));
        // A Pan that puts the Cursor off the console's top-left, inside the
        // Grid's bounds at every size below.
        let away = Vec2::new(-160.0, -160.0);
        view.pan = away;
        view.settle(still(CONSOLE, grid, (9, 5)));
        assert_eq!(view.pan, away, "the fixture's Pan did not hold");

        // A shrink and a growth: either one, followed, would Pan the Cursor
        // back to the console's top-left.
        for size in [Vec2::new(160.0, 160.0), Vec2::new(240.0, 240.0)] {
            view.settle(still(size, grid, (9, 5)));
            assert_eq!(
                view.pan, away,
                "a resize to {size:?} pulled the Pan back to the Cursor: {:?}",
                view.pan
            );
        }
    }

    ///
    /// A zoom that would open a gap past an edge settles the Source View back
    /// inside the margin, through the same `clamp_pan` a Pan uses.
    ///
    /// egui's zoom changes the console's size in points and never the Source's,
    /// so zooming out widens the console around a Pan pinned to the far edge.
    /// The Cursor sits on the far corner the pinned Pan already shows, so the
    /// Cursor follow has nothing to do: what settles the gap is `clamp_pan`.
    ///
    #[test]
    fn a_zoom_that_would_open_a_gap_settles_the_source_view_back_inside() {
        let grid = Grid::with_shape(32, 32);
        let mut view = SourceView::new(CELL_SIZE);

        // 32 Cells and a margin of two either side is 36 Cells, so the far edge
        // of a console is 36 Cells less its size in.
        let far_edge = |console: f32| Vec2::splat(console) - 36.0 * CELL_SIZE;
        view.pan = far_edge(200.0);
        view.settle(still(CONSOLE, grid, (31, 31)));
        assert_eq!(
            view.pan,
            far_edge(200.0),
            "the fixture did not open already pinned to the far edge"
        );

        // At half zoom the same window is 400 points across, at the Cell the
        // zoom snaps to.
        let cell = snapped_cell(CELL_SIZE, 0.5);
        assert_eq!(cell, CELL_SIZE, "the zoom changed the Cell's points");
        view.settle(PanInput {
            cell,
            ..still(Vec2::splat(400.0), grid, (31, 31))
        });
        assert_eq!(
            view.pan,
            far_edge(400.0),
            "the zoom left a gap past the Grid: {:?}",
            view.pan
        );
    }

    ///
    /// A zoom that would leave the Cursor outside the Source View Pans the
    /// least distance that shows it, the same as a Cursor move does.
    ///
    #[test]
    fn a_zoom_that_would_leave_the_cursor_outside_the_view_pans_to_show_it() {
        let grid = Grid::with_shape(32, 32);
        let mut view = SourceView::new(CELL_SIZE);
        // Column 13 and row 8 end 16 and 11 Cells along, margin counted:
        // inside a 200 point console and past a 160 point one.
        view.settle(still(CONSOLE, grid, (13, 8)));
        assert_eq!(
            view.pan,
            Vec2::ZERO,
            "the fixture's first frame already Panned"
        );

        // At 1.25 the same window is 160 points across, and the Cell snaps to
        // 12 by 17.6 points, so the Cursor's Cell ends 16 and 11 of those
        // along, and the least Pan puts that end on the console's.
        let cell = snapped_cell(CELL_SIZE, 1.25);
        let zoomed_in = PanInput {
            cell,
            ..still(Vec2::splat(160.0), grid, (13, 8))
        };
        view.settle(zoomed_in);
        assert_eq!(
            view.pan,
            Vec2::splat(160.0) - Vec2::new(16.0, 11.0) * cell,
            "the zoom did not Pan the least distance that shows the Cursor: {:?}",
            view.pan
        );

        // A frame at the zoom it already had is not a zoom, so a Pan away from
        // the Cursor stays where it was put.
        view.pan = Vec2::ZERO;
        view.settle(PanInput {
            cell,
            ..still(Vec2::splat(160.0), grid, (13, 8))
        });
        assert_eq!(view.pan, Vec2::ZERO, "an unchanged zoom chased the Cursor");
    }

    ///
    /// Releasing a drag held past the edge does not jump the Source View to
    /// the Cursor: the frame the button comes up still scrolls at most a Cell.
    ///
    #[test]
    fn releasing_a_drag_past_the_edge_scrolls_no_more_than_a_cell() {
        let grid = Grid::with_shape(64, 32);
        let console = Vec2::new(400.0, 200.0);
        let mut view = SourceView::new(CELL_SIZE);
        view.settle(still(console, grid, (20, 5)));
        view.region_drag = grid.position(20, 5);
        // Far enough out that the Cursor, the Cell nearest the pointer, is
        // many Cells past the edge.
        let far = Pos2::new(console.x + 400.0, 100.0);

        // The release frame still has the drag; the caller clears it after.
        for _ in 0..3 {
            let before = view.pan.x;
            view.settle(PanInput {
                pointer: Some(far),
                ..still(console, grid, (63, 5))
            });
            view.region_drag = None;
            assert!(
                before - view.pan.x <= CELL_SIZE.x,
                "a frame around the release scrolled {}",
                before - view.pan.x
            );
        }
    }

    ///
    /// A Cursor follow and a requested centre on one frame settle at the
    /// centre: the request runs after the follow.
    ///
    #[test]
    fn a_follow_and_a_requested_centre_on_one_frame_settle_at_the_centre() {
        let grid = Grid::with_shape(32, 32);
        let mut view = SourceView::new(CELL_SIZE);
        view.settle(still(CONSOLE, grid, (0, 0)));

        view.centre_on(SourceCellPoint::new(16.0, 16.0));
        view.settle(still(CONSOLE, grid, (30, 30)));

        // Cell 16 is 18 Cells along, margin counted, and the console's centre
        // is 100 points in.
        assert_eq!(view.pan, Vec2::splat(100.0) - 18.0 * CELL_SIZE);
    }

    ///
    /// A requested centre past an edge settles at the clamp, as any Pan does.
    ///
    #[test]
    fn a_requested_centre_past_an_edge_settles_at_the_clamp() {
        let grid = Grid::with_shape(32, 32);
        let mut view = SourceView::new(CELL_SIZE);
        view.settle(still(CONSOLE, grid, (0, 0)));

        view.centre_on(SourceCellPoint::new(40.0, -10.0));
        view.settle(still(CONSOLE, grid, (0, 0)));

        assert_eq!(view.pan, Vec2::new(200.0 - 36.0 * CELL_SIZE.x, 0.0));
    }

    ///
    /// Alt pressed partway through a Region drag leaves it a Region drag: the
    /// primary drag's delta does not Pan, and a pointer past the edge scrolls
    /// at its pace instead.
    ///
    #[test]
    fn alt_pressed_during_a_region_drag_edge_scrolls_rather_than_pans() {
        let grid = Grid::with_shape(32, 32);
        let mut view = SourceView::new(CELL_SIZE);
        view.settle(still(CONSOLE, grid, (0, 0)));
        view.region_drag = grid.position(0, 0);

        // Column 16 ends 19 Cells along, 28 points past the console; the
        // pointer is 8 points past it, so the frame scrolls 2.
        let settled = view.settle(PanInput {
            primary_drag: Some(Vec2::new(-40.0, 0.0)),
            alt: true,
            contains_pointer: true,
            pointer: Some(Pos2::new(208.0, 100.0)),
            ..still(CONSOLE, grid, (16, 5))
        });

        assert_eq!(settled.pan, Vec2::new(-2.0, 0.0));
        assert!(
            !settled.drag_panned,
            "Alt turned the Region drag into a Pan"
        );
    }

    ///
    /// A resize follows a Cursor the view showed in full, and leaves one the
    /// viewer Panned partly out of view where the Pan put it.
    ///
    #[test]
    fn a_resize_follows_a_cursor_shown_in_full_and_leaves_one_panned_partly_out_of_view() {
        let grid = Grid::with_shape(32, 32);
        // Column 13 and row 8 span 180..192 across and 180..198 down at rest.
        for (pan, followed) in [(Vec2::ZERO, true), (Vec2::new(-185.0, 0.0), false)] {
            let mut view = SourceView::new(CELL_SIZE);
            view.settle(still(CONSOLE, grid, (13, 8)));
            view.pan = pan;
            view.settle(still(CONSOLE, grid, (13, 8)));

            view.settle(still(Vec2::splat(160.0), grid, (13, 8)));

            let follow = Vec2::splat(160.0) - Vec2::new(16.0, 11.0) * CELL_SIZE;
            let expected = if followed { follow } else { pan };
            assert_eq!(view.pan, expected, "from a Pan of {pan:?}");
        }
    }

    ///
    /// A Region drag's edge scroll takes one point for every
    /// [`super::EDGE_SCROLL_REACH`] the pointer is past the edge, never more
    /// than the follow wants, and never more than one Cell on either axis.
    ///
    #[test]
    fn edge_scroll_is_paced_by_the_pointer_and_capped_at_a_cell() {
        let wanted = Vec2::new(-100.0, 100.0);
        for (past, scrolled) in [
            (Vec2::new(8.0, 0.0), Vec2::new(-2.0, 0.0)),
            (Vec2::new(0.0, 20.0), Vec2::new(0.0, 5.0)),
            (Vec2::splat(400.0), Vec2::new(-CELL_SIZE.x, CELL_SIZE.y)),
        ] {
            assert_eq!(edge_scroll(wanted, past, CELL_SIZE), scrolled, "{past:?}");
        }
        assert_eq!(
            edge_scroll(Vec2::new(-1.0, 0.0), Vec2::splat(400.0), CELL_SIZE),
            Vec2::new(-1.0, 0.0),
            "the edge scroll overshot what the follow wanted"
        );
    }

    ///
    /// `repaint` asks for the next frame exactly while a Region drag's pointer
    /// is past the console's edge.
    ///
    #[test]
    fn repaint_is_asked_for_exactly_while_a_region_drags_pointer_is_past_the_edge() {
        let grid = Grid::with_shape(32, 32);
        let past = Some(Pos2::new(250.0, 100.0));
        let inside = Some(Pos2::new(100.0, 100.0));
        for (region_drag, pointer, repaint) in [
            (true, past, true),
            (true, inside, false),
            (true, None, false),
            (false, past, false),
        ] {
            let mut view = SourceView::new(CELL_SIZE);
            view.settle(still(CONSOLE, grid, (16, 5)));
            view.region_drag = grid.position(16, 5).filter(|_| region_drag);
            let settled = view.settle(PanInput {
                pointer,
                ..still(CONSOLE, grid, (16, 5))
            });
            assert_eq!(
                settled.repaint, repaint,
                "a Region drag {region_drag} with the pointer at {pointer:?}"
            );
        }
    }

    ///
    /// `can_pan` answers whether the extent leaves anywhere to Pan.
    ///
    #[test]
    fn can_pan_is_whether_the_extent_overflows_the_console() {
        // Eight Cells and their margins are 144 by 216 points, inside a 400
        // point console; 32 are 432 by 648.
        let console = Vec2::splat(400.0);
        for (grid, can_pan) in [
            (Grid::with_shape(8, 8), false),
            (Grid::with_shape(32, 32), true),
        ] {
            let settled = SourceView::new(CELL_SIZE).settle(still(console, grid, (0, 0)));
            assert_eq!(settled.can_pan, can_pan);
        }
    }
}
