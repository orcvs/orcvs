use std::time::Duration;

use egui::{Color32, Pos2, Rect, Shape, Stroke, Vec2, epaint::RectShape};

use crate::grid_viewport::CELL_SIZE;

pub(crate) const DEFAULT_CURSOR_COLOUR: Color32 = Color32::from_rgb(234, 235, 229);
pub(crate) const DEFAULT_AREA_COLOUR: Color32 = Color32::from_rgb(76, 190, 156);
/// White at 17% opacity: 0.17 × 255, rounded.
pub(crate) const DEFAULT_REGION_COLOUR: Color32 =
    Color32::from_rgba_unmultiplied_const(255, 255, 255, 43);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CursorEffectSettings {
    cursor_colour: Color32,
    area_colour: Color32,
    cell_colour: Option<Color32>,
    region_colour: Color32,
    region_cursor_colour: Option<Color32>,
    amount: u8,
    frequency: u8,
}

impl Default for CursorEffectSettings {
    fn default() -> Self {
        Self {
            cursor_colour: DEFAULT_CURSOR_COLOUR,
            area_colour: DEFAULT_AREA_COLOUR,
            cell_colour: None,
            region_colour: DEFAULT_REGION_COLOUR,
            region_cursor_colour: None,
            amount: 60,
            frequency: 55,
        }
    }
}

impl CursorEffectSettings {
    pub(crate) fn cursor_colour(self) -> Color32 {
        self.cursor_colour
    }
    pub(crate) fn area_colour(self) -> Color32 {
        self.area_colour
    }
    pub(crate) fn cell_colour(self) -> Option<Color32> {
        self.cell_colour
    }
    /// The fill of every Cell a Region larger than one Cell covers, with its
    /// opacity.
    pub(crate) fn region_colour(self) -> Color32 {
        self.region_colour
    }
    pub(crate) fn amount(self) -> u8 {
        self.amount
    }
    pub(crate) fn frequency(self) -> u8 {
        self.frequency
    }

    pub(crate) fn cursor_colour_mut(&mut self) -> &mut Color32 {
        &mut self.cursor_colour
    }
    pub(crate) fn area_colour_mut(&mut self) -> &mut Color32 {
        &mut self.area_colour
    }
    pub(crate) fn region_colour_mut(&mut self) -> &mut Color32 {
        &mut self.region_colour
    }
    /// The fill of the Cursor's Cell while a Region larger than one Cell is
    /// selected, independent of the rest of the Region. `None` leaves that
    /// Cell as the Cursor cell colour would have it.
    pub(crate) fn region_cursor_colour(self) -> Option<Color32> {
        self.region_cursor_colour
    }
    pub(crate) fn set_region_cursor_colour(&mut self, colour: Option<Color32>) {
        self.region_cursor_colour = colour;
    }
    pub(crate) fn set_cell_colour(&mut self, colour: Option<Color32>) {
        self.cell_colour = colour;
    }
    pub fn amount_mut(&mut self) -> &mut u8 {
        &mut self.amount
    }
    pub(crate) fn frequency_mut(&mut self) -> &mut u8 {
        &mut self.frequency
    }

    pub(crate) fn respecting_reduced_motion(mut self, reduced_motion: bool) -> Self {
        if reduced_motion {
            self.amount = 0;
            self.frequency = 0;
        }
        self
    }

    pub(crate) fn interval(self, variation: u8) -> Option<Duration> {
        (self.frequency > 0).then(|| {
            let fastest = 45_u64;
            let slowest = 190_u64;
            let base = slowest - (slowest - fastest) * u64::from(self.frequency) / 100;
            Duration::from_millis(base + u64::from(variation) * base / 255)
        })
    }

