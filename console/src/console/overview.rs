//! The Overview: the whole Grid and its margin, one texel per Cell in the
//! Theme's colours, floating over the window's bottom-right corner.

use std::iter::repeat_n;
use std::ops::Range;
use std::sync::Arc;

use egui::emath::GuiRounding as _;
use egui::{
    Color32, ColorImage, CursorIcon, Painter, PointerButton, Rect, Response, Sense, Stroke,
    StrokeKind, TextureHandle, TextureOptions, Vec2,
};
use orcvs::grid::Grid;
use orcvs::render_frame::{RenderCell, RenderFrame};
use orcvs::source::RevisionId;

use super::Console;
use super::source_view::{ShownSourceView, extent};
use crate::grid_viewport::{PictureMap, SourceCellPoint, SourceCellRect};
use crate::style::{CellColours, CursorPlacement};
use crate::theme::{Appearance, Theme, ThemeIdentity};

///
/// The Overview's picture, kept across Render Frames, and the texture it is
/// drawn from.
///
/// The texture holds no galley, only the Theme's colours, so nothing it holds
/// goes stale when egui rebuilds the font atlas.
///
#[derive(Default)]
pub(super) struct Overview {
    /// The picture, shared with the texture update in flight. Once the
    /// renderer has taken that update this is the only owner again, so the
    /// next redraw writes in place rather than copying.
    picture: Arc<ColorImage>,
    texture: Option<TextureHandle>,
    /// The Source revision, Theme and appearance `picture` was drawn from.
    drawn_from: Option<(RevisionId, ThemeIdentity, Appearance)>,
    /// From the pointer to the point the Source View is centred on, in Cells,
    /// fixed when the press began: zero for a press outside the outline, so
    /// the Source View centres on the pointer, and the offset to the outline's
    /// centre for one inside it, so the Source View does not jump.
    grab: Vec2,
}

impl Overview {
    ///
    /// Redraws the picture and updates the texture when `revision`, the Theme
    /// or `appearance` differs from what the picture was drawn from, and
    /// answers the texture. `revision` has to be read before `frame` was
    /// taken.
    ///
    fn texture(
        &mut self,
        ctx: &egui::Context,
        revision: RevisionId,
        frame: &RenderFrame,
        theme: &Theme,
        appearance: Appearance,
    ) -> &TextureHandle {
        let current = self
            .drawn_from
            .as_ref()
            .is_some_and(|(drawn, identity, shown)| {
                *drawn == revision && *identity == theme.identity && *shown == appearance
            });
        if !current {
            draw_picture(frame, theme, Arc::make_mut(&mut self.picture));
            self.drawn_from = Some((revision, theme.identity.clone(), appearance));
            // A partial update must lie inside the texture, so a picture of
            // another Grid's size replaces the texture's image outright.
            if let Some(texture) = &mut self.texture {
                let picture = Arc::clone(&self.picture);
                if texture.size() == picture.size {
                    texture.set_partial([0, 0], picture, TextureOptions::NEAREST);
                } else {
                    texture.set(picture, TextureOptions::NEAREST);
                }
            }
        }
        self.texture.get_or_insert_with(|| {
            ctx.load_texture(
                "overview",
                Arc::clone(&self.picture),
                TextureOptions::NEAREST,
            )
        })
    }
}

impl Overview {
    ///
    /// The centre a press or drag on `response` asks the Source View to Pan
    /// to: on the press frame the grab is fixed from where the press landed
    /// against `source`'s outline, and each Render Frame the button is held
    /// answers the pointer plus that grab.
    ///
    /// Nothing is asked where `source` has no outline, as the Source View
    /// offers no Pan there either. It never selects, so the Cursor and the
    /// Region stay where they were.
    ///
    fn requested_centre(
        &mut self,
        response: &Response,
        picture: Rect,
        source: &ShownSourceView,
    ) -> Option<SourceCellPoint> {
        let Some(outline) = source.outline() else {
            // No press fixes a grab here, so an earlier gesture's is dropped and a
            // drag that can Pan only partway through centres on the pointer.
            self.grab = Vec2::ZERO;
            return None;
        };
        if let Some(pointer) = response.interact_pointer_pos()
            && response.dragged_by(PointerButton::Primary)
        {
            let at = PictureMap::new(source.extent(), picture).to_cells(pointer);
            if response.drag_started_by(PointerButton::Primary) {
                self.grab = if outline.contains(at) {
                    outline.center() - at
                } else {
                    Vec2::ZERO
                };
            }
            response.ctx.set_cursor_icon(CursorIcon::Grabbing);
            response.ctx.request_repaint();
            Some(at + self.grab)
        } else {
            if response.hovered() {
                response.ctx.set_cursor_icon(CursorIcon::Grab);
            }
            None
        }
    }
}

