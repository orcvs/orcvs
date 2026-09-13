use crate::{
    Atom, Error, Note, Sequence, SequenceError, TypeError, Value, atom::operands,
    interpreter::Context,
};

/// Reverse `:<`: reverse Atom order, preserving each member's type and encoding.
#[inline(always)]
pub fn reverse(ctx: &mut Context) -> Result<Value, Error> {
    let operands::Reverse { sequence } = ctx.stack.extract_values::<operands::Reverse>()?;
    let mut atoms: Vec<Atom> = sequence.atoms().to_vec();
    atoms.reverse();
    Ok(Sequence::new(atoms)?.into())
}

/// Concatenate `:&`: promote Atoms, stay flat, treat empty Sequence as identity.
#[inline(always)]
pub fn concatenate(ctx: &mut Context) -> Result<Value, Error> {
    let operands::Concatenate { left, right } =
        ctx.stack.extract_values::<operands::Concatenate>()?;
    let mut atoms = left.atoms().to_vec();
    atoms.extend_from_slice(right.atoms());
    Ok(Sequence::new(atoms)?.into())
}

/// Select `:?`: zero-based index modulo length; empty Sequence diagnoses.
#[inline(always)]
pub fn select(ctx: &mut Context) -> Result<Value, Error> {
    let operands::Select { index, sequence } = ctx.stack.extract_values::<operands::Select>()?;
    require_non_empty(&sequence)?;
    Ok(sequence.atoms()[wrapping_index(index, sequence.len())].into())
}

/// Replace `:=`: same-length Sequence with one Atom replaced; input unchanged.
#[inline(always)]
pub fn replace(ctx: &mut Context) -> Result<Value, Error> {
    let operands::Replace {
        index,
        replacement,
        sequence,
    } = ctx.stack.extract_values::<operands::Replace>()?;
    require_non_empty(&sequence)?;
    let mut atoms: Vec<Atom> = sequence.atoms().to_vec();
    let selected = wrapping_index(index, atoms.len());
    atoms[selected] = replacement;
    Ok(Sequence::new(atoms)?.into())
}

/// Number Range `:-`: inclusive unit-step Sequence between two Numbers.
#[inline(always)]
pub fn number_range(ctx: &mut Context) -> Result<Value, Error> {
    let operands::NumberRange { lower, upper } =
        ctx.stack.extract_values::<operands::NumberRange>()?;
    Ok(inclusive_number_range(lower, upper)?.into())
}

/// Note Range `:#`: inclusive chromatic Sequence between two Notes.
#[inline(always)]
pub fn note_range(ctx: &mut Context) -> Result<Value, Error> {
    let operands::NoteRange { lower, upper } = ctx.stack.extract_values::<operands::NoteRange>()?;
    Ok(inclusive_note_range(lower, upper)?.into())
}

#[inline(always)]
fn require_non_empty(sequence: &Sequence) -> Result<(), Error> {
    if sequence.is_empty() {
        Err(SequenceError::EmptyNotAllowed.into())
    } else {
        Ok(())
    }
}

#[inline(always)]
fn wrapping_index(index: u8, length: usize) -> usize {
    usize::from(index) % length
}

fn inclusive_byte_range(lower: u8, upper: u8) -> (i16, i16, i16) {
    if lower <= upper {
        (i16::from(lower), 1, i16::from(upper) - i16::from(lower) + 1)
    } else {
        (
            i16::from(lower),
            -1,
            i16::from(lower) - i16::from(upper) + 1,
        )
    }
}

fn inclusive_number_range(lower: u8, upper: u8) -> Result<Sequence, Error> {
    let (start, step, count) = inclusive_byte_range(lower, upper);
    build_ranged_sequence(start, step, count, |value| {
        Ok(Atom::Number(number_in_range(value)?))
    })
}

fn inclusive_note_range(lower: Note, upper: Note) -> Result<Sequence, Error> {
    let (start, step, count) = inclusive_byte_range(lower.value(), upper.value());
    build_ranged_sequence(start, step, count, |value| {
        Ok(Atom::Note(Note::try_from(number_in_range(value)?)?))
    })
}

