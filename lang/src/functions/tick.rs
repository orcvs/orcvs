use crate::{
    Atom, Error, Function, InterpretationError, Value,
    atom::operands::{Clock, Delay, Euclidean, Increment, Interpolation, Random},
    interpreter::Context,
};
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::{Rng, SeedableRng};

// ADR 0012's Tick-reading Functions. Each reads the absolute Tick from the
// interpretation `Context` and nothing else: no clock, no static, no counter of
// its own. That is what makes the same Source Snapshot at the same Tick answer
// the same way, and it is why the Tick is lifted out of `ctx` once here rather
// than reached for inside each element's closure — the element answer is about
// one pair of Numbers at one Tick, exactly as `math::add` is about one pair of
// Numbers, and the Tick is shared by the whole operation because an Expression
// is evaluated at one Tick.
//
// Two of the five answer a pulse rather than a Number, and ADR 0039 declares
// those two Scalar: they refuse a Sequence operand rather than widening. A
// widened pulse would need one answer per element, an element that does not
// Bang has only the absence marker to offer, and `Sequence::new` refuses that
// as a member because it has no Source encoding. What is left is a reduction
// over the elements, and every reduction fixes a meaning for two rhythms
// layered on one Cell that could not later be changed without breaking Source,
// so the operand is refused instead — a refusal ADR 0039 can relax once the
// Sequence Functions make one spellable. Increment and Interpolation are
// Scalar for a different reason, ADR 0012's: their previous is one visible
// Atom at the ordinary result Portal, and a Sequence previous would need
// hidden element identity their one Cell pair cannot hold. The refusal comes
// from the declaration alone: `Stack::broadcast` raises `ExpectedAtom` for a
// Sequence at any operand of a Function that does not pervade, so none of the
// bodies checks for one. They bind through `Stack::extract`, the scalar
// seam; Clock and Random each answer a Number and broadcast through
// `Stack::apply` or `Stack::apply_indexed` like every other Atomic Function.
// Random uses `Stack::apply_indexed` so Sequence index participates in each
// element's stream.
//
// Every formula is evaluated in `u64`. The Tick is already one, and the two
// operands are Numbers whose product is a cycle length rather than a value the
// Source wrote: `~* 10 20` is a cycle of 512 Ticks, which a byte cannot hold and
// a byte multiply would fold to zero.

///
/// The diagnostic a cycle factor of zero raises, built in one place.
///
/// All three Functions refuse one fault under three role names, so the fault is
/// constructed once rather than once per site: a second construction beside
/// Euclidean's own validation is a second wording waiting to drift from this
/// one, and a Source told two different things about one mistake has to work
/// out that they are the same mistake.
///
#[inline(always)]
fn zero_cycle(function: Function, role: &'static str) -> Error {
    InterpretationError::ZeroCycle { function, role }.into()
}

///
/// The two factors of a cycle length, widened, and refused where either is
/// zero.
///
/// Clock and Delay both measure their cycle as `rate * modulus`, so both refuse
/// the same two zeroes, and refusing them in one place is what keeps the two
/// diagnostics saying the same thing about the same fault. The rate is answered
/// before the modulus because that is signature order, which is the order every
/// other operand fault in the language is reported in — `Stack::checked` walks
/// operands in it and `Operands::from_operands` binds in it — so a Source that
/// wrote two zeroes is told about the earlier Cell pair.
///
#[inline(always)]
fn cycle_factors(function: Function, rate: u8, modulus: u8) -> Result<(u64, u64), Error> {
    if rate == 0 {
        return Err(zero_cycle(function, "rate"));
    }

    if modulus == 0 {
        return Err(zero_cycle(function, "modulus"));
    }

    Ok((u64::from(rate), u64::from(modulus)))
}

///
/// The Atom a pulse answers with, which is a Bang or nothing at all.
///
/// Both pulse Functions answer the same pair, so the pair is named once: the
/// absence marker is what the Interpreter already reads as "no result write",
/// and answering a Number for the silent Tick would put a Cell meaning "no" in
/// the Source for the next Tick to read as an operand. It is also the Atom
/// `Sequence::new` refuses, which is why ADR 0039 has these two refuse a
/// Sequence operand rather than answer one.
///
#[inline(always)]
fn pulse(banged: bool) -> Value {
    if banged { Atom::Bang } else { Atom::Empty }.into()
}

/// Clock: `~. rate modulus`.
///
/// The step a cycle of `rate * modulus` Ticks is at, as a Number: `rate` Ticks
/// to a step and `modulus` steps to the cycle, so the answer counts `00`,
/// `01`, … up to `modulus - 1` and begins again. It is the one Function of the
/// three that answers a Number, so it is the one ADR 0039 leaves pervasive: it
/// broadcasts through `Stack::apply` and a Sequence operand answers a Sequence
/// of steps, because every element has a step to contribute.
#[inline(always)]
pub fn clock(ctx: &mut Context) -> Result<Value, Error> {
    let tick = ctx.inputs.tick().get();

    ctx.stack.apply(move |Clock { rate, modulus }: Clock| {
        let (rate, modulus) = cycle_factors(Function::Clock, rate, modulus)?;
        let step = (tick / rate) % modulus;

        // A remainder of `modulus` is below it and `modulus` came out of a
        // Number, so narrowing the step back into one is total. What the
        // conversion does if that ever stops being true is still a decision,
        // and the three candidates do not cost the same. A panic states the
        // invariant and is ruled out: this runs inside a Tick under the Source
        // write guard ADR 0028 forbids panicking under. A fallback Number is
        // the worst of the three rather than the cautious one — `00` is the
        // first step of every cycle, so a broken proof would write a step no
        // reader could tell from a counted one. So the impossible state
        // diagnoses, and what it costs is a diagnostic rather than Playback.
        let step =
            u8::try_from(step).map_err(|_| InterpretationError::ClockStepOutOfRange { step })?;

        Ok(Atom::Number(step))
    })
}

/// Delay: `~* rate modulus`.
///
/// One Bang every `rate * modulus` Ticks, beginning at Tick `0` — ADR 0012
/// counts the first Tick of a Playback run as a Tick like any other, so a Delay
/// fires as the run starts rather than one cycle into it. Modulus `01` is
/// therefore a Bang once per `rate` Ticks and not one every Tick, which is what
/// makes the two operands a rate and a step count rather than two names for the
/// same period.
///
/// It answers a pulse, so ADR 0039 keeps it scalar: a Sequence at either
/// operand is refused by the declaration before this body runs, and the one
/// pair `Stack::extract` binds is the whole operation.
#[inline(always)]
pub fn delay(ctx: &mut Context) -> Result<Value, Error> {
    let tick = ctx.inputs.tick().get();
    let Delay { rate, modulus } = ctx.stack.extract::<Delay>()?;
    let (rate, modulus) = cycle_factors(Function::Delay, rate, modulus)?;

    // The product is the cycle length, not a Number: two bytes multiply to at
    // most 0xFE01, and `~* 10 20` is a cycle of 512 Ticks that a byte multiply
    // would fold to zero and then divide by.
    //
    // ADR 0012 writes this as `Tick % (rate * modulus) == 0`, and
    // `is_multiple_of` is that test rather than a different one: the two differ
    // only at a zero divisor, which `cycle_factors` has already refused.
    Ok(pulse(tick.is_multiple_of(rate * modulus)))
}

