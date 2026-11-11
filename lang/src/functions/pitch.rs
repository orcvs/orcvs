use arrayvec::ArrayVec;

use crate::{
    Atom, Error, Function, InterpretationError, IntervalSet, Note,
    atom::operands::{Down, Pick, Stack, Up},
    functions::chord::MAX_TONES,
    interpreter::Context,
};

/// The semitones one tone moves when it moves an octave.
pub(crate) const OCTAVE: i16 = 12;

/// The Note at `value` semitones, or the diagnostic naming `function`.
///
/// `value` is reached in a type wider than a byte so that an answer past
/// either end of the range is refused and never folded back into it.
#[inline(always)]
fn answer(function: Function, value: i16) -> Result<Atom, Error> {
    note_at(value)
        .map(Atom::Note)
        .ok_or_else(|| InterpretationError::NoteOutOfRange { function, value }.into())
}

/// The Note at `value` semitones, or `None` outside the Note range.
///
/// Every Function that reaches a pitch in a type wider than a byte reads it
/// back through here, so each refuses exactly the values the others refuse
/// and names the refusal in its own diagnostic.
#[inline(always)]
pub(crate) fn note_at(value: i16) -> Option<Note> {
    u8::try_from(value)
        .ok()
        .and_then(|value| Note::try_from(value).ok())
}

/// Up: `#+ note number`.
///
/// Answers the Note `number` semitones above `note`.
#[inline(always)]
pub fn up(ctx: &mut Context) -> Result<Atom, Error> {
    ctx.stack.apply(|Up { note, number }: Up| {
        answer(Function::Up, i16::from(note.value()) + i16::from(number))
    })
}

/// Down: `#- note number`.
///
/// Answers the Note `number` semitones below `note`.
#[inline(always)]
pub fn down(ctx: &mut Context) -> Result<Atom, Error> {
    ctx.stack.apply(|Down { note, number }: Down| {
        answer(Function::Down, i16::from(note.value()) - i16::from(number))
    })
}

/// The semitones above a root of the tone at `position` in `set`: the entry
/// it names, raised an octave for each whole trip through the set before it.
///
/// `i16` holds every value Pick and Stack reach through here, for any entry
/// a byte can hold. Pick's index is a byte, and Stack reads positions up to
/// `FF + 2 * (MAX_TONES - 1)` from its degree, 277 while the longest set
/// holds twelve tones, so no position is past that. A set holds at least one
/// tone, so a tone is at most `FF + 12 * 277`, 3579 semitones above its root.
/// Pick adds a root no higher than `7F`, reaching 3706, and Stack subtracts
/// one tone from another, so neither leaves `i16`. The assertion after this
/// function keeps the bound for whatever length the longest set has.
#[inline(always)]
fn tone(set: IntervalSet, position: usize) -> i16 {
    let semitones = set.semitones();
    let octaves = (position / semitones.len()) as i16;
    i16::from(semitones[position % semitones.len()]) + OCTAVE * octaves
}

// `tone`'s bound: the highest root, the largest byte entry and an octave for
// every position up to Stack's furthest fit in an `i16`.
const _: () = assert!(
    0x7F + 0xFF + (OCTAVE as usize) * (0xFF + 2 * (MAX_TONES - 1)) <= i16::MAX as usize,
    "a Pick or Stack tone above the highest root fits in an i16"
);

/// Pick: `#p root set index`.
///
/// Answers the Note at `index` in `set` above `root`. An index past the end
/// of `set` wraps up an octave.
#[inline(always)]
pub fn pick(ctx: &mut Context) -> Result<Atom, Error> {
    ctx.stack.apply(|Pick { root, set, index }: Pick| {
        answer(
            Function::Pick,
            i16::from(root.value()) + tone(set, usize::from(index)),
        )
    })
}

