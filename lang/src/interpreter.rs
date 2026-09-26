use crate::{
    Atom, Error, Function, FunctionInputs, Performance, Sequence, SourceEffect, Stack, Value,
    functions::{self, math, numeric_conversion, tick},
};

pub struct Interpreter {}

#[derive(Clone, Debug, PartialEq)]
pub enum Interpretation {
    Cell(Atom),
    /// A Sequence value leaving evaluation intact.
    ///
    /// The Atomic Functions broadcast over a Sequence operand and answer one,
    /// Structural and Range Functions reach this variant from Source text.
    /// It exists because the whole point of the Sequence value
    /// is that it can cross Function evaluation and leave it without first
    /// becoming Source writes; adding it later would mean the consumer had
    /// already been written as though it could not.
    Sequence(Sequence),
    /// The ordered group of Play Commands one active Terminal Output Function
    /// root performs.
    ///
    /// ADR 0030 extends `!>` and `!~` over a Sequence operand, so one
    /// Expression answers many commands, ordered by element index, while still
    /// answering no value. It carries a group rather than one command because
    /// this is the seam `lang` publishes to `orcvs`: a consumer written against
    /// a single command would have to be rewritten when Control Change, Pitch
    /// Bend, and Monophonic Play arrive, which is the cost ADR 0030 names as
    /// its reason to decide this before the family grows. The scalar shape
    /// [`Performance::One`] is what every Source-spelled Play answers today,
    /// because the Range Functions that would spell a Sequence operand are
    /// unbuilt.
    Play(Performance),
    /// The lock one active locking Function root places on the Expression
    /// root at its Output Portal.
    ///
    /// The Portal lives on the Function, not on this answer: ADR 0009 keeps
    /// destination resolution in `orcvs` and this crate holds no Grid.
    Lock,
    /// The Cells one active Source-writing Function root plans to write.
    ///
    /// It carries a displacement and a spelling rather than Positions, because
    /// ADR 0009 keeps destination resolution in `orcvs` and this crate holds no
    /// Grid. The consumer turns the offset into a Portal, refuses a destination
    /// the Grid does not hold, and orders the writes of the bundle.
    Source(SourceEffect),
}

///
/// What one evaluation has to work with: the operands it has resolved so far,
/// and the explicit inputs ADR 0012 supplies alongside the Source Snapshot.
///
/// A Function reaching for the Tick, its anchor, or a declared Portal input
/// takes `&mut Context` exactly as an arithmetic Function does today, so a
/// Tick-reading Function is a new arm in the Interpreter's Function match
/// rather than a new evaluation path.
///
pub struct Context<'a> {
    pub stack: Stack,
    /// Playback Tick, anchor, and working Source at any Portal the Turn resolved.
    pub inputs: FunctionInputs<'a>,
}

impl<'a> Context<'a> {
    pub fn new(inputs: FunctionInputs<'a>, stack_limit: usize) -> Self {
        Self {
            stack: Stack::new(stack_limit),
            inputs,
        }
    }
}