fn build_ranged_sequence(
    start: i16,
    step: i16,
    count: i16,
    mut map: impl FnMut(i16) -> Result<Atom, Error>,
) -> Result<Sequence, Error> {
    // Inclusive ranges between two u8 bounds produce a positive count no
    // larger than 256; the conversion is checked so a future miscalculation
    // diagnoses rather than silently truncating.
    let count =
        usize::try_from(count).map_err(|_| Error::Type(TypeError::Number(format!("{count}"))))?;
    let mut atoms = Vec::with_capacity(count);
    let mut value = start;
    for _ in 0..count {
        atoms.push(map(value)?);
        value += step;
    }
    Sequence::new(atoms)
}

fn number_in_range(value: i16) -> Result<u8, Error> {
    u8::try_from(value).map_err(|_| Error::Type(TypeError::Number(format!("{value:02X}"))))
}

#[cfg(test)]
mod test {
    use crate::{
        Anchor, ArgumentError, Atom, Error, Function, Interpretation, Interpreter, Note, Parser,
        Sequence, SequenceError, Tick, TickInputs, TypeError, Value,
    };

    fn inputs() -> TickInputs {
        TickInputs::new(Tick::ZERO, Anchor::new(0, 0))
    }

    fn interpret(source: &str) -> Result<Interpretation, Error> {
        let mut source = source.to_string();
        let atoms = Parser::from(&mut source).try_parse()?;
        Interpreter::execute(&atoms, inputs())
    }

    fn evaluate(function: Function, operands: &[Value]) -> Result<Interpretation, Error> {
        Interpreter::execute_function(function, operands, inputs().into())
    }

    fn numbers(values: impl IntoIterator<Item = u8>) -> Sequence {
        Sequence::new(values.into_iter().map(Atom::Number)).unwrap()
    }

    fn notes(values: impl IntoIterator<Item = u8>) -> Sequence {
        Sequence::new(
            values
                .into_iter()
                .map(|value| Atom::Note(Note::try_from(value).unwrap())),
        )
        .unwrap()
    }

    #[test]
    fn reverse_preserves_atom_type_and_encoding() {
        let sequence = Sequence::new([
            Atom::Number(0x0A),
            Atom::Note(Note::try_from(60).unwrap()),
            Atom::Bang,
        ])
        .unwrap();

        assert_eq!(
            evaluate(Function::Reverse, &[Value::Sequence(sequence.clone())]).unwrap(),
            Interpretation::Sequence(
                Sequence::new([
                    Atom::Bang,
                    Atom::Note(Note::try_from(60).unwrap()),
                    Atom::Number(0x0A)
                ])
                .unwrap()
            )
        );
        assert_eq!(
            interpret(":<:-0003").unwrap(),
            Interpretation::Sequence(numbers([0x03, 0x02, 0x01, 0x00]))
        );
    }

    #[test]
    fn reverse_leaves_singleton_and_empty_sequences_unchanged() {
        let singleton = Sequence::promote(Atom::Number(0x0A)).unwrap();
        assert_eq!(
            evaluate(Function::Reverse, &[Value::Sequence(singleton.clone())]).unwrap(),
            Interpretation::Sequence(singleton)
        );

        let empty = Sequence::empty();
        assert_eq!(
            evaluate(Function::Reverse, &[Value::Sequence(empty.clone())]).unwrap(),
            Interpretation::Sequence(empty)
        );
    }