/// Euclidean: `~% hits steps`.
///
/// ADR 0012's bucket distribution: `hits` onsets spread as evenly as the whole
/// numbers allow across a cycle of `steps` Ticks, which is the family of
/// rhythms `~% 03 08` names — `X..X..X.` — without a pattern being written
/// anywhere.
///
/// The absolute Tick is reduced into the cycle *before* the phase offset is
/// added. That is not a rearrangement for tidiness: `(hits * (t + steps - 1))`
/// over an absolute Tick would overflow the counter the Tick was read from, and
/// reducing first is exactly equivalent because every term after it is taken
/// modulo `steps` anyway. ADR 0012 asks for the reduction and says only that
/// the counter must not overflow; what the counter itself does at its end is
/// decided by [`crate::Tick::next`], which saturates rather than wraps.
///
/// It answers a pulse, so ADR 0039 keeps it scalar for the reason Delay is:
/// a Sequence at either operand is refused by the declaration, and the one pair
/// `Stack::extract` binds is the whole operation.
#[inline(always)]
pub fn euclidean(ctx: &mut Context) -> Result<Value, Error> {
    let tick = ctx.inputs.tick().get();
    let Euclidean { hits, steps } = ctx.stack.extract::<Euclidean>()?;

    // Zero steps is answered before hits are compared to steps, which is what
    // ADR 0012 requires and not an accident of the order the two checks are
    // written in: `~% 00 00` is a cycle with no positions rather than a cycle
    // with no onsets, so it diagnoses instead of falling to the zero-hits rule
    // below.
    if steps == 0 {
        return Err(zero_cycle(Function::Euclidean, "step count"));
    }

    if hits > steps {
        return Err(InterpretationError::EuclideanOverfull { hits, steps }.into());
    }

    let hits = u64::from(hits);
    let steps = u64::from(steps);
    let tick_in_cycle = tick % steps;
    let phase = (tick_in_cycle + steps - 1) % steps;

    // Zero hits falls out of this rather than being special-cased: `0 + 0 >=
    // steps` is false for every positive `steps`. So does a full cycle, where
    // `hits == steps` makes the remainder zero and the comparison an equality
    // that always holds.
    Ok(pulse((hits * phase) % steps + hits >= steps))
}

/// Increment: `~+ step modulus`.
///
/// The next Number of a wrap of `modulus`, advanced by `step` from the
/// previous visible Number at the ordinary result Portal. ADR 0012 writes
/// this as `(previous + step) % modulus`, and the addition is taken in `u64`
/// so `FF + 02` is 257 rather than a wrapped `01` that would then take the
/// modulus of the wrong total.
///
/// It is a scalar exception under ADR 0012: a Sequence at either operand is
/// refused by cell operand binding before the Portal input is decoded.
/// Both bindings must succeed before the formula runs.
#[inline(always)]
pub fn increment(ctx: &mut Context) -> Result<Value, Error> {
    let (Increment { step, modulus }, previous) =
        crate::portal::bind_operands(&mut ctx.stack, ctx.inputs.portal_source())?;
    let previous = previous.number();

    if modulus == 0 {
        return Err(InterpretationError::ZeroWrap {
            function: Function::Increment,
            role: "modulus",
        }
        .into());
    }

    let step = (u64::from(previous) + u64::from(step)) % u64::from(modulus);
    let step = u8::try_from(step).map_err(|_| InterpretationError::TickNumberOutOfRange {
        function: Function::Increment,
        value: step,
    })?;

    Ok(Atom::Number(step).into())
}

/// Interpolation: `~> rate target`.
///
/// The Number one Tick closer to `target`, moving by at most `rate` and
/// never past it. ADR 0012 spells the three orderings: below, above, and
/// equal. Each distance is taken only in the branch whose subtraction is
/// non-negative, so a step that would underflow is never asked, and the
/// remaining addition or subtraction is taken in `u64` before the answer
/// becomes a Number. Rate `00` holds because a step of nothing is still a
/// step of at most `rate`.
///
/// It is a scalar exception under ADR 0012 for the reason Increment is: the
/// previous is one Atom, and a Sequence at either operand is refused by the
/// binding before the formula runs.
#[inline(always)]
pub fn interpolation(ctx: &mut Context) -> Result<Value, Error> {
    let (Interpolation { rate, target }, previous) =
        crate::portal::bind_operands(&mut ctx.stack, ctx.inputs.portal_source())?;
    let previous = previous.number();

    let previous = u64::from(previous);
    let rate = u64::from(rate);
    let target = u64::from(target);

    let next = if previous < target {
        if rate >= target - previous {
            target
        } else {
            previous + rate
        }
    } else if previous > target {
        if rate >= previous - target {
            target
        } else {
            previous - rate
        }
    } else {
        target
    };

    let next = u8::try_from(next).map_err(|_| InterpretationError::TickNumberOutOfRange {
        function: Function::Interpolation,
        value: next,
    })?;

    Ok(Atom::Number(next).into())
}

/// Random: `~? seed minimum maximum`.
///
/// A Number selected inclusively between normalized bounds. ADR 0013 derives
/// each result from the explicit seed, the absolute Tick, this Function's
/// own Position, and the zero-based Sequence index, rather than from
/// activation history: the same Source Snapshot at the same Tick answers the
/// same Number, a skipped activation skips that sample, and two Randoms at
/// different Positions have independent reproducible streams.
///
/// The stream is a fresh ChaCha8 seeded from those four facts for every
/// scalar result. Reversed bounds describe the same range; equal bounds
/// return that value without asking the generator. The mapping widens the
/// inclusive width so `00`–`FF` is 256 values rather than a wrapping 0.
///
/// It answers a Number, so it is pervasive like Clock: it broadcasts through
/// `Stack::apply_indexed` and a Sequence operand answers a Sequence of
/// draws. Sequence index participates per element so two equal bounds at
/// different positions do not share a stream.
#[inline(always)]
pub fn random(ctx: &mut Context) -> Result<Value, Error> {
    let tick = ctx.inputs.tick().get();
    let anchor = ctx.inputs.anchor();
    // ADR 0013 writes the coordinates as little-endian i64. The Grid mints
    // non-negative usize that fit; this is that narrowing, not a signed
    // Position type in lang.
    let column = anchor.column() as i64;
    let row = anchor.row() as i64;

    ctx.stack.apply_indexed(
        move |Random {
                  seed,
                  minimum,
                  maximum,
              }: Random,
              index| {
            let sequence = index as u32;
            Ok(Atom::Number(draw(
                seed, tick, column, row, sequence, minimum, maximum,
            )))
        },
    )
}

///
/// One scalar draw: normalize the bounds, then map the first ChaCha8 `u64`
/// into that inclusive range.
///
/// Equal bounds return that value before a stream is built, because the
/// range has one member and the word cannot choose another. Reversed bounds
/// swap so the width is taken from the ordered pair, which is what makes
/// `10 00` the same range as `00 10`.
///
fn draw(seed: u8, tick: u64, column: i64, row: i64, sequence: u32, minimum: u8, maximum: u8) -> u8 {
    let (low, high) = if minimum <= maximum {
        (minimum, maximum)
    } else {
        (maximum, minimum)
    };
    if low == high {
        return low;
    }

    let word = chacha_word(seed, tick, column, row, sequence);
    let width = u16::from(high) - u16::from(low) + 1;
    let selected = u16::from(low) + (word % u64::from(width)) as u16;
    // `width` is at most 256 and `low + (word % width)` is at most 255.
    selected as u8
}

