//! ADR 0012's explicit interpretation inputs.
//!
//! A Function whose result depends on time or on where it sits reads it from
//! here rather than from a clock or a static. ADR 0003 puts every piece of
//! language state in the Source Snapshot, so anything that is *not* in the
//! Snapshot has to arrive as an explicit input: that is what makes an
//! identical Source Snapshot interpreted at an identical Tick produce an
//! identical Tick Plan.
//!
//! Clock, Delay, and Euclidean read the Tick; Random also reads the anchor.
//! None of them exists yet — they are the consumers this seam is built for.
//!
//! Nothing here derives `Default`. There is no Tick a Playback run has not
//! reached and no Cell an evaluation is not anchored at, so a caller that
//! cannot say which Tick and which anchor it means has nothing to interpret.
//! Withholding `Default` is what makes that a compile error rather than a
//! silent zero, which is the whole of what ADR 0012 means by an explicit input.

///
/// One absolute Tick of one Playback run.
///
/// The Playback Engine owns musical time, so it owns this counter: the first
/// Tick of a run is [`Tick::ZERO`] and each executed Tick is one
/// [`Tick::next`] on from the last. It is a type of its own rather than a bare
/// `u64` because the language threads several unrelated counts — a Tick, a
/// Number, a rate, a Cell index — and nothing but the type tells them apart at
/// a call site.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tick(u64);

impl Tick {
    /// The first Tick of a Playback run.
    pub const ZERO: Self = Self(0);

    /// The absolute Tick numbered `tick` within its Playback run.
    #[inline]
    #[must_use]
    pub const fn new(tick: u64) -> Self {
        Self(tick)
    }

    /// This Tick's number within its Playback run.
    #[inline]
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    ///
    /// The Tick after this one.
    ///
    /// Saturating, and unreachably so: at one Tick per nanosecond a `u64`
    /// counter takes over five hundred years to exhaust, and a musical Tick
    /// period is many orders of magnitude longer than that. The behaviour is
    /// still chosen rather than left implicit, because the two candidates
    /// differ in what they would do to the language. Wrapping would return a
    /// Playback run to Tick `0` and re-fire every Delay and Euclidean at once;
    /// saturating instead stops time, which is the failure a listener can
    /// recognise.
    ///
    #[inline]
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

///
/// The column and row an evaluation is anchored at.
///
/// This is the Grid-minted anchor Position of the evaluating root, carried
/// across the crate boundary as the column and row it names. It is
/// deliberately not a Position: a Position can be obtained only from the Grid
/// that contains it, and that invariant belongs to the crate that owns the
/// Grid. What interpretation needs is the two numbers, which is what ADR 0013
/// folds into Random's seed.
///
/// Deliberately unordered, exactly as the Position it is minted from is. An
/// anchor's order is its Grid's `y * cols + x` — the row-major Source order ADR
/// 0020 takes turns in — and that index can only be computed by the crate that
/// owns the Grid, because only it knows how wide a row is. A derived ordering
/// here would sort by the fields in declaration order, column before row, which
/// is the opposite of Source order: it would place a root far along row 0 after
/// a root near the start of row 1. Withholding `PartialOrd` and `Ord` is what
/// stops a later `BTreeMap<Anchor, _>` or `sort` from quietly emitting a Tick's
/// effects in an order no ADR describes.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Anchor {
    column: usize,
    row: usize,
}

impl Anchor {
    /// The anchor at `column` and `row`.
    #[inline]
    #[must_use]
    pub const fn new(column: usize, row: usize) -> Self {
        Self { column, row }
    }

    /// The anchor's column.
    #[inline]
    #[must_use]
    pub const fn column(self) -> usize {
        self.column
    }

    /// The anchor's row.
    #[inline]
    #[must_use]
    pub const fn row(self) -> usize {
        self.row
    }
}

///
/// Everything one evaluation is told that the Source Snapshot does not say.
///
/// One struct rather than one parameter per input: the inputs ADR 0012 and ADR
/// 0013 name are already two, and visible Portal feedback is a third. A
/// Function reaches them through the interpretation `Context` exactly as it
/// reaches its operands, so adding an input later is a field here rather than
/// a new parameter at every call site between the Playback Engine and the
/// Function.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TickInputs {
    tick: Tick,
    anchor: Anchor,
}

impl TickInputs {
    /// The inputs for an evaluation anchored at `anchor` during `tick`.
    #[inline]
    #[must_use]
    pub const fn new(tick: Tick, anchor: Anchor) -> Self {
        Self { tick, anchor }
    }

    /// The absolute Tick this evaluation belongs to.
    #[inline]
    #[must_use]
    pub const fn tick(self) -> Tick {
        self.tick
    }

    /// The anchor Position of the root being evaluated.
    #[inline]
    #[must_use]
    pub const fn anchor(self) -> Anchor {
        self.anchor
    }
}

#[cfg(test)]
mod test {
    use super::{Anchor, Tick, TickInputs};

    #[test]
    fn a_playback_run_counts_from_tick_zero_by_ones() {
        // ADR 0012's first-Tick rule and its increment are one property of the
        // counter, not two: Tick `0` is only the first Tick of a run if the
        // Tick after it is `1`.
        assert_eq!(Tick::ZERO.get(), 0);
        assert_eq!(Tick::ZERO.next(), Tick::new(1));

        let mut tick = Tick::ZERO;
        for expected in 0..8 {
            assert_eq!(tick.get(), expected);
            tick = tick.next();
        }
    }

    #[test]
    fn the_absolute_tick_saturates_rather_than_returning_to_the_first_tick() {
        // Unreachable in a Playback run, and pinned anyway: wrapping here would
        // silently restart every Tick-reading Function's cycle, which is the
        // one outcome the saturating choice exists to rule out.
        assert_eq!(Tick::new(u64::MAX).next(), Tick::new(u64::MAX));
        assert_ne!(Tick::new(u64::MAX).next(), Tick::ZERO);
    }

    #[test]
    fn tick_inputs_carry_the_tick_and_anchor_they_were_given() {
        // Column and row are both `usize`, so a transposed pair is a silent
        // change of meaning rather than a type error. An asymmetric anchor is
        // what makes the transposition visible.
        let inputs = TickInputs::new(Tick::new(9), Anchor::new(3, 7));

        assert_eq!(inputs.tick(), Tick::new(9));
        assert_eq!(inputs.anchor().column(), 3);
        assert_eq!(inputs.anchor().row(), 7);
    }
}