    #[test]
    fn concatenate_promotes_atoms_stays_flat_and_treats_empty_as_identity() {
        let left = numbers([0x01, 0x02]);
        let right = numbers([0x03]);

        assert_eq!(
            evaluate(
                Function::Concatenate,
                &[
                    Value::Sequence(left.clone()),
                    Value::Sequence(right.clone())
                ]
            )
            .unwrap(),
            Interpretation::Sequence(numbers([0x01, 0x02, 0x03]))
        );

        assert_eq!(
            evaluate(
                Function::Concatenate,
                &[
                    Value::Atom(Atom::Number(0x01)),
                    Value::Sequence(right.clone())
                ]
            )
            .unwrap(),
            Interpretation::Sequence(numbers([0x01, 0x03]))
        );

        assert_eq!(
            evaluate(
                Function::Concatenate,
                &[Value::Sequence(left), Value::Atom(Atom::Number(0x03))]
            )
            .unwrap(),
            Interpretation::Sequence(numbers([0x01, 0x02, 0x03]))
        );

        let empty = Sequence::empty();
        assert_eq!(
            evaluate(
                Function::Concatenate,
                &[
                    Value::Sequence(empty.clone()),
                    Value::Sequence(numbers([0x01]))
                ]
            )
            .unwrap(),
            Interpretation::Sequence(numbers([0x01]))
        );
        assert_eq!(
            evaluate(
                Function::Concatenate,
                &[
                    Value::Sequence(numbers([0x01])),
                    Value::Sequence(empty.clone())
                ]
            )
            .unwrap(),
            Interpretation::Sequence(numbers([0x01]))
        );
    }

    #[test]
    fn select_uses_a_wrapping_number_index_and_preserves_the_chosen_atom() {
        let sequence = numbers([0x0A, 0x0B, 0x0C]);

        assert_eq!(
            evaluate(
                Function::Select,
                &[
                    Value::Atom(Atom::Number(0x00)),
                    Value::Sequence(sequence.clone())
                ]
            )
            .unwrap(),
            Interpretation::Cell(Atom::Number(0x0A))
        );
        assert_eq!(
            evaluate(
                Function::Select,
                &[Value::Atom(Atom::Number(0x04)), Value::Sequence(sequence)]
            )
            .unwrap(),
            Interpretation::Cell(Atom::Number(0x0B))
        );
    }

    #[test]
    fn select_diagnoses_empty_sequences_and_non_number_indices() {
        assert!(matches!(
            evaluate(
                Function::Select,
                &[
                    Value::Atom(Atom::Number(0x00)),
                    Value::Sequence(Sequence::empty())
                ]
            ),
            Err(Error::Sequence(SequenceError::EmptyNotAllowed))
        ));

        let sequence = numbers([0x0A]);
        assert!(matches!(
            evaluate(
                Function::Select,
                &[
                    Value::Atom(Atom::Note(Note::try_from(0).unwrap())),
                    Value::Sequence(sequence.clone())
                ]
            ),
            Err(Error::Type(TypeError::Number(_)))
        ));
        assert!(matches!(
            evaluate(
                Function::Select,
                &[Value::Sequence(sequence), Value::Sequence(numbers([0x0A]))]
            ),
            Err(Error::Sequence(SequenceError::ExpectedAtom(_)))
        ));
    }

    #[test]
    fn replace_returns_a_new_same_length_sequence_and_allows_a_different_replacement_type() {
        let sequence = Sequence::new([
            Atom::Number(0x0A),
            Atom::Note(Note::try_from(60).unwrap()),
            Atom::Number(0x0C),
        ])
        .unwrap();

        assert_eq!(
            evaluate(
                Function::Replace,
                &[
                    Value::Atom(Atom::Number(0x01)),
                    Value::Atom(Atom::Bang),
                    Value::Sequence(sequence.clone())
                ]
            )
            .unwrap(),
            Interpretation::Sequence(
                Sequence::new([Atom::Number(0x0A), Atom::Bang, Atom::Number(0x0C),]).unwrap()
            )
        );
        assert_eq!(sequence.atoms()[1], Atom::Note(Note::try_from(60).unwrap()));
    }