///
/// The first `u64` of a fresh ChaCha8 stream seeded from ADR 0013's layout.
///
/// The 32-byte seed is zero-initialized. Byte `0` is the explicit seed;
/// `[1, 9)` is the Tick as little-endian `u64`; `[9, 17)` and `[17, 25)` are
/// the Function column and row as little-endian `i64`; `[25, 29)` is the
/// Sequence index as little-endian `u32`; `[29, 32)` stay zero. A scalar
/// call uses Sequence index `0`.
///
fn chacha_word(seed: u8, tick: u64, column: i64, row: i64, sequence: u32) -> u64 {
    let mut bytes = [0u8; 32];
    bytes[0] = seed;
    bytes[1..9].copy_from_slice(&tick.to_le_bytes());
    bytes[9..17].copy_from_slice(&column.to_le_bytes());
    bytes[17..25].copy_from_slice(&row.to_le_bytes());
    bytes[25..29].copy_from_slice(&sequence.to_le_bytes());
    ChaCha8Rng::from_seed(bytes).next_u64()
}

#[cfg(test)]
mod test {
    use crate::{
        Anchor, Atom, Error, Function, FunctionInputs, Interpretation, Interpreter, Note,
        PortalSource, Sequence, SequenceError, Tick, TickInputs, TypeError, Value,
    };
    use rand_chacha::ChaCha8Rng;
    use rand_chacha::rand_core::{Rng, SeedableRng};

    /// Evaluates one Tick-reading Function at absolute Tick `tick`, with its
    /// operands in signature order and an empty previous.
    ///
    /// Clock, Delay, and Euclidean ignore the previous; Increment and
    /// Interpolation treat empty as Number `00`. The anchor is the Grid origin
    /// throughout: Clock, Delay, and Euclidean do not read a Position. Random
    /// has its own helpers below, because it does.
    fn evaluate(
        function: Function,
        tick: u64,
        left: impl Into<Value>,
        right: impl Into<Value>,
    ) -> Result<Interpretation, Error> {
        evaluate_previous(function, tick, left, right, "  ")
    }

    /// Evaluates one feedback Function with a stated previous at the Portal.
    fn evaluate_previous(
        function: Function,
        tick: u64,
        left: impl Into<Value>,
        right: impl Into<Value>,
        previous: &str,
    ) -> Result<Interpretation, Error> {
        Interpreter::execute_function(
            function,
            [left.into(), right.into()],
            FunctionInputs::with_portal_source(
                TickInputs::new(Tick::new(tick), Anchor::new(0, 0)),
                PortalSource::from_cells(Some(previous)),
            ),
        )
    }

    /// The Atom `function` answers at `tick` for one pair of Numbers.
    fn answer(function: Function, tick: u64, left: u8, right: u8) -> Result<Atom, Error> {
        match evaluate(function, tick, Atom::Number(left), Atom::Number(right))? {
            Interpretation::Cell(atom) => Ok(atom),
            other => panic!("expected a Cell result, found {other:?}"),
        }
    }

    /// Whether `function` Banged at `tick`, refusing anything but the two Atoms
    /// a pulse answers with.
    fn banged(function: Function, tick: u64, left: u8, right: u8) -> bool {
        match answer(function, tick, left, right) {
            Ok(Atom::Bang) => true,
            Ok(Atom::Empty) => false,
            other => panic!("{function:?}({left:02X}, {right:02X}) at {tick} answered {other:?}"),
        }
    }

    fn numbers(values: impl IntoIterator<Item = u8>) -> Sequence {
        Sequence::new(values.into_iter().map(Atom::Number)).unwrap()
    }

    #[test]
    fn clock_counts_one_step_per_rate_ticks_and_wraps_at_its_modulus() {
        // The formula is small enough to enumerate against rather than sample:
        // every rate and modulus from `01` to `08` over four cycles of the
        // widest of them, which is enough Ticks for every pair to wrap at least
        // twice. The reference here is ADR 0012's expression retyped, so what
        // the sweep pins is operand order and the width the arithmetic is done
        // in — a rate read as a modulus fails at every asymmetric pair — and
        // not the shape of the expression itself, which a retyped reference
        // agrees with by construction.
        // `clock_counts_the_literal_step_sequence_its_operands_name` is the
        // independent half, written from what the operands mean.
        for rate in 1..=8u8 {
            for modulus in 1..=8u8 {
                for tick in 0..256u64 {
                    let expected = (tick / u64::from(rate)) % u64::from(modulus);

                    assert_eq!(
                        answer(Function::Clock, tick, rate, modulus).unwrap(),
                        Atom::Number(u8::try_from(expected).unwrap()),
                        "~. {rate:02X} {modulus:02X} at Tick {tick}"
                    );
                }
            }
        }
    }

    #[test]
    fn clock_counts_the_literal_step_sequence_its_operands_name() {
        // Hand-written step sequences, one entry per Tick of one cycle, read
        // from what the operands mean rather than from the expression the body
        // evaluates: `rate` Ticks to a step and `modulus` steps to the cycle
        // make `~. 02 04` the sequence `00 00 01 01 02 02 03 03` and nothing
        // else. A body that divided by the modulus, or wrapped at the rate,
        // would have to land on that same sequence to pass here, which is what
        // the enumeration above cannot ask because it computes its reference
        // the way the body does.
        //
        // Each sequence is walked three times over, which separates a cycle
        // that begins again from a counter that keeps counting.
        for (rate, modulus, steps) in [
            (
                0x02u8,
                0x04u8,
                &[0x00u8, 0x00, 0x01, 0x01, 0x02, 0x02, 0x03, 0x03][..],
            ),
            (0x01, 0x03, &[0x00, 0x01, 0x02]),
            (0x03, 0x02, &[0x00, 0x00, 0x00, 0x01, 0x01, 0x01]),
            (0x02, 0x03, &[0x00, 0x00, 0x01, 0x01, 0x02, 0x02]),
            (0x04, 0x01, &[0x00, 0x00, 0x00, 0x00]),
        ] {
            assert_eq!(
                steps.len(),
                usize::from(rate) * usize::from(modulus),
                "~. {rate:02X} {modulus:02X} names a cycle of {} Ticks",
                usize::from(rate) * usize::from(modulus)
            );

            for (tick, expected) in steps.iter().cycle().take(steps.len() * 3).enumerate() {
                assert_eq!(
                    answer(Function::Clock, tick as u64, rate, modulus).unwrap(),
                    Atom::Number(*expected),
                    "~. {rate:02X} {modulus:02X} at Tick {tick}"
                );
            }
        }
    }

    #[test]
    fn clock_counts_a_cycle_of_more_ticks_than_a_byte_holds() {
        // `~. 80 03` is 128 Ticks to a step and three steps to the cycle: 384
        // Ticks, which no byte holds. Writing the whole sequence out would be
        // 384 entries saying one thing, so what is written instead is the Tick
        // each step ends and the Tick the next begins — the only places a
        // sequence of steps can be wrong — for three cycles.
        //
        // Every boundary after the first is beyond `FF`, so a Tick narrowed
        // before the division would answer about Tick `00` where this asks
        // about Tick 512, and a cycle length folded into a byte would put a
        // boundary at 128 where this says there is none.
        for (tick, step) in [
            (0u64, 0x00u8),
            (127, 0x00),
            (128, 0x01),
            (255, 0x01),
            (256, 0x02),
            (383, 0x02),
            (384, 0x00),
            (511, 0x00),
            (512, 0x01),
            (767, 0x02),
            (768, 0x00),
        ] {
            assert_eq!(
                answer(Function::Clock, tick, 0x80, 0x03).unwrap(),
                Atom::Number(step),
                "~. 80 03 at Tick {tick}"
            );
        }
    }