impl Interpreter {
    /// Evaluates one Function with already resolved, typed inputs. Literal
    /// decoding and nested ownership belong to the caller; evaluation applies
    /// the Function's declared type, domain, absence and Sequence rules.
    /// [`FunctionInputs::portal_source`] borrows working Source when the
    /// Function declares a Portal input. Functions without one ignore it.
    ///
    /// `inputs` is the whole of what evaluation knows beyond the operands: no
    /// clock, static, or thread-local is read, so the same Function over the
    /// same operands and inputs answers the same way every time.
    ///
    /// ```
    /// use lang::{Anchor, Atom, Function, Interpretation, Interpreter, Sequence, Tick, TickInputs, Value};
    /// let sequence = Sequence::new([Atom::Number(5), Atom::Number(9)]).unwrap();
    /// let answer = Interpreter::execute_function(
    ///     Function::Subtract, &[Value::Sequence(sequence), Value::Atom(Atom::Number(2))],
    ///     TickInputs::new(Tick::ZERO, Anchor::new(0, 0)).into(),
    /// ).unwrap();
    /// assert_eq!(answer, Interpretation::Sequence(
    ///     Sequence::new([Atom::Number(3), Atom::Number(7)]).unwrap()));
    /// ```
    pub fn execute_function(
        function: Function,
        operands: &[Value],
        inputs: FunctionInputs<'_>,
    ) -> Result<Interpretation, Error> {
        let expected = function.signature().len();
        if operands.len() != expected {
            return Err(crate::ArgumentError::Arity {
                expected,
                found: operands.len(),
            }
            .into());
        }
        if function.locks_root() {
            return Ok(Interpretation::Lock);
        }
        if let Some(effect) = function.source_effect() {
            // The Source-writing Functions take no operand and read no
            // Context: the whole of the effect is declared in the table, so
            // this reads the declaration rather than repeating eight offsets
            // and two bundles. ADR 0029's asymmetry lives in the activation
            // column and the bundle, and both are settled before this.
            return Ok(Interpretation::Source(effect));
        }
        let mut ctx = Context::new(inputs, operands.len());
        for operand in operands.iter().rev() {
            ctx.stack.push(operand.clone())?;
        }
        // Every Function answers a language Value, so a Function that returns
        // a Sequence needs an arm here and nothing else: the match below
        // already carries whichever shape the Value holds.
        let value = match function {
            Function::AbsoluteDifference => math::absolute_difference(&mut ctx)?,
            Function::Add => math::add(&mut ctx)?,
            Function::Clock => tick::clock(&mut ctx)?,
            Function::ConvertToNote => numeric_conversion::to_note(&mut ctx)?,
            Function::ConvertToNumber => numeric_conversion::to_number(&mut ctx)?,
            Function::Delay => tick::delay(&mut ctx)?,
            Function::Divide => math::divide(&mut ctx)?,
            Function::Equality => math::equality(&mut ctx)?,
            Function::Euclidean => tick::euclidean(&mut ctx)?,
            Function::Increment => tick::increment(&mut ctx)?,
            Function::Interpolation => tick::interpolation(&mut ctx)?,
            Function::JumpEast | Function::JumpNorth | Function::JumpSouth | Function::JumpWest => {
                functions::jump::jump(&mut ctx, function)?
            }
            Function::Random => tick::random(&mut ctx)?,
            Function::Concatenate => functions::sequence::concatenate(&mut ctx)?,
            Function::NoteRange => functions::sequence::note_range(&mut ctx)?,
            Function::NumberRange => functions::sequence::number_range(&mut ctx)?,
            Function::Replace => functions::sequence::replace(&mut ctx)?,
            Function::Reverse => functions::sequence::reverse(&mut ctx)?,
            Function::Select => functions::sequence::select(&mut ctx)?,
            Function::Maximum => math::maximum(&mut ctx)?,
            Function::Minimum => math::minimum(&mut ctx)?,
            Function::Modulo => math::modulo(&mut ctx)?,
            Function::Multiply => math::multiply(&mut ctx)?,
            Function::Subtract => math::subtract(&mut ctx)?,
            Function::ControlChange => {
                return Ok(Interpretation::Play(functions::control_change(&mut ctx)?));
            }
            Function::MonophonicPlay => {
                return Ok(Interpretation::Play(functions::monophonic_play(&mut ctx)?));
            }
            Function::PitchBend => {
                return Ok(Interpretation::Play(functions::pitch_bend(&mut ctx)?));
            }
            Function::RawPlay => {
                return Ok(Interpretation::Play(functions::raw_play(&mut ctx)?));
            }
            Function::TimedPlay => {
                return Ok(Interpretation::Play(functions::timed_play(&mut ctx)?));
            }
            Function::Halt
            | Function::DirectionalBangEast
            | Function::DirectionalBangNorth
            | Function::DirectionalBangSouth
            | Function::DirectionalBangWest
            | Function::SelfBangingEast
            | Function::SelfBangingNorth
            | Function::SelfBangingSouth
            | Function::SelfBangingWest => {
                unreachable!("{function} returns as a lock or Source write before dispatch")
            }
        };
        Ok(match value {
            Value::Atom(atom) => Interpretation::Cell(atom),
            Value::Sequence(sequence) => Interpretation::Sequence(sequence),
        })
    }
}