/// Stack: `#s set degree size`.
///
/// Answers the Interval Set with the semitones of `size` tones of `set` from
/// `degree`, each two positions on from the last, less the first; or diagnoses
/// when no name holds exactly those. The build refuses a table in which two
/// names hold the same semitones, so the answer is never a choice.
#[inline(always)]
pub fn stack(ctx: &mut Context) -> Result<Atom, Error> {
    ctx.stack.apply(|Stack { set, degree, size }: Stack| {
        stacked(set, usize::from(degree), usize::from(size))
            .and_then(|stacked| {
                IntervalSet::ALL.iter().copied().find(|name| {
                    name.semitones()
                        .iter()
                        .map(|&semitone| i16::from(semitone))
                        .eq(stacked.iter().copied())
                })
            })
            .map(Atom::IntervalSet)
            .ok_or_else(|| {
                InterpretationError::NoIntervalSet {
                    function: Function::Stack,
                    set,
                    degree,
                    size,
                }
                .into()
            })
    })
}

/// Stack's tones as semitones above the first, read once; or `None` when
/// `size` is past the longest name, so that no name could hold them.
#[inline(always)]
fn stacked(set: IntervalSet, degree: usize, size: usize) -> Option<ArrayVec<i16, MAX_TONES>> {
    (size <= MAX_TONES).then(|| {
        let first = tone(set, degree);
        (0..size)
            .map(|step| tone(set, degree + 2 * step) - first)
            .collect()
    })
}

#[cfg(test)]
mod test {
    use super::{OCTAVE, down, pick, stack, up};
    use crate::{
        Anchor, ArgumentError, Atom, Error, Function, Interpretation, InterpretationError,
        Interpreter, IntervalSet, Note, Tick, TickInputs, Token, TypeError, interpreter::Context,
    };

    /// The Tick inputs for a test about operands rather than about time.
    fn inputs() -> TickInputs {
        TickInputs::new(Tick::ZERO, Anchor::new(0, 0))
    }

    /// A Pitch body, called directly so a claim about its own arity demand is
    /// not answered by `execute_function`'s guard in front of it.
    type Body = fn(&mut Context) -> Result<Atom, Error>;

    const BODIES: [(Function, Body); 2] = [(Function::Up, up), (Function::Down, down)];

    /// Exercises Function dispatch with resolved operands in signature order.
    fn evaluate(function: Function, note: Atom, number: Atom) -> Result<Interpretation, Error> {
        Interpreter::execute_function(function, [note, number], inputs().into())
    }

    fn note(value: u8) -> Atom {
        Atom::Note(Note::try_from(value).unwrap())
    }

    /// Evaluates `outer` with the Return of the flat Source text `inner` in
    /// its note operand: `lang` evaluates one Function per Turn, so the inner
    /// Turn's two-Cell answer reaches the outer one decoded by its signature.
    fn nested(outer: Function, inner: &str, number: u8) -> Result<Interpretation, Error> {
        let [token, _] = outer.signature() else {
            panic!("{outer:?} declares two operands");
        };
        evaluate(outer, token.decode(&returned(inner))?, Atom::Number(number))
    }

    /// Asserts the Source text answers Note `expected`.
    fn answers(source: &str, expected: u8) {
        assert_eq!(
            crate::interpret_source(source).unwrap(),
            Interpretation::Cell(note(expected)),
            "{source}"
        );
    }

    /// Whether `answer` is the diagnostic for `function` reaching semitone
    /// `expected`.
    fn out_of_range(
        answer: &Result<Interpretation, Error>,
        function: Function,
        expected: i16,
    ) -> bool {
        matches!(
            answer,
            Err(Error::Interpretation(InterpretationError::NoteOutOfRange {
                function: found,
                value,
            })) if *found == function && *value == expected
        )
    }

    /// Asserts the Source text diagnoses `function` reaching `expected`.
    fn diagnoses(source: &str, function: Function, expected: i16) {
        assert!(
            out_of_range(&crate::interpret_source(source), function, expected),
            "{source}"
        );
    }

    #[test]
    fn up_answers_the_note_a_number_of_semitones_above() {
        answers("#+C402", 0x3E); // D4
        answers("#+C40C", 0x48); // C5, an octave
        answers("#+C400", 0x3C); // raised by no semitones
        answers("#+C/7F", 0x7F); // the whole range from the lowest Note
    }

