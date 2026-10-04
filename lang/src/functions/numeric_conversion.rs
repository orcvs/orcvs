use crate::{
    Atom, Error, Note,
    atom::operands::{ConvertToNote, ConvertToNumber},
    interpreter::Context,
};

/// Convert to Number: `.v note`.
///
/// The operand is a Note, however its characters arrived: a nested Function's
/// Return is decoded by this operand's declared type, so a value that is
/// already a Number is read as a Note spelling or refused.
#[inline]
pub fn to_number(ctx: &mut Context) -> Result<Atom, Error> {
    ctx.stack
        .apply(|ConvertToNumber { value }: ConvertToNumber| Ok(Atom::Number(value.value())))
}

/// Convert to Note: `.^ number`.
///
/// `80` through `FF` name no MIDI Note, so they diagnose rather than being
/// folded into the range.
#[inline]
pub fn to_note(ctx: &mut Context) -> Result<Atom, Error> {
    ctx.stack.apply(|ConvertToNote { value }: ConvertToNote| {
        // `Note`'s own conversion is the one range check, and its refusal
        // is the diagnostic.
        Ok(Atom::Note(Note::try_from(value)?))
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

    /// Evaluates `outer` over the Return of the flat Source text `inner`, as
    /// the Turn nesting it does: `lang` evaluates one Function per Turn, the
    /// inner Turn returns its answer's two-Cell encoding, and the outer
    /// Turn's operand is that encoding decoded by the outer signature.
    fn nested(outer: Function, inner: &str) -> Result<Interpretation, Error> {
        let returned = match interpret(inner).expect("the inner Turn answers a value") {
            Interpretation::Cell(atom) => atom.to_string(),
            other => panic!("{inner:?} answers {other:?}, which no Turn returns"),
        };
        let [token] = outer.signature() else {
            panic!("{outer:?} declares one operand");
        };
        evaluate(outer, token.decode(&returned)?)
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
    fn conversion_to_note_refuses_every_number_above_the_range() {
        // Every Number above the range diagnoses and produces no result at
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
    fn each_conversion_refuses_a_value_of_the_type_it_answers() {
        // The operand is the declared literal type and nothing else: a value
        // already of the result type is not passed through.
        for value in 0x00..=0x7F {
            assert!(
                matches!(
                    evaluate(Function::ConvertToNumber, Atom::Number(value)),
                    Err(Error::Type(TypeError::Note(_)))
                ),
                "{value:02X}"
            );
            assert!(
                matches!(
                    evaluate(Function::ConvertToNote, note(value)),
                    Err(Error::Type(TypeError::Number(_)))
                ),
                "{value:02X}"
            );
        }
    }

    #[test]
    fn a_non_numeric_operand_diagnoses_as_the_declared_literal_type() {
        for value in [Atom::Bang, Atom::Function(Function::Add)] {
            let spelling = value.to_string();
            assert!(
                matches!(
                    evaluate(Function::ConvertToNumber, value),
                    Err(Error::Type(TypeError::Note(found))) if found == spelling
                ),
                "{spelling}"
            );
            assert!(
                matches!(
                    evaluate(Function::ConvertToNote, value),
                    Err(Error::Type(TypeError::Number(found))) if found == spelling
                ),
                "{spelling}"
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
        for ((function, body), operand) in BODIES.into_iter().zip([Atom::Number(0x3C), note(0x3C)])
        {
            let mut ctx = Context::new(inputs().into(), 4);
            ctx.stack.push(Atom::Bang).unwrap();
            ctx.stack.push(operand).unwrap();

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
    fn nested_conversions_compose_and_read_their_return_by_the_receiving_operand() {
        // Opposite directions compose: each Return spells the type the outer
        // conversion declares.
        assert_eq!(
            nested(Function::ConvertToNumber, ".^3C").unwrap(),
            Interpretation::Cell(Atom::Number(0x3C))
        );
        assert_eq!(
            nested(Function::ConvertToNote, ".vC4").unwrap(),
            Interpretation::Cell(note(0x3C))
        );
        // The same direction does not pass the child's type through. `.^3C`
        // returns `C4`, which `.^` reads as Number `C4`, outside the Note
        // range; `.vC4` returns `3C`, which is not a Note spelling.
        assert!(matches!(
            nested(Function::ConvertToNote, ".^3C"),
            Err(Error::Interpretation(InterpretationError::NoteConversion(
                0xC4
            )))
        ));
        assert!(matches!(
            nested(Function::ConvertToNumber, ".vC4"),
            Err(Error::Type(TypeError::Note(found))) if found == "3C"
        ));
    }

    #[test]
    fn a_nested_bang_return_diagnoses_as_the_receiving_literal_type() {
        // Equal operands make `.=` answer a Bang, whose Return `**` spells
        // neither a Note nor a Number.
        assert!(matches!(
            nested(Function::ConvertToNumber, ".=0101"),
            Err(Error::Type(TypeError::Note(found))) if found == "**"
        ));
        assert!(matches!(
            nested(Function::ConvertToNote, ".=0101"),
            Err(Error::Type(TypeError::Number(found))) if found == "**"
        ));
    }
}
