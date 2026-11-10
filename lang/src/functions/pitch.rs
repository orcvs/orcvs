use crate::{
    Atom, Error, Function, InterpretationError, Note,
    atom::operands::{Down, Up},
    interpreter::Context,
};

/// Up: `#+ note number`.
///
/// Answers the Note `number` semitones above `note`. A sum above `7F` names no
/// MIDI Note, so it diagnoses rather than wrapping into a lower octave.
#[inline(always)]
pub fn up(ctx: &mut Context) -> Result<Atom, Error> {
    ctx.stack.apply(|Up { note, number }: Up| {
        transposed(Function::Up, i16::from(note.value()) + i16::from(number))
    })
}

/// Down: `#- note number`.
///
/// Answers the Note `number` semitones below `note`. A difference below `00`
/// names no MIDI Note, so it diagnoses rather than wrapping into a higher
/// octave.
#[inline(always)]
pub fn down(ctx: &mut Context) -> Result<Atom, Error> {
    ctx.stack.apply(|Down { note, number }: Down| {
        transposed(Function::Down, i16::from(note.value()) - i16::from(number))
    })
}

/// The Note at `semitone`, or the diagnostic `function` raises when no Note
/// is there. The arithmetic is widened to `i16` so neither direction can
/// overflow before the range is checked.
fn transposed(function: Function, semitone: i16) -> Result<Atom, Error> {
    u8::try_from(semitone)
        .ok()
        .and_then(|value| Note::try_from(value).ok())
        .map(Atom::Note)
        .ok_or_else(|| {
            InterpretationError::NoteOutOfRange {
                function,
                value: semitone,
            }
            .into()
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

    /// Evaluates Source text as a complete parsed Expression, so a Pitch
    /// Function is reached through its literal slots or through nesting.
    fn interpret(source: &str) -> Result<Interpretation, Error> {
        crate::interpret_source(source)
    }

    /// Exercises Function dispatch with resolved operands in signature order.
    fn evaluate(function: Function, note: Atom, number: Atom) -> Result<Interpretation, Error> {
        Interpreter::execute_function(function, [note, number], inputs().into())
    }

    /// A Pitch body, called directly so a claim about its own arity demand is
    /// not answered by `execute_function`'s guard in front of it.
    type Body = fn(&mut Context) -> Result<Atom, Error>;

    const BODIES: [(Function, Body); 2] = [(Function::Up, up), (Function::Down, down)];

    /// What a transposition should answer for one operand pair, in signature
    /// order: the Note `semitones` away, or no Note at all.
    type Reference = fn(u8, u8) -> Result<Atom, InterpretationError>;

    fn note(value: u8) -> Atom {
        Atom::Note(Note::try_from(value).unwrap())
    }

    #[test]
    fn up_and_down_move_every_note_by_every_number_and_never_wrap() {
        // The whole operand square, against a reference that states the
        // range by its bounds rather than by the body's arithmetic: an answer
        // inside `00`–`7F` is that Note, and any other answer diagnoses with
        // the semitone it would have been, which `00 - 01` and `7F + 01`
        // both reach at the boundary.
        let cases: [(Function, Reference); 2] = [
            (Function::Up, |note, number| {
                let answer = i16::from(note) + i16::from(number);
                match answer {
                    0x00..=0x7F => Ok(self::note(answer as u8)),
                    _ => Err(InterpretationError::NoteOutOfRange {
                        function: Function::Up,
                        value: answer,
                    }),
                }
            }),
            (Function::Down, |note, number| {
                let answer = i16::from(note) - i16::from(number);
                match answer {
                    0x00..=0x7F => Ok(self::note(answer as u8)),
                    _ => Err(InterpretationError::NoteOutOfRange {
                        function: Function::Down,
                        value: answer,
                    }),
                }
            }),
        ];

        for (function, reference) in cases {
            for value in 0x00..=0x7F {
                for number in 0..=u8::MAX {
                    match (
                        evaluate(function, note(value), Atom::Number(number)),
                        reference(value, number),
                    ) {
                        (Ok(answer), Ok(expected)) => assert_eq!(
                            answer,
                            Interpretation::Cell(expected),
                            "{function} {value:02X} {number:02X}"
                        ),
                        // `InterpretationError` derives no `PartialEq`, and the
                        // wording is what the Source is shown.
                        (Err(answer), Err(expected)) => assert_eq!(
                            answer.to_string(),
                            expected.to_string(),
                            "{function} {value:02X} {number:02X}"
                        ),
                        (answer, expected) => panic!(
                            "{function} {value:02X} {number:02X}: {answer:?} is not {expected:?}"
                        ),
                    }
                }
            }
        }
    }

    #[test]
    fn an_answer_outside_the_note_range_names_the_function_and_the_semitone() {
        // Hexadecimal, as every other Number the Source is shown, with a sign
        // where Down passes below `00`.
        assert_eq!(
            interpret("#+G901").unwrap_err().to_string(),
            "#+ answers 80, outside the Note range 00–7F"
        );
        assert_eq!(
            interpret("#+G9FF").unwrap_err().to_string(),
            "#+ answers 17E, outside the Note range 00–7F"
        );
        assert_eq!(
            interpret("#-C/01").unwrap_err().to_string(),
            "#- answers -01, outside the Note range 00–7F"
        );
    }

    #[test]
    fn each_literal_slot_reads_the_type_its_signature_declares() {
        // `#+C40C` reads Note `C4` and Number `0C`. The same Cells in the
        // other slot read the other type: `C4` as the Number is 196
        // semitones, which no Note is.
        assert_eq!(
            interpret("#+C40C").unwrap(),
            Interpretation::Cell(note(0x48))
        );
        assert_eq!(
            interpret("#-C40C").unwrap(),
            Interpretation::Cell(note(0x30))
        );
        assert!(matches!(
            interpret("#+3C0C"),
            Err(Error::Type(TypeError::Note(found))) if found == "3C"
        ));
        assert!(matches!(
            interpret("#-C/C4"),
            Err(Error::Interpretation(InterpretationError::NoteOutOfRange {
                function: Function::Down,
                ..
            }))
        ));
    }

    #[test]
    fn a_number_note_and_a_note_number_are_refused() {
        // A Note is not a Number in disguise and a Number is not a Note: the
        // Pitch family converts nothing implicitly, as the numeric family
        // does not.
        for (function, _) in BODIES {
            assert!(
                matches!(
                    evaluate(function, Atom::Number(0x3C), Atom::Number(1)),
                    Err(Error::Type(TypeError::Note(found))) if found == "3C"
                ),
                "{function}"
            );
            assert!(
                matches!(
                    evaluate(function, note(0x3C), note(0x01)),
                    Err(Error::Type(TypeError::Number(found))) if found == "c/"
                ),
                "{function}"
            );
        }
    }

    #[test]
    fn a_non_value_operand_diagnoses_as_the_declared_literal_type() {
        for (function, _) in BODIES {
            for value in [Atom::Bang, Atom::Function(Function::Add)] {
                let spelling = value.to_string();
                assert!(
                    matches!(
                        evaluate(function, value, Atom::Number(1)),
                        Err(Error::Type(TypeError::Note(ref found))) if *found == spelling
                    ),
                    "{function} {spelling}"
                );
                assert!(
                    matches!(
                        evaluate(function, note(0x3C), value),
                        Err(Error::Type(TypeError::Number(ref found))) if *found == spelling
                    ),
                    "{function} {spelling}"
                );
            }
        }
    }

    #[test]
    fn a_missing_operand_diagnoses_as_arity_before_anything_is_read() {
        for (function, body) in BODIES {
            for present in [vec![], vec![Atom::Number(1)]] {
                let mut ctx = Context::new(inputs().into(), 2);
                for operand in &present {
                    ctx.stack.push(*operand).unwrap();
                }

                assert!(
                    matches!(
                        body(&mut ctx),
                        Err(Error::Argument(ArgumentError::Arity { expected: 2, found }))
                            if found == present.len()
                    ),
                    "{function} over {present:?}"
                );
            }
        }
    }

    #[test]
    fn a_transposition_reads_only_its_own_operands_from_a_deeper_stack() {
        for (function, body) in BODIES {
            let mut ctx = Context::new(inputs().into(), 4);
            ctx.stack.push(Atom::Bang).unwrap();
            ctx.stack.push(Atom::Number(0x02)).unwrap();
            ctx.stack.push(note(0x3C)).unwrap();

            assert!(body(&mut ctx).is_ok(), "{function}");
            assert_eq!(ctx.stack.pop_value(), Some(Atom::Bang), "{function}");
        }
    }

    /// The Return of the flat Source text `inner`, decoded by `outer`'s
    /// operand at `slot`: a Turn returns its answer's two-Cell encoding and
    /// the receiving signature decides what those Cells are.
    fn returned(outer: Function, slot: usize, inner: &str) -> Result<Atom, Error> {
        let spelled = match interpret(inner).expect("the inner Turn answers a value") {
            Interpretation::Cell(atom) => atom.to_string(),
            other => panic!("{inner:?} answers {other:?}, which no Turn returns"),
        };
        outer.signature()[slot].decode(&spelled)
    }

    #[test]
    fn a_nested_return_is_read_by_the_operand_that_receives_it() {
        // A Note answered by Up, Down or `.^` stands in a note operand, and a
        // Number answered by the numeric family stands in a number operand.
        let down = returned(Function::Up, 0, "#-C40C").unwrap();
        assert_eq!(
            evaluate(Function::Up, down, Atom::Number(0x07)).unwrap(),
            Interpretation::Cell(note(0x37))
        );
        let converted = returned(Function::Down, 0, ".^3C").unwrap();
        let sum = returned(Function::Down, 1, ".+0102").unwrap();
        assert_eq!(
            evaluate(Function::Down, converted, sum).unwrap(),
            Interpretation::Cell(note(0x39))
        );
        let up = returned(Function::ConvertToNumber, 0, "#+C40C").unwrap();
        assert_eq!(
            Interpreter::execute_function(Function::ConvertToNumber, [up], inputs().into())
                .unwrap(),
            Interpretation::Cell(Atom::Number(0x48))
        );
        // A Note Return in the number operand reads as the Number its Cells
        // spell, or is refused: `D/` spells none.
        assert!(matches!(
            returned(Function::Up, 1, "#+C/02"),
            Err(Error::Type(TypeError::Number(found))) if found == "D/"
        ));
    }
}