    #[test]
    fn down_answers_the_note_a_number_of_semitones_below() {
        answers("#-D402", 0x3C); // C4
        answers("#-C40C", 0x30); // C3, an octave
        answers("#-C400", 0x3C); // lowered by no semitones
        answers("#-G97F", 0x00); // the whole range from the highest Note
    }

    #[test]
    fn up_diagnoses_an_answer_above_the_note_range_rather_than_wrapping() {
        // `G9` is 7F, the highest Note. One semitone more names no Note, and
        // `#+C4FF` is the case byte arithmetic would wrap to the legal Note 3B.
        answers("#+G900", 0x7F);
        diagnoses("#+G901", Function::Up, 128);
        diagnoses("#+C4FF", Function::Up, 315);
    }

    #[test]
    fn down_diagnoses_an_answer_below_the_note_range_rather_than_wrapping() {
        // `C/` is 00, the lowest Note, so nothing lies below it. `#-C4FF`
        // is the case byte arithmetic would wrap to the legal Note 3D.
        answers("#-C/00", 0x00);
        diagnoses("#-C/01", Function::Down, -1);
        diagnoses("#-C4FF", Function::Down, -195);
    }

    #[test]
    fn pick_answers_the_tone_at_an_index_inside_the_interval_set() {
        answers("#pC4MA00", 0x3C); // C4, the root
        answers("#pC4MA04", 0x43); // G4, MA's fifth entry is 7
        answers("#pC4MA06", 0x47); // B4, MA's last entry is 11
        answers("#pC4M_02", 0x43); // G4, a chord as well as a scale
    }

    #[test]
    fn pick_wraps_an_index_past_the_end_of_the_set_up_an_octave() {
        answers("#pC4MA07", 0x48); // C5: index 7 is entry 0, one octave up
        answers("#pC4MA08", 0x4A); // D5: entry 1 is 2, one octave up
        answers("#pC4M_03", 0x48); // C5
        answers("#pC4M_07", 0x58); // E6: 7 % 3 is entry 1, two octaves up
    }

    #[test]
    fn pick_steps_back_down_at_the_wrap_of_a_set_wider_than_an_octave() {
        // MD's last entry is 21, so its sixth tone is above the octave the
        // wrap adds: index 06 is entry 0 one octave up, below index 05.
        answers("#pC4MD05", 0x51); // A5
        answers("#pC4MD06", 0x48); // C5
    }

    #[test]
    fn pick_diagnoses_an_answer_above_the_note_range_rather_than_wrapping() {
        // `C/` is 00 and `ch` has twelve tones a semitone apart, so index 7F
        // is the Note 7F itself and index 80 is the first past the range.
        answers("#pC/ch7F", 0x7F);
        diagnoses("#pC/ch80", Function::Pick, 128);
        // `G9` is 7F, so any tone above the root leaves the range.
        answers("#pG9M_00", 0x7F);
        diagnoses("#pG9M_01", Function::Pick, 131);
        // The largest reach: index FF in a three-tone set is 85 octaves up.
        diagnoses("#pG9M_FF", Function::Pick, 1147);
    }

    /// Asserts the Source text answers the Interval Set spelled `expected`.
    fn stacks(source: &str, expected: &str) {
        assert_eq!(
            crate::interpret_source(source).unwrap(),
            Interpretation::Cell(Token::IntervalSet.decode(expected).unwrap()),
            "{source}"
        );
    }

    #[test]
    fn stack_answers_the_chord_built_on_a_scale_degree() {
        // Positions 0 2 4 of MA are 0 4 7.
        stacks("#sMA0003", "M_");
        // Positions 1 3 5 are 2 5 9, which less the first are 0 3 7.
        stacks("#sMA0103", "m_");
        // Positions 0 2 4 6 and on through 8, wrapping up an octave.
        stacks("#sMA0004", "M7");
        stacks("#sMA0005", "M9");
    }