    #[cfg(any(feature = "persistence", test))]
    pub(crate) fn encode(self) -> String {
        let [red, green, blue, alpha] = self.region_colour.to_srgba_unmultiplied();
        let region = (red, green, blue, alpha);
        format!(
            "{},{},{};{},{},{};{};{};{};{},{},{},{};{}",
            self.cursor_colour.r(),
            self.cursor_colour.g(),
            self.cursor_colour.b(),
            self.area_colour.r(),
            self.area_colour.g(),
            self.area_colour.b(),
            self.amount,
            self.frequency,
            self.cell_colour.map_or_else(
                || "none".to_owned(),
                |colour| { format!("{},{},{}", colour.r(), colour.g(), colour.b()) }
            ),
            region.0,
            region.1,
            region.2,
            region.3,
            self.region_cursor_colour.map_or_else(
                || "none".to_owned(),
                |colour| {
                    let [red, green, blue, alpha] = colour.to_srgba_unmultiplied();
                    format!("{red},{green},{blue},{alpha}")
                }
            ),
        )
    }

    #[cfg(any(feature = "persistence", test))]
    pub(crate) fn decode(value: &str) -> Option<Self> {
        let mut groups = value.split(';');
        let colour = |group: &str| -> Option<Color32> {
            let channels = group
                .split(',')
                .map(str::parse::<u8>)
                .collect::<Result<Vec<_>, _>>()
                .ok()?;
            (channels.len() == 3).then(|| Color32::from_rgb(channels[0], channels[1], channels[2]))
        };
        let translucent = |group: &str| -> Option<Color32> {
            let channels = group
                .split(',')
                .map(str::parse::<u8>)
                .collect::<Result<Vec<_>, _>>()
                .ok()?;
            (channels.len() == 4).then(|| {
                Color32::from_rgba_unmultiplied(channels[0], channels[1], channels[2], channels[3])
            })
        };
        let settings = Self {
            cursor_colour: colour(groups.next()?)?,
            area_colour: colour(groups.next()?)?,
            amount: groups.next()?.parse().ok()?,
            frequency: groups.next()?.parse().ok()?,
            cell_colour: match groups.next() {
                None | Some("none") => None,
                Some(group) => colour(group),
            },
            // Settings saved before the Region fill was themeable carry no
            // sixth group, and take the default.
            region_colour: match groups.next() {
                None => DEFAULT_REGION_COLOUR,
                Some(group) => translucent(group)?,
            },
            // Likewise for the Cursor's colour within a Region, which is unset
            // by default.
            region_cursor_colour: match groups.next() {
                None | Some("none") => None,
                Some(group) => Some(translucent(group)?),
            },
        };
        (groups.next().is_none() && settings.amount <= 100 && settings.frequency <= 100)
            .then_some(settings)
    }
}

const AREA_RADIUS_CELLS: f32 = 7.0;

/// The console-owned clock for the Cursor's changing presentation.
///
/// Deadlines move from the time they are observed rather than from their old
/// value. A sleeping or clipped console therefore advances to one fresh state
/// instead of replaying every state nobody saw.
#[derive(Clone, Copy, Debug)]
pub struct CursorEffectAnimation {
    seed: u64,
    frame_epoch: u64,
    field_epoch: u64,
    next_frame: Duration,
    next_field: Duration,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CursorEffectSample {
    frame: u64,
    grain: u64,
    field: u64,
}

fn animation_seed() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        // `SystemTime::now()` panics on `wasm32-unknown-unknown`; per-session
        // variation comes from the console's elapsed-time clock in `advance`.
        0x6f72_6376_735f_6375
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0x6f72_6376_735f_6375, |elapsed| elapsed.as_nanos() as u64)
    }
}

impl Default for CursorEffectAnimation {
    fn default() -> Self {
        Self {
            seed: hash(animation_seed()),
            frame_epoch: 0,
            field_epoch: 0,
            next_frame: Duration::ZERO,
            next_field: Duration::ZERO,
        }
    }
}

