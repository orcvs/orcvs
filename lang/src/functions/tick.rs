use crate::{
    Atom, Error, Function, InterpretationError, Value,
    atom::operands::{Clock, Delay, Euclidean},
    interpreter::Context,
};

// ADR 0012's Tick-reading Functions. Each reads the absolute Tick from the
// interpretation `Context` and nothing else: no clock, no static, no counter of
// its own. That is what makes the same Source Snapshot at the same Tick answer
// the same way, and it is why the Tick is lifted out of `ctx` once here rather
// than reached for inside each element's closure — the element answer is about
// one pair of Numbers at one Tick, exactly as `math::add` is about one pair of
// Numbers, and the Tick is shared by the whole operation because an Expression
// is evaluated at one Tick.
//
// Two of the three answer a pulse rather than a Number, so they go through
// `Stack::predicate` and not `Stack::apply`. ADR 0012 has each broadcast
// element apply the formula independently at the same Tick, which `predicate`
// does — it binds and asks every element — but the answer is one whole value
// because the alternative does not exist: an element that does not Bang has
// nothing to put at its position, the only Atom meaning nothing is the absence
// marker, and `Sequence::new` refuses that as a member because it has no Source
// encoding. This is the reasoning `Stack::predicate` and the Absence Marker
// already record for Equality, reaching its second and third Functions.
//
// Every formula is evaluated in `u64`. The Tick is already one, and the two
// operands are Numbers whose product is a cycle length rather than a value the
// Source wrote: `~* 10 20` is a cycle of 512 Ticks, which a byte cannot hold and
// a byte multiply would fold to zero.

///
/// The two factors of a cycle length, refused where either is zero.
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
fn cycle(function: Function, rate: u8, modulus: u8) -> Result<(u64, u64), Error> {
    if rate == 0 {
        return Err(InterpretationError::ZeroCycle {
            function,
            operand: "rate",
        }
        .into());
    }

    if modulus == 0 {
        return Err(InterpretationError::ZeroCycle {
            function,
            operand: "modulus",
        }
        .into());
    }

    Ok((u64::from(rate), u64::from(modulus)))
}