///
/// Outlines `source`'s [`ShownSourceView::outline`] on `picture`: clipped to
/// the picture on an axis the console shows all of, and not drawn when it
/// shows all of both, where there is nowhere to Pan. The stroke lies inside
/// the outline, so a clipped edge stays on the picture.
///
fn outline_source_view(painter: &Painter, picture: Rect, source: &ShownSourceView, theme: &Theme) {
    let Some(outline) = source.outline() else {
        return;
    };
    painter.rect_stroke(
        PictureMap::new(source.extent(), picture).rect_to_picture(outline),
        0.0,
        Stroke::new(
            theme.overview_source_view_width.points(),
            theme.overview_source_view,
        ),
        StrokeKind::Inside,
    );
}

///
/// Where the picture of `grid`'s extent lies in `window`: pinned so its
/// corner is the window's bottom-right corner, on whole physical pixels.
///
/// It is square, at a whole number of physical pixels per texel, so every
/// Cell is a crisp block of one size: the largest number up to the whole
/// pixels per point that fits, with a `border` along its inner edges, in
/// half the window's width and the height from `below_menu` to the window's
/// bottom. It answers `None` where one pixel per texel does not fit.
///
fn place_picture(
    window: Rect,
    below_menu: f32,
    border: f32,
    grid: Grid,
    pixels_per_point: f32,
) -> Option<Rect> {
    let texels = extent(grid).size();
    let room = Vec2::new(window.width() / 2.0, window.bottom() - below_menu) - Vec2::splat(border);
    let pixels_per_texel = pixels_per_point
        .floor()
        .max(1.0)
        .min((room * pixels_per_point / texels).min_elem().floor());
    if pixels_per_texel < 1.0 {
        return None;
    }
    let side = texels * pixels_per_texel / pixels_per_point;
    Some(Rect::from_min_size(
        (window.max - side).round_to_pixels(pixels_per_point),
        side,
    ))
}

impl Console {
    ///
    /// Shows the Overview pinned to the window's bottom-right corner, when it
    /// is open and fits below `below_menu`, the menu bar's bottom, and answers
    /// the centre a press or drag on it asks the Source View to Pan to. It
    /// floats over the Source and the bottom panel, so it takes no width from
    /// the Source, with no margin: the bottom panel's border along the two
    /// edges that face the rest of the console, and the picture filling the
    /// rest, placed by [`place_picture`].
    ///
    /// It marks the Region and the Cursor and outlines the Source View as the
    /// Source was presented this Render Frame, so it is shown after the
    /// Source. The picture is not clipped to fit: a press maps to a Pan only
    /// across the whole picture. Like every egui Area it is drawn from its
    /// second pass, the first only measuring it, and it does not fade in. It
    /// senses a drag and nothing else: sensing focus would put it in the Tab
    /// order, where a focused widget keeps every key from the Source.
    ///
    pub(super) fn show_overview(
        &mut self,
        root: &egui::Ui,
        below_menu: f32,
        revision: RevisionId,
        frame: &RenderFrame,
        appearance: Appearance,
    ) -> Option<SourceCellPoint> {
        if !self.overview_open {
            return None;
        }
        let ctx = root.ctx().clone();
        // The stroke egui's panel separator draws on the bottom panel.
        let border = root.visuals().widgets.noninteractive.bg_stroke;
        let rect = place_picture(
            ctx.content_rect(),
            below_menu,
            border.width,
            frame.grid(),
            ctx.pixels_per_point(),
        )?;
        let theme = self.themes.presented(appearance);
        let texture = self
            .overview
            .texture(&ctx, revision, frame, theme, appearance)
            .id();
        let source = self.source_view.shown();
        let overview = &mut self.overview;
        let bordered = Rect::from_min_max(rect.min - Vec2::splat(border.width), rect.max);
        egui::Area::new(egui::Id::new("overview"))
            .fixed_pos(bordered.min)
            .movable(false)
            .fade_in(false)
            // Sized as the bordered picture from the first pass and never
            // moved to fit the window, so the layer is that rect and takes no
            // pointer from the Source around it.
            .default_size(bordered.size())
            .constrain(false)
            .sense(Sense::hover())
            .show(&ctx, |ui| {
                // Each line just outside the picture, as the separator lies
                // just outside the bottom panel's frame.
                let inset = border.width / 2.0;
                ui.painter()
                    .hline(bordered.x_range(), rect.top() - inset, border);
                ui.painter()
                    .vline(rect.left() - inset, bordered.y_range(), border);
                let response = ui.allocate_rect(rect, Sense::DRAG);
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::Other, true, "Source overview")
                });
                ui.painter().image(
                    texture,
                    rect,
                    Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                let painter = ui.painter_at(rect);
                mark_region_and_cursor(&painter, rect, frame, theme);
                outline_source_view(&painter, rect, &source, theme);
                overview.requested_centre(&response, rect, &source)
            })
            .inner
    }
}

