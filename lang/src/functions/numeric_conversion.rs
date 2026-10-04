use crate::{
    Atom, Error, Note,
    atom::operands::{ConvertToNote, ConvertToNumber},
    interpreter::Context,
    operand::NumericValue,
};

/// Convert to Number: `.v value`.
///
/// Idempotence over the result type is what makes the Number arm not a
/// coercion: a value that is already a Number arrives from nested evaluation,
/// never from this Function's own literal operand slot, which the parser reads
/// as a Note.
#[inline]
pub fn to_number(ctx: &mut Context) -> Result<Atom, Error> {
    ctx.stack
        .apply(|ConvertToNumber { value }: ConvertToNumber| {
            Ok(Atom::Number(match value {
                NumericValue::Note(value) => value.value(),
                NumericValue::Number(value) => value,
            }))
        })
}

/// Convert to Note: `.^ value`.
///
/// `80` through `FF` name no MIDI Note, so they diagnose rather than being
/// folded into the range.
#[inline]
pub fn to_note(ctx: &mut Context) -> Result<Atom, Error> {
    ctx.stack
        .apply(|ConvertToNote { value }: ConvertToNote| match value {
            NumericValue::Note(value) => Ok(Atom::Note(value)),
            // `Note`'s own conversion is the one range check, and its refusal
            // is the diagnostic.
            NumericValue::Number(value) => Ok(Atom::Note(Note::try_from(value)?)),
        })
}

#[cfg(test)]
mod test {
    use super::{to_note, to_number};
    use crate::{
        Anchor, ArgumentError, Atom, Error, Function, Interpretation, InterpretationError,
        Interpreter, Note, Tick, TickInputs, TypeError, interpreter::Context,
    };

    /// The Tick inputs for a test about operands rather than about time.
    fn inputs() -> TickInputs {
        TickInputs::new(Tick::ZERO, Anchor::new(0, 0))
    }

    /// Evaluates Source text as a complete parsed Expression, so a conversion
    /// is reached through its literal slot or through nested evaluation.
    fn interpret(source: &str) -> Result<Interpretation, Error> {
        crate::interpret_source(source)
    }

    /// A conversion body, called directly so a claim about its own arity
    /// demand is not answered by `execute_function`'s guard in front of it.
    type Body = fn(&mut Context) -> Result<Atom, Error>;

    const BODIES: [(Function, Body); 2] = [
        (Function::ConvertToNote, to_note),
        (Function::ConvertToNumber, to_number),
    ];

    /// Exercises conversion dispatch with one resolved language value.
    fn evaluate(function: Function, value: Atom) -> Result<Interpretation, Error> {
        Interpreter::execute_function(function, [value], inputs().into())
    }

    /// Evaluates `outer` over the answer of the flat Source text `inner`, as
    /// the Turn nesting it does: `lang` evaluates one Function per Turn, and
    /// the inner Turn's answer is the outer Turn's operand.
    fn nested(outer: Function, inner: &str) -> Result<Interpretation, Error> {
        let operand = match interpret(inner).expect("the inner Turn answers a value") {
            Interpretation::Cell(atom) => atom,
            other => panic!("{inner:?} answers {other:?}, which no Turn passes on"),
        };
        evaluate(outer, operand)
    }

    fn note(value: u8) -> Atom {
        Atom::Note(Note::try_from(value).unwrap())
    }

    #[test]
    fn a_conversion_over_one_atom_answers_one_atom() {
        for value in 0x00..=0x7F {
            assert_eq!(
                evaluate(Function::ConvertToNumber, note(value)).unwrap(),
                Interpretation::Cell(Atom::Number(value))
            );
            assert_eq!(
                evaluate(Function::ConvertToNote, Atom::Number(value)).unwrap(),
                Interpretation::Cell(note(value))
            );
        }
    }

    #[test]
    fn conversion_to_note_is_the_identity_over_every_note_and_refuses_every_number_above_the_range()
    {
        // `.^` has a monomorphic literal signature and an
        // evaluation-time identity, and the two together are what the whole
        // domain is enumerated for. Every Note is left exactly as it is —
        // a value arriving from nested evaluation, never from the Function's
        // own operand slot, which the parser reads as a
        // Number.
        for value in 0x00..=0x7F {
            assert_eq!(
                evaluate(Function::ConvertToNote, note(value)).unwrap(),
                Interpretation::Cell(note(value)),
                "{value:02X}"
            );
        }

        // And every Number above the range diagnoses and produces no result at
        // all. Not `Atom::Empty`, which is the Interpreter's silent "no result
        // write", and not a value folded into the range: `80`–`FF` name no
        // pitch, so the Source is told rather than answered.
        for value in 0x80..=u8::MAX {
            assert!(
                matches!(
                    evaluate(Function::ConvertToNote, Atom::Number(value)),
                    Err(Error::Interpretation(InterpretationError::NoteConversion(found)))
                        if found == value
                ),
                "{value:02X}"
            );
        }
    }