    #[test]
    fn clock_counts_a_tick_far_beyond_the_byte_the_step_is_answered_as() {
        // The Tick is a `u64` and the answer is a Number, so the division has
        // to happen in the wider integer: a Tick folded into a byte first would
        // answer about Tick 232 here rather than about Tick 1000.
        assert_eq!(
            answer(Function::Clock, 1000, 0x03, 0x04).unwrap(),
            Atom::Number(1)
        );

        // And at a Tick no run reaches, which is where a narrower intermediate
        // would have wrapped many times over.
        assert_eq!(
            answer(Function::Clock, u64::from(u32::MAX), 0xFF, 0x10).unwrap(),
            Atom::Number(u8::try_from((u64::from(u32::MAX) / 0xFF) % 0x10).unwrap())
        );
    }

    #[test]
    fn clock_diagnoses_a_zero_rate_before_a_zero_modulus() {
        // A cycle needs both factors, so either zero refuses. The order is
        // signature order, which is the order every other operand fault is
        // reported in, and `00 00` is what pins it: a Source that wrote two
        // zeroes is told about the rate, the earlier Cell pair.
        for tick in [0u64, 1, 7] {
            let rate = answer(Function::Clock, tick, 0x00, 0x04).unwrap_err();
            assert_eq!(rate.to_string(), "~. cannot count a cycle with a zero rate");

            let modulus = answer(Function::Clock, tick, 0x04, 0x00).unwrap_err();
            assert_eq!(
                modulus.to_string(),
                "~. cannot count a cycle with a zero modulus"
            );

            let both = answer(Function::Clock, tick, 0x00, 0x00).unwrap_err();
            assert_eq!(both.to_string(), "~. cannot count a cycle with a zero rate");
        }
    }

    #[test]
    fn delay_bangs_once_per_cycle_beginning_at_the_first_tick() {
        // ADR 0012 counts Tick `0` as a Tick like any other, so a Delay fires
        // as a Playback run starts rather than one cycle into it. The sweep
        // states the whole rule — Bang exactly on a multiple of the cycle — and
        // Tick `0` is the case a reference computed as "some Ticks have passed"
        // would get wrong. Like Clock's sweep it reaches every pair rather than
        // every claim: the cycle it compares against is the product the body
        // takes, so it pins operand order and the width of the multiply and
        // leaves the shape to
        // `delay_bangs_on_the_literal_ticks_its_operands_name`.
        for rate in 1..=8u8 {
            for modulus in 1..=8u8 {
                for tick in 0..256u64 {
                    let cycle = u64::from(rate) * u64::from(modulus);

                    assert_eq!(
                        banged(Function::Delay, tick, rate, modulus),
                        tick % cycle == 0,
                        "~* {rate:02X} {modulus:02X} at Tick {tick}"
                    );
                }
            }
        }
    }

    #[test]
    fn delay_bangs_on_the_literal_ticks_its_operands_name() {
        // The Ticks written out rather than a multiple recomputed: `~* 03 02`
        // is a cycle of six Ticks, so it Bangs at `0`, `6`, `12`, `18` and
        // nowhere between, and every Tick up to the last of those is asked so
        // that "nowhere between" is checked rather than assumed. The sweep
        // above compares against the same product the body multiplies, so this
        // is where the cycle length itself is claimed.
        //
        // The last pair is 384 Ticks long, which is the case a byte product
        // would fold: `40 * 06` wraps to `80`, and a Delay counting that would
        // Bang three times inside every cycle written here.
        for (rate, modulus, bangs) in [
            (0x03u8, 0x02u8, &[0u64, 6, 12, 18][..]),
            (0x01, 0x04, &[0, 4, 8, 12, 16]),
            (0x05, 0x01, &[0, 5, 10, 15, 20]),
            (0x02, 0x03, &[0, 6, 12, 18]),
            (0x40, 0x06, &[0, 384, 768]),
        ] {
            let last = *bangs.last().unwrap();

            for tick in 0..=last {
                assert_eq!(
                    banged(Function::Delay, tick, rate, modulus),
                    bangs.contains(&tick),
                    "~* {rate:02X} {modulus:02X} at Tick {tick}"
                );
            }
        }
    }

    #[test]
    fn delay_with_modulus_one_bangs_once_per_rate_ticks_rather_than_every_tick() {
        // Named by ADR 0012 because it is what makes the two operands a rate
        // and a step count instead of two spellings of one period: a modulus of
        // `01` is a cycle of one step, and that step is `rate` Ticks long.
        for tick in 0..32u64 {
            assert_eq!(
                banged(Function::Delay, tick, 0x04, 0x01),
                tick % 4 == 0,
                "Tick {tick}"
            );
        }
    }

    #[test]
    fn delay_counts_a_cycle_product_wider_than_a_byte() {
        // The test that catches a byte multiply. `10 * 20` is 512, which wraps
        // a byte to zero — a cycle length that would then be divided by — and
        // `10 * 11` is 272, which wraps to 16 and would Bang seventeen times
        // per cycle instead of once.
        for tick in [0u64, 512, 1024] {
            assert!(
                banged(Function::Delay, tick, 0x10, 0x20),
                "512-Tick cycle at Tick {tick}"
            );
        }

        for tick in [1u64, 16, 256, 511, 513] {
            assert!(
                !banged(Function::Delay, tick, 0x10, 0x20),
                "512-Tick cycle at Tick {tick}"
            );
        }

        for tick in [0u64, 272, 544] {
            assert!(
                banged(Function::Delay, tick, 0x10, 0x11),
                "272-Tick cycle at Tick {tick}"
            );
        }

        // Every multiple of the wrapped product below the real one, which is
        // exactly what a byte multiply would have Banged on.
        for tick in (16..272).step_by(16) {
            assert!(
                !banged(Function::Delay, tick, 0x10, 0x11),
                "272-Tick cycle at Tick {tick}"
            );
        }
    }

    #[test]
    fn delay_diagnoses_a_zero_rate_before_a_zero_modulus() {
        // The same two faults Clock refuses, under a diagnostic that names
        // Delay: a Source shown "cannot count a cycle" has to be able to tell
        // which of the two Cell pairs it wrote is the one to edit.
        let rate = answer(Function::Delay, 5, 0x00, 0x04).unwrap_err();
        assert_eq!(rate.to_string(), "~* cannot count a cycle with a zero rate");

        let modulus = answer(Function::Delay, 5, 0x04, 0x00).unwrap_err();
        assert_eq!(
            modulus.to_string(),
            "~* cannot count a cycle with a zero modulus"
        );

        // Tick `0` is a multiple of every cycle, so it is the Tick a Delay that
        // skipped its validation would answer Bang at for any operands at all.
        let zero = answer(Function::Delay, 0, 0x00, 0x00).unwrap_err();
        assert_eq!(zero.to_string(), "~* cannot count a cycle with a zero rate");
    }