/// Clock: `~. rate modulus`.
///
/// The step a cycle of `rate * modulus` Ticks is at, as a Number: `rate` Ticks
/// to a step and `modulus` steps to the cycle, so the answer counts `00`,
/// `01`, … up to `modulus - 1` and begins again. It is the one Function of the
/// three that answers a value, so it broadcasts through `Stack::apply` and a
/// Sequence operand answers a Sequence of steps.
#[inline(always)]
pub fn clock(ctx: &mut Context) -> Result<Value, Error> {
    let tick = ctx.inputs.tick().get();

    ctx.stack.apply(move |Clock { rate, modulus }: Clock| {
        let (rate, modulus) = cycle(Function::Clock, rate, modulus)?;
        let step = (tick / rate) % modulus;

        // A remainder of `modulus` is below it and `modulus` came out of a
        // Number, so narrowing the step back into one is total. It is written
        // as a conversion with a named fallback rather than an unwrap because
        // this runs inside a Tick under the Source write guard ADR 0028 rules a
        // panic out of, exactly as `Stack::convert` does for the state its own
        // bound rules out. The fallback is the first step of every cycle, and
        // it is unreachable.
        Ok(Atom::Number(u8::try_from(step).unwrap_or(0)))
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
#[inline(always)]
pub fn delay(ctx: &mut Context) -> Result<Value, Error> {
    let tick = ctx.inputs.tick().get();

    ctx.stack.predicate(move |Delay { rate, modulus }: Delay| {
        let (rate, modulus) = cycle(Function::Delay, rate, modulus)?;

        // The product is the cycle length, not a Number: two bytes multiply to
        // at most 0xFE01, and `~* 10 20` is a cycle of 512 Ticks that a byte
        // multiply would fold to zero and then divide by.
        //
        // ADR 0012 writes this as `Tick % (rate * modulus) == 0`, and
        // `is_multiple_of` is that test rather than a different one: the two
        // differ only at a zero divisor, which `cycle` has already refused.
        Ok(tick.is_multiple_of(rate * modulus))
    })
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
/// over an absolute Tick would overflow a counter that ADR 0012 saturates
/// rather than wraps, and reducing first is exactly equivalent because every
/// term after it is taken modulo `steps` anyway.
#[inline(always)]
pub fn euclidean(ctx: &mut Context) -> Result<Value, Error> {
    let tick = ctx.inputs.tick().get();

    ctx.stack
        .predicate(move |Euclidean { hits, steps }: Euclidean| {
            // Zero steps is answered before hits are compared to steps, which
            // is what ADR 0012 requires and not an accident of the order the
            // two checks are written in: `~% 00 00` is a cycle with no
            // positions rather than a cycle with no onsets, so it diagnoses
            // instead of falling to the zero-hits rule below.
            if steps == 0 {
                return Err(InterpretationError::ZeroCycle {
                    function: Function::Euclidean,
                    operand: "step count",
                }
                .into());
            }

            if hits > steps {
                return Err(InterpretationError::EuclideanOverfull { hits, steps }.into());
            }

            let hits = u64::from(hits);
            let steps = u64::from(steps);
            let tick_in_cycle = tick % steps;
            let phase = (tick_in_cycle + steps - 1) % steps;

            // Zero hits falls out of this rather than being special-cased:
            // `0 + 0 >= steps` is false for every positive `steps`. So does a
            // full cycle, where `hits == steps` makes the remainder zero and
            // the comparison an equality that always holds.
            Ok((hits * phase) % steps + hits >= steps)
        })
}

#[cfg(test)]
mod test {
    use crate::{
        Anchor, Atom, Error, Function, Interpretation, Interpreter, Note, Sequence, SequenceError,
        Tick, TickInputs, TypeError, Value,
    };

    /// Evaluates one Tick-reading Function at absolute Tick `tick`, with its
    /// operands in signature order.
    ///
    /// The anchor is the Grid origin throughout: none of these three Functions
    /// reads a Position, and ADR 0013's Random is the one that will.
    fn evaluate(
        function: Function,
        tick: u64,
        left: impl Into<Value>,
        right: impl Into<Value>,
    ) -> Result<Interpretation, Error> {
        Interpreter::execute_function(
            function,
            &[left.into(), right.into()],
            TickInputs::new(Tick::new(tick), Anchor::new(0, 0)),
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
        // twice. A reference written here rather than reused from the body is
        // what makes this a claim about ADR 0012 instead of a restatement.
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
        // would get wrong.
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
        assert_eq!(
            error.to_string(),
            "Euclidean cannot fit 05 hits into 04 steps"
        );
    }

    #[test]
    fn a_note_operand_diagnoses_in_every_tick_function() {
        // All six operands are declared Number, so a Note is refused at each of
        // them rather than converted: ADR 0021 makes the Numeric Conversion
        // Functions the only crossing between the two numeric types, and a rate
        // that silently read a Note's byte would be a seventh crossing.
        let note = Atom::Note(Note::try_from(0x3C).unwrap());

        for function in [Function::Clock, Function::Delay, Function::Euclidean] {
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
        // Clock answers a value, so it extends element-wise like any other
        // Atomic Function: a scalar operand repeats and equal lengths pair.
        assert_eq!(
            evaluate(Function::Clock, 7, Atom::Number(0x02), numbers([2, 4, 8])).unwrap(),
            Interpretation::Sequence(numbers([1, 3, 3])),
        );

        assert_eq!(
            evaluate(Function::Clock, 7, numbers([1, 2, 4]), numbers([4, 4, 4])).unwrap(),
            Interpretation::Sequence(numbers([3, 3, 1])),
        );
    }

    #[test]
    fn a_pulse_broadcasts_to_find_its_elements_and_still_answers_one_atom() {
        // ADR 0012 has each broadcast element apply the formula independently
        // at the same Tick, and ADR 0011's rule for a whole-value answer is
        // what shapes the result: an element that does not Bang has nothing to
        // put at its position, so the answer is one Bang or the Absence Marker
        // about all of them rather than a Sequence with a hole in it.
        //
        // Tick 12 is a multiple of 4 and 6 and not of 8, so the three cases
        // below separate "every element" from "some element".
        assert_eq!(
            evaluate(Function::Delay, 12, Atom::Number(0x02), numbers([2, 3])).unwrap(),
            Interpretation::Cell(Atom::Bang),
        );

        assert_eq!(
            evaluate(Function::Delay, 12, Atom::Number(0x02), numbers([2, 3, 4])).unwrap(),
            Interpretation::Cell(Atom::Empty),
        );

        // Euclidean the same way, at Tick 14: it is the third step of a cycle
        // of four, where `03 04` Bangs and `01 04` does not, so one element is
        // enough to settle the answer.
        assert_eq!(
            evaluate(Function::Euclidean, 14, numbers([3, 4]), Atom::Number(0x04)).unwrap(),
            Interpretation::Cell(Atom::Bang),
        );

        assert_eq!(
            evaluate(Function::Euclidean, 14, numbers([3, 1]), Atom::Number(0x04)).unwrap(),
            Interpretation::Cell(Atom::Empty),
        );

        // An operation of no elements is vacuously true, exactly as Equality's
        // is: the empty Sequence is a width of no elements, not a scalar.
        assert_eq!(
            evaluate(Function::Delay, 5, Atom::Number(0x02), Sequence::empty()).unwrap(),
            Interpretation::Cell(Atom::Bang),
        );
    }

    #[test]
    fn an_element_fault_diagnoses_the_complete_operation() {
        // The all-or-nothing rule reaching a fallible predicate. A zero step
        // count at the last element refuses the whole answer rather than
        // letting the elements that did answer settle a Bang, which is the
        // failure a body that ignored its own diagnostic would produce.
        let error = evaluate(
            Function::Euclidean,
            0,
            Atom::Number(0x02),
            numbers([4, 8, 0]),
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "~% cannot count a cycle with a zero step count"
        );

        let error = evaluate(Function::Delay, 0, numbers([1, 0]), Atom::Number(0x04)).unwrap_err();
        assert_eq!(
            error.to_string(),
            "~* cannot count a cycle with a zero rate"
        );
    }

    #[test]
    fn two_non_scalar_operands_of_different_lengths_diagnose() {
        // Ordinary ADR 0007 shape rules, including an empty Sequence against a
        // non-empty one: a shape fault is settled before any element is read,
        // so it precedes every diagnostic the formulas above could raise.
        for function in [Function::Clock, Function::Delay, Function::Euclidean] {
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
                        evaluate(function, 5, left.clone(), right.clone()),
                        Err(Error::Sequence(SequenceError::IncompatibleLengths { left: l, right: r }))
                            if (l, r) == lengths
                    ),
                    "{function:?}({left:?}, {right:?})"
                );
            }
        }
    }
}