impl CursorEffectAnimation {
    pub fn advance(&mut self, now: Duration, settings: CursorEffectSettings) -> CursorEffectSample {
        if settings.frequency() > 0 && now >= self.next_frame {
            self.frame_epoch = self.frame_epoch.wrapping_add(1);
            let variation = hash(self.seed ^ self.frame_epoch) as u8;
            self.next_frame = now
                + settings
                    .interval(variation)
                    .expect("a positive frequency has an interval");
        }
        if settings.frequency() > 0 && now >= self.next_field {
            self.field_epoch = self.field_epoch.wrapping_add(1);
            let variation = hash(self.seed ^ self.field_epoch.rotate_left(23)) as u8;
            self.next_field = now
                + settings
                    .interval(variation)
                    .expect("a positive frequency has an interval")
                    .saturating_mul(3);
        }

        CursorEffectSample {
            frame: hash(self.seed ^ self.frame_epoch),
            grain: hash(self.seed.rotate_left(17) ^ self.frame_epoch),
            field: hash(self.seed.rotate_left(37) ^ self.field_epoch),
        }
    }

    pub(crate) fn repaint_after(
        self,
        now: Duration,
        settings: CursorEffectSettings,
    ) -> Option<Duration> {
        (settings.frequency() > 0).then(|| self.next_frame.min(self.next_field).saturating_sub(now))
    }
}

#[derive(Debug, Default)]
pub struct CursorEffectShapes {
    pub(crate) area: Vec<Shape>,
    pub(crate) frame: Vec<Shape>,
}

pub(crate) fn effect_bounds(cursor: Rect, cell_size: f32) -> Rect {
    cursor.expand((AREA_RADIUS_CELLS + 1.0) * cell_size)
}

/// Builds the approved green damaged field and off-white eroded frame.
///
/// The field centres on `cursor`, the Cursor's Cell. The frame outlines
/// `outline`: the Cursor's Cell on its own, or the whole Region when the
/// Region is larger than one Cell, which is the lasso. Either way the frame is
/// built one Cell-length of edge at a time, so a Region's lasso has the
/// Cursor's fragment density and stroke weight however far it reaches.
///
/// Candidates are fixed in number per Cell-length and rejected against `clip`
/// before a Shape is allocated, so work follows the visible edge rather than
/// the Grid.
pub fn cursor_effect_shapes(
    cursor: Rect,
    outline: Rect,
    clip: Rect,
    cell_size: f32,
    sample: CursorEffectSample,
    settings: CursorEffectSettings,
) -> CursorEffectShapes {
    let mut shapes = CursorEffectShapes::default();
    if !effect_bounds(cursor, cell_size)
        .union(outline.expand(cell_size))
        .intersects(clip)
    {
        return shapes;
    }

    let amount = f32::from(settings.amount()) / 100.0;
    if amount > 0.0 {
        area_shapes(
            &mut shapes.area,
            cursor,
            clip,
            cell_size,
            sample,
            settings,
            amount,
        );
    }
    frame_shapes(
        &mut shapes.frame,
        outline,
        cell_size,
        clip,
        sample,
        settings,
        amount,
    );
    shapes
}

