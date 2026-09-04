use crate::{
    Atom, Error, InterpretationError, Note, Value,
    atom::operands::{ConvertToNote, ConvertToNumber},
    interpreter::Context,
    stack::NumericValue,
};

/// Convert to Number: `.v value`.
///
/// One expression per element, and `Stack::convert` decides whether that
/// element is the whole operation or one member of a Sequence. ADR 0021's
/// idempotence is what makes the Number arm not a coercion: a value that is
/// already a Number arrives from nested evaluation or from broadcasting, never
/// from this Function's own literal operand slot, which the parser reads as a
/// Note.
#[inline(always)]
pub fn to_number(ctx: &mut Context) -> Result<Value, Error> {
    ctx.stack.convert::<ConvertToNumber, _>(|value| {
        Ok(Atom::Number(match value {
            NumericValue::Note(value) => value.value(),
            NumericValue::Number(value) => value,
        }))
    })
}

/// Convert to Note: `.^ value`.
///
/// `80` through `FF` name no MIDI Note, so they diagnose rather than being
/// folded into the range. Over a Sequence that is a diagnostic about the
/// complete operation: `convert` assembles nothing until every element has
/// answered, so one unconvertible member leaves no partial Sequence of the
/// members that did convert.
#[inline(always)]
pub fn to_note(ctx: &mut Context) -> Result<Value, Error> {
    ctx.stack.convert::<ConvertToNote, _>(|value| match value {
        NumericValue::Note(value) => Ok(Atom::Note(value)),
        NumericValue::Number(value @ 0x00..=0x7F) => Ok(Atom::Note(Note::try_from(value)?)),
        NumericValue::Number(value) => Err(InterpretationError::NoteConversion(value).into()),
    })
}

#[cfg(test)]
mod test {
    use super::{to_note, to_number};
    use crate::{
        Anchor, Atom, Error, InterpretationError, Note, Sequence, Tick, TickInputs, TypeError,
        Value, interpreter::Context,
    };

    type Conversion = fn(&mut Context) -> Result<Value, Error>;

    /// Evaluates one conversion against the whole language value its single
    /// operand position holds.
    fn evaluate(conversion: Conversion, value: impl Into<Value>) -> Result<Value, Error> {
        // A conversion reads no Tick and no Position, so the first Tick at the
        // Grid origin is as good as any other.
        let mut ctx = Context::new(TickInputs::new(Tick::ZERO, Anchor::new(0, 0)));
        ctx.stack.push(value.into()).unwrap();
        conversion(&mut ctx)
    }

    fn note(value: u8) -> Atom {
        Atom::Note(Note::try_from(value).unwrap())
    }

    fn notes(values: impl IntoIterator<Item = u8>) -> Sequence {
        Sequence::new(values.into_iter().map(note)).unwrap()
    }

    fn numbers(values: impl IntoIterator<Item = u8>) -> Sequence {
        Sequence::new(values.into_iter().map(Atom::Number)).unwrap()
    }

    #[test]
    fn a_conversion_over_one_atom_answers_one_atom() {
        // Broadcasting must leave the scalar spelling the Source already writes
        // exactly as it was: a singleton Sequence would encode the same and
        // reach tick planning through a different arm.
        for value in 0x00..=0x7F {
            assert_eq!(
                evaluate(to_number, note(value)).unwrap(),
                Value::Atom(Atom::Number(value))
            );
            assert_eq!(
                evaluate(to_note, Atom::Number(value)).unwrap(),
                Value::Atom(note(value))
            );
        }
    }

    #[test]
    fn a_conversion_extends_atom_wise_and_preserves_order() {
        // Members that are not in ascending order, so a conversion that sorted
        // or reversed its Sequence answers a different value rather than the
        // same one.
        assert_eq!(
            evaluate(to_number, notes([0x3C, 0x00, 0x7F])).unwrap(),
            Value::Sequence(numbers([0x3C, 0x00, 0x7F]))
        );
        assert_eq!(
            evaluate(to_note, numbers([0x3C, 0x00, 0x7F])).unwrap(),
            Value::Sequence(notes([0x3C, 0x00, 0x7F]))
        );

        // An empty Sequence converts to the empty Sequence, and a singleton
        // stays a Sequence rather than collapsing to the Atom it holds.
        assert_eq!(
            evaluate(to_note, Sequence::empty()).unwrap(),
            Value::Sequence(Sequence::empty())
        );
        assert_eq!(
            evaluate(to_number, notes([0x3C])).unwrap(),
            Value::Sequence(numbers([0x3C]))
        );
    }

    #[test]
    fn evaluation_time_idempotence_survives_broadcasting() {
        // ADR 0021 makes each conversion an identity over its own result type
        // for values that arrive from nested evaluation or from a Sequence.
        // Without it a broadcast conversion could not compose with another one
        // over the same Sequence.
        assert_eq!(
            evaluate(to_number, numbers([0x3C, 0x7F])).unwrap(),
            Value::Sequence(numbers([0x3C, 0x7F]))
        );
        assert_eq!(
            evaluate(to_note, notes([0x3C, 0x7F])).unwrap(),
            Value::Sequence(notes([0x3C, 0x7F]))
        );
    }

    #[test]
    fn a_member_outside_the_midi_range_diagnoses_the_complete_conversion() {
        // The unconvertible member is the last one, so the two that already
        // converted are discarded rather than answered as a shorter Sequence.
        for value in 0x80..=u8::MAX {
            assert!(
                matches!(
                    evaluate(to_note, numbers([0x00, 0x3C, value])),
                    Err(Error::Interpretation(InterpretationError::NoteConversion(found)))
                        if found == value
                ),
                "{value:02X}"
            );
        }
    }

    #[test]
    fn a_non_numeric_member_diagnoses_before_any_member_converts() {
        // A Bang is a legitimate Sequence member and is not a numeric value, so
        // it is the type layer that refuses it. It stands after a member that
        // would itself fail to convert, which is what makes the ordering
        // observable rather than incidental.
        assert!(matches!(
            evaluate(to_note, Sequence::new([Atom::Number(0x80), Atom::Bang]).unwrap()),
            Err(Error::Type(TypeError::Numeric(found))) if found == "**"
        ));
    }
}