    #[test]
    fn both_conversions_accept_both_numeric_types_as_one_atom() {
        // Idempotence, at the scalar shape: each conversion leaves a
        // value already of its result type unchanged, and converts the other.
        for value in 0x00..=0x7F {
            assert_eq!(
                evaluate(Function::ConvertToNumber, Atom::Number(value)).unwrap(),
                Interpretation::Cell(Atom::Number(value)),
                "{value:02X}"
            );
            assert_eq!(
                evaluate(Function::ConvertToNumber, note(value)).unwrap(),
                Interpretation::Cell(Atom::Number(value)),
                "{value:02X}"
            );
            assert_eq!(
                evaluate(Function::ConvertToNote, note(value)).unwrap(),
                Interpretation::Cell(note(value)),
                "{value:02X}"
            );
            assert_eq!(
                evaluate(Function::ConvertToNote, Atom::Number(value)).unwrap(),
                Interpretation::Cell(note(value)),
                "{value:02X}"
            );
        }

        // `.v` has no range to refuse: every Number is its own identity.
        for value in 0x80..=u8::MAX {
            assert_eq!(
                evaluate(Function::ConvertToNumber, Atom::Number(value)).unwrap(),
                Interpretation::Cell(Atom::Number(value)),
                "{value:02X}"
            );
        }
    }

    #[test]
    fn a_non_numeric_operand_diagnoses_as_non_numeric_for_both_conversions() {
        for (function, _) in BODIES {
            assert!(
                matches!(
                    evaluate(function, Atom::Bang),
                    Err(Error::Type(TypeError::Numeric(found))) if found == "**"
                ),
                "{function:?}"
            );
            assert!(
                matches!(
                    evaluate(function, Atom::Function(Function::Add)),
                    Err(Error::Type(TypeError::Numeric(found))) if found == ".+"
                ),
                "{function:?}"
            );
        }
    }

    #[test]
    fn a_missing_operand_diagnoses_as_arity_before_anything_is_read() {
        for (function, body) in BODIES {
            let mut ctx = Context::new(inputs().into(), 2);

            assert!(
                matches!(
                    body(&mut ctx),
                    Err(Error::Argument(ArgumentError::Arity {
                        expected: 1,
                        found: 0
                    }))
                ),
                "{function:?}"
            );
        }
    }

    #[test]
    fn a_conversion_reads_only_its_own_operand_from_a_deeper_stack() {
        for (function, body) in BODIES {
            let mut ctx = Context::new(inputs().into(), 4);
            ctx.stack.push(Atom::Bang).unwrap();
            ctx.stack.push(Atom::Number(0x3C)).unwrap();

            assert!(body(&mut ctx).is_ok(), "{function:?}");
            assert_eq!(ctx.stack.pop_value(), Some(Atom::Bang));
        }
    }

    #[test]
    fn each_literal_slot_reads_the_type_its_signature_declares() {
        // `.vA0` consumes Note `A0`; `.^A0` consumes Number `A0`, which names
        // no MIDI Note. The spelling is the same and the slot decides.
        assert_eq!(
            interpret(".vA0").unwrap(),
            Interpretation::Cell(Atom::Number(0x15))
        );
        assert!(matches!(
            interpret(".^A0"),
            Err(Error::Interpretation(InterpretationError::NoteConversion(
                0xA0
            )))
        ));
        assert_eq!(interpret(".^3C").unwrap(), Interpretation::Cell(note(0x3C)));
    }

    #[test]
    fn nested_conversions_compose_and_stay_idempotent() {
        assert_eq!(
            nested(Function::ConvertToNumber, ".^3C").unwrap(),
            Interpretation::Cell(Atom::Number(0x3C))
        );
        assert_eq!(
            nested(Function::ConvertToNote, ".vC4").unwrap(),
            Interpretation::Cell(note(0x3C))
        );
        assert_eq!(
            nested(Function::ConvertToNote, ".^3C").unwrap(),
            Interpretation::Cell(note(0x3C))
        );
        assert_eq!(
            nested(Function::ConvertToNumber, ".vC4").unwrap(),
            Interpretation::Cell(Atom::Number(0x3C))
        );
    }

    #[test]
    fn a_nested_non_numeric_answer_diagnoses_where_the_conversion_pops_it() {
        // Equal operands make `.=` answer a Bang, which each conversion pops.
        for function in [Function::ConvertToNote, Function::ConvertToNumber] {
            assert!(
                matches!(
                    nested(function, ".=0101"),
                    Err(Error::Type(TypeError::Numeric(found))) if found == "**"
                ),
                "{function:?}"
            );
        }
    }
}
