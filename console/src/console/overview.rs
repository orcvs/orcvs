//! The Overview: the whole Grid and its margin, one texel per Cell in the
//! Theme's colours, docked at the right of the Source View.

use std::iter::repeat_n;
use std::sync::Arc;

use egui::emath::GuiRounding as _;
use egui::{Color32, ColorImage, Rect, Sense, TextureHandle, TextureOptions, Vec2};
use orcvs::render_frame::RenderFrame;
use orcvs::source::RevisionId;

use super::Console;
use super::source_view::SOURCE_MARGIN_CELLS;
use crate::style::{CellColours, CursorPlacement};
use crate::theme::{Appearance, Theme, ThemeIdentity};

/// The margin around the Grid in texels: one texel per Cell of margin.
const MARGIN: usize = SOURCE_MARGIN_CELLS as usize;

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
            if let Some(texture) = &mut self.texture {
                texture.set_partial([0, 0], Arc::clone(&self.picture), TextureOptions::NEAREST);
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

impl Console {
    ///
    /// Shows the Overview in a panel docked at the right, when it is open.
    /// Shown before the Source, so it takes width rather than covering Cells.
    ///
    /// The picture is drawn square, at a whole number of physical pixels per
    /// texel, on whole physical pixels, so every Cell is a crisp block of one
    /// size: the most that keeps it within the height beside the Source and
    /// half its width. Where not even one pixel per texel fits, it is scaled
    /// down to fit, so the whole Grid stays on it. It senses a drag and nothing else: sensing focus would put it in
    /// the Tab order, where a focused widget keeps every key from the Source.
    ///
    pub(super) fn show_overview(
        &mut self,
        root: &mut egui::Ui,
        revision: RevisionId,
        frame: &RenderFrame,
        appearance: Appearance,
    ) {
        if !self.overview_open {
            return;
        }
        let ctx = root.ctx().clone();
        let theme = self.themes.presented(appearance);
        let texture = self
            .overview
            .texture(&ctx, revision, frame, theme, appearance);
        let pixels_per_point = ctx.pixels_per_point();
        let panel_frame = egui::Frame::side_top_panel(root.style());
        let room = root.available_rect_before_wrap().size() * Vec2::new(0.5, 1.0)
            - panel_frame.total_margin().sum();
        let fits = (room / texture.size_vec2()).min_elem().max(0.0) * pixels_per_point;
        let whole = pixels_per_point.floor().max(1.0).min(fits.floor());
        let pixels_per_texel = if whole >= 1.0 { whole } else { fits };
        let side = texture.size_vec2() * pixels_per_texel / pixels_per_point;
        egui::Panel::right("overview")
            .resizable(false)
            .exact_size(side.x + panel_frame.total_margin().sum().x)
            .frame(panel_frame)
            .show(root, |ui| {
                let corner = ui
                    .available_rect_before_wrap()
                    .min
                    .round_to_pixels(pixels_per_point);
                let rect = Rect::from_min_size(corner, side);
                let response = ui.allocate_rect(rect, Sense::DRAG);
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::Other, true, "Source overview")
                });
                ui.painter().image(
                    texture.id(),
                    rect,
                    Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            });
    }
}

///
/// Draws `frame` into `picture` in `theme`'s colours: one texel per Cell,
/// row by row, with the margin around the Grid as Grid background.
///
/// Each Cell's colour is its Plain colours from the Cell colour table reduced
/// by a fixed rule: the role's foreground where the Cell is occupied,
/// otherwise its background where it has one, otherwise the Grid background;
/// composited over the Grid background, as the Source draws it. The Cursor and
/// the Region are not drawn, so moving them changes no texel.
///
/// `picture`'s pixels are cleared and refilled, so a picture redrawn at the
/// same size allocates nothing.
///
pub fn draw_picture(frame: &RenderFrame, theme: &Theme, picture: &mut ColorImage) {
    let grid = frame.grid();
    let width = grid.columns() + 2 * MARGIN;
    let height = grid.rows() + 2 * MARGIN;
    let background = theme.grid_background;
    let mut colours = CellColours::new(theme);

    picture.size = [width, height];
    picture.source_size = egui::vec2(width as f32, height as f32);
    let pixels = &mut picture.pixels;
    pixels.clear();
    pixels.extend(repeat_n(background, MARGIN * width));
    for row in frame.cells().chunks(grid.columns()) {
        pixels.extend(repeat_n(background, MARGIN));
        pixels.extend(row.iter().map(|cell| {
            let visuals = colours.colours(
                cell.source_paint(),
                cell.portal_roles(),
                CursorPlacement::Plain,
            );
            let colour = if cell.content().is_some() {
                visuals.foreground
            } else {
                visuals.background.unwrap_or(Color32::TRANSPARENT)
            };
            background.blend(colour)
        }));
        pixels.extend(repeat_n(background, MARGIN));
    }
    pixels.extend(repeat_n(background, MARGIN * width));
}

#[cfg(test)]
mod tests {
    use egui::{Color32, ColorImage};
    use orcvs::app::Orcvs;
    use orcvs::source::Tick;

    use super::{MARGIN, draw_picture};
    use crate::theme::{Theme, okabe_ito, orcvs_light};

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
        picture.pixels[(row + MARGIN) * picture.size[0] + column + MARGIN]
    }

    #[tokio::test]
    async fn an_empty_grid_and_its_margin_are_all_grid_background() {
        let orcvs = running_orcvs(6, 4);
        let theme = okabe_ito();
        let picture = picture(&orcvs, &theme);

        assert_eq!(picture.size, [6 + 2 * MARGIN, 4 + 2 * MARGIN]);
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
        for y in 0..height {
            for x in 0..width {
                let in_margin =
                    x < MARGIN || y < MARGIN || x >= width - MARGIN || y >= height - MARGIN;
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
    async fn another_theme_draws_another_picture() {
        let mut orcvs = running_orcvs(8, 2);
        write_row(&mut orcvs, 0, ".+0304");
        let dark = picture(&orcvs, &okabe_ito());
        let light = picture(&orcvs, &orcvs_light());

        assert_ne!(dark, light);
        assert_eq!(texel(&light, 7, 1), orcvs_light().grid_background);
        assert_eq!(texel(&light, 0, 0), orcvs_light().source_function);
    }
}