///
/// Marks `frame`'s Region and then its Cursor's Cell on `picture` in the
/// Theme's Region and Cursor fills and borders, so moving either changes no
/// texel. A Region of one Cell is the Cursor's Cell and gets no mark of its
/// own. The Region's mark shows where the Region is, not how its Cells look:
/// it is one rect of `region.background` over the picture, so it tints Cells
/// the Source leaves untinted inside a Region, such as a Portal's, and an
/// opaque `region.background` hides the texels it covers. The Cursor keeps
/// its border inside a spanning Region, where the Source shows only the
/// Region's, so the Overview always shows where the Cursor is. The Cursor's
/// Cell is the texel the picture would draw for it at the Cursor's placement,
/// so its fill is the one the Source gives it.
///
fn mark_region_and_cursor(painter: &Painter, picture: Rect, frame: &RenderFrame, theme: &Theme) {
    let extent = extent(frame.grid());
    let map = PictureMap::new(extent, picture);
    let texel = picture.size() / extent.size();
    let cells = |columns: Range<usize>, rows: Range<usize>| {
        let corner = |column: usize, row: usize| SourceCellPoint::new(column as f32, row as f32);
        map.rect_to_picture(SourceCellRect::from_min_max(
            corner(columns.start, rows.start),
            corner(columns.end, rows.end),
        ))
    };
    // An inside stroke half a texel wide covers a one-texel mark entirely, so
    // a border keeps to a quarter texel and leaves the middle half to the fill.
    let border =
        |width: f32, colour: Color32| Stroke::new(width.min(texel.min_elem() / 4.0), colour);
    let region = frame.region();
    let placement = if region.is_one_cell() {
        CursorPlacement::Cursor
    } else {
        painter.rect(
            cells(region.columns(), region.rows()),
            0.0,
            theme.region_background,
            border(theme.region_border_width.points(), theme.region_border),
            StrokeKind::Inside,
        );
        CursorPlacement::RegionCursor
    };
    let cursor = frame.cursor();
    painter.rect(
        map.rect_to_picture(SourceCellRect::cell(cursor)),
        0.0,
        cell_texel(&mut CellColours::new(theme), frame.at(cursor), placement),
        border(theme.cursor_border_width.points(), theme.cursor_border),
        StrokeKind::Inside,
    );
}