    #[test]
    fn euclidean_places_its_hits_where_the_adr_formula_does() {
        // Hand-written patterns rather than the formula recomputed: one `X` per
        // Tick of one cycle, read left to right from Tick `0`. These are the
        // Euclidean rhythms the distribution is named for — `03 08` is the
        // tresillo — so a formula that drifted would have to drift into another
        // recognisable pattern to pass.
        //
        // Each is checked over three cycles, which is what pins the reduction
        // of the absolute Tick: a phase computed from an unreduced Tick agrees
        // with this on the first cycle and disagrees afterwards.
        for (hits, steps, pattern) in [
            (0x03, 0x04, "X.XX"),
            (0x03, 0x08, "X..X..X."),
            (0x05, 0x08, "X.X.XX.X"),
            (0x02, 0x05, "X..X."),
            (0x02, 0x03, "X.X"),
            (0x01, 0x04, "X..."),
            (0x07, 0x10, "X..X.X.X..X.X.X."),
        ] {
            assert_eq!(usize::from(steps), pattern.len(), "{hits} of {steps}");

            for (tick, expected) in pattern.chars().cycle().take(pattern.len() * 3).enumerate() {
                assert_eq!(
                    banged(Function::Euclidean, tick as u64, hits, steps),
                    expected == 'X',
                    "~% {hits:02X} {steps:02X} at Tick {tick}"
                );
            }
        }
    }

    #[test]
    fn euclidean_answers_no_hits_and_a_full_cycle_from_the_same_formula() {
        // ADR 0012 states both as consequences rather than as cases: with
        // positive steps, zero hits never Bangs and equal hits and steps Bang
        // every Tick. A body that special-cased either would pass this too —
        // what the test is for is the opposite, that neither needs a case.
        for steps in 1..=16u8 {
            for tick in 0..64u64 {
                assert!(
                    !banged(Function::Euclidean, tick, 0x00, steps),
                    "~% 00 {steps:02X} at Tick {tick}"
                );
                assert!(
                    banged(Function::Euclidean, tick, steps, steps),
                    "~% {steps:02X} {steps:02X} at Tick {tick}"
                );
            }
        }
    }

    #[test]
    fn euclidean_reduces_a_tick_no_multiplication_could_have_survived() {
        // Reducing before the phase offset is what ADR 0012 asks for, and the
        // reason is arithmetic rather than style: `hits * (t + steps - 1)` over
        // an absolute Tick near the end of a saturating counter overflows,
        // while the reduced form is the same pattern at the same position.
        //
        // The last eight Ticks of the counter are exactly one cycle of `08`,
        // because `u64::MAX` is seven more than a multiple of eight.
        let pattern = b"X..X..X.";
        let steps = 0x08u8;

        for tick in (u64::MAX - 7)..=u64::MAX {
            let position = usize::try_from(tick % u64::from(steps)).unwrap();

            assert_eq!(
                banged(Function::Euclidean, tick, 0x03, steps),
                pattern[position] == b'X',
                "Tick {tick}"
            );
        }
    }

    #[test]
    fn euclidean_refuses_a_cycle_of_no_steps_before_it_counts_its_hits() {
        // The ordering ADR 0012 states outright. `00 00` is the case it is
        // stated for: a cycle with no positions rather than a pattern with no
        // onsets, so it diagnoses instead of quietly answering the Absence
        // Marker forever the way `~% 00 04` does.
        for hits in [0x00u8, 0x01, 0xFF] {
            let error = answer(Function::Euclidean, 3, hits, 0x00).unwrap_err();
            assert_eq!(
                error.to_string(),
                "~% cannot count a cycle with a zero step count",
                "~% {hits:02X} 00"
            );
        }

        // More onsets than positions is the second check, and it names both
        // operands because either Cell pair is the one to edit.
        let error = answer(Function::Euclidean, 3, 0x05, 0x04).unwrap_err();
        assert_eq!(error.to_string(), "~% cannot fit 05 hits into 04 steps");
    }

    #[test]
    fn a_note_operand_diagnoses_in_every_tick_function() {
        // All six operands are declared Number, so a Note is refused at each of
        // them rather than converted: ADR 0021 makes the Numeric Conversion
        // Functions the only crossing between the two numeric types, and a rate
        // that silently read a Note's byte would be a seventh crossing.
        let note = Atom::Note(Note::try_from(0x3C).unwrap());

        for function in [
            Function::Clock,
            Function::Delay,
            Function::Euclidean,
            Function::Increment,
            Function::Interpolation,
        ] {
            for (left, right) in [
                (note, Atom::Number(0x04)),
                (Atom::Number(0x04), note),
                (note, note),
            ] {
                let error = evaluate(function, 5, left, right).unwrap_err();
                assert!(
                    matches!(error, Error::Type(TypeError::Number(ref found)) if found == "C4"),
                    "{function:?}({left:?}, {right:?}) gave {error:?}"
                );
            }
        }
    }

    #[test]
    fn a_clock_broadcasts_one_step_per_element() {
        // Clock answers a Number, so ADR 0039 leaves it pervasive and it
        // extends element-wise like any other Atomic Function: a scalar operand
        // repeats and equal lengths pair.
        assert_eq!(
            evaluate(Function::Clock, 7, Atom::Number(0x02), numbers([2, 4, 8])).unwrap(),
            Interpretation::Sequence(numbers([1, 3, 3])),
        );

        assert_eq!(
            evaluate(Function::Clock, 7, numbers([1, 2, 4]), numbers([4, 4, 4])).unwrap(),
            Interpretation::Sequence(numbers([3, 3, 1])),
        );

        // An empty Sequence operand is a width of no elements rather than a
        // scalar, so the answer is the empty Sequence: Clock is the one of the
        // three that has an answer of that shape to give.
        for (left, right) in [
            (
                Value::from(Atom::Number(0x02)),
                Value::from(Sequence::empty()),
            ),
            (Sequence::empty().into(), Atom::Number(0x02).into()),
            (Sequence::empty().into(), Sequence::empty().into()),
        ] {
            assert_eq!(
                evaluate(Function::Clock, 7, left.clone(), right.clone()).unwrap(),
                Interpretation::Sequence(Sequence::empty()),
                "~. {left:?} {right:?}"
            );
        }
    }

    #[test]
    fn a_clock_element_fault_diagnoses_the_complete_operation() {
        // The all-or-nothing rule on the one Function of the three that can
        // still meet it. A zero modulus at the last element refuses the whole
        // answer rather than leaving a Sequence of the steps that did count,
        // and the fault is raised at whichever element holds it.
        for (left, right, message) in [
            (
                Value::from(Atom::Number(0x02)),
                Value::from(numbers([4, 8, 0])),
                "~. cannot count a cycle with a zero modulus",
            ),
            (
                numbers([1, 0]).into(),
                Atom::Number(0x04).into(),
                "~. cannot count a cycle with a zero rate",
            ),
        ] {
            let error = evaluate(Function::Clock, 7, left, right).unwrap_err();
            assert_eq!(error.to_string(), message);
        }
    }

    #[test]
    fn a_clock_diagnoses_two_non_scalar_operands_of_different_lengths() {
        // Ordinary ADR 0007 shape rules, including an empty Sequence against a
        // non-empty one: a shape fault is settled before any element is read,
        // so it precedes every diagnostic the formula could raise. Only Clock
        // can reach this now — the other two refuse the first Sequence they see
        // and never compare two lengths.
        for (left, right, lengths) in [
            (
                Value::from(numbers([1, 2])),
                Value::from(numbers([1, 2, 3])),
                (2, 3),
            ),
            (Sequence::empty().into(), numbers([1, 2]).into(), (0, 2)),
            (numbers([1, 2]).into(), Sequence::empty().into(), (2, 0)),
        ] {
            assert!(
                matches!(
                    evaluate(Function::Clock, 5, left.clone(), right.clone()),
                    Err(Error::Sequence(SequenceError::IncompatibleLengths { left: l, right: r }))
                        if (l, r) == lengths
                ),
                "~. {left:?} {right:?}"
            );
        }
    }

