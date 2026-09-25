//! The Glyphs the Source is drawn in: the scale a Glyph is laid out at, and
//! the table of laid-out Glyphs one Render Frame paints from.

use std::sync::Arc;

use egui::{FontId, text::Galley};

///
/// The step the scale is quantised to before it reaches a [`FontId`].
///
/// # Why this is an atlas budget, not a cache-hit rate
///
/// A Glyph is laid out at the size it is drawn at, so the scale has to reach
/// the font size. A *continuous* scale would reach it as a fresh size per
/// Render Frame, and epaint rasterises a fresh glyph set per distinct size —
/// `FontImpl::glyph_info` scales by `font_size * pixels_per_point` and rounds
/// nothing (`epaint-0.36.2/src/text/font.rs:567`). `subpixel_binning` is on by
/// default (`epaint-0.36.2/src/text/mod.rs:62`) and renders each glyph at up to
/// four fractional offsets, so a zoom sweep across `N` sizes costs up to
/// `N x alphabet x 4` rasters into one atlas.
///
/// That is the budget, because the atlas is not merely wasted when it fills:
/// `Fonts::begin_pass` replaces the whole `FontsImpl` — a new atlas with empty
/// glyph caches — as soon as `atlas.fill_ratio()` passes 0.8
/// (`epaint-0.36.2/src/text/fonts.rs:728-742`), restarting glyph rasterisation
/// mid-session for every size already paid for.
///
/// At a step of an eighth, the zoom range `MIN_ZOOM..=MAX_ZOOM` holds fifteen
/// distinct scales, so a viewer who sweeps that range spends at most
/// `15 x 94 x 4 = 5,640` rasters — roughly two megapixels of a 2048-square
/// atlas at one device pixel per point, which stays inside the fill ratio.
/// Both zoom limits and the Source's own scale are exact multiples of the step,
/// so the default window and either end of the range land on it rather than
/// beside it.
///
/// Fifteen is the whole range, not a floor a large window can widen: Zoom is
/// a stated step between [`MIN_ZOOM`](super::source_view::MIN_ZOOM) and [`MAX_ZOOM`](super::source_view::MAX_ZOOM), and no window size
/// changes the Cell size. The atlas budget ADR 0038 and ADR 0040 state covers
/// the whole range rather than its floor.
///
/// The step costs a Glyph at most an eighth of the Source's Cell scale in size,
/// taken downwards so a Glyph is never larger than its share of the Cell — see
/// [`glyph_scale`], which states why the rounding goes that way. It is still
/// strictly sharper than what it replaces: a Scene bilinearly resamples one
/// rasterised size at *every* zoom.
///
pub(super) const GLYPH_SCALE_STEP: f32 = 0.125;

///
/// The scale a [`FontId`] is derived from, quantised to [`GLYPH_SCALE_STEP`].
///
/// Never zero or negative: a font size of zero lays nothing out, and the
/// smallest step still draws something a viewer can see is there.
///
/// # Why the step is taken downwards
///
/// The step is absolute, so rounding to the nearest one is disproportionate at
/// a small scale: a console fitting at 0.2 would round up to 0.25 and lay an
/// 11.5 point Glyph out at 2.875 points inside a 3.2 point Cell, where the same Glyph
/// at the Source's own scale takes 11.5 of 16. Flooring keeps a Glyph's share of
/// its Cell at or under what the fit gave it at every scale, and costs at most
/// one step of sharpness rather than a Cell's worth of proportion. Both zoom
/// limits and the Source's own scale are exact multiples of the step, so
/// flooring leaves them exactly where rounding did, and the step count the
/// atlas budget above is stated over is unchanged.
///
pub(super) fn glyph_scale(scaling: f32) -> f32 {
    if !scaling.is_finite() || scaling <= 0.0 {
        return GLYPH_SCALE_STEP;
    }

    ((scaling / GLYPH_SCALE_STEP).floor() * GLYPH_SCALE_STEP).max(GLYPH_SCALE_STEP)
}

///
/// The alphabet the Glyph table covers: the printable ASCII a Source Cell can
/// show, less the space.
///
/// `orcvs::source::CellContent` accepts exactly `0x20..=0x7e`, and every
/// spelling a Token prints for an empty Cell is inside that range, so no
/// Cell of a Source can ask for a character outside it. The space is the one
/// printable character deliberately left out: a Cell showing one paints no
/// Glyph at all, and a Shape and a galley clone spent showing nothing is the
/// cost this drawing exists to stop paying.
///
pub(super) const ALPHABET_FIRST: u8 = b'!';
pub(super) const ALPHABET_LAST: u8 = b'~';