fn area_shapes(
    out: &mut Vec<Shape>,
    cursor: Rect,
    clip: Rect,
    cell_size: f32,
    sample: CursorEffectSample,
    settings: CursorEffectSettings,
    amount: f32,
) {
    let reach = AREA_RADIUS_CELLS * cell_size;
    let centre = cursor.center();
    let colour = settings.area_colour();

    // Long-tailed horizontal tears. Several nearby candidates share the slow
    // field seed, making concentrations and empty patches move as regions;
    // the grain seed changes the detail within them more often.
    for candidate in 0..240_u64 {
        let field_bucket = candidate / 12;
        let density = unit(hash(sample.field ^ field_bucket.wrapping_mul(0x9e37_79b9)));
        if density < 0.34 || unit(hash(sample.grain ^ candidate)) > density * amount {
            continue;
        }
        let x = centre.x + signed(hash(sample.field ^ candidate.rotate_left(9))) * reach;
        let y = centre.y + signed(hash(sample.field ^ candidate.rotate_left(29))) * reach;
        let envelope = 1.0 - ((x - centre.x).abs().max((y - centre.y).abs()) / reach);
        if envelope <= 0.0 {
            continue;
        }
        let length_noise = unit(hash(sample.grain ^ candidate.rotate_left(41)));
        let width = cell_size * (0.025 + length_noise.powi(3) * 0.9);
        let height = (0.35 + unit(hash(sample.grain ^ !candidate)) * 0.8)
            * (cell_size / CELL_SIZE).max(0.25);
        let rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(width, height));
        if !rect.intersects(clip) {
            continue;
        }
        let alpha = (colour.a() as f32 * envelope * (0.08 + density * 0.22) * amount)
            .round()
            .clamp(1.0, 255.0) as u8;
        out.push(Shape::Rect(RectShape::filled(
            rect,
            0.0,
            Color32::from_rgba_unmultiplied(colour.r(), colour.g(), colour.b(), alpha),
        )));
    }

    // A few fine connections stop the islands reading as loose confetti.
    for strand in 0..14_u64 {
        let anchor = hash(sample.field ^ strand.wrapping_mul(0xd6e8_feb8));
        let start = Pos2::new(
            centre.x + signed(anchor) * reach * 0.85,
            centre.y + signed(hash(anchor)) * reach * 0.85,
        );
        let end = start
            + Vec2::new(
                signed(hash(anchor.rotate_left(17))) * cell_size * 1.4,
                signed(hash(anchor.rotate_left(43))) * cell_size * 0.5,
            );
        let bound = Rect::from_two_pos(start, end).expand(1.0);
        if bound.intersects(clip) {
            out.push(Shape::line_segment(
                [start, end],
                Stroke::new(
                    (cell_size / 50.0).max(0.35),
                    Color32::from_rgba_unmultiplied(
                        colour.r(),
                        colour.g(),
                        colour.b(),
                        (20.0 * amount) as u8,
                    ),
                ),
            ));
        }
    }
}