    #[test]
    fn a_pulse_refuses_a_sequence_at_either_operand_position() {
        // ADR 0039. A widened pulse would need one answer per element and an
        // element that does not Bang has only the Absence Marker to offer,
        // which ADR 0025 refuses as a Sequence member; every reduction to one
        // answer fixes a meaning for layered rhythms that could not be changed
        // later without breaking Source, so the operand is refused instead.
        //
        // The refusal is the declaration's, not a check in either body, so it
        // is claimed at both operand positions of both Functions — including
        // the empty Sequence, which a body checking for members to walk would
        // let through as an operation of nothing.
        for function in [Function::Delay, Function::Euclidean] {
            for (left, right, found) in [
                (
                    Value::from(numbers([2, 3])),
                    Value::from(Atom::Number(0x04)),
                    "0203",
                ),
                (Atom::Number(0x04).into(), numbers([2, 3]).into(), "0203"),
                (numbers([2, 3]).into(), numbers([4, 8]).into(), "0203"),
                (Sequence::empty().into(), Atom::Number(0x04).into(), ""),
                (Atom::Number(0x04).into(), Sequence::empty().into(), ""),
            ] {
                assert!(
                    matches!(
                        evaluate(function, 12, left.clone(), right.clone()),
                        Err(Error::Sequence(SequenceError::ExpectedAtom(ref rendered)))
                            if rendered == found
                    ),
                    "{function:?}({left:?}, {right:?})"
                );
            }
        }
    }

    #[test]
    fn a_pulse_refuses_a_sequence_before_it_reads_the_numbers_inside_it() {
        // The refusal is settled in `Stack::broadcast`, which runs before any
        // element binds, so a Sequence carrying operands the formula would also
        // refuse is answered as the shape fault it is. A body that walked the
        // members first would report the zero and leave the Source believing a
        // Sequence operand is admissible once its members are fixed.
        let error = evaluate(Function::Delay, 0, numbers([1, 0]), Atom::Number(0x04)).unwrap_err();
        assert_eq!(
            error.to_string(),
            r#"expected an Atom, found the Sequence "0100""#
        );

        let error = evaluate(
            Function::Euclidean,
            0,
            Atom::Number(0x02),
            numbers([4, 8, 0]),
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            r#"expected an Atom, found the Sequence "040800""#
        );

        // And two Sequences of different lengths, which is the other shape
        // fault: the pervasion answer precedes the length comparison, so the
        // first Sequence in signature order is the one the Source is shown.
        let error = evaluate(Function::Delay, 0, numbers([1, 2]), numbers([1, 2, 3])).unwrap_err();
        assert_eq!(
            error.to_string(),
            r#"expected an Atom, found the Sequence "0102""#
        );
    }

    /// The Atom `function` answers for one pair of Numbers at a stated previous.
    fn feedback(function: Function, previous: &str, left: u8, right: u8) -> Result<Atom, Error> {
        match evaluate_previous(
            function,
            0,
            Atom::Number(left),
            Atom::Number(right),
            previous,
        )? {
            Interpretation::Cell(atom) => Ok(atom),
            other => panic!("expected a Cell result, found {other:?}"),
        }
    }

    #[test]
    fn increment_advances_by_step_and_wraps_at_its_modulus() {
        // The formula is small enough to enumerate against rather than sample:
        // every step and modulus from `01` to `08` from every previous in the
        // byte, which is enough wraps for every pair. The reference here is
        // ADR 0012's expression retyped, so what the sweep pins is operand
        // order and the width the arithmetic is done in — a step read as a
        // modulus fails at every asymmetric pair — and not the shape of the
        // expression itself.
        // `increment_counts_the_literal_sequence_its_operands_name` is the
        // independent half, written from what the operands mean.
        for step in 1..=8u8 {
            for modulus in 1..=8u8 {
                for previous in 0..=u8::MAX {
                    let expected = (u64::from(previous) + u64::from(step)) % u64::from(modulus);

                    assert_eq!(
                        feedback(
                            Function::Increment,
                            &format!("{previous:02X}"),
                            step,
                            modulus
                        )
                        .unwrap(),
                        Atom::Number(u8::try_from(expected).unwrap()),
                        "~+ {step:02X} {modulus:02X} from {previous:02X}"
                    );
                }
            }
        }
    }

    #[test]
    fn increment_counts_the_literal_sequence_its_operands_name() {
        // Hand-written wraps, one entry per Tick, read from what the operands
        // mean rather than from the expression the body evaluates: `~+ 01 04`
        // from `00` is `01 02 03 00` and nothing else. A body that added the
        // modulus, or wrapped at the step, would have to land on that same
        // sequence to pass here.
        //
        // Each sequence is walked three times over, which separates a wrap
        // that begins again from a counter that keeps counting.
        for (step, modulus, wraps) in [
            (0x01u8, 0x04u8, &[0x01u8, 0x02, 0x03, 0x00][..]),
            (0x02, 0x08, &[0x02, 0x04, 0x06, 0x00]),
            (0x03, 0x05, &[0x03, 0x01, 0x04, 0x02, 0x00]),
            (0x01, 0x01, &[0x00]),
        ] {
            let mut previous = "  ".to_owned();

            for expected in wraps.iter().cycle().take(wraps.len() * 3) {
                let atom = feedback(Function::Increment, &previous, step, modulus).unwrap();
                assert_eq!(
                    atom,
                    Atom::Number(*expected),
                    "~+ {step:02X} {modulus:02X} from {previous:?}"
                );
                previous = format!("{expected:02X}");
            }
        }
    }

    #[test]
    fn increment_adds_in_a_wider_integer_before_the_modulus() {
        // `FF + 02` is 257, which wraps a byte to `01` before the modulus
        // would ever see it. Taking the modulus of that wrapped total at
        // `10` would answer `01`; the wider sum answers `01` too at this
        // pair, so the case that tells them apart is a modulus that is not
        // a factor of the wrap: `FF + 02` modulo `0F` is `02`, and a byte
        // add would have taken `01 % 0F`.
        assert_eq!(
            feedback(Function::Increment, "FF", 0x02, 0x0F).unwrap(),
            Atom::Number(0x02)
        );

        // Empty is Number `00`, so the first increment of an unused Portal
        // is the step itself, wrapped.
        assert_eq!(
            feedback(Function::Increment, "  ", 0x03, 0x08).unwrap(),
            Atom::Number(0x03)
        );
    }

    #[test]
    fn increment_diagnoses_a_zero_modulus() {
        // A wrap needs a length, so a zero modulus refuses. The diagnostic
        // names Increment and the role rather than Modulo: a Source shown
        // "cannot modulo by zero" would be told about a Function it did not
        // write.
        let error = feedback(Function::Increment, "04", 0x01, 0x00).unwrap_err();
        assert_eq!(error.to_string(), "~+ cannot wrap at a zero modulus");

        // Empty previous is `00`, and a zero step does not skip the check:
        // `(0 + 0) % 0` is the same fault as any other pair.
        let error = feedback(Function::Increment, "  ", 0x00, 0x00).unwrap_err();
        assert_eq!(error.to_string(), "~+ cannot wrap at a zero modulus");
    }