    #[test]
    fn replace_diagnoses_empty_sequences_non_number_indices_and_sequence_replacements() {
        let sequence = numbers([0x0A, 0x0B]);
        assert!(matches!(
            evaluate(
                Function::Replace,
                &[
                    Value::Atom(Atom::Number(0x00)),
                    Value::Atom(Atom::Number(0x01)),
                    Value::Sequence(Sequence::empty())
                ]
            ),
            Err(Error::Sequence(SequenceError::EmptyNotAllowed))
        ));
        assert!(matches!(
            evaluate(
                Function::Replace,
                &[
                    Value::Atom(Atom::Note(Note::try_from(0).unwrap())),
                    Value::Atom(Atom::Number(0x01)),
                    Value::Sequence(sequence.clone())
                ]
            ),
            Err(Error::Type(TypeError::Number(_)))
        ));
        assert!(matches!(
            evaluate(
                Function::Replace,
                &[
                    Value::Atom(Atom::Number(0x00)),
                    Value::Sequence(numbers([0x01])),
                    Value::Sequence(sequence)
                ]
            ),
            Err(Error::Sequence(SequenceError::ExpectedAtom(_)))
        ));
    }

    #[test]
    fn number_range_is_inclusive_and_respects_bound_order() {
        assert_eq!(
            evaluate(
                Function::NumberRange,
                &[
                    Value::Atom(Atom::Number(0x00)),
                    Value::Atom(Atom::Number(0x03))
                ]
            )
            .unwrap(),
            Interpretation::Sequence(numbers([0x00, 0x01, 0x02, 0x03]))
        );
        assert_eq!(
            evaluate(
                Function::NumberRange,
                &[
                    Value::Atom(Atom::Number(0x03)),
                    Value::Atom(Atom::Number(0x00))
                ]
            )
            .unwrap(),
            Interpretation::Sequence(numbers([0x03, 0x02, 0x01, 0x00]))
        );
        assert_eq!(
            evaluate(
                Function::NumberRange,
                &[
                    Value::Atom(Atom::Number(0x05)),
                    Value::Atom(Atom::Number(0x05))
                ]
            )
            .unwrap(),
            Interpretation::Sequence(numbers([0x05]))
        );
        assert_eq!(
            interpret(":-0003").unwrap(),
            evaluate(
                Function::NumberRange,
                &[
                    Value::Atom(Atom::Number(0x00)),
                    Value::Atom(Atom::Number(0x03))
                ]
            )
            .unwrap()
        );
    }

    #[test]
    fn note_range_preserves_note_identity_and_respects_bound_order() {
        assert_eq!(
            evaluate(
                Function::NoteRange,
                &[
                    Value::Atom(Atom::Note(Note::try_from(60).unwrap())),
                    Value::Atom(Atom::Note(Note::try_from(62).unwrap()))
                ]
            )
            .unwrap(),
            Interpretation::Sequence(notes([60, 61, 62]))
        );
        assert_eq!(
            evaluate(
                Function::NoteRange,
                &[
                    Value::Atom(Atom::Note(Note::try_from(62).unwrap())),
                    Value::Atom(Atom::Note(Note::try_from(60).unwrap()))
                ]
            )
            .unwrap(),
            Interpretation::Sequence(notes([62, 61, 60]))
        );
        assert_eq!(
            interpret(":#C4C4").unwrap(),
            evaluate(
                Function::NoteRange,
                &[
                    Value::Atom(Atom::Note(Note::try_from(60).unwrap())),
                    Value::Atom(Atom::Note(Note::try_from(60).unwrap()))
                ]
            )
            .unwrap()
        );
    }

