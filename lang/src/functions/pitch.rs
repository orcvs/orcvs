use crate::{
    Atom, Error, Function, InterpretationError, Note,
    atom::operands::{Down, Up},
    interpreter::Context,
};

/// The Note at `value` semitones, or the diagnostic naming `function`.
///
/// `value` is reached in a type wider than a byte so that an answer past
/// either end of the range is refused and never folded back into it.
#[inline(always)]
fn answer(function: Function, value: i16) -> Result<Atom, Error> {
    u8::try_from(value)
        .ok()
        .and_then(|value| Note::try_from(value).ok())
        .map(Atom::Note)
        .ok_or_else(|| InterpretationError::NoteOutOfRange { function, value }.into())
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

#[cfg(test)]
mod test {
    use super::{down, up};
    use crate::{
        Anchor, ArgumentError, Atom, Error, Function, Interpretation, InterpretationError,
        Interpreter, Note, Tick, TickInputs, TypeError, interpreter::Context,
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
        let returned = match crate::interpret_source(inner).expect("the inner Turn answers") {
            Interpretation::Cell(atom) => atom.to_string(),
            other => panic!("{inner:?} answers {other:?}, which no Turn returns"),
        };
        let [token, _] = outer.signature() else {
            panic!("{outer:?} declares two operands");
        };
        evaluate(outer, token.decode(&returned)?, Atom::Number(number))
    }

    /// Asserts the Source text answers Note `expected`.
    fn answers(source: &str, expected: u8) {
        assert_eq!(
            crate::interpret_source(source).unwrap(),
            Interpretation::Cell(note(expected)),
            "{source}"
        );
    }

    /// Asserts the Source text diagnoses `function` reaching `expected`.
    fn out_of_range(source: &str, function: Function, expected: i16) {
        assert!(
            matches!(
                crate::interpret_source(source),
                Err(Error::Interpretation(InterpretationError::NoteOutOfRange {
                    function: found,
                    value,
                })) if found == function && value == expected
            ),
            "{source}"
        );
    }

    #[test]
    fn up_answers_the_note_a_number_of_semitones_above() {
        answers("#+C402", 0x3E); // D4
        answers("#+C40C", 0x48); // C5, an octave
        answers("#+C400", 0x3C); // no transposition
        answers("#+C/7F", 0x7F); // the whole range from the lowest Note
    }

    #[test]
    fn down_answers_the_note_a_number_of_semitones_below() {
        answers("#-D402", 0x3C); // C4
        answers("#-C40C", 0x30); // C3, an octave
        answers("#-C400", 0x3C); // no transposition
        answers("#-G97F", 0x00); // the whole range from the highest Note
    }

    #[test]
    fn up_diagnoses_an_answer_above_the_note_range_rather_than_wrapping() {
        // `G9` is 7F, the highest Note. One semitone more names no Note, and
        // `#+C4FF` is the case byte arithmetic would wrap to the legal Note 3B.
        answers("#+G900", 0x7F);
        out_of_range("#+G901", Function::Up, 128);
        out_of_range("#+C4FF", Function::Up, 315);
    }

    #[test]
    fn down_diagnoses_an_answer_below_the_note_range_rather_than_wrapping() {
        // `C/` is 00, the lowest Note, so nothing lies below it. `#-C4FF`
        // is the case byte arithmetic would wrap to the legal Note 3D.
        answers("#-C/00", 0x00);
        out_of_range("#-C/01", Function::Down, -1);
        out_of_range("#-C4FF", Function::Down, -195);
    }

    #[test]
    fn every_transposition_up_is_undone_by_the_same_transposition_down() {
        // Over the whole Note and Number domains, Down inverts every Up that
        // answers, and an Up that refuses refuses as out of range.
        for start in 0x00..=0x7F {
            for number in 0x00..=u8::MAX {
                let raised = evaluate(Function::Up, note(start), Atom::Number(number));
                let Ok(Interpretation::Cell(raised)) = raised else {
                    assert!(
                        matches!(
                            raised,
                            Err(Error::Interpretation(
                                InterpretationError::NoteOutOfRange { .. }
                            ))
                        ),
                        "{start:02X} {number:02X}"
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
        out_of_range("#+C4C4", Function::Up, 256);
        out_of_range("#-C4C4", Function::Down, -136);
    }

    #[test]
    fn a_missing_operand_diagnoses_as_arity() {
        for (function, body) in BODIES {
            let mut ctx = Context::new(inputs().into(), 4);
            ctx.stack.push(note(0x3C)).unwrap();

            assert!(
                matches!(
                    body(&mut ctx),
                    Err(Error::Argument(ArgumentError::Arity {
                        expected: 2,
                        found: 1
                    }))
                ),
                "{function:?}"
            );
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