fn frame_shapes(
    out: &mut Vec<Shape>,
    outline: Rect,
    cell_size: f32,
    clip: Rect,
    sample: CursorEffectSample,
    settings: CursorEffectSettings,
    amount: f32,
) {
    let colour = settings.cursor_colour();
    let side = cell_size;
    let scale = side / CELL_SIZE;
    if amount == 0.0 {
        out.push(Shape::Rect(RectShape::stroke(
            outline,
            0.0,
            Stroke::new(scale.max(0.5), colour),
            egui::StrokeKind::Inside,
        )));
        return;
    }
    // How many Cell-lengths each pair of edges spans. A Cell's frame is one of
    // each, and its keys are the ones the unit index zero leaves unchanged.
    let across = ((outline.width() / side).round() as u64).max(1);
    let down = ((outline.height() / side).round() as u64).max(1);
    for edge in 0..4_u64 {
        let units = if edge % 2 == 0 { across } else { down };
        for unit_index in 0..units {
            let span = 1.0 / units as f32;
            let offset = unit_index as f32 * span;
            for fragment in 0..8_u64 {
                let key = sample.frame
                    ^ edge.wrapping_mul(0x9e37_79b9)
                    ^ fragment.rotate_left(19)
                    ^ unit_index.wrapping_mul(0xc2b2_ae3d_27d4_eb4f);
                // Two fragments per Cell-length are unconditional, keeping the
                // outline exact and legible even when a noisy sample erodes
                // most of a side.
                if fragment >= 2 && unit(hash(key)) > 0.42 + amount * 0.42 {
                    continue;
                }
                let start = unit(hash(key.rotate_left(11))) * 0.88;
                let length = 0.05 + unit(hash(key.rotate_left(31))).powi(2) * (0.2 + amount * 0.18);
                let end = (start + length).min(0.98);
                let outward = signed(hash(key.rotate_left(47))) * side * 0.045 * amount;
                let (a, b) = edge_segment(
                    outline,
                    edge,
                    offset + start * span,
                    offset + end * span,
                    outward,
                );
                let bound = Rect::from_two_pos(a, b).expand(2.0 * scale);
                if bound.intersects(clip) {
                    let alpha = (105.0 + unit(hash(!key)) * 150.0) as u8;
                    out.push(Shape::line_segment(
                        [a, b],
                        Stroke::new(
                            (0.45 + unit(hash(key ^ 73)) * 0.8) * scale,
                            colour.gamma_multiply(f32::from(alpha) / 255.0),
                        ),
                    ));
                }
            }
        }
    }

    // Short horizontal displaced signal tears, independently placed from the
    // edge fragments so the frame never settles into one diagonal motif. Each
    // Cell-length of height carries its own.
    for row in 0..down {
        let top = outline.top() + row as f32 * side;
        for tear in 0..7_u64 {
            let key =
                hash(sample.frame ^ tear.wrapping_mul(0xa076_1d65) ^ row.wrapping_mul(0x1656_67b1));
            let y = top + unit(key) * side;
            let from_right = key & 1 == 0;
            let length = side * (0.1 + unit(hash(key)) * 0.45) * amount;
            let x = if from_right {
                outline.right()
            } else {
                outline.left() - length
            };
            let rect = Rect::from_min_size(
                Pos2::new(x, y + signed(hash(!key)) * side * 0.08),
                Vec2::new(length, (0.4 + unit(hash(key.rotate_left(7)))) * scale),
            );
            if rect.intersects(clip) {
                out.push(Shape::Rect(RectShape::filled(
                    rect,
                    0.0,
                    colour.gamma_multiply(0.2 + unit(hash(key.rotate_left(13))) * 0.5),
                )));
            }
        }
    }
}

fn edge_segment(rect: Rect, edge: u64, start: f32, end: f32, outward: f32) -> (Pos2, Pos2) {
    match edge {
        0 => (
            Pos2::new(rect.left() + rect.width() * start, rect.top() + outward),
            Pos2::new(rect.left() + rect.width() * end, rect.top() + outward),
        ),
        1 => (
            Pos2::new(rect.right() + outward, rect.top() + rect.height() * start),
            Pos2::new(rect.right() + outward, rect.top() + rect.height() * end),
        ),
        2 => (
            Pos2::new(rect.right() - rect.width() * start, rect.bottom() + outward),
            Pos2::new(rect.right() - rect.width() * end, rect.bottom() + outward),
        ),
        _ => (
            Pos2::new(rect.left() - outward, rect.bottom() - rect.height() * start),
            Pos2::new(rect.left() - outward, rect.bottom() - rect.height() * end),
        ),
    }
}