    #[test]
    fn interpolation_moves_toward_target_without_overshoot() {
        // ADR 0012's three orderings, written as the Numbers they name
        // rather than as the expression the body evaluates. Below steps up
        // by at most `rate` and lands on the target rather than past it;
        // above steps down the same way; equal is the target unchanged.
        for (previous, rate, target, expected) in [
            (0x00u8, 0x02u8, 0x10u8, 0x02u8),
            (0x0E, 0x03, 0x10, 0x10),
            (0x0F, 0x03, 0x10, 0x10),
            (0x10, 0x03, 0x10, 0x10),
            (0x14, 0x03, 0x10, 0x11),
            (0x11, 0x03, 0x10, 0x10),
            (0x02, 0xFF, 0x01, 0x01),
            (0x00, 0xFF, 0xFF, 0xFF),
            (0xFF, 0xFF, 0x00, 0x00),
        ] {
            assert_eq!(
                feedback(
                    Function::Interpolation,
                    &format!("{previous:02X}"),
                    rate,
                    target
                )
                .unwrap(),
                Atom::Number(expected),
                "~> {rate:02X} {target:02X} from {previous:02X}"
            );
        }
    }

    #[test]
    fn interpolation_holds_when_its_rate_is_zero() {
        // Rate `00` is a step of nothing, so the current value is unchanged
        // whether it sits below the target, above it, or on it. The hold is
        // the formula, not a case: a body that special-cased zero would pass
        // this too — what the test is for is the opposite, that it needs none.
        for (previous, target) in [(0x05u8, 0x10u8), (0x80, 0x10), (0x10, 0x10)] {
            assert_eq!(
                feedback(
                    Function::Interpolation,
                    &format!("{previous:02X}"),
                    0x00,
                    target
                )
                .unwrap(),
                Atom::Number(previous),
                "~> 00 {target:02X} from {previous:02X}"
            );
        }

        // Empty is Number `00`, so a hold from an unused Portal stays `00`.
        assert_eq!(
            feedback(Function::Interpolation, "  ", 0x00, 0x10).unwrap(),
            Atom::Number(0x00)
        );
    }

    #[test]
    fn a_feedback_function_diagnoses_an_occupied_previous() {
        // A previous that is not a Number is refused rather than converted:
        // A Note spelling that cannot decode as a Number is supplied, and both
        // Functions name themselves so a Source shown the message knows which
        // of the two Cell pairs it wrote is the one that cannot read.
        for function in [Function::Increment, Function::Interpolation] {
            let error = feedback(function, "G4", 0x01, 0x04).unwrap_err();
            assert_eq!(
                error.to_string(),
                format!("{function} cannot read a previous value that is not a Number"),
                "{function:?}"
            );
        }
    }

    #[test]
    fn cell_operand_faults_precede_an_invalid_portal_input() {
        for function in [Function::Increment, Function::Interpolation] {
            let note = Atom::Note(Note::try_from(60).unwrap());
            for (left, right) in [(note, Atom::Number(4)), (Atom::Number(1), note)] {
                assert!(matches!(
                    evaluate_previous(function, 0, left, right, "G4"),
                    Err(Error::Type(TypeError::Number(_)))
                ));
            }
            for (left, right) in [
                (Value::from(numbers([1, 2])), Value::from(Atom::Number(4))),
                (Atom::Number(1).into(), numbers([3, 4]).into()),
            ] {
                assert!(matches!(
                    evaluate_previous(function, 0, left, right, "G4"),
                    Err(Error::Sequence(SequenceError::ExpectedAtom(_)))
                ));
            }
        }
    }

    #[test]
    fn a_feedback_function_refuses_a_sequence_at_either_operand_position() {
        // ADR 0012. The previous is one Atom at the ordinary result Portal,
        // and a Sequence operand would need hidden element identity that one
        // Cell pair cannot hold, so the operand is refused by the declaration
        // before either body runs — including the empty Sequence, which a
        // body checking for members to walk would let through as an operation
        // of nothing.
        for function in [Function::Increment, Function::Interpolation] {
            for (left, right, found) in [
                (
                    Value::from(numbers([2, 3])),
                    Value::from(Atom::Number(0x04)),
                    "0203",
                ),
                (Atom::Number(0x04).into(), numbers([2, 3]).into(), "0203"),
                (numbers([2, 3]).into(), numbers([4, 8]).into(), "0203"),
                (Sequence::empty().into(), Atom::Number(0x04).into(), ""),
                (Atom::Number(0x04).into(), Sequence::empty().into(), ""),
            ] {
                assert!(
                    matches!(
                        evaluate(function, 0, left.clone(), right.clone()),
                        Err(Error::Sequence(SequenceError::ExpectedAtom(ref rendered)))
                            if rendered == found
                    ),
                    "{function:?}({left:?}, {right:?})"
                );
            }
        }
    }

    #[test]
    fn a_feedback_function_refuses_a_sequence_before_it_reads_the_numbers_inside_it() {
        // The refusal is settled in `Stack::broadcast`, which runs before any
        // element binds, so a Sequence carrying a zero modulus is answered as
        // the shape fault it is. A body that walked the members first would
        // report the wrap and leave the Source believing a Sequence operand
        // is admissible once its members are fixed.
        let error =
            evaluate(Function::Increment, 0, numbers([1, 0]), Atom::Number(0x04)).unwrap_err();
        assert_eq!(
            error.to_string(),
            r#"expected an Atom, found the Sequence "0100""#
        );

        let error = evaluate(
            Function::Interpolation,
            0,
            Atom::Number(0x02),
            numbers([4, 8, 0]),
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            r#"expected an Atom, found the Sequence "040800""#
        );
    }

    /// Evaluates Random at a stated Tick and anchor.
    ///
    /// The anchor is part of the stream, so these helpers take it rather than
    /// defaulting to the origin the two-operand Tick Functions use.
    fn evaluate_random(
        tick: u64,
        column: usize,
        row: usize,
        seed: impl Into<Value>,
        minimum: impl Into<Value>,
        maximum: impl Into<Value>,
    ) -> Result<Interpretation, Error> {
        Interpreter::execute_function(
            Function::Random,
            [seed.into(), minimum.into(), maximum.into()],
            TickInputs::new(Tick::new(tick), Anchor::new(column, row)).into(),
        )
    }

    /// The Atom Random answers for one triple of Numbers.
    fn random_answer(
        tick: u64,
        column: usize,
        row: usize,
        seed: u8,
        minimum: u8,
        maximum: u8,
    ) -> Result<Atom, Error> {
        match evaluate_random(
            tick,
            column,
            row,
            Atom::Number(seed),
            Atom::Number(minimum),
            Atom::Number(maximum),
        )? {
            Interpretation::Cell(atom) => Ok(atom),
            other => panic!("expected a Cell result, found {other:?}"),
        }
    }

