//! The Overview: the whole Grid, one texel per Cell in the Theme's colours,
//! floating over the Source's bottom-right corner.

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
/// Where the picture of `grid`'s extent lies on `source`, the Source's rect:
/// pinned so its corner is the Source's bottom-right corner, on whole
/// physical pixels.
///
/// It is square at one physical pixel per texel, so every Cell is a crisp
/// block of one pixel. It answers `None` where the picture, with a `border`
/// along its inner edges, does not fit in half the Source's width and its
/// whole height.
///
fn place_picture(source: Rect, border: f32, grid: Grid, pixels_per_point: f32) -> Option<Rect> {
    let side = extent(grid).size() / pixels_per_point;
    let room = Vec2::new(source.width() / 2.0, source.height()) - Vec2::splat(border);
    (side.x <= room.x && side.y <= room.y)
        .then(|| Rect::from_min_size((source.max - side).round_to_pixels(pixels_per_point), side))
}

impl Console {
    ///
    /// Shows the Overview pinned to the Source's bottom-right corner, when it
    /// is open and fits, and answers the centre a press or drag on it asks
    /// the Source View to Pan to. It floats over the Source, so it takes no
    /// width from it, and stays above the bottom panel, with no margin: the
    /// bottom panel's border along the two edges that face the rest of the
    /// Source, and the picture filling the rest, placed by [`place_picture`].
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
        revision: RevisionId,
        frame: &RenderFrame,
        appearance: Appearance,
    ) -> Option<SourceCellPoint> {
        if !self.overview_open {
            return None;
        }
        let ctx = root.ctx().clone();
        let theme = self.themes.presented(appearance);
        // The line the bottom panel draws along its edge facing the Source.
        let border = crate::style::bar_separator(theme);
        let source = self.source_view.shown();
        let rect = place_picture(
            source.console(),
            border.width,
            frame.grid(),
            ctx.pixels_per_point(),
        )?;
        let texture = self
            .overview
            .texture(&ctx, revision, frame, theme, appearance);
        // `place_picture` sized the picture from the extent, so whole pixels
        // per texel hold only while the texture is drawn from that extent.
        debug_assert_eq!(texture.size_vec2(), extent(frame.grid()).size());
        let texture = texture.id();
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
/// Cell of the extent, row by row.
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
    let extent = extent(frame.grid());
    let (width, height) = (extent.size().x as usize, extent.size().y as usize);
    let mut colours = CellColours::new(theme);

    picture.size = [width, height];
    picture.source_size = egui::vec2(width as f32, height as f32);
    let pixels = &mut picture.pixels;
    pixels.clear();
    pixels.extend(
        frame
            .cells()
            .iter()
            .map(|cell| cell_texel(&mut colours, cell, CursorPlacement::Plain)),
    );
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
    let occupied = cell.content().is_some();
    let displayed = colours.displayed(
        cell.source_paint(),
        cell.portal_roles(),
        occupied,
        placement,
    );
    if occupied {
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
    use crate::theme::{Appearance, Theme};
    use crate::theme_registry::tests_support::default_theme;

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

    /// The texel of the Cell at `column` and `row`.
    fn texel(picture: &ColorImage, column: usize, row: usize) -> Color32 {
        picture.pixels[row * picture.size[0] + column]
    }

    #[tokio::test]
    async fn an_empty_grid_is_all_grid_background() {
        let orcvs = running_orcvs(6, 4);
        let theme = default_theme(Appearance::Dark);
        let picture = picture(&orcvs, &theme);

        assert_eq!(picture.size, [6, 4]);
        assert!(
            picture
                .pixels
                .iter()
                .all(|texel| *texel == theme.grid_background),
            "an empty Grid drew something other than the Grid background"
        );
    }

    #[tokio::test]
    async fn each_occupied_cell_takes_its_roles_foreground_and_an_empty_one_stays_grid_background()
    {
        let mut orcvs = running_orcvs(8, 4);
        write_row(&mut orcvs, 0, ".+0304");
        write_row(&mut orcvs, 1, "**");
        write_row(&mut orcvs, 2, "||ab");
        write_row(&mut orcvs, 3, ".+XY");
        orcvs.select(orcvs.grid().position(7, 3).expect("inside the grid"));
        let theme = default_theme(Appearance::Dark);
        let picture = picture(&orcvs, &theme);

        for column in 0..2 {
            assert_eq!(texel(&picture, column, 0), theme.source_function);
            assert_eq!(texel(&picture, column, 1), theme.source_bang);
            assert_eq!(texel(&picture, column, 2), theme.source_comment);
            assert_eq!(texel(&picture, column + 2, 0), theme.source_number);
            assert_eq!(texel(&picture, column + 2, 3), theme.diagnostic_foreground);
        }
        assert_eq!(picture.size, [8, 4]);
        for (column, row) in [(6, 0), (2, 1), (7, 2), (6, 3)] {
            assert_eq!(
                texel(&picture, column, row),
                theme.grid_background,
                "empty texel ({column}, {row})"
            );
        }
    }

    #[tokio::test]
    async fn an_empty_cell_a_committed_tick_read_takes_its_portal_background() {
        // `&v` reads the pair one row south of its operand, (2, 1) and (3, 1).
        let mut orcvs = running_orcvs(4, 4);
        write_row(&mut orcvs, 0, "&v01");
        let theme = default_theme(Appearance::Dark);
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
        let mut theme = default_theme(Appearance::Dark);
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
        assert!(
            picture.pixels.iter().all(|texel| texel.a() == 255),
            "a texel is translucent"
        );
    }

    #[tokio::test]
    async fn another_theme_draws_another_picture() {
        let mut orcvs = running_orcvs(8, 2);
        write_row(&mut orcvs, 0, ".+0304");
        let dark = picture(&orcvs, &default_theme(Appearance::Dark));
        let light = picture(&orcvs, &default_theme(Appearance::Light));

        assert_ne!(dark, light);
        assert_eq!(
            texel(&light, 7, 1),
            default_theme(Appearance::Light).grid_background
        );
        assert_eq!(
            texel(&light, 0, 0),
            default_theme(Appearance::Light).source_function
        );
    }

    /// The bottom panel's separator, which borders the picture's inner edges.
    const BORDER: f32 = 1.0;

    /// The picture's rect on a Source of `size` below a menu bar.
    fn placed(size: Vec2, pixels_per_point: f32) -> Option<Rect> {
        place_picture(source(size), BORDER, Grid::new(), pixels_per_point)
    }

    fn source(size: Vec2) -> Rect {
        Rect::from_min_size(Pos2::new(0.0, 52.0), size)
    }

    ///
    /// The 256 by 256 Grid is 256 texels a side, placed square in the Source's
    /// bottom-right corner at one physical pixel per texel at every pixels per
    /// point: 256 points a side at 1×, 128 at 2×.
    ///
    #[test]
    fn the_picture_is_one_physical_pixel_per_texel_in_the_sources_corner() {
        let size = Vec2::new(2000.0, 1400.0);
        for (pixels_per_point, side) in [
            (1.0, 256.0),
            (1.5, 170.666_67),
            (2.0, 128.0),
            (3.5, 73.142_86),
        ] {
            let picture = placed(size, pixels_per_point)
                .unwrap_or_else(|| panic!("at {pixels_per_point}×: the picture was not placed"));
            assert!(
                (picture.size() - Vec2::splat(side)).length() < 1e-3,
                "at {pixels_per_point}×: the picture is {:?} points",
                picture.size()
            );
            assert!(
                (picture.max - source(size).max).length() * pixels_per_point
                    <= 0.5_f32.hypot(0.5) + 1e-3,
                "at {pixels_per_point}×: the picture {picture:?} is not at the Source's corner"
            );
        }
    }

    ///
    /// A Source whose corner is not on a whole physical pixel gets the
    /// picture's corner rounded to one, so every texel lands on whole physical
    /// pixels, and keeps the picture's size.
    ///
    #[test]
    fn the_picture_is_placed_on_whole_physical_pixels_at_a_fractional_corner() {
        for pixels_per_point in [1.0, 1.25, 1.5, 2.0] {
            let size = Vec2::new(1000.3, 620.8);
            let picture = placed(size, pixels_per_point).expect("the picture fits");
            let physical = picture.min * pixels_per_point;
            assert_eq!(
                physical,
                physical.round(),
                "at {pixels_per_point}×: the corner {:?} is not on a whole physical pixel",
                picture.min
            );
            assert!(
                (picture.max - source(size).max).length() * pixels_per_point
                    <= 0.5_f32.hypot(0.5) + 1e-3,
                "at {pixels_per_point}×: the picture {picture:?} is not at the Source's corner"
            );
        }
    }

    ///
    /// The picture and its one-point border fit in half the Source's width and
    /// its whole height, or there is no Overview: at 1× a 256-point picture
    /// needs a Source 514 points wide and 257 high, at 2× a 128-point one
    /// needs 258 wide and 129 high.
    ///
    #[test]
    fn the_overview_hides_where_one_pixel_per_texel_does_not_fit_in_the_source() {
        for (size, pixels_per_point, shown) in [
            (Vec2::new(514.0, 600.0), 1.0, true),
            (Vec2::new(513.0, 600.0), 1.0, false),
            (Vec2::new(1000.0, 257.0), 1.0, true),
            (Vec2::new(1000.0, 256.0), 1.0, false),
            (Vec2::new(258.0, 600.0), 2.0, true),
            (Vec2::new(257.0, 600.0), 2.0, false),
            (Vec2::new(1000.0, 129.0), 2.0, true),
            (Vec2::new(1000.0, 128.0), 2.0, false),
        ] {
            assert_eq!(
                placed(size, pixels_per_point).is_some(),
                shown,
                "a {size:?} Source at {pixels_per_point}×"
            );
        }
    }
}