///
/// Draws `frame` into `picture` in `theme`'s colours: one texel per Source
/// Cell of the extent, row by row, with the margin around the Grid as the
/// surface the Source is displayed on.
///
/// Each Cell's colour is its Plain displayed colours reduced by a fixed rule:
/// the displayed foreground where the Cell is occupied, otherwise the
/// displayed background. Both are composited down to the console's opaque
/// surface, as the Source shows them, so every texel is opaque and the
/// floating Overview never shows the Source beneath it. The Cursor and the
/// Region are not drawn, so moving them changes no texel.
///
/// `picture`'s pixels are cleared and refilled, so a picture redrawn at the
/// same size allocates nothing.
///
pub fn draw_picture(frame: &RenderFrame, theme: &Theme, picture: &mut ColorImage) {
    let grid = frame.grid();
    let extent = extent(grid);
    let (width, height) = (extent.size().x as usize, extent.size().y as usize);
    // The margin's texels before the Grid on each axis, and after it.
    let (left, top) = (-extent.min().x() as usize, -extent.min().y() as usize);
    let (right, bottom) = (width - left - grid.columns(), height - top - grid.rows());
    let mut colours = CellColours::new(theme);
    let background = colours.surface();

    picture.size = [width, height];
    picture.source_size = egui::vec2(width as f32, height as f32);
    let pixels = &mut picture.pixels;
    pixels.clear();
    pixels.extend(repeat_n(background, top * width));
    for row in frame.cells().chunks(grid.columns()) {
        pixels.extend(repeat_n(background, left));
        pixels.extend(
            row.iter()
                .map(|cell| cell_texel(&mut colours, cell, CursorPlacement::Plain)),
        );
        pixels.extend(repeat_n(background, right));
    }
    pixels.extend(repeat_n(background, bottom * width));
}

///
/// The texel for `cell` at `placement`: its displayed foreground where it is
/// occupied, otherwise its displayed background.
///
fn cell_texel(
    colours: &mut CellColours<'_>,
    cell: &RenderCell,
    placement: CursorPlacement,
) -> Color32 {
    let displayed = colours.displayed(cell.source_paint(), cell.portal_roles(), placement);
    if cell.content().is_some() {
        displayed.displayed_foreground()
    } else {
        displayed.background
    }
}

#[cfg(test)]
mod tests {
    use egui::{Color32, ColorImage};
    use orcvs::app::Orcvs;
    use orcvs::grid::Grid;
    use orcvs::source::Tick;

    use egui::{Pos2, Rect, Vec2};

    use super::{draw_picture, place_picture};
    use crate::console::menu_bar::TOP_PANEL_HEIGHT;
    use crate::console::source_view::extent;
    use crate::console::{DEFAULT_VIEW_SIZE, DEFAULT_VIEW_SIZE_MIN};
    use crate::theme::{Theme, okabe_ito, orcvs_light};

    /// The margin around the Grid in texels, read from the extent.
    fn margin() -> usize {
        -extent(Grid::new()).min().x() as usize
    }

    fn running_orcvs(cols: usize, rows: usize) -> Orcvs {
        Orcvs::with_shape(cols, rows).expect("the test runtime")
    }

    fn write_row(orcvs: &mut Orcvs, row: usize, text: &str) {
        let grid = orcvs.grid();
        for (x, character) in text.chars().enumerate() {
            orcvs.select(grid.position(x, row).expect("inside the grid"));
            orcvs.write(&character.to_string());
        }
    }

    fn picture(orcvs: &Orcvs, theme: &Theme) -> ColorImage {
        let mut picture = ColorImage::default();
        draw_picture(&orcvs.render_frame(), theme, &mut picture);
        picture
    }

    /// The texel of the Cell at `column` and `row`, inside the margin.
    fn texel(picture: &ColorImage, column: usize, row: usize) -> Color32 {
        picture.pixels[(row + margin()) * picture.size[0] + column + margin()]
    }

    #[tokio::test]
    async fn an_empty_grid_and_its_margin_are_all_grid_background() {
        let orcvs = running_orcvs(6, 4);
        let theme = okabe_ito();
        let picture = picture(&orcvs, &theme);

        assert_eq!(picture.size, [6 + 2 * margin(), 4 + 2 * margin()]);
        assert!(
            picture
                .pixels
                .iter()
                .all(|texel| *texel == theme.grid_background),
            "an empty Grid drew something other than the Grid background"
        );
    }