    #[test]
    fn range_functions_diagnose_mixed_type_and_invalid_bounds() {
        assert!(matches!(
            evaluate(
                Function::NumberRange,
                &[
                    Value::Atom(Atom::Note(Note::try_from(60).unwrap())),
                    Value::Atom(Atom::Number(0x03))
                ]
            ),
            Err(Error::Type(TypeError::Number(_)))
        ));
        assert!(matches!(
            evaluate(
                Function::NoteRange,
                &[
                    Value::Atom(Atom::Number(0x03)),
                    Value::Atom(Atom::Note(Note::try_from(60).unwrap()))
                ]
            ),
            Err(Error::Type(TypeError::Note(_)))
        ));
        assert!(matches!(
            evaluate(
                Function::NumberRange,
                &[Value::Atom(Atom::Bang), Value::Atom(Atom::Number(0x03))]
            ),
            Err(Error::Type(TypeError::Number(_)))
        ));
        assert!(matches!(
            evaluate(
                Function::NoteRange,
                &[
                    Value::Atom(Atom::Bang),
                    Value::Atom(Atom::Note(Note::try_from(60).unwrap()))
                ]
            ),
            Err(Error::Type(TypeError::Note(_)))
        ));
        assert!(matches!(
            interpret(":#8080"),
            Err(Error::Type(TypeError::Note(found))) if found == "80"
        ));
    }

    #[test]
    fn structural_operations_preserve_bang_type_and_encoding() {
        let sequence = Sequence::new([Atom::Bang, Atom::Number(0x00), Atom::Bang]).unwrap();
        assert_eq!(
            evaluate(Function::Reverse, &[Value::Sequence(sequence.clone())]).unwrap(),
            Interpretation::Sequence(
                Sequence::new([Atom::Bang, Atom::Number(0x00), Atom::Bang]).unwrap()
            )
        );
        assert_eq!(
            evaluate(
                Function::Replace,
                &[
                    Value::Atom(Atom::Number(0x01)),
                    Value::Atom(Atom::Bang),
                    Value::Sequence(sequence)
                ]
            )
            .unwrap(),
            Interpretation::Sequence(Sequence::new([Atom::Bang, Atom::Bang, Atom::Bang]).unwrap())
        );
    }

    #[test]
    fn every_sequence_function_parses_and_round_trips_its_spelling() {
        for (source, function) in [
            (":-0003", Function::NumberRange),
            (":#C4C5", Function::NoteRange),
            (":&:-0101:-0202", Function::Concatenate),
            (":?00:-0103", Function::Select),
            (":=01.+0102:-0103", Function::Replace),
        ] {
            let mut text = source.to_string();
            let atoms = Parser::from(&mut text).try_parse().unwrap();
            assert_eq!(atoms[0], Atom::Function(function), "{source}");
            assert_eq!(function.to_string(), &source[..2], "{source}");
        }

        for (spelling, function) in [
            (":<", Function::Reverse),
            (":?", Function::Select),
            (":=", Function::Replace),
            (":#", Function::NoteRange),
            (":-", Function::NumberRange),
        ] {
            assert_eq!(function.to_string(), spelling);
            assert_eq!(
                Function::try_from(spelling).unwrap(),
                function,
                "{spelling} round-trips through TryFrom",
            );
        }
    }

    #[test]
    fn select_and_replace_parse_round_trips_execute_through_nested_operands() {
        assert_eq!(
            interpret(":?00:-0103").unwrap(),
            Interpretation::Cell(Atom::Number(0x01))
        );
        assert_eq!(
            interpret(":=01.+0102:-0103").unwrap(),
            Interpretation::Sequence(numbers([0x01, 0x03, 0x03]))
        );
    }

    #[test]
    fn select_declares_that_it_can_emit_bang_and_returns_a_bang_member() {
        assert!(Function::Select.can_emit_bang());
        let sequence = Sequence::new([Atom::Bang, Atom::Number(0x00)]).unwrap();
        assert_eq!(
            evaluate(
                Function::Select,
                &[Value::Atom(Atom::Number(0x00)), Value::Sequence(sequence)]
            )
            .unwrap(),
            Interpretation::Cell(Atom::Bang)
        );
        assert_eq!(
            interpret(":?00:<:=00.=0101:-0101").unwrap(),
            Interpretation::Cell(Atom::Bang)
        );
    }