#[cfg(test)]
mod test {

    use crate::{
        Anchor, ArgumentError, Atom, Error, Function, Interpretation, InterpretationError, Note,
        Tick, TickInputs, Token, TypeError, Value, interpreter::Interpreter, trace,
    };

    ///
    /// The explicit inputs for a test that is about neither time nor Position:
    /// the first Tick of a Playback run, at the Grid origin.
    ///
    fn inputs() -> TickInputs {
        TickInputs::new(Tick::ZERO, Anchor::new(0, 0))
    }

    /// Evaluates one Function over literal operands, the way a Turn hands
    /// them over once it has resolved them.
    fn evaluate(function: Function, operands: &[Atom]) -> Result<Interpretation, Error> {
        let operands: Vec<Value> = operands.iter().copied().map(Value::Atom).collect();
        Interpreter::execute_function(function, &operands, inputs().into())
    }

    /// `evaluate` for a Function that answers one Cell.
    fn interpret_stack(atoms: Vec<Atom>) -> Result<Atom, Error> {
        let Some((Atom::Function(function), operands)) = atoms.split_first() else {
            panic!("{atoms:?} does not start with a Function");
        };
        evaluate(*function, operands).map(|result| match result {
            Interpretation::Cell(atom) => atom,
            other => panic!("expected a Cell result, found {other:?}"),
        })
    }

    /// Source text for one Function over literal operands, read by the Parser
    /// so a transposed spelling in the table is caught.
    fn interpret_source(source: &str) -> Result<Atom, Error> {
        crate::interpret_source(source).map(|result| match result {
            Interpretation::Cell(atom) => atom,
            other => panic!("expected a Cell result, found {other:?}"),
        })
    }

    fn interpret(source: &str) -> Atom {
        interpret_source(source).unwrap()
    }