fn hash(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn unit(value: u64) -> f32 {
    (value as u32) as f32 / u32::MAX as f32
}

fn signed(value: u64) -> f32 {
    unit(value) * 2.0 - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(frame: u64, grain: u64, field: u64) -> CursorEffectSample {
        CursorEffectSample {
            frame,
            grain,
            field,
        }
    }

    #[test]
    fn default_animation_constructs_on_every_target() {
        let _ = CursorEffectAnimation::default();
    }

    #[test]
    fn animation_seed_avoids_system_time_on_wasm() {
        #[cfg(target_arch = "wasm32")]
        assert_eq!(animation_seed(), 0x6f72_6376_735f_6375);
    }

    #[test]
    fn defaults_are_the_approved_colours_and_an_irregular_live_cadence() {
        let settings = CursorEffectSettings::default();
        assert_eq!(settings.cursor_colour(), Color32::from_rgb(234, 235, 229));
        assert_eq!(settings.area_colour(), Color32::from_rgb(76, 190, 156));
        assert_eq!(settings.cell_colour(), None);
        assert_ne!(settings.interval(0), settings.interval(255));
    }

    #[test]
    fn zero_frequency_stops_animation_and_valid_settings_round_trip() {
        let mut settings = CursorEffectSettings::default();
        *settings.frequency_mut() = 0;
        *settings.amount_mut() = 73;
        assert_eq!(settings.interval(127), None);
        assert_eq!(
            CursorEffectSettings::decode(&settings.encode()),
            Some(settings)
        );
    }

    #[test]
    fn reduced_motion_uses_a_clear_static_cursor() {
        let settings = CursorEffectSettings::default().respecting_reduced_motion(true);
        assert_eq!(settings.amount(), 0);
        assert_eq!(settings.frequency(), 0);
    }

    #[test]
    fn malformed_or_out_of_range_settings_are_refused_whole() {
        assert_eq!(CursorEffectSettings::decode("garbage"), None);
        assert_eq!(CursorEffectSettings::decode("1,2,3;4,5,6;101;50"), None);
        assert_eq!(CursorEffectSettings::decode("1,2,3;4,5,6;50;101"), None);
    }

    #[test]
    fn animation_changes_only_at_deadlines_and_does_not_replay_a_backlog() {
        let settings = CursorEffectSettings::default();
        let mut animation = CursorEffectAnimation::default();
        let first = animation.advance(Duration::ZERO, settings);
        let before = animation.advance(Duration::from_millis(99), settings);
        assert_eq!(before, first);

        let changed = animation.advance(Duration::from_secs(30), settings);
        assert_ne!(changed.frame, first.frame);
        let after_jump = animation.advance(Duration::from_secs(30), settings);
        assert_eq!(
            after_jump, changed,
            "one observation advances only one state"
        );
        assert!(
            animation
                .repaint_after(Duration::from_secs(30), settings)
                .is_some()
        );
    }

    #[test]
    fn zero_frequency_freezes_the_sample_and_requests_no_repaint() {
        let mut settings = CursorEffectSettings::default();
        *settings.frequency_mut() = 0;
        let mut animation = CursorEffectAnimation::default();
        let first = animation.advance(Duration::ZERO, settings);
        assert_eq!(animation.advance(Duration::from_secs(100), settings), first);
        assert_eq!(animation.repaint_after(Duration::ZERO, settings), None);
    }

    #[test]
    fn frame_fragments_cover_all_four_edges_and_evolve() {
        let cursor = Rect::from_min_size(Pos2::new(200.0, 200.0), Vec2::splat(CELL_SIZE));
        let clip = cursor.expand(200.0);
        let settings = CursorEffectSettings::default();
        let first =
            cursor_effect_shapes(cursor, cursor, clip, CELL_SIZE, sample(1, 2, 3), settings);
        let second =
            cursor_effect_shapes(cursor, cursor, clip, CELL_SIZE, sample(4, 2, 3), settings);
        assert_ne!(format!("{:?}", first.frame), format!("{:?}", second.frame));

        for edge in [
            Rect::from_min_max(
                cursor.min - Vec2::splat(2.0),
                Pos2::new(cursor.max.x + 2.0, cursor.min.y + 2.0),
            ),
            Rect::from_min_max(
                Pos2::new(cursor.max.x - 2.0, cursor.min.y - 2.0),
                cursor.max + Vec2::splat(2.0),
            ),
            Rect::from_min_max(
                Pos2::new(cursor.min.x - 2.0, cursor.max.y - 2.0),
                cursor.max + Vec2::splat(2.0),
            ),
            Rect::from_min_max(
                cursor.min - Vec2::splat(2.0),
                Pos2::new(cursor.min.x + 2.0, cursor.max.y + 2.0),
            ),
        ] {
            assert!(
                first
                    .frame
                    .iter()
                    .any(|shape| shape.visual_bounding_rect().intersects(edge))
            );
        }
        assert!(
            first
                .frame
                .iter()
                .all(|shape| shape.visual_bounding_rect().intersects(cursor.expand(8.0)))
        );
    }

    ///
    /// A Region's lasso is the Cursor's frame drawn around the whole Region:
    /// every Cell-length of every edge carries fragments, and no fragment is
    /// heavier than the Cursor's heaviest.
    ///
    #[test]
    fn a_lasso_frames_every_cell_length_of_a_region_at_the_cursors_weight() {
        let cell = Vec2::splat(CELL_SIZE);
        let outline = Rect::from_min_size(Pos2::new(200.0, 200.0), Vec2::new(5.0, 3.0) * CELL_SIZE);
        let cursor = Rect::from_min_size(outline.max - cell, cell);
        let clip = outline.expand(200.0);
        let settings = CursorEffectSettings::default();
        let heaviest = |shapes: &[Shape]| {
            shapes
                .iter()
                .filter_map(|shape| match shape {
                    Shape::LineSegment { stroke, .. } => Some(stroke.width),
                    _ => None,
                })
                .fold(0.0_f32, f32::max)
        };

        for frame in 0..16 {
            let lasso = cursor_effect_shapes(
                cursor,
                outline,
                clip,
                CELL_SIZE,
                sample(frame, 2, 3),
                settings,
            );
            let alone = cursor_effect_shapes(
                cursor,
                cursor,
                clip,
                CELL_SIZE,
                sample(frame, 2, 3),
                settings,
            );
            assert!(heaviest(&lasso.frame) <= 1.25 + 1e-3);
            assert!(heaviest(&alone.frame) <= 1.25 + 1e-3);

            let touches = |probe: Rect| {
                lasso
                    .frame
                    .iter()
                    .any(|shape| shape.visual_bounding_rect().intersects(probe))
            };
            for column in 0..5 {
                let left = outline.left() + column as f32 * CELL_SIZE;
                for y in [outline.top(), outline.bottom()] {
                    let probe = Rect::from_min_max(
                        Pos2::new(left, y - 2.0),
                        Pos2::new(left + CELL_SIZE, y + 2.0),
                    );
                    assert!(touches(probe), "column {column} of an edge at {y} was bare");
                }
            }
            for row in 0..3 {
                let top = outline.top() + row as f32 * CELL_SIZE;
                for x in [outline.left(), outline.right()] {
                    let probe = Rect::from_min_max(
                        Pos2::new(x - 2.0, top),
                        Pos2::new(x + 2.0, top + CELL_SIZE),
                    );
                    assert!(touches(probe), "row {row} of an edge at {x} was bare");
                }
            }
            assert!(
                lasso
                    .frame
                    .iter()
                    .all(|shape| shape.visual_bounding_rect().intersects(outline.expand(8.0))),
                "a lasso fragment strayed from the Region"
            );
        }
    }

    ///
    /// A Cell's frame is the Cursor's frame whichever way it is asked for, so
    /// generalising the frame to a Region left the Cursor's presentation as it
    /// was.
    ///
    #[test]
    fn a_one_cell_outline_is_the_cursors_own_frame() {
        let cursor = Rect::from_min_size(Pos2::new(200.0, 200.0), Vec2::splat(CELL_SIZE));
        let settings = CursorEffectSettings::default();
        let effects = cursor_effect_shapes(
            cursor,
            cursor,
            cursor.expand(200.0),
            CELL_SIZE,
            sample(9, 2, 3),
            settings,
        );
        // Eight fragments on each of four edges at most, and seven tears.
        assert!(effects.frame.len() <= 4 * 8 + 7);
        assert!(effects.frame.len() >= 4 * 2);
    }

    #[test]
    fn the_region_colour_defaults_to_white_at_seventeen_percent_and_is_persisted() {
        let settings = CursorEffectSettings::default();
        assert_eq!(
            settings.region_colour().to_srgba_unmultiplied(),
            [255, 255, 255, 43]
        );

        let mut changed = settings;
        *changed.region_colour_mut() = Color32::from_rgba_unmultiplied(255, 0, 0, 128);
        let decoded = CursorEffectSettings::decode(&changed.encode()).expect("round trips");
        assert_eq!(decoded.region_colour(), changed.region_colour());
    }

    #[test]
    fn the_cursors_colour_within_a_region_is_unset_by_default_and_is_persisted() {
        let settings = CursorEffectSettings::default();
        assert_eq!(settings.region_cursor_colour(), None);
        assert_eq!(
            CursorEffectSettings::decode(&settings.encode()),
            Some(settings)
        );

        let mut changed = settings;
        changed.set_region_cursor_colour(Some(Color32::from_rgba_unmultiplied(0, 0, 255, 200)));
        let decoded = CursorEffectSettings::decode(&changed.encode()).expect("round trips");
        assert_eq!(
            decoded.region_cursor_colour(),
            changed.region_cursor_colour()
        );
    }

    #[test]
    fn settings_saved_before_the_region_colour_existed_take_its_default() {
        let decoded =
            CursorEffectSettings::decode("1,2,3;4,5,6;50;50;none").expect("the older form decodes");
        assert_eq!(decoded.region_colour(), DEFAULT_REGION_COLOUR);
        assert_eq!(decoded.region_cursor_colour(), None);
        assert_eq!(
            CursorEffectSettings::decode("1,2,3;4,5,6;50;50;none;1,2,3"),
            None
        );
    }

    #[test]
    fn area_is_bounded_crosses_cell_boundaries_and_can_reach_in_from_outside_clip() {
        let cursor = Rect::from_min_size(Pos2::new(-35.0, 100.0), Vec2::splat(CELL_SIZE));
        let clip = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::splat(300.0));
        let effects = cursor_effect_shapes(
            cursor,
            cursor,
            clip,
            CELL_SIZE,
            sample(1, 0x1234, 0x5678),
            CursorEffectSettings::default(),
        );
        assert!(!effects.area.is_empty());
        assert!(effects.area.len() <= 254);
        let bounds = effect_bounds(cursor, CELL_SIZE).expand(2.0);
        assert!(
            effects
                .area
                .iter()
                .all(|shape| shape.visual_bounding_rect().intersects(bounds))
        );
        assert!(effects.area.iter().any(|shape| {
            let rect = shape.visual_bounding_rect();
            (rect.left() / CELL_SIZE).floor() != (rect.right() / CELL_SIZE).floor()
        }));
    }

    #[test]
    fn advertised_bounds_contain_the_outer_area_tails() {
        let cursor = Rect::from_min_size(Pos2::new(200.0, 200.0), Vec2::splat(CELL_SIZE));
        let bounds = effect_bounds(cursor, CELL_SIZE);
        for field in 0..32 {
            let effects = cursor_effect_shapes(
                cursor,
                cursor,
                bounds.expand(100.0),
                CELL_SIZE,
                sample(1, field, field.rotate_left(7)),
                CursorEffectSettings::default(),
            );
            assert!(effects.area.iter().all(|shape| {
                let shape = shape.visual_bounding_rect();
                bounds.contains(shape.min) && bounds.contains(shape.max)
            }));
        }
    }

    #[test]
    fn a_fully_clipped_effect_builds_no_shapes() {
        let cursor = Rect::from_min_size(Pos2::ZERO, Vec2::splat(CELL_SIZE));
        let clip = Rect::from_min_size(Pos2::new(1_000.0, 1_000.0), Vec2::splat(100.0));
        let effects = cursor_effect_shapes(
            cursor,
            cursor,
            clip,
            CELL_SIZE,
            sample(1, 2, 3),
            CursorEffectSettings::default(),
        );
        assert!(effects.area.is_empty());
        assert!(effects.frame.is_empty());
    }
}
