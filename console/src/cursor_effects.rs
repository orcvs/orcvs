use std::time::Duration;

use egui::{Color32, Pos2, Rect, Shape, Stroke, Vec2, epaint::RectShape};

use crate::grid_viewport::CELL_SIZE;

///
/// The Cursor Effect's non-colour motion preferences: Glitch amount and
/// Glitch frequency, beside the operating system's reduced-motion preference
/// (ADR 0053: "A Theme decides how things look, never how much they move.").
///
/// The Effect's colours — the Cursor frame, the living-field area, the
/// Region fill and the two optional Cell fills — moved to the resolved
/// `crate::theme::Theme` (`.scratch/theming/issues/06`): `cursor.border`,
/// `region.border`, `cursor.area`, `region.background`,
/// `cursor.background` and `region.cursor.background`. `crate::style::style`
/// and `crate::paint` read those directly; this settings value carries only
/// what `.scratch/theming/issues/09` still owns.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CursorEffectSettings {
    amount: u8,
    frequency: u8,
}

impl Default for CursorEffectSettings {
    fn default() -> Self {
        Self {
            amount: 60,
            frequency: 55,
        }
    }
}

impl CursorEffectSettings {
    pub(crate) fn amount(self) -> u8 {
        self.amount
    }
    pub(crate) fn frequency(self) -> u8 {
        self.frequency
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

    ///
    /// `amount;frequency`. A value stored before `.scratch/theming/issues/06`
    /// moved the Effect's colours out of this settings value carries more
    /// groups (colours, in `r,g,b` form, which contains a comma `parse::<u8>`
    /// below refuses outright) and is refused whole by [`Self::decode`],
    /// the same "malformed value falls back to the default" rule every
    /// other settings decode in this crate already holds — not a migration,
    /// since the two amount/frequency values a viewer had stored moved index
    /// with the colours removed around them and reading them positionally
    /// from the old string would misinterpret a colour group as one.
    ///
    #[cfg(any(feature = "persistence", test))]
    pub(crate) fn encode(self) -> String {
        format!("{};{}", self.amount, self.frequency)
    }

    #[cfg(any(feature = "persistence", test))]
    pub(crate) fn decode(value: &str) -> Option<Self> {
        let mut groups = value.split(';');
        let settings = Self {
            amount: groups.next()?.parse().ok()?,
            frequency: groups.next()?.parse().ok()?,
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
///
/// `area_colour` and `frame_colour` are the resolved Theme's `cursor.area`
/// and `cursor.border`/`region.border` (the caller picks the outline colour:
/// `cursor.border` for the Cursor's own frame, `region.border` for the lasso
/// around a Region larger than one Cell) — `.scratch/theming/issues/06`
/// moved both out of [`CursorEffectSettings`], which now carries only the
/// motion `settings` decide from here. `frame_width` is the same caller's
/// choice of `cursor.border.width`/`region.border.width`: a fixed display-point
/// nominal width, never scaled by `cell_size`/Grid zoom (`.scratch/theming/
/// issues/06` slice C). Zero hides every frame stroke outright — see this
/// module's private `frame_shapes` — without touching the living-area fill `area_colour` and
/// `amount` still control. One over clippy's default: every parameter is an
/// independent, already-tested value with nowhere smaller to group into, the
/// same reasoning `console.rs::show_source`'s own
/// `#[allow(clippy::too_many_arguments)]` states.
///
#[allow(clippy::too_many_arguments)]
pub fn cursor_effect_shapes(
    cursor: Rect,
    outline: Rect,
    clip: Rect,
    cell_size: f32,
    sample: CursorEffectSample,
    settings: CursorEffectSettings,
    area_colour: Color32,
    frame_colour: Color32,
    frame_width: f32,
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
            area_colour,
            amount,
        );
    }
    frame_shapes(
        &mut shapes.frame,
        outline,
        cell_size,
        clip,
        sample,
        frame_colour,
        amount,
        frame_width,
    );
    shapes
}

///
/// The living-field fill: long-tailed horizontal tears and a few fine
/// connecting strands, both at a fraction of `colour`'s own alpha.
///
/// `colour` is `Color32`, whose byte layout is premultiplied alpha
/// (`ecolor::Color32`'s own documentation — the same fact
/// `crate::theme::straight_rgba` states). Scaling that alpha down therefore
/// has to scale every channel together, which is exactly what
/// [`Color32::gamma_multiply`] does: `r'/g'/b'/a' = r/g/b/a * fraction`, still
/// premultiplied and still `colour`'s own hue. The `fraction` each site below
/// computes is what a fully opaque `colour` would want as its literal alpha
/// byte, divided back out of 255 — `gamma_multiply` reapplies it through
/// `colour.a()` rather than through an assumed 255, so a transparent
/// `colour` (`a() == 0`) multiplies to exactly zero for every candidate
/// rather than the flat `alpha.clamp(1.0, 255.0)` floor this replaced, which
/// used to paint a faint minimum speck even when the Theme's own `cursor.area`
/// asked for nothing at all. Reading `colour.r()/g()/b()` directly and handing
/// them to [`Color32::from_rgba_unmultiplied`] — this function's own bug
/// before this fix — would instead treat those premultiplied bytes as a
/// straight triple and premultiply them a second time by the new alpha,
/// shifting the hue and, at `colour.a() == 0`, discarding the caller's colour
/// entirely in favour of black.
///
fn area_shapes(
    out: &mut Vec<Shape>,
    cursor: Rect,
    clip: Rect,
    cell_size: f32,
    sample: CursorEffectSample,
    colour: Color32,
    amount: f32,
) {
    let reach = AREA_RADIUS_CELLS * cell_size;
    let centre = cursor.center();

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
        let fraction = envelope * (0.08 + density * 0.22) * amount;
        let painted = colour.gamma_multiply(fraction);
        if painted.a() == 0 {
            continue;
        }
        out.push(Shape::Rect(RectShape::filled(rect, 0.0, painted)));
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
            let painted = colour.gamma_multiply(20.0 * amount / 255.0);
            if painted.a() != 0 {
                out.push(Shape::line_segment(
                    [start, end],
                    Stroke::new((cell_size / 50.0).max(0.35), painted),
                ));
            }
        }
    }
}