    #[test]
    fn test_add_function() {
        trace();

        let result = interpret(".+0102");

        let expected = Atom::Number(3);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_add_wraps_over_the_byte_domain() {
        trace();

        // 0xFF + 0xFF wraps modulo 256.
        let result = interpret(".+FFFF");
        assert_eq!(result, Atom::Number(254));

        // Exact boundary: 0xFF + 0x01 wraps to zero.
        let result = interpret(".+FF01");
        assert_eq!(result, Atom::Number(0));

        // Just below the boundary still computes normally
        let result = interpret(".+FE01");
        assert_eq!(result, Atom::Number(255));

        let result = interpret(".+FD01");
        assert_eq!(result, Atom::Number(254));
    }

    #[test]
    fn test_multiply_wraps_over_the_byte_domain() {
        trace();

        // 0x99 * 0x99 == 23409, whose low byte is 0x71.
        let result = interpret(".x9999");
        assert_eq!(result, Atom::Number(113));

        let result = interpret(".xFFFF");
        assert_eq!(result, Atom::Number(1));

        // Exact boundary: 0x10 * 0x10 wraps to zero.
        let result = interpret(".x1010");
        assert_eq!(result, Atom::Number(0));

        // Largest exactly-representable product is computed, not saturated
        let result = interpret(".x0F11");
        assert_eq!(result, Atom::Number(255));

        let result = interpret(".x0F10");
        assert_eq!(result, Atom::Number(240));
    }

    #[test]
    fn test_sub_function() {
        trace();

        let result = interpret(".-0201");

        let expected = Atom::Number(1);
        assert_eq!(result, expected);

        let result = interpret(".-0102");

        let expected = Atom::Number(255);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_multiply_function() {
        trace();

        let result = interpret(".x0201");

        let expected = Atom::Number(2);
        assert_eq!(result, expected);

        let result = interpret(".x0002");

        let expected = Atom::Number(0);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_divide() {
        trace();

        let result = interpret("./0402");

        let expected = Atom::Number(2);
        assert_eq!(result, expected);

        assert!(matches!(
            interpret_source("./0100"),
            Err(Error::Interpretation(InterpretationError::DivisionByZero))
        ));
    }

    #[test]
    fn test_with_missing_argument() {
        trace();

        let stack = vec![Atom::Function(Function::Add), Atom::Number(1)];

        let result = interpret_stack(stack);

        let error = result.unwrap_err();

        assert!(matches!(
            error,
            Error::Argument(ArgumentError::Arity {
                expected: 2,
                found: 1
            })
        ));
    }

    #[test]
    fn test_with_invalid_argument() {
        trace();

        let stack = vec![Atom::Function(Function::Add), Atom::Number(1), Atom::Bang];

        let result = interpret_stack(stack);

        let error = result.unwrap_err();

        assert!(matches!(error, Error::Type(TypeError::Number(_))));
    }

    #[test]
    fn general_arithmetic_rejects_note_operands() {
        // The numeric family is Number-only in both operand positions: a Note
        // is converted explicitly with `.v` or not at all, so neither position
        // may quietly read a Note's MIDI number as a Number.
        let note = Atom::Note(crate::Note::try_from(60).unwrap());
        for function in [
            Function::AbsoluteDifference,
            Function::Add,
            Function::Divide,
            Function::Equality,
            Function::Maximum,
            Function::Minimum,
            Function::Modulo,
            Function::Multiply,
            Function::Subtract,
        ] {
            for operands in [[note, Atom::Number(1)], [Atom::Number(1), note]] {
                let result = interpret_stack(
                    std::iter::once(Atom::Function(function))
                        .chain(operands)
                        .collect(),
                );
                assert!(
                    matches!(result, Err(Error::Type(TypeError::Number(_)))),
                    "{function:?} accepted {operands:?}",
                );
            }
        }
    }

    #[test]
    fn the_numeric_family_rejects_every_non_number_operand() {
        // Bang reaches the stack from Source text and Empty from an unequal
        // Equality answer, so neither may coerce into a Number either.
        for function in [
            Function::AbsoluteDifference,
            Function::Equality,
            Function::Maximum,
            Function::Minimum,
            Function::Modulo,
        ] {
            for operand in [Atom::Bang, Atom::Empty] {
                // Both slots, because a nested Function answers into either
                // one: an unequal `.=` puts Empty wherever it is written.
                for operands in [[operand, Atom::Number(1)], [Atom::Number(1), operand]] {
                    assert!(
                        matches!(
                            interpret_stack(
                                std::iter::once(Atom::Function(function))
                                    .chain(operands)
                                    .collect(),
                            ),
                            Err(Error::Type(TypeError::Number(_)))
                        ),
                        "{function:?} accepted {operands:?}",
                    );
                }
            }
        }
    }

    #[test]
    fn an_operand_type_diagnostic_names_the_operand_it_refused() {
        // A Note, and the Empty an unequal Equality answers into whichever
        // operand it stands in: neither reads as a Number, and the diagnostic
        // spells what it found.
        let note = Atom::Note(crate::Note::try_from(60).unwrap());
        for (operand, spelled) in [(note, "C4"), (Atom::Empty, "_")] {
            assert!(matches!(
                interpret_stack(vec![Atom::Function(Function::Add), operand, Atom::Number(1)]),
                Err(Error::Type(TypeError::Number(found))) if found == spelled
            ));
        }
    }

    #[test]
    fn direct_play_evaluation_enforces_each_operand_type() {
        let note = Atom::Note(crate::Note::try_from(60).unwrap());
        for atoms in [
            vec![Function::RawPlay.into(), note, Atom::Number(0x7F), note],
            vec![Function::RawPlay.into(), Atom::Number(0), note, note],
            vec![
                Function::RawPlay.into(),
                Atom::Number(0),
                Atom::Number(0x7F),
                Atom::Number(60),
            ],
            vec![
                Function::RawPlay.into(),
                Atom::Number(0),
                Atom::Number(0x7F),
                Atom::Bang,
            ],
        ] {
            assert!(matches!(interpret_stack(atoms), Err(Error::Type(_))));
        }
    }

    #[test]
    fn halt_locks_at_its_output_portal() {
        // Halt names no coordinates: its Output Portal is one row south, the
        // same Portal Add writes. The lock is the Effect, not a second geometry.
        assert_eq!(
            Function::Halt.output_portal(),
            Some(crate::PortalCoords::SOUTH)
        );
        assert_eq!(Function::Halt.input_portal(), None);
        assert!(Function::Halt.locks_root());
        assert_eq!(evaluate(Function::Halt, &[]).unwrap(), Interpretation::Lock);
        assert!(!Function::Halt.answers_value());
        assert!(!Function::Halt.can_emit_bang());
        assert!(!Function::Halt.is_intrinsically_active());
        assert!(Function::Halt.source_effect().is_none());
        assert!(!Function::Halt.performs_terminal_output());
    }

    #[test]
    fn locking_and_source_writing_functions_are_interpreted_from_their_kind() {
        for function in Function::ALL {
            if function.locks_root() {
                assert_eq!(
                    evaluate(*function, &[]).unwrap(),
                    Interpretation::Lock,
                    "{function:?}"
                );
            } else if let Some(effect) = function.source_effect() {
                assert_eq!(
                    evaluate(*function, &[]).unwrap(),
                    Interpretation::Source(effect),
                    "{function:?}"
                );
            }
        }
    }

    #[test]
    fn explicit_numeric_conversions_have_fixed_result_types() {
        // `.v` is the identity over Numbers across the whole byte domain, not
        // only the MIDI part of it. A Number reaches this Function from nested
        // evaluation or from broadcasting rather than from its literal operand
        // slot, and the arithmetic that produced it wraps over `00`–`FF`, so
        // `80`–`FF` arrive as often as anything else. Folding them into the
        // Note range would be the coercion ADR 0021 refuses, and diagnosing
        // them would make `.v` reject values `.^` never had to accept.
        for value in 0..=u8::MAX {
            assert_eq!(
                interpret_stack(vec![
                    Atom::Function(Function::ConvertToNumber),
                    Atom::Number(value),
                ])
                .unwrap(),
                Atom::Number(value),
                "{value:02X}"
            );
        }

        // The typed conversions themselves are defined over the MIDI range,
        // which is every value a Note can hold.
        for value in 0..=0x7F {
            assert_eq!(
                interpret_stack(vec![
                    Atom::Function(Function::ConvertToNumber),
                    Atom::Note(crate::Note::try_from(value).unwrap()),
                ])
                .unwrap(),
                Atom::Number(value)
            );
            assert_eq!(
                interpret_stack(vec![
                    Atom::Function(Function::ConvertToNote),
                    Atom::Number(value),
                ])
                .unwrap(),
                Atom::Note(crate::Note::try_from(value).unwrap())
            );
            assert_eq!(
                interpret_stack(vec![
                    Atom::Function(Function::ConvertToNote),
                    Atom::Note(crate::Note::try_from(value).unwrap()),
                ])
                .unwrap(),
                Atom::Note(crate::Note::try_from(value).unwrap())
            );
        }
    }

    #[test]
    fn conversion_to_note_rejects_numbers_outside_the_midi_range() {
        for value in 0x80..=u8::MAX {
            assert!(matches!(
                interpret_stack(vec![
                    Atom::Function(Function::ConvertToNote),
                    Atom::Number(value),
                ]),
                Err(Error::Interpretation(InterpretationError::NoteConversion(n))) if n == value
            ));
        }
    }

    #[test]
    fn conversion_source_literals_use_the_monomorphic_operand_type() {
        assert_eq!(interpret(".vA0"), Atom::Number(21));

        assert!(matches!(
            interpret_source(".^C4"),
            Err(Error::Interpretation(InterpretationError::NoteConversion(
                0xC4
            )))
        ));
    }

    #[test]
    fn absolute_difference_is_symmetric_and_never_underflows() {
        // `.-` wraps, so an ordered difference cannot express distance without
        // borrowing. `.|` exists precisely to answer the distance instead, so
        // both operand orders must agree for every pair a Source can write.
        for left in 0..=u8::MAX {
            for right in 0..=u8::MAX {
                let expected =
                    Atom::Number((i16::from(left) - i16::from(right)).unsigned_abs() as u8);

                for (a, b) in [(left, right), (right, left)] {
                    assert_eq!(
                        interpret_stack(vec![
                            Atom::Function(Function::AbsoluteDifference),
                            Atom::Number(a),
                            Atom::Number(b),
                        ])
                        .unwrap(),
                        expected,
                        "AbsoluteDifference({a:02X}, {b:02X})",
                    );
                }
            }
        }
    }

    #[test]
    fn modulo_returns_the_unsigned_remainder_for_every_non_zero_divisor() {
        for left in 0..=u8::MAX {
            for right in 1..=u8::MAX {
                assert_eq!(
                    interpret_stack(vec![
                        Atom::Function(Function::Modulo),
                        Atom::Number(left),
                        Atom::Number(right),
                    ])
                    .unwrap(),
                    Atom::Number((u16::from(left) % u16::from(right)) as u8),
                    "Modulo({left:02X}, {right:02X})",
                );
            }
        }
    }

    #[test]
    fn modulo_by_zero_diagnoses_distinctly_from_division_by_zero() {
        // A zero divisor has no remainder to invent, so `.%` produces no Atom
        // at all. It carries its own diagnostic rather than borrowing `./`'s,
        // so the Source is told which Function it wrote.
        for left in 0..=u8::MAX {
            assert!(
                matches!(
                    interpret_stack(vec![
                        Atom::Function(Function::Modulo),
                        Atom::Number(left),
                        Atom::Number(0),
                    ]),
                    Err(Error::Interpretation(InterpretationError::ModuloByZero))
                ),
                "Modulo({left:02X}, 00)",
            );
        }

        assert_ne!(
            InterpretationError::ModuloByZero.to_string(),
            InterpretationError::DivisionByZero.to_string()
        );
    }

    #[test]
    fn minimum_and_maximum_select_one_of_their_operands_for_every_pair_of_bytes() {
        // Selection, not arithmetic: the answer is always one operand
        // unchanged, so neither Function can wrap or clamp.
        for left in 0..=u8::MAX {
            for right in 0..=u8::MAX {
                for (function, expected) in [
                    (
                        Function::Minimum,
                        u16::from(left).min(u16::from(right)) as u8,
                    ),
                    (
                        Function::Maximum,
                        u16::from(left).max(u16::from(right)) as u8,
                    ),
                ] {
                    assert_eq!(
                        interpret_stack(vec![
                            Atom::Function(function),
                            Atom::Number(left),
                            Atom::Number(right),
                        ])
                        .unwrap(),
                        Atom::Number(expected),
                        "{function:?}({left:02X}, {right:02X})",
                    );
                }
            }
        }
    }

    #[test]
    fn only_a_function_that_declares_it_ever_answers_with_bang() {
        // `can_emit_bang` is a declaration Tick scheduling trusts to decide
        // which roots can supply activation, and a declaration checked only
        // against itself catches nothing: a Function that began answering Bang
        // without flipping its flag would build no activation edge, and the
        // neighbouring terminal root would fall silent with no diagnostic
        // anywhere.
        //
        // So the flag is checked against what the Interpreter actually
        // answers, over operands built from each Function's own signature.
        // Every operand of one attempt carries the same value, which is what
        // reaches Equality's Bang at all: an unequal pair answers Empty.
        for &function in Function::ALL {
            if !function.answers_value() {
                continue;
            }
            if function.answers_sequence()
                || function.input_portal().is_some()
                || function
                    .signature()
                    .iter()
                    .any(|token| matches!(token, Token::Atom | Token::Sequence))
            {
                // Sequence answers, Portal-read answers, and Atom/Sequence
                // operands are exercised on their own paths rather than
                // through this Atom-only sweep.
                continue;
            }

            let answers_bang = (0..=u8::MAX).any(|value| {
                let mut atoms = vec![Atom::Function(function)];
                atoms.extend(function.signature().iter().map(|token| match token {
                    Token::Number => Atom::Number(value),
                    Token::Note => Atom::Note(Note::try_from(value & 0x7F).expect("a MIDI note")),
                    other => panic!("no operand is declared as {other:?}"),
                }));
                matches!(interpret_stack(atoms), Ok(Atom::Bang))
            });

            assert_eq!(
                answers_bang,
                function.can_emit_bang(),
                "{function:?} answers Bang == {answers_bang} but declares \
                 can_emit_bang() == {}",
                function.can_emit_bang(),
            );
        }
    }

    #[test]
    fn equality_answers_a_bang_only_for_equal_numbers() {
        // Equality is a pulse, not a truth value: an unequal comparison answers
        // `Atom::Empty`, the Interpreter's existing "no result write" signal, so
        // the Source never gains a Cell meaning "false".
        for left in 0..=u8::MAX {
            for right in 0..=u8::MAX {
                let expected = if left == right {
                    Atom::Bang
                } else {
                    Atom::Empty
                };

                assert_eq!(
                    interpret_stack(vec![
                        Atom::Function(Function::Equality),
                        Atom::Number(left),
                        Atom::Number(right),
                    ])
                    .unwrap(),
                    expected,
                    "Equality({left:02X}, {right:02X})",
                );
            }
        }
    }

    #[test]
    fn the_numeric_family_evaluates_from_its_source_spellings() {
        // Ties each spelling to its behaviour end to end: the exhaustive tests
        // above build stacks directly and would not catch two definitions whose
        // spellings were transposed in the table.
        assert_eq!(interpret(".|050A"), Atom::Number(5));
        assert_eq!(interpret(".%0A03"), Atom::Number(1));
        assert_eq!(interpret(".<0A03"), Atom::Number(3));
        assert_eq!(interpret(".>0A03"), Atom::Number(10));
        assert_eq!(interpret(".=0A0A"), Atom::Bang);
    }

    #[test]
    fn general_arithmetic_wraps_for_every_pair_of_bytes() {
        for left in 0..=u8::MAX {
            for right in 0..=u8::MAX {
                for (function, expected) in [
                    (Function::Add, (u16::from(left) + u16::from(right)) as u8),
                    (
                        Function::Subtract,
                        (i16::from(left) - i16::from(right)).rem_euclid(256) as u8,
                    ),
                    (
                        Function::Multiply,
                        (u16::from(left) * u16::from(right)) as u8,
                    ),
                ] {
                    assert_eq!(
                        interpret_stack(vec![
                            Atom::Function(function),
                            Atom::Number(left),
                            Atom::Number(right),
                        ])
                        .unwrap(),
                        Atom::Number(expected),
                        "{function:?}({left:02X}, {right:02X})",
                    );
                }
            }
        }
    }

    #[test]
    fn division_is_the_asymmetry_that_diagnoses_every_zero_divisor() {
        // ADR 0011 wraps the rest of general arithmetic modulo 256, so every
        // other Function of the family answers a Number for every pair the
        // Source can write. Division is the one that cannot: a quotient by
        // zero has no cyclic position to wrap into, so `./` produces a
        // diagnostic and no result instead of inventing a Number. Both halves
        // of that asymmetry are enumerated here — a diagnostic for all 256
        // zero divisors, and a Number for all 65,280 pairs that have one.
        //
        // The expected quotient is counted by repeated subtraction rather than
        // written as `left / right`, so what it asserts is the definition of
        // floor division rather than a second spelling of the implementation.
        for left in 0..=u8::MAX {
            assert!(
                matches!(
                    interpret_stack(vec![
                        Atom::Function(Function::Divide),
                        Atom::Number(left),
                        Atom::Number(0),
                    ]),
                    Err(Error::Interpretation(InterpretationError::DivisionByZero))
                ),
                "Divide({left:02X}, 00)",
            );

            for right in 1..=u8::MAX {
                let mut remainder = left;
                let mut quotient = 0u8;
                while remainder >= right {
                    remainder -= right;
                    quotient += 1;
                }

                assert_eq!(
                    interpret_stack(vec![
                        Atom::Function(Function::Divide),
                        Atom::Number(left),
                        Atom::Number(right),
                    ])
                    .unwrap(),
                    Atom::Number(quotient),
                    "Divide({left:02X}, {right:02X})",
                );
            }
        }
    }
}