    #[tokio::test]
    async fn each_occupied_cell_takes_its_roles_foreground_and_the_margin_stays_grid_background() {
        let mut orcvs = running_orcvs(8, 4);
        write_row(&mut orcvs, 0, ".+0304");
        write_row(&mut orcvs, 1, "**");
        write_row(&mut orcvs, 2, "||ab");
        write_row(&mut orcvs, 3, ".+XY");
        orcvs.select(orcvs.grid().position(7, 3).expect("inside the grid"));
        let theme = okabe_ito();
        let picture = picture(&orcvs, &theme);

        for column in 0..2 {
            assert_eq!(texel(&picture, column, 0), theme.source_function);
            assert_eq!(texel(&picture, column, 1), theme.source_bang);
            assert_eq!(texel(&picture, column, 2), theme.source_comment);
            assert_eq!(texel(&picture, column + 2, 0), theme.source_number);
            assert_eq!(texel(&picture, column + 2, 3), theme.diagnostic_foreground);
        }
        let [width, height] = picture.size;
        let margin = margin();
        for y in 0..height {
            for x in 0..width {
                let in_margin =
                    x < margin || y < margin || x >= width - margin || y >= height - margin;
                if in_margin {
                    assert_eq!(
                        picture.pixels[y * width + x],
                        theme.grid_background,
                        "margin texel ({x}, {y})"
                    );
                }
            }
        }
    }

    #[tokio::test]
    async fn an_empty_cell_a_committed_tick_read_takes_its_portal_background() {
        // `&v` reads the pair one row south of its operand, (2, 1) and (3, 1).
        let mut orcvs = running_orcvs(4, 4);
        write_row(&mut orcvs, 0, "&v01");
        let theme = okabe_ito();
        let before = picture(&orcvs, &theme);
        assert_eq!(texel(&before, 2, 1), theme.grid_background);

        orcvs.source().execute(Tick::ZERO);
        let after = picture(&orcvs, &theme);
        for column in [2, 3] {
            assert_eq!(
                texel(&after, column, 1),
                theme.grid_background.blend(theme.input_portal_background),
                "column {column}"
            );
        }
    }

    #[tokio::test]
    async fn a_translucent_grid_and_token_still_draw_an_opaque_picture() {
        let mut orcvs = running_orcvs(4, 2);
        write_row(&mut orcvs, 0, ".+");
        let mut theme = okabe_ito();
        theme.grid_background = Color32::from_rgba_unmultiplied(40, 60, 80, 128);
        theme.source_function = Color32::from_rgba_unmultiplied(200, 100, 50, 128);
        let picture = picture(&orcvs, &theme);

        let surface = theme.window_background.blend(theme.grid_background);
        assert_eq!(
            texel(&picture, 0, 0),
            surface
                .blend(theme.source_function_background)
                .blend(theme.source_function),
            "an occupied Function Cell"
        );
        assert_eq!(texel(&picture, 3, 1), surface, "an empty Cell");
        assert_eq!(picture.pixels[0], surface, "the margin");
        assert!(
            picture.pixels.iter().all(|texel| texel.a() == 255),
            "a texel is translucent"
        );
    }

    #[tokio::test]
    async fn another_theme_draws_another_picture() {
        let mut orcvs = running_orcvs(8, 2);
        write_row(&mut orcvs, 0, ".+0304");
        let dark = picture(&orcvs, &okabe_ito());
        let light = picture(&orcvs, &orcvs_light());

        assert_ne!(dark, light);
        assert_eq!(texel(&light, 7, 1), orcvs_light().grid_background);
        assert_eq!(texel(&light, 0, 0), orcvs_light().source_function);
    }

    /// The bottom panel's separator, which borders the picture's inner edges.
    const BORDER: f32 = 1.0;

    /// The picture's rect in a window of `size` at the origin.
    fn placed(size: Vec2, pixels_per_point: f32) -> Option<Rect> {
        let window = Rect::from_min_size(Pos2::ZERO, size);
        place_picture(
            window,
            TOP_PANEL_HEIGHT,
            BORDER,
            Grid::new(),
            pixels_per_point,
        )
    }

