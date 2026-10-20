//! PROTOTYPE — throwaway. Do not merge to main.
//!
//! Draws one small Grid two ways in "My Grayscale Dark" colours, to compare
//! how Cell borders meet. Today's drawing is the console itself.
//!
//! B. Lattice + overlays, as egui apps draw it: full-length grid lines once,
//!    then every bordered Cell strokes its whole outline on top.
//! C. Each edge once: every edge and every junction where lines cross is one
//!    non-overlapping, pixel-aligned rectangle, coloured by the highest-priority
//!    Cell touching it. Nothing is painted twice.
//!
//! cargo run -p console --example prototype_cell_borders

use std::sync::atomic::{AtomicBool, Ordering};

use eframe::egui::{
    self, Color32, CornerRadius, Event, FontId, Painter, Pos2, Rect, Shape, Stroke, StrokeKind,
    TextureHandle, TextureOptions, Vec2, ViewportCommand, emath::GuiRounding, epaint::RectShape,
};

const CELL: f32 = 16.0;

const KINDS: [&str; 6] = [
    "............",
    ".ooo.nn.ccc.",
    ".OOO.II.LLL.",
    ".oooLL.xxx..",
    ".ff.!.ooo...",
    "............",
];
const GLYPHS: [&str; 6] = [
    "            ",
    " add 12 #hi ",
    " 042 07 abc ",
    " addxy +*/  ",
    " fn ! abc   ",
    "            ",
];
const CURSOR: (usize, usize) = (7, 4);

/// When set, role borders are a visible translucent white instead of the
/// Theme's near-background `#111111A8`.
static VISIBLE_ROLE_BORDER: AtomicBool = AtomicBool::new(false);

fn hex(s: &str) -> Color32 {
    Color32::from_hex(s).expect("valid hex")
}

fn blend(base: Color32, on_top: Color32) -> Color32 {
    if on_top.a() == 255 {
        on_top
    } else {
        base.blend(on_top)
    }
}

fn grid_border() -> Color32 {
    hex("#33333380")
}

fn role_border() -> Color32 {
    if VISIBLE_ROLE_BORDER.load(Ordering::Relaxed) {
        hex("#FFFFFF60")
    } else {
        hex("#111111A8")
    }
}

#[derive(Clone, Copy)]
struct Look {
    background: Color32,
    foreground: Color32,
    /// The border's own colour, before it is composited over anything.
    border: Color32,
    width: f32,
    priority: u8,
}

fn look(code: char) -> Look {
    let role = |background: &str, foreground: &str| Look {
        background: hex(background),
        foreground: hex(foreground),
        border: role_border(),
        width: 0.5,
        priority: 1,
    };
    match code {
        'o' => role("#111111A8", "#C8C8C8"),
        'n' => role("#222222", "#A8A8A8"),
        'c' => role("#33333321", "#757070"),
        'f' => role("#333333", "#E6E6E6"),
        '!' => Look {
            background: hex("#CCCCCC"),
            foreground: hex("#111111A8"),
            border: hex("#CCCCCC"),
            width: 1.0,
            priority: 1,
        },
        'x' => Look {
            background: hex("#111111A8"),
            foreground: hex("#E6E6E6"),
            border: hex("#C8C8C8"),
            width: 0.5,
            priority: 2,
        },
        'O' | 'I' => Look {
            background: hex("#CCCCCC"),
            foreground: hex("#111111A8"),
            border: hex("#CCCCCC"),
            width: 1.0,
            priority: 3,
        },
        'L' => Look {
            background: hex("#000000CC"),
            foreground: hex("#111111A8"),
            border: hex("#C8C8C880"),
            width: 0.5,
            priority: 3,
        },
        // Unclaimed: the ordinary background, and only the grid line.
        _ => Look {
            background: hex("#111111A8"),
            foreground: Color32::TRANSPARENT,
            border: grid_border(),
            width: 0.5,
            priority: 0,
        },
    }
}

fn cursor_look() -> Look {
    Look {
        background: Color32::TRANSPARENT,
        foreground: hex("#FFFFFF"),
        border: hex("#E6E6E6"),
        width: 1.0,
        priority: 4,
    }
}

/// The colour an edge shows when it is painted once over the background: the
/// border composited over the grid line, and Diagnostic over the role border.
fn composited(l: Look) -> Color32 {
    match l.priority {
        0 => grid_border(),
        1 => blend(grid_border(), l.border),
        4 => l.border,
        // Diagnostic, Portal and Locked Cells here all hold content, so their
        // border composites over the role border as well as the grid line.
        _ => blend(blend(grid_border(), role_border()), l.border),
    }
}