///
/// The eroded Cursor/Region frame: a stationary outline when `amount` is
/// zero, or the animated fragments otherwise. `width` is the nominal
/// display-point width `cursor_effect_shapes` resolved from the Theme
/// (`cursor.border.width`/`region.border.width`), fixed in points and never
/// scaled by `cell_size`/`CELL_SIZE` — `cell_size` still drives every
/// fragment's *position* and *length* along the edge, which is Grid geometry
/// rather than stroke weight and stays proportional to the Cell side.
///
/// `width <= 0.0` hides every stroke this function would otherwise build,
/// without touching the living-area fill `area_shapes` paints independently:
/// no shape is pushed at all, at every `amount`, rather than a zero-width one
/// left for the painter to drop.
///
/// Eight parameters, one over clippy's default, for the same reason
/// `console.rs::show_source`'s own `#[allow(clippy::too_many_arguments)]`
/// states: every parameter is an independent, already-tested value with
/// nowhere smaller to group into.
///
#[allow(clippy::too_many_arguments)]
fn frame_shapes(
    out: &mut Vec<Shape>,
    outline: Rect,
    cell_size: f32,
    clip: Rect,
    sample: CursorEffectSample,
    colour: Color32,
    amount: f32,
    width: f32,
) {
    if width <= 0.0 {
        return;
    }
    let side = cell_size;
    let scale = side / CELL_SIZE;
    if amount == 0.0 {
        out.push(Shape::Rect(RectShape::stroke(
            outline,
            0.0,
            Stroke::new(width, colour),
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
                let bound = Rect::from_two_pos(a, b).expand(2.0 * width);
                if bound.intersects(clip) {
                    let alpha = (105.0 + unit(hash(!key)) * 150.0) as u8;
                    out.push(Shape::line_segment(
                        [a, b],
                        Stroke::new(
                            (0.45 + unit(hash(key ^ 73)) * 0.8) * width,
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

    /// A stand-in Cursor frame/area colour pair for the geometry tests
    /// below, which do not care what the resolved Theme's own `cursor.border`
    /// and `cursor.area` are — only that a colour reaches the Shapes.
    /// `crate::theme::okabe_ito()`'s actual values are cross-checked in
    /// `theme::tests::okabe_ito_defines_every_key_at_the_schema_values`.
    const AREA_COLOUR: Color32 = Color32::from_rgb(76, 190, 156);
    const FRAME_COLOUR: Color32 = Color32::from_rgb(234, 235, 229);
    /// The stationary Cursor/Region outline's nominal display-point default
    /// (`.scratch/theming/schema.md`'s `cursor.border.width`/
    /// `region.border.width`: 1 point), reused as every geometry test's
    /// nominal width unless a test states otherwise.
    const FRAME_WIDTH: f32 = 1.0;

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
    fn defaults_are_an_irregular_live_cadence() {
        let settings = CursorEffectSettings::default();
        assert_eq!(settings.amount(), 60);
        assert_eq!(settings.frequency(), 55);
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
        assert_eq!(CursorEffectSettings::decode("101;50"), None);
        assert_eq!(CursorEffectSettings::decode("50;101"), None);
        assert_eq!(CursorEffectSettings::decode("50"), None);
        assert_eq!(CursorEffectSettings::decode("50;50;50"), None);
        // A value saved before `.scratch/theming/issues/06` moved the
        // Effect's colours out of this settings value: its colour groups
        // contain commas, which `str::parse::<u8>` refuses outright, so the
        // whole value is refused rather than misread positionally.
        assert_eq!(
            CursorEffectSettings::decode("234,235,229;76,190,156;60;55;none"),
            None
        );
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
        let first = cursor_effect_shapes(
            cursor,
            cursor,
            clip,
            CELL_SIZE,
            sample(1, 2, 3),
            settings,
            AREA_COLOUR,
            FRAME_COLOUR,
            FRAME_WIDTH,
        );
        let second = cursor_effect_shapes(
            cursor,
            cursor,
            clip,
            CELL_SIZE,
            sample(4, 2, 3),
            settings,
            AREA_COLOUR,
            FRAME_COLOUR,
            FRAME_WIDTH,
        );
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
                AREA_COLOUR,
                FRAME_COLOUR,
                FRAME_WIDTH,
            );
            let alone = cursor_effect_shapes(
                cursor,
                cursor,
                clip,
                CELL_SIZE,
                sample(frame, 2, 3),
                settings,
                AREA_COLOUR,
                FRAME_COLOUR,
                FRAME_WIDTH,
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
            AREA_COLOUR,
            FRAME_COLOUR,
            FRAME_WIDTH,
        );
        // Eight fragments on each of four edges at most, and seven tears.
        assert!(effects.frame.len() <= 4 * 8 + 7);
        assert!(effects.frame.len() >= 4 * 2);
    }

    // The Region fill, the Cursor's optional Cell fill, and the Cursor's
    // optional fill within a Region moved to the resolved Theme
    // (`.scratch/theming/issues/06`): `region.background`,
    // `cursor.background` and `region.cursor.background`. Their defaults,
    // optional-fill states and inheritance are `theme::tests`' subject now
    // — `okabe_ito_defines_every_key_at_the_schema_values`,
    // `omitted_optional_fills_inherit_the_parent`,
    // `explicit_none_clears_the_optional_fill_even_over_a_parent_colour` and
    // `explicit_transparent_colour_is_a_supplied_value_not_a_clear`.

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
            AREA_COLOUR,
            FRAME_COLOUR,
            FRAME_WIDTH,
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
                AREA_COLOUR,
                FRAME_COLOUR,
                FRAME_WIDTH,
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
            AREA_COLOUR,
            FRAME_COLOUR,
            FRAME_WIDTH,
        );
        assert!(effects.area.is_empty());
        assert!(effects.frame.is_empty());
    }

    /// The alpha byte a `Shape::Rect` fill or `Shape::LineSegment` stroke was
    /// painted with — the two Shape kinds [`area_shapes`] builds.
    fn shape_alpha(shape: &Shape) -> u8 {
        match shape {
            Shape::Rect(rect) => rect.fill.a(),
            Shape::LineSegment { stroke, .. } => stroke.color.a(),
            other => panic!("{other:?} is neither a filled Rect nor a LineSegment"),
        }
    }

    ///
    /// Width zero hides every Cursor/Region frame stroke this function
    /// builds — the stationary outline `frame_shapes` draws at `amount ==
    /// 0.0` and every animated fragment it draws otherwise — without
    /// disabling the living-area fill `amount` and `area_colour` still
    /// control, at several Grid zoom levels from `MIN_ZOOM` to `MAX_ZOOM`.
    /// `.scratch/theming/issues/06`: "Width 0 hides every affected stroke
    /// without disabling fills or changing motion preferences."
    ///
    #[test]
    fn zero_frame_width_hides_every_stroke_at_every_amount_and_zoom() {
        let mut static_settings = CursorEffectSettings::default();
        *static_settings.amount_mut() = 0;

        for cell_size in [
            CELL_SIZE * 0.25,
            CELL_SIZE * 0.5,
            CELL_SIZE,
            CELL_SIZE * 2.0,
        ] {
            let cursor = Rect::from_min_size(Pos2::new(200.0, 200.0), Vec2::splat(cell_size));
            let clip = cursor.expand(500.0);

            for settings in [static_settings, CursorEffectSettings::default()] {
                let effects = cursor_effect_shapes(
                    cursor,
                    cursor,
                    clip,
                    cell_size,
                    sample(3, 5, 7),
                    settings,
                    AREA_COLOUR,
                    FRAME_COLOUR,
                    0.0,
                );
                assert!(
                    effects.frame.is_empty(),
                    "a Cursor frame stroke survived width 0 at cell_size {cell_size}"
                );
            }

            // The living-area fill has no width of its own and must not be
            // disabled by a frame width of zero.
            let effects = cursor_effect_shapes(
                cursor,
                cursor,
                clip,
                cell_size,
                sample(3, 5, 7),
                CursorEffectSettings::default(),
                AREA_COLOUR,
                FRAME_COLOUR,
                0.0,
            );
            assert!(
                !effects.area.is_empty(),
                "width 0 wrongly suppressed the living-area fill at cell_size {cell_size}"
            );
        }
    }

    ///
    /// The stationary outline (`amount == 0.0`) is stroked at exactly the
    /// nominal width, at every Grid zoom level — no `scale.max(0.5)` floor
    /// and no proportional shrink as `cell_size` falls.
    ///
    #[test]
    fn stationary_frame_stroke_is_the_nominal_width_at_every_zoom() {
        let mut settings = CursorEffectSettings::default();
        *settings.amount_mut() = 0;

        for cell_size in [
            CELL_SIZE * 0.25,
            CELL_SIZE * 0.5,
            CELL_SIZE,
            CELL_SIZE * 2.0,
        ] {
            let cursor = Rect::from_min_size(Pos2::new(200.0, 200.0), Vec2::splat(cell_size));
            let clip = cursor.expand(500.0);
            let effects = cursor_effect_shapes(
                cursor,
                cursor,
                clip,
                cell_size,
                sample(3, 5, 7),
                settings,
                AREA_COLOUR,
                FRAME_COLOUR,
                FRAME_WIDTH,
            );
            assert_eq!(
                effects.frame.len(),
                1,
                "the stationary outline is one stroke"
            );
            let Shape::Rect(stroked) = &effects.frame[0] else {
                panic!("the stationary outline was {:?}", effects.frame[0])
            };
            assert_eq!(
                stroked.stroke.width, FRAME_WIDTH,
                "the stationary outline scaled with cell_size {cell_size}"
            );
        }
    }

    ///
    /// The animated fragments' 0.45–1.25× nominal-width modulation, exactly
    /// reproduced across every Grid zoom level from `MIN_ZOOM` to `MAX_ZOOM`:
    /// the same `sample` and `width` produce the identical set of stroke
    /// widths whatever `cell_size` is, because the modulation is a function
    /// of the sample and the nominal width alone
    /// (`.scratch/theming/issues/06`: "Cursor/Region effect width is
    /// nominal... without Grid zoom scaling").
    ///
    #[test]
    fn animated_fragment_widths_modulate_the_nominal_width_and_never_the_grid_zoom() {
        let clip = Rect::from_min_size(Pos2::new(-500.0, -500.0), Vec2::splat(2000.0));
        let mut widths_by_zoom = Vec::new();

        for cell_size in [
            CELL_SIZE * 0.25,
            CELL_SIZE * 0.5,
            CELL_SIZE,
            CELL_SIZE * 2.0,
        ] {
            // One Cell's own outline at this zoom, so the loop's fragment
            // count (`across`/`down`) stays the same at every cell_size and
            // only the stroke width can differ.
            let outline = Rect::from_min_size(Pos2::ZERO, Vec2::splat(cell_size));
            let mut out = Vec::new();
            frame_shapes(
                &mut out,
                outline,
                cell_size,
                clip,
                sample(11, 13, 17),
                FRAME_COLOUR,
                0.6,
                FRAME_WIDTH,
            );
            let widths: Vec<f32> = out
                .iter()
                .filter_map(|shape| match shape {
                    Shape::LineSegment { stroke, .. } => Some(stroke.width),
                    _ => None,
                })
                .collect();
            assert!(
                !widths.is_empty(),
                "cell_size {cell_size} produced no fragment strokes to compare"
            );
            for width in &widths {
                assert!(
                    (0.45 * FRAME_WIDTH - 1e-4..=1.25 * FRAME_WIDTH + 1e-4).contains(width),
                    "a fragment at cell_size {cell_size} was {width} points wide, outside \
                     0.45x-1.25x the nominal width {FRAME_WIDTH}"
                );
            }
            widths_by_zoom.push(widths);
        }

        for widths in &widths_by_zoom[1..] {
            assert_eq!(
                widths, &widths_by_zoom[0],
                "fragment stroke widths changed with Grid zoom"
            );
        }
    }

    ///
    /// A fully transparent `cursor.area` colour paints neither a tear nor a
    /// strand — the fix for the bug this function used to carry: reading
    /// `colour.r()/g()/b()` (already zero at zero alpha, since `Color32`'s
    /// premultiplied bytes collapse a transparent colour to `(0, 0, 0, 0)`
    /// regardless of its straight RGB) and handing them to
    /// `Color32::from_rgba_unmultiplied` with a `.clamp(1.0, 255.0)`-floored
    /// alpha always produced a faint but nonzero speck.
    /// `.scratch/theming/issues/06`: "Fix Cursor area alpha propagation so
    /// transparent area colour produces no visible tears or strands."
    ///
    #[test]
    fn zero_alpha_area_colour_paints_no_tears_or_strands() {
        let cursor = Rect::from_min_size(Pos2::new(200.0, 200.0), Vec2::splat(CELL_SIZE));
        let clip = cursor.expand(200.0);
        let mut out = Vec::new();

        area_shapes(
            &mut out,
            cursor,
            clip,
            CELL_SIZE,
            sample(1, 0x1234, 0x5678),
            Color32::TRANSPARENT,
            1.0,
        );

        assert!(
            out.is_empty(),
            "a fully transparent area colour painted {} shapes",
            out.len()
        );
    }

    ///
    /// A partial-alpha `cursor.area` colour scales every tear's and strand's
    /// alpha down from its own — never past it — and, crucially, keeps the
    /// same hue: every produced Shape un-premultiplies back to the Theme
    /// colour's own straight RGB regardless of the alpha `area_shapes`
    /// computed for it. This is what distinguishes correct premultiplied
    /// scaling (`Color32::gamma_multiply`, which scales r/g/b/a together)
    /// from the old bug (reading the premultiplied bytes as a straight
    /// triple and re-premultiplying them a second time), which would have
    /// shifted the hue here since the fixture's colour is not fully opaque.
    /// `.scratch/theming/issues/06`: "partial alpha is respected consistently
    /// without treating premultiplied RGB as straight RGB."
    ///
    #[test]
    fn partial_alpha_area_colour_is_respected_and_keeps_the_colours_hue() {
        let cursor = Rect::from_min_size(Pos2::new(200.0, 200.0), Vec2::splat(CELL_SIZE));
        let clip = cursor.expand(200.0);
        let colour = Color32::from_rgba_unmultiplied(76, 190, 156, 200);
        let mut out = Vec::new();

        area_shapes(
            &mut out,
            cursor,
            clip,
            CELL_SIZE,
            sample(1, 0x1234, 0x5678),
            colour,
            1.0,
        );

        assert!(!out.is_empty(), "the fixture painted nothing to check");
        for shape in &out {
            let alpha = shape_alpha(shape);
            assert!(alpha > 0, "a painted shape must carry nonzero alpha");
            assert!(
                alpha <= colour.a(),
                "a shape's alpha {alpha} exceeded the area colour's own {}",
                colour.a()
            );
            let painted = match shape {
                Shape::Rect(rect) => rect.fill,
                Shape::LineSegment { stroke, .. } => stroke.color,
                other => panic!("{other:?} is neither a filled Rect nor a LineSegment"),
            };
            // Cross-multiplied rather than un-premultiplied and compared
            // straight-on: `Color32::to_srgba_unmultiplied` divides by a
            // small alpha, which amplifies one rounding step into several
            // sRGB units of noise — noise a correct
            // `Color32::gamma_multiply` scaling is entitled to, since it
            // rounds each of r/g/b/a independently. Comparing
            // `painted.channel() * colour.a()` against
            // `colour.channel() * painted.a()` proves the same proportion
            // without a division, and the old bug's magnitude — reading
            // `colour.r()/g()/b()` as already-straight bytes and
            // re-premultiplying them a second time by the new alpha — is
            // still far outside the ~500 a correct scaling's rounding can
            // reach here (`colour.a()` and `painted.a()` both under 256, so
            // one rounding step on either side of the cross product moves it
            // by at most `colour.a()` or `painted.a()`, well under 256 each).
            for (painted_channel, colour_channel) in [painted.r(), painted.g(), painted.b()]
                .into_iter()
                .zip([colour.r(), colour.g(), colour.b()])
            {
                let cross_a = i32::from(painted_channel) * i32::from(colour.a());
                let cross_b = i32::from(colour_channel) * i32::from(alpha);
                assert!(
                    (cross_a - cross_b).abs() <= 500,
                    "a shape's channel drifted from the area colour's own hue: painted \
                     {painted:?} against area colour {colour:?}"
                );
            }
        }
    }
}