    #[test]
    fn stack_answers_every_seventh_of_the_major_scale() {
        stacks("#sMA0204", "m7"); // E G B D: 4 7 11 14
        stacks("#sMA0304", "M7"); // F A C E: 5 9 12 16
        stacks("#sMA0404", "7_"); // G B D F: 7 11 14 17
        stacks("#sMA0504", "m7"); // A C E G: 9 12 16 19
        stacks("#sMA0604", "h7"); // B D F A: 11 14 17 21
        stacks("#sMA0603", "o_"); // B D F: 11 14 17
    }

    #[test]
    fn stack_wraps_up_octaves_as_pick_does_past_the_end_of_the_set() {
        stacks("#sMA0703", "M_"); // Position 7 is the root an octave up.
        // Positions FF, 101 and 103 are 437, 441 and 444: a position runs past
        // a byte, where a byte's wrap would read positions FF, 01 and 03.
        stacks("#sMAFF03", "M_");
    }

    #[test]
    fn stack_answers_a_scale_name_when_the_tones_spell_one() {
        stacks("#sch0006", "wt"); // Every second chromatic tone.
        stacks("#sch0106", "wt"); // The same steps from one semitone up.
        stacks("#swt0003", "+_"); // A chord name from a non-diatonic scale.
    }

    /// Asserts the Source text diagnoses that no name fits Stack's operands.
    fn no_name(source: &str, expected_degree: u8, expected_size: u8) {
        assert!(
            matches!(
                crate::interpret_source(source),
                Err(Error::Interpretation(InterpretationError::NoIntervalSet {
                    degree,
                    size,
                    ..
                })) if degree == expected_degree && size == expected_size
            ),
            "{source}"
        );
    }

    #[test]
    fn stack_diagnoses_a_major_pentatonic_stacked_from_c() {
        // Positions 0 2 4 of p5 are 0 4 9, which is no name: a Major Triad is
        // 0 4 7 and a Sixth 0 4 7 9.
        no_name("#sp50003", 0, 3);
    }

    #[test]
    fn stack_diagnoses_when_no_name_fits_rather_than_answering_a_near_one() {
        // A Major Eleventh is 0 4 7 11 14 17, and no name holds it.
        no_name("#sMA0006", 0, 6);
        // Positions 0 2 4 of a chord read 0 7 16, which no name holds.
        no_name("#sM_0003", 0, 3);
        // Positions 5, 7 and 9 of MD are 21, 16 and 23, and no name descends.
        no_name("#sMD0503", 5, 3);
    }

    #[test]
    fn stack_diagnoses_a_size_outside_every_name() {
        // No name is empty or a single tone, and none is longer than twelve.
        no_name("#sMA0000", 0, 0);
        no_name("#sMA0001", 0, 1);
        no_name("#sch000D", 0, 13);
        no_name("#schFFFF", 0xFF, 0xFF);
    }

    #[test]
    fn up_and_down_move_every_note_by_every_number_and_never_wrap() {
        // The whole operand square, against an expectation stated by the
        // range's bounds: an answer inside `00`–`7F` is that Note, and any
        // other answer diagnoses the Function and the semitone it reached.
        for (function, direction) in [(Function::Up, 1), (Function::Down, -1)] {
            for start in 0x00..=0x7F {
                for number in 0x00..=u8::MAX {
                    let semitone = i16::from(start) + direction * i16::from(number);
                    let answer = evaluate(function, note(start), Atom::Number(number));
                    match semitone {
                        0x00..=0x7F => assert_eq!(
                            answer.unwrap(),
                            Interpretation::Cell(note(semitone as u8)),
                            "{function:?} {start:02X} {number:02X}"
                        ),
                        _ => assert!(
                            out_of_range(&answer, function, semitone),
                            "{function:?} {start:02X} {number:02X}: {answer:?}"
                        ),
                    }
                }
            }
        }
    }