///
/// One laid-out Glyph per character of the alphabet, for the font and size the
/// Source is painted at.
///
/// The table has one owner and one construction site — [`show_source`](super::shapes::show_source) — so
/// nothing lays the alphabet out at a second size and thrashes it.
///
/// # Scratch for exactly one Render Frame
///
/// Retaining this table across Render Frames is unsound, not merely wasteful,
/// and that is why it is rebuilt every frame rather than memoised.
///
/// A galley's `RowVisuals::mesh` holds *texel* coordinates into the live font
/// atlas, normalised against that atlas's size at tessellation
/// (`epaint-0.36.2/src/text/text_layout_types.rs`, `tessellator.rs`), and
/// `Fonts::begin_pass` (`epaint-0.36.2/src/text/fonts.rs`) replaces the whole
/// `FontsImpl` — a fresh atlas with empty glyph caches — whenever the text
/// options change or the atlas passes its fill ratio. A galley held across that
/// recreate indexes unrelated texels and paints a *different character*. Atlas
/// *growth* is safe, because it only extends the image height and every texel
/// keeps its coordinates; *recreation* is what corrupts, and `font_image_size()`
/// cannot tell the two apart.
///
/// epaint's own `GalleyCache` is the memo, and it is the only cache in the
/// stack that `begin_pass` invalidates alongside the atlas. An
/// `egui::cache::FrameCache` or a `ctx.data()` entry evicts on last-frame use
/// and knows nothing about fonts, so either would carry exactly that
/// corruption. Do not "optimise" this into one.
///
/// Rebuilding costs one `Context` write lock for the whole table instead of one
/// per Cell, and one `String` per character of the alphabet instead of one per
/// Cell. Those are the wins; retaining the table is needed for none of them.
///
pub(super) struct GlyphTable {
    /// The `Context` the alphabet was laid out through, for the one character
    /// the table cannot cover. Cloning a `Context` clones an `Arc`, and the
    /// table lives for one Render Frame, so this holds nothing open.
    ctx: egui::Context,
    /// The font the alphabet was laid out at, for the one character the table
    /// cannot cover.
    font: FontId,
    /// The alphabet, indexed by `byte - ALPHABET_FIRST`.
    characters: Vec<Arc<Galley>>,
}

impl GlyphTable {
    ///
    /// Lays the alphabet out for this Render Frame, inside a single
    /// `ctx.fonts_mut` closure.
    ///
    /// One galley per character, never one per row: egui 0.36 shapes through
    /// harfrust with `liga` and `calt` enabled and does not apply
    /// `extra_letter_spacing` within a shaping cluster, so a row laid out as one
    /// galley would let a ligature consume two Cells and shift the rest of the
    /// row. MonaspaceNeon has ligatures and Orcvs Source is full of the pairs
    /// that trigger them.
    ///
    pub(super) fn lay_out(ctx: &egui::Context, font: FontId) -> Self {
        let characters = ctx.fonts_mut(|fonts| {
            (ALPHABET_FIRST..=ALPHABET_LAST)
                .map(|byte| {
                    // Laid out with `Color32::PLACEHOLDER`, so one galley serves
                    // every Cell whatever colour that Cell's Glyph is painted
                    // in: the tessellator substitutes the fallback colour for
                    // placeholder vertices alone.
                    fonts.layout_delayed_color(
                        char::from(byte).to_string(),
                        font.clone(),
                        f32::INFINITY,
                    )
                })
                .collect()
        });

        Self {
            ctx: ctx.clone(),
            font,
            characters,
        }
    }

    /// The laid-out Glyph for `character`, or `None` for a character outside
    /// the alphabet — the space included.
    pub(super) fn galley(&self, character: char) -> Option<&Arc<Galley>> {
        let byte = u8::try_from(character).ok()?;
        self.characters
            .get(usize::from(byte.checked_sub(ALPHABET_FIRST)?))
    }

    ///
    /// The laid-out Glyph for a character a Cell shows, laying out the one the
    /// alphabet does not cover.
    ///
    /// A Source Cell holds printable ASCII by construction, so the fallback
    /// lays out at most the odd galley for a Source that found a way to hold
    /// something else — and it costs that one Cell the allocation and the
    /// whole-`Context` lock the table exists to take once. It answers a galley
    /// rather than painting one, so that Cell stays a Shape in the ordered
    /// sequence rather than a `Painter::text` painted out of turn.
    ///
    pub(super) fn glyph(&self, character: char) -> Arc<Galley> {
        match self.galley(character) {
            Some(galley) => galley.clone(),
            None => self.ctx.fonts_mut(|fonts| {
                fonts.layout_delayed_color(character.to_string(), self.font.clone(), f32::INFINITY)
            }),
        }
    }
}