fn cols() -> usize {
    KINDS[0].len()
}
fn rows() -> usize {
    KINDS.len()
}
fn code(x: usize, y: usize) -> char {
    KINDS[y].as_bytes()[x] as char
}
/// The border a Cell shows, the Cursor overriding its own.
fn border_at(x: isize, y: isize) -> Option<Look> {
    (x >= 0 && y >= 0 && (x as usize) < cols() && (y as usize) < rows()).then(|| {
        let (x, y) = (x as usize, y as usize);
        if (x, y) == CURSOR {
            cursor_look()
        } else {
            look(code(x, y))
        }
    })
}
fn winner(looks: impl IntoIterator<Item = Option<Look>>) -> Option<Look> {
    looks
        .into_iter()
        .flatten()
        .fold(None, |best: Option<Look>, l| match best {
            Some(b) if b.priority >= l.priority => Some(b),
            _ => Some(l),
        })
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Overlays,
    EachEdgeOnce,
}

fn paint_cells(painter: &Painter, origin: Pos2, cell: f32, ppp: f32) {
    let grid = Rect::from_min_size(origin, Vec2::new(cols() as f32, rows() as f32) * cell);
    painter.rect_filled(grid, CornerRadius::ZERO, hex("#111111A8"));
    for y in 0..rows() {
        for x in 0..cols() {
            let l = look(code(x, y));
            let r = Rect::from_min_size(
                origin + Vec2::new(x as f32, y as f32) * cell,
                Vec2::splat(cell),
            );
            painter.rect_filled(
                r.round_to_pixels(ppp),
                CornerRadius::ZERO,
                blend(hex("#111111A8"), l.background),
            );
        }
    }
}

fn paint_glyphs(painter: &Painter, origin: Pos2, cell: f32) {
    for y in 0..rows() {
        for x in 0..cols() {
            let ch = GLYPHS[y].as_bytes()[x] as char;
            if ch != ' ' {
                let fg = if (x, y) == CURSOR {
                    cursor_look().foreground
                } else {
                    look(code(x, y)).foreground
                };
                let centre = origin + (Vec2::new(x as f32, y as f32) + Vec2::splat(0.5)) * cell;
                painter.text(
                    centre,
                    egui::Align2::CENTER_CENTER,
                    ch,
                    FontId::monospace(11.5),
                    fg,
                );
            }
        }
    }
}

/// B: the common egui pattern. Full-length lines, then outlines on top in
/// priority order; painter order is the only priority rule.
fn paint_overlays(painter: &Painter, origin: Pos2, cell: f32) {
    let grid = Rect::from_min_size(origin, Vec2::new(cols() as f32, rows() as f32) * cell);
    let line = Stroke::new(0.5, grid_border());
    for x in 0..=cols() {
        let px = grid.min.x + x as f32 * cell;
        painter.line_segment([Pos2::new(px, grid.min.y), Pos2::new(px, grid.max.y)], line);
    }
    for y in 0..=rows() {
        let py = grid.min.y + y as f32 * cell;
        painter.line_segment([Pos2::new(grid.min.x, py), Pos2::new(grid.max.x, py)], line);
    }
    let mut groups: [Vec<Shape>; 5] = Default::default();
    for y in 0..rows() {
        for x in 0..cols() {
            let l = border_at(x as isize, y as isize).expect("in grid");
            if l.priority > 0 {
                let r = Rect::from_min_size(
                    origin + Vec2::new(x as f32, y as f32) * cell,
                    Vec2::splat(cell),
                );
                groups[l.priority as usize].push(Shape::Rect(RectShape::stroke(
                    r,
                    CornerRadius::ZERO,
                    Stroke::new(l.width, l.border),
                    StrokeKind::Middle,
                )));
            }
        }
    }
    for g in groups {
        painter.extend(g);
    }
}