    #[test]
    fn every_up_is_undone_by_the_same_down() {
        // Over the whole Note and Number domains, Down inverts every Up that
        // answers, and an Up that refuses refuses as out of range.
        for start in 0x00..=0x7F {
            for number in 0x00..=u8::MAX {
                let raised = evaluate(Function::Up, note(start), Atom::Number(number));
                let Ok(Interpretation::Cell(raised)) = raised else {
                    let semitone = i16::from(start) + i16::from(number);
                    assert!(
                        out_of_range(&raised, Function::Up, semitone),
                        "{start:02X} {number:02X}: {raised:?}"
                    );
                    continue;
                };
                assert_eq!(
                    evaluate(Function::Down, raised, Atom::Number(number)).unwrap(),
                    Interpretation::Cell(note(start)),
                    "{start:02X} {number:02X}"
                );
            }
        }
    }

    #[test]
    fn a_number_in_the_note_operand_is_refused() {
        // The note operand decodes by the signature, so a value already a
        // Number is read as a Note spelling and refused rather than coerced.
        for (function, _) in BODIES {
            assert!(
                matches!(
                    evaluate(function, Atom::Number(0x3C), Atom::Number(2)),
                    Err(Error::Type(TypeError::Note(found))) if found == "3C"
                ),
                "{function:?}"
            );
        }
        assert!(matches!(
            crate::interpret_source("#+3C02"),
            Err(Error::Type(TypeError::Note(found))) if found == "3C"
        ));
    }

    #[test]
    fn a_note_in_the_number_operand_is_refused() {
        for (function, _) in BODIES {
            assert!(
                matches!(
                    evaluate(function, note(0x3C), note(0x3C)),
                    Err(Error::Type(TypeError::Number(found))) if found == "C4"
                ),
                "{function:?}"
            );
        }
    }

    #[test]
    fn a_non_value_operand_in_either_slot_diagnoses_as_its_declared_type() {
        for (function, _) in BODIES {
            for value in [Atom::Bang, Atom::Function(Function::Add)] {
                let spelling = value.to_string();
                assert!(
                    matches!(
                        evaluate(function, value, Atom::Number(2)),
                        Err(Error::Type(TypeError::Note(found))) if found == spelling
                    ),
                    "{function:?} note {spelling}"
                );
                assert!(
                    matches!(
                        evaluate(function, note(0x3C), value),
                        Err(Error::Type(TypeError::Number(found))) if found == spelling
                    ),
                    "{function:?} number {spelling}"
                );
            }
        }
    }

    #[test]
    fn the_number_operand_reads_a_note_spelling_as_the_number_it_spells() {
        // A Cell pair is typed by the slot it stands in, so `C4` in the
        // number slot is Number C4, and either direction passes the range.
        diagnoses("#+C4C4", Function::Up, 256);
        diagnoses("#-C4C4", Function::Down, -136);
    }

    #[test]
    fn a_missing_operand_diagnoses_as_arity() {
        for (function, body) in BODIES {
            for present in [&[][..], &[note(0x3C)][..]] {
                let mut ctx = Context::new(inputs().into(), 4);
                for &operand in present {
                    ctx.stack.push(operand).unwrap();
                }

                assert!(
                    matches!(
                        body(&mut ctx),
                        Err(Error::Argument(ArgumentError::Arity { expected: 2, found }))
                            if found == present.len()
                    ),
                    "{function:?} over {present:?}"
                );
            }
        }
    }

    #[test]
    fn a_pitch_function_reads_only_its_own_operands_from_a_deeper_stack() {
        for (function, body) in BODIES {
            let mut ctx = Context::new(inputs().into(), 4);
            ctx.stack.push(Atom::Bang).unwrap();
            ctx.stack.push(Atom::Number(2)).unwrap();
            ctx.stack.push(note(0x3C)).unwrap();

            assert!(body(&mut ctx).is_ok(), "{function:?}");
            assert_eq!(ctx.stack.pop_value(), Some(Atom::Bang));
        }
    }

    fn scale() -> Atom {
        Atom::IntervalSet(IntervalSet::MajorScale)
    }

    /// Pick and Stack, which each read three operands, with arguments the
    /// tests can name.
    fn evaluate_pick(root: Atom, set: Atom, index: Atom) -> Result<Interpretation, Error> {
        Interpreter::execute_function(Function::Pick, [root, set, index], inputs().into())
    }