    ///
    /// The 256 by 256 Grid and its two-Cell margin are 260 texels a side,
    /// placed square at the window's bottom-right corner at a whole number of
    /// physical pixels per texel, up to the whole pixels per point: one at 1×
    /// and 1.5×, two at 2× and 3 at 3.5×.
    ///
    #[test]
    fn the_picture_is_whole_physical_pixels_per_texel_up_to_the_whole_pixels_per_point() {
        let window = Vec2::new(2000.0, 1400.0);
        for (pixels_per_point, pixels_per_texel) in [(1.0, 1.0), (1.5, 1.0), (2.0, 2.0), (3.5, 3.0)]
        {
            let picture = placed(window, pixels_per_point)
                .unwrap_or_else(|| panic!("at {pixels_per_point}×: the picture was not placed"));
            let physical = picture.size() * pixels_per_point;
            assert!(
                (physical - Vec2::splat(260.0 * pixels_per_texel)).length() < 1e-3,
                "at {pixels_per_point}×: the picture is {physical:?} physical pixels"
            );
            assert!(
                (picture.max - window.to_pos2()).length() < 1e-3,
                "at {pixels_per_point}×: the picture {picture:?} is not at the corner"
            );
        }
    }

    ///
    /// A window whose corner is not on a whole physical pixel gets the
    /// picture's corner rounded to one, so every texel lands on whole physical
    /// pixels, and keeps the picture's size.
    ///
    #[test]
    fn the_picture_is_placed_on_whole_physical_pixels_at_a_fractional_corner() {
        for pixels_per_point in [1.0, 1.25, 1.5, 2.0] {
            let window = Vec2::new(1000.3, 700.7);
            let picture = placed(window, pixels_per_point).expect("the picture fits");
            let physical = picture.min * pixels_per_point;
            assert_eq!(
                physical,
                physical.round(),
                "at {pixels_per_point}×: the corner {:?} is not on a whole physical pixel",
                picture.min
            );
            assert!(
                (picture.max - window.to_pos2()).length() * pixels_per_point <= 0.5_f32.hypot(0.5),
                "at {pixels_per_point}×: the picture {picture:?} is not at the corner"
            );
        }
    }

    ///
    /// A window too small for the picture at its usual size gets it at the
    /// largest whole number of physical pixels per texel that fits, in half the
    /// window's width and the height below the menu bar, and no Overview
    /// at all where one pixel per texel does not fit. A window that holds it
    /// again gets it back at its usual size.
    ///
    #[test]
    fn the_overview_shrinks_to_whole_pixels_per_texel_that_fit_and_hides_below_one() {
        let narrow = Vec2::new(500.0, DEFAULT_VIEW_SIZE[1]);
        let short = Vec2::new(DEFAULT_VIEW_SIZE[0], TOP_PANEL_HEIGHT + 259.0);
        let smallest = Vec2::from(DEFAULT_VIEW_SIZE_MIN);
        let usual = Vec2::from(DEFAULT_VIEW_SIZE);
        for (size, pixels_per_point, pixels_per_texel) in [
            (narrow, 2.0, Some(1.0)),
            (narrow, 1.5, Some(1.0)),
            (narrow, 1.0, None),
            (short, 2.0, Some(1.0)),
            (short, 1.0, None),
            (smallest, 1.0, None),
            (smallest, 2.0, Some(1.0)),
            (usual, 1.0, Some(1.0)),
            (usual, 1.5, Some(1.0)),
            (usual, 2.0, Some(2.0)),
        ] {
            let picture = placed(size, pixels_per_point);
            match pixels_per_texel {
                Some(pixels_per_texel) => {
                    let picture = picture.unwrap_or_else(|| {
                        panic!("{size:?} at {pixels_per_point}×: the Overview is hidden")
                    });
                    let physical = picture.size() * pixels_per_point;
                    assert!(
                        (physical - Vec2::splat(260.0 * pixels_per_texel)).length() < 1e-3,
                        "{size:?} at {pixels_per_point}×: the picture is {physical:?} physical \
                         pixels"
                    );
                    assert!(
                        picture.left() >= size.x / 2.0,
                        "{size:?} at {pixels_per_point}×: the picture {picture:?} takes more \
                         than half the width"
                    );
                    assert!(
                        (picture.max - size.to_pos2()).length() < 1e-3,
                        "{size:?} at {pixels_per_point}×: the picture {picture:?} is not in the \
                         window's bottom-right corner"
                    );
                }
                None => assert_eq!(
                    picture, None,
                    "{size:?} at {pixels_per_point}×: the Overview was shown"
                ),
            }
        }
    }
}