    #[test]
    fn reverse_and_concatenate_diagnose_atoms_where_sequences_are_required() {
        assert!(matches!(
            evaluate(Function::Reverse, &[Value::Atom(Atom::Number(0x0A))]),
            Err(Error::Sequence(SequenceError::ExpectedSequence(_)))
        ));
        assert!(matches!(
            evaluate(
                Function::Concatenate,
                &[
                    Value::Atom(Atom::Function(Function::SelfBangingEast)),
                    Value::Sequence(numbers([0x0A]))
                ]
            ),
            Err(Error::Sequence(SequenceError::Member(_)))
        ));
    }

    #[test]
    fn structural_functions_diagnose_missing_operands() {
        assert!(matches!(
            evaluate(Function::Reverse, &[]),
            Err(Error::Argument(ArgumentError::Arity {
                expected: 1,
                found: 0
            }))
        ));
        assert!(matches!(
            evaluate(Function::Concatenate, &[Value::Sequence(numbers([0x01]))]),
            Err(Error::Argument(ArgumentError::Arity {
                expected: 2,
                found: 1
            }))
        ));
        assert!(matches!(
            evaluate(
                Function::Replace,
                &[
                    Value::Atom(Atom::Number(0x00)),
                    Value::Atom(Atom::Number(0x01))
                ]
            ),
            Err(Error::Argument(ArgumentError::Arity {
                expected: 3,
                found: 2
            }))
        ));
    }

    #[test]
    fn number_range_interpret_diagnoses_a_note_operand_bound() {
        assert!(matches!(
            interpret(":-.^3C03"),
            Err(Error::Type(TypeError::Number(found))) if found == "C4"
        ));
    }

    #[test]
    fn range_functions_diagnose_function_and_sequence_bounds() {
        assert!(matches!(
            evaluate(
                Function::NumberRange,
                &[
                    Value::Atom(Atom::Function(Function::Add)),
                    Value::Atom(Atom::Number(0x03))
                ]
            ),
            Err(Error::Type(TypeError::Number(_)))
        ));
        assert!(matches!(
            evaluate(
                Function::NoteRange,
                &[
                    Value::Atom(Atom::Function(Function::Add)),
                    Value::Atom(Atom::Note(Note::try_from(60).unwrap()))
                ]
            ),
            Err(Error::Type(TypeError::Note(_)))
        ));
        assert!(matches!(
            evaluate(
                Function::NumberRange,
                &[
                    Value::Sequence(numbers([0x00, 0x01])),
                    Value::Atom(Atom::Number(0x03))
                ]
            ),
            Err(Error::Sequence(SequenceError::ExpectedAtom(_)))
        ));
        assert!(matches!(
            evaluate(
                Function::NoteRange,
                &[
                    Value::Sequence(notes([60, 61])),
                    Value::Atom(Atom::Note(Note::try_from(62).unwrap()))
                ]
            ),
            Err(Error::Sequence(SequenceError::ExpectedAtom(_)))
        ));
        assert!(matches!(
            interpret(":-**0003"),
            Err(Error::Type(TypeError::Number(found))) if found == "**"
        ));
        assert!(matches!(
            interpret(":#**C4"),
            Err(Error::Type(TypeError::Note(found))) if found == "**"
        ));
    }

    #[test]
    fn number_range_spans_the_full_byte_without_wrapping() {
        let Interpretation::Sequence(sequence) = evaluate(
            Function::NumberRange,
            &[
                Value::Atom(Atom::Number(0x00)),
                Value::Atom(Atom::Number(0xFF)),
            ],
        )
        .unwrap() else {
            panic!("Number Range answers a Sequence");
        };
        assert_eq!(sequence.len(), 256);
        assert_eq!(sequence.atoms().first(), Some(&Atom::Number(0x00)));
        assert_eq!(sequence.atoms().last(), Some(&Atom::Number(0xFF)));
        assert_eq!(
            sequence
                .atoms()
                .iter()
                .filter(|atom| matches!(atom, Atom::Number(0x00)))
                .count(),
            1,
            "a wrapped range would duplicate zero"
        );
    }
}