/// C: every edge and every junction is one rectangle in physical pixels, and
/// no two rectangles overlap, so a translucent colour is never painted twice.
///
/// A line `w` physical pixels wide sits on its boundary pixel `b` at
/// `b - w/2 .. b - w/2 + w`: a 1 px line falls in the pixel right of / below the
/// boundary, a 2 px line straddles it. Lines thinner than a pixel draw one
/// pixel at proportionally less alpha, as epaint's thin-line path does.
fn paint_each_edge_once(painter: &Painter, origin: Pos2, cell: f32, ppp: f32) {
    let px = |p: f32| (p * ppp).round() as i32;
    let span = |l: Look| -> (i32, Color32) {
        let w = l.width * ppp;
        let c = composited(l);
        if w < 1.0 {
            (1, c.gamma_multiply(w))
        } else {
            (w.round() as i32, c)
        }
    };
    let band = |boundary: i32, width: i32| (boundary - width / 2, boundary - width / 2 + width);
    let x_at = |x: usize| px(origin.x + x as f32 * cell);
    let y_at = |y: usize| px(origin.y + y as f32 * cell);
    // One Mesh, straight to the GPU: epaint turns a filled rect a pixel wide into
    // a line segment and rounds a one-pixel one away entirely.
    let mesh = std::cell::RefCell::new(egui::Mesh::default());
    let fill = |x0: i32, x1: i32, y0: i32, y1: i32, c: Color32| {
        if x1 > x0 && y1 > y0 {
            let r = Rect::from_min_max(
                Pos2::new(x0 as f32 / ppp, y0 as f32 / ppp),
                Pos2::new(x1 as f32 / ppp, y1 as f32 / ppp),
            );
            mesh.borrow_mut().add_colored_rect(r, c);
        }
    };
    let (ix, iy) = (|v: usize| v as isize, |v: usize| v as isize);

    // The winning Look of each edge: vertical edge (x, y) lies on column line
    // x between rows y and y+1; horizontal edge (x, y) on row line y.
    let v_edge =
        |x: usize, y: usize| winner([border_at(ix(x) - 1, iy(y)), border_at(ix(x), iy(y))]);
    let h_edge =
        |x: usize, y: usize| winner([border_at(ix(x), iy(y) - 1), border_at(ix(x), iy(y))]);
    let h_band_at = |x: usize, y: usize| {
        // A junction is as wide as the widest vertical edge meeting it, and as
        // tall as the widest horizontal one.
        let verticals = [
            (y > 0).then(|| v_edge(x, y - 1)).flatten(),
            (y < rows()).then(|| v_edge(x, y)).flatten(),
        ];
        let horizontals = [
            (x > 0).then(|| h_edge(x - 1, y)).flatten(),
            (x < cols()).then(|| h_edge(x, y)).flatten(),
        ];
        let w = verticals
            .iter()
            .flatten()
            .map(|l| span(*l).0)
            .max()
            .unwrap_or(0);
        let h = horizontals
            .iter()
            .flatten()
            .map(|l| span(*l).0)
            .max()
            .unwrap_or(0);
        let top = winner(verticals.into_iter().chain(horizontals));
        (band(x_at(x), w), band(y_at(y), h), top)
    };

    for y in 0..=rows() {
        for x in 0..=cols() {
            let ((x0, x1), (y0, y1), top) = h_band_at(x, y);
            if let Some(top) = top {
                fill(x0, x1, y0, y1, span(top).1);
            }
            // The vertical edge below this junction, up to the next one.
            if y < rows() {
                if let Some(l) = v_edge(x, y) {
                    let (next_top, _) = h_band_at(x, y + 1).1;
                    let (w, c) = span(l);
                    let (lx0, lx1) = band(x_at(x), w);
                    fill(lx0, lx1, y1, next_top, c);
                }
            }
            // The horizontal edge right of this junction, up to the next one.
            if x < cols() {
                if let Some(l) = h_edge(x, y) {
                    let (next_left, _) = h_band_at(x + 1, y).0;
                    let (h, c) = span(l);
                    let (ly0, ly1) = band(y_at(y), h);
                    fill(x1, next_left, ly0, ly1, c);
                }
            }
        }
    }
    painter.add(Shape::mesh(mesh.into_inner()));
}

fn paint(painter: &Painter, origin: Pos2, mode: Mode, ppp: f32) {
    // The console's own snap: a whole number of physical pixels per Cell.
    let cell = (CELL * ppp).floor().max(1.0) / ppp;
    paint_cells(painter, origin, cell, ppp);
    match mode {
        Mode::Overlays => paint_overlays(painter, origin, cell),
        Mode::EachEdgeOnce => paint_each_edge_once(painter, origin, cell, ppp),
    }
    paint_glyphs(painter, origin, cell);
}

const MODES: [(Mode, &str, &str); 2] = [
    (
        Mode::Overlays,
        "B. Lattice + overlays (as egui apps draw it)",
        "Grid lines drawn once as full-length lines, then every bordered Cell strokes its whole outline on top. A shared edge between two bordered Cells is painted twice, and so is every crossing.",
    ),
    (
        Mode::EachEdgeOnce,
        "C. Each edge once",
        "Every edge, and every point where lines cross, is one rectangle coloured by the highest-priority Cell touching it: Cursor > Portal/Locked > Diagnostic > role > grid. Nothing is painted twice.",
    ),
];