    /// ADR 0013's 32-byte ChaCha seed, assembled here from the stated layout
    /// rather than from the Function body.
    fn adr_seed(seed: u8, tick: u64, column: i64, row: i64, sequence: u32) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        bytes[0] = seed;
        bytes[1..9].copy_from_slice(&tick.to_le_bytes());
        bytes[9..17].copy_from_slice(&column.to_le_bytes());
        bytes[17..25].copy_from_slice(&row.to_le_bytes());
        bytes[25..29].copy_from_slice(&sequence.to_le_bytes());
        bytes
    }

    #[test]
    fn random_normalizes_reversed_bounds_and_returns_an_equal_bound() {
        // Reversed bounds are the same range, so `10 00` answers what `00 10`
        // answers. Equal bounds have one member and return it, which is the
        // case a body that always asked ChaCha8 would still pass — what the
        // test is for is the opposite, that the range is settled before the
        // stream is.
        assert_eq!(
            random_answer(0, 0, 0, 0x01, 0x00, 0x10).unwrap(),
            Atom::Number(0x02)
        );
        assert_eq!(
            random_answer(0, 0, 0, 0x01, 0x10, 0x00).unwrap(),
            Atom::Number(0x02)
        );
        assert_eq!(
            random_answer(0, 0, 0, 0x01, 0x2A, 0x2A).unwrap(),
            Atom::Number(0x2A)
        );
    }

    #[test]
    fn random_golden_vectors_pin_the_adr_seed_chacha8_word_and_range_mapping() {
        // Computed independently from ADR 0013's layout plus `rand_chacha`
        // 0.10's ChaCha8Rng, not by reading the Function body. A dependency
        // upgrade that changes the first `u64`, or a body that writes the
        // Tick into the column slot, fails a literal rather than agreeing
        // with itself.
        //
        // Origin: seed `01`, Tick `0`, column `0`, row `0`, Sequence `0`.
        const ORIGIN_SEED: [u8; 32] = [
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        const ORIGIN_WORD: u64 = 0x61a9_4a49_a0e9_5ecf;

        assert_eq!(adr_seed(0x01, 0, 0, 0, 0), ORIGIN_SEED);
        assert_eq!(
            ChaCha8Rng::from_seed(ORIGIN_SEED).next_u64(),
            ORIGIN_WORD,
            "ChaCha8 first u64 at the origin seed"
        );
        // Width of `00`–`FF` is 256, not a wrapping 0. The first word's low
        // byte is `CF`; modulo 256 is that byte, which is the accepted
        // bias the ADR documents.
        assert_eq!(ORIGIN_WORD % 256, 0xCF);
        assert_eq!(
            random_answer(0, 0, 0, 0x01, 0x00, 0xFF).unwrap(),
            Atom::Number(0xCF)
        );
        // Width 17: `7037237572835827407 % 17 == 2`.
        assert_eq!(ORIGIN_WORD % 17, 2);
        assert_eq!(
            random_answer(0, 0, 0, 0x01, 0x00, 0x10).unwrap(),
            Atom::Number(0x02)
        );

        // Asymmetric Position and a non-zero Tick, so a transposition of
        // column and row, or a Tick written as `i64`, is visible.
        const OFFSET_SEED: [u8; 32] = [
            0x01, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        const OFFSET_WORD: u64 = 0x9142_5add_444f_79fb;

        assert_eq!(adr_seed(0x01, 7, 3, 5, 0), OFFSET_SEED);
        assert_eq!(ChaCha8Rng::from_seed(OFFSET_SEED).next_u64(), OFFSET_WORD);
        assert_eq!(
            random_answer(7, 3, 5, 0x01, 0x00, 0xFF).unwrap(),
            Atom::Number(0xFB)
        );
    }

    #[test]
    fn random_sequence_index_distinguishes_broadcast_elements() {
        // Sequence index occupies bytes `[25, 29)`. Two elements with the
        // same bounds at the same Position would share a stream if the
        // index were left at `0` for every one.
        const SEQ1_SEED: [u8; 32] = [
            0x01, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        const SEQ1_WORD: u64 = 0x2487_bf0e_d164_dbd0;

        assert_eq!(adr_seed(0x01, 7, 3, 5, 1), SEQ1_SEED);
        assert_eq!(ChaCha8Rng::from_seed(SEQ1_SEED).next_u64(), SEQ1_WORD);
        assert_ne!(SEQ1_WORD, 0x9142_5add_444f_79fb);

        assert_eq!(
            evaluate_random(
                0,
                0,
                0,
                Atom::Number(0x01),
                numbers([0, 0, 0]),
                numbers([0x10, 0x10, 0x10])
            )
            .unwrap(),
            Interpretation::Sequence(numbers([0x02, 0x0B, 0x07])),
        );
    }

    #[test]
    fn two_randoms_at_different_anchors_differ_and_moving_one_changes_its_stream() {
        // Own-anchor, not Expression-root: `(3, 5)` and `(4, 5)` are two
        // Functions, and moving one column is enough to change the stream.
        // Identical inputs reproduce it.
        let at_three = random_answer(7, 3, 5, 0x01, 0x00, 0xFF).unwrap();
        let at_four = random_answer(7, 4, 5, 0x01, 0x00, 0xFF).unwrap();

        assert_eq!(at_three, Atom::Number(0xFB));
        assert_eq!(at_four, Atom::Number(0xB5));
        assert_ne!(at_three, at_four);
        assert_eq!(random_answer(7, 3, 5, 0x01, 0x00, 0xFF).unwrap(), at_three);
    }

    #[test]
    fn a_note_operand_diagnoses_in_random() {
        // All three operands are declared Number, so a Note is refused at
        // each of them rather than converted: ADR 0021 makes the Numeric
        // Conversion Functions the only crossing between the two numeric
        // types.
        let note = Atom::Note(Note::try_from(0x3C).unwrap());
        let number = Atom::Number(0x04);

        for (seed, minimum, maximum) in [
            (note, number, number),
            (number, note, number),
            (number, number, note),
            (note, note, note),
        ] {
            let error = evaluate_random(5, 0, 0, seed, minimum, maximum).unwrap_err();
            assert!(
                matches!(error, Error::Type(TypeError::Number(ref found)) if found == "C4"),
                "~?({seed:?}, {minimum:?}, {maximum:?}) gave {error:?}"
            );
        }
    }

    #[test]
    fn a_random_broadcasts_one_draw_per_element() {
        // Random answers a Number, so it extends element-wise like Clock: a
        // scalar operand repeats and equal lengths pair. Sequence index is
        // what keeps the three draws from collapsing to one stream.
        assert_eq!(
            evaluate_random(
                0,
                0,
                0,
                Atom::Number(0x01),
                Atom::Number(0x00),
                numbers([0x10, 0x10, 0x10])
            )
            .unwrap(),
            Interpretation::Sequence(numbers([0x02, 0x0B, 0x07])),
        );

        for (seed, minimum, maximum) in [
            (
                Value::from(Atom::Number(0x01)),
                Value::from(Atom::Number(0x00)),
                Value::from(Sequence::empty()),
            ),
            (
                Sequence::empty().into(),
                Atom::Number(0x00).into(),
                Atom::Number(0x10).into(),
            ),
            (
                Sequence::empty().into(),
                Sequence::empty().into(),
                Sequence::empty().into(),
            ),
        ] {
            assert_eq!(
                evaluate_random(0, 0, 0, seed.clone(), minimum.clone(), maximum.clone()).unwrap(),
                Interpretation::Sequence(Sequence::empty()),
                "~? {seed:?} {minimum:?} {maximum:?}"
            );
        }
    }

    #[test]
    fn a_random_diagnoses_two_non_scalar_operands_of_different_lengths() {
        assert!(matches!(
            evaluate_random(
                5,
                0,
                0,
                Atom::Number(0x01),
                numbers([1, 2]),
                numbers([1, 2, 3]),
            ),
            Err(Error::Sequence(SequenceError::IncompatibleLengths {
                left: 2,
                right: 3
            }))
        ));
    }
}