    fn evaluate_stack(set: Atom, degree: Atom, size: Atom) -> Result<Interpretation, Error> {
        Interpreter::execute_function(Function::Stack, [set, degree, size], inputs().into())
    }

    #[test]
    fn pick_refuses_an_operand_of_the_wrong_type_in_each_slot() {
        assert!(matches!(
            evaluate_pick(Atom::Number(0x3C), scale(), Atom::Number(0)),
            Err(Error::Type(TypeError::Note(found))) if found == "3C"
        ));
        assert!(matches!(
            evaluate_pick(note(0x3C), note(0x3C), Atom::Number(0)),
            Err(Error::Type(TypeError::IntervalSet(found))) if found == "C4"
        ));
        assert!(matches!(
            evaluate_pick(note(0x3C), scale(), note(0x3C)),
            Err(Error::Type(TypeError::Number(found))) if found == "C4"
        ));
        assert!(matches!(
            evaluate_pick(note(0x3C), scale(), scale()),
            Err(Error::Type(TypeError::Number(found))) if found == "MA"
        ));
        assert!(matches!(
            evaluate_pick(note(0x3C), Atom::Bang, Atom::Number(0)),
            Err(Error::Type(TypeError::IntervalSet(found))) if found == "**"
        ));
    }

    #[test]
    fn stack_refuses_an_operand_of_the_wrong_type_in_each_slot() {
        assert!(matches!(
            evaluate_stack(Atom::Number(0x3C), Atom::Number(0), Atom::Number(3)),
            Err(Error::Type(TypeError::IntervalSet(found))) if found == "3C"
        ));
        assert!(matches!(
            evaluate_stack(scale(), note(0x3C), Atom::Number(3)),
            Err(Error::Type(TypeError::Number(found))) if found == "C4"
        ));
        assert!(matches!(
            evaluate_stack(scale(), Atom::Number(0), note(0x3C)),
            Err(Error::Type(TypeError::Number(found))) if found == "C4"
        ));
        assert!(matches!(
            evaluate_stack(scale(), Atom::Bang, Atom::Number(3)),
            Err(Error::Type(TypeError::Number(found))) if found == "**"
        ));
    }

    #[test]
    fn an_interval_set_operand_reads_a_literal_only_when_it_spells_a_name() {
        // `69` and `13` are also Numbers, and the set slot reads them as names.
        answers("#pC46900", 0x3C);
        answers("#pC41301", 0x40);
        assert!(matches!(
            crate::interpret_source("#pC43C00"),
            Err(Error::Type(TypeError::IntervalSet(found))) if found == "3C"
        ));
        assert!(matches!(
            crate::interpret_source("#sC40003"),
            Err(Error::Type(TypeError::IntervalSet(found))) if found == "C4"
        ));
    }

    /// Pick and Stack, called directly so a claim about their own arity demand
    /// is not answered by `execute_function`'s guard in front of them, each
    /// beside operands it answers, in the order they push: last first, so the
    /// first operand is on top.
    fn three_operand_bodies() -> [(Function, Body, [Atom; 3]); 2] {
        [
            (Function::Pick, pick, [Atom::Number(0), scale(), note(0x3C)]),
            (
                Function::Stack,
                stack,
                [Atom::Number(3), Atom::Number(0), scale()],
            ),
        ]
    }

    #[test]
    fn pick_and_stack_diagnose_a_missing_operand_as_arity() {
        for (function, body, operands) in three_operand_bodies() {
            let mut ctx = Context::new(inputs().into(), 4);
            for &operand in &operands[1..] {
                ctx.stack.push(operand).unwrap();
            }

            assert!(
                matches!(
                    body(&mut ctx),
                    Err(Error::Argument(ArgumentError::Arity {
                        expected: 3,
                        found: 2
                    }))
                ),
                "{function:?}"
            );
        }
    }