/// Marks this app's own screenshots, so an inspection client's are ignored.
struct Loupe;

struct App {
    panels: [Rect; 2],
    loupes: Option<[TextureHandle; 2]>,
    captured_at: f32,
    scale: f32,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let ppp = ctx.pixels_per_point();

        let shot = ctx.input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                Event::Screenshot {
                    image, user_data, ..
                } if user_data.data.as_ref().is_some_and(|d| d.is::<Loupe>()) => {
                    Some(image.clone())
                }
                _ => None,
            })
        });
        // A screenshot taken at another scale or window size cannot be cropped
        // by this frame's panel rects; skip it rather than read out of bounds.
        let fits = |image: &egui::ColorImage| {
            let screen = ctx.content_rect();
            let [w, h] = image.size;
            (w as f32 - screen.width() * ppp).abs() < 2.0
                && (h as f32 - screen.height() * ppp).abs() < 2.0
                && self
                    .panels
                    .iter()
                    .all(|p| p.is_positive() && screen.contains_rect(*p))
        };
        let mut capture = false;
        if let Some(image) = shot.filter(|image| fits(image)) {
            self.loupes = Some(std::array::from_fn(|i| {
                let crop = image.region(&self.panels[i], Some(ppp));
                ctx.load_texture(format!("loupe{i}"), crop, TextureOptions::NEAREST)
            }));
            self.captured_at = ppp;
        } else if self.captured_at != ppp && self.panels[0].is_positive() {
            // Capture once after the first layout and after every change.
            self.captured_at = ppp;
            capture = true;
        }

        egui::Frame::new().fill(hex("#111111")).inner_margin(16.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.colored_label(hex("#C8C8C8"), format!("{ppp} physical pixels per point"));
                for (label, wanted) in [("1 px/pt (non-Retina)", 1.0), ("2 px/pt (Retina)", 2.0)] {
                    if ui.button(label).clicked() {
                        ctx.set_zoom_factor(wanted / ctx.native_pixels_per_point().unwrap_or(1.0));
                    }
                }
                let mut visible = VISIBLE_ROLE_BORDER.load(Ordering::Relaxed);
                if ui.checkbox(&mut visible, "visible translucent role border (#FFFFFF60)").changed() {
                    VISIBLE_ROLE_BORDER.store(visible, Ordering::Relaxed);
                    self.captured_at = 0.0;
                }
                if ui.button("Re-capture").clicked() {
                    capture = true;
                }
            });
            ui.colored_label(
                hex("#A8A8A8"),
                "Rows: empty · `add` ordinary, `12` number, `#hi` comment · Output Portal, Input Portal, Locked · `add` next to Locked `xy`, Diagnostic `+*/` · function `fn`, bang `!`, `abc` with the Cursor on `b`. Right of each Grid: the same pixels magnified.",
            );

            let size = Vec2::new(cols() as f32, rows() as f32) * CELL;
            for (i, (mode, title, explain)) in MODES.into_iter().enumerate() {
                ui.add_space(16.0);
                ui.colored_label(hex("#E6E6E6"), title);
                ui.colored_label(hex("#A8A8A8"), explain);
                ui.horizontal_top(|ui| {
                    let (rect, _) = ui.allocate_exact_size(size + Vec2::splat(4.0), egui::Sense::hover());
                    let origin = (rect.min + Vec2::splat(2.0)).round_to_pixels(ppp);
                    paint(ui.painter(), origin, mode, ppp);
                    self.panels[i] = rect;
                    ui.add_space(16.0);
                    if let Some(loupes) = &self.loupes {
                        let tex = &loupes[i];
                        ui.image((tex.id(), tex.size_vec2() * self.scale / ppp));
                    }
                });
            }
        });

        if capture {
            ctx.send_viewport_cmd(ViewportCommand::Screenshot(egui::UserData::new(Loupe)));
        }
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "PROTOTYPE cell borders",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 820.0]),
            ..Default::default()
        },
        Box::new(|cc| {
            // The console's own font: egui's default set is off in this crate.
            let mut fonts = egui::FontDefinitions::empty();
            fonts.font_data.insert(
                "MonaspaceNeon".to_owned(),
                egui::FontData::from_static(include_bytes!("../assets/MonaspaceNeon-Regular.otf"))
                    .into(),
            );
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                fonts
                    .families
                    .insert(family, vec!["MonaspaceNeon".to_owned()]);
            }
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(App {
                panels: [Rect::NOTHING; 2],
                loupes: None,
                captured_at: 0.0,
                scale: 3.0,
            }))
        }),
    )
}