    #[test]
    fn pick_and_stack_read_only_their_own_operands_from_a_deeper_stack() {
        for (function, body, operands) in three_operand_bodies() {
            let mut ctx = Context::new(inputs().into(), 4);
            ctx.stack.push(Atom::Bang).unwrap();
            for operand in operands {
                ctx.stack.push(operand).unwrap();
            }

            assert!(body(&mut ctx).is_ok(), "{function:?}");
            assert_eq!(ctx.stack.pop_value(), Some(Atom::Bang));
        }
    }

    #[test]
    fn one_more_trip_through_a_set_picks_an_octave_higher() {
        // Over every set and every index whose answer and whose next trip's
        // answer are both Notes.
        for set in IntervalSet::ALL {
            let length = u8::try_from(set.semitones().len()).unwrap();
            let picked = |index| match evaluate_pick(
                note(0),
                Atom::IntervalSet(*set),
                Atom::Number(index),
            ) {
                Ok(Interpretation::Cell(Atom::Note(answer))) => Some(answer.value()),
                _ => None,
            };
            for index in 0..=(u8::MAX - length) {
                if let (Some(lower), Some(upper)) = (picked(index), picked(index + length)) {
                    assert_eq!(
                        i16::from(upper),
                        i16::from(lower) + OCTAVE,
                        "{set:?} {index:02X}"
                    );
                }
            }
        }
    }

    /// What the flat Source text `inner` returns, as the two Cells a Turn
    /// hands the operand it stands in.
    fn returned(inner: &str) -> String {
        match crate::interpret_source(inner).expect("the inner Turn answers") {
            Interpretation::Cell(atom) => atom.to_string(),
            other => panic!("{inner:?} answers {other:?}, which no Turn returns"),
        }
    }

    #[test]
    fn pick_and_stack_nest_and_compose() {
        // Stack's answer reaches Pick's set slot decoded by Pick's signature:
        // `#sMA0003` answers M_, and index 02 of it is the fifth.
        assert_eq!(returned("#sMA0003"), "M_");
        let set = Function::Pick.signature()[1]
            .decode(&returned("#sMA0003"))
            .unwrap();
        assert_eq!(
            evaluate_pick(note(0x3C), set, Atom::Number(2)).unwrap(),
            Interpretation::Cell(note(0x43))
        );
        // Pick's Note answer serves another Note operand.
        assert_eq!(
            nested(Function::Up, "#pC4MA04", 2).unwrap(),
            Interpretation::Cell(note(0x45))
        );
        // `.^3C` answers C4 to Pick's root.
        let root = Function::Pick.signature()[0]
            .decode(&returned(".^3C"))
            .unwrap();
        assert_eq!(
            evaluate_pick(root, scale(), Atom::Number(0)).unwrap(),
            Interpretation::Cell(note(0x3C))
        );
    }

    #[test]
    fn a_nested_answer_of_the_wrong_type_is_refused() {
        // `.^3C` answers a Note, which names no Interval Set.
        assert!(matches!(
            Function::Stack.signature()[0].decode(&returned(".^3C")),
            Err(Error::Type(TypeError::IntervalSet(found))) if found == "C4"
        ));
        // `#sMA0003` answers a name, which spells no Note.
        assert!(matches!(
            Function::Pick.signature()[0].decode(&returned("#sMA0003")),
            Err(Error::Type(TypeError::Note(found))) if found == "M_"
        ));
    }

    #[test]
    fn up_and_down_nest_and_compose_with_the_other_note_answers() {
        // `#+C402` answers D4 to the outer Down, which takes two away again.
        assert_eq!(
            nested(Function::Down, "#+C402", 2).unwrap(),
            Interpretation::Cell(note(0x3C))
        );
        // `.^3C` answers C4 to the Note operand.
        assert_eq!(
            nested(Function::Up, ".^3C", 2).unwrap(),
            Interpretation::Cell(note(0x3E))
        );
    }

    #[test]
    fn a_nested_number_answer_in_the_note_operand_is_refused() {
        // `.vC4` returns `3C`, which spells no Note.
        assert!(matches!(
            nested(Function::Up, ".vC4", 2),
            Err(Error::Type(TypeError::Note(found))) if found == "3C"
        ));
    }
}
