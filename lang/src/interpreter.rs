use crate::{
    Atom, Atoms, Error, Function, InterpretationError, PlayCommand, Sequence, Stack, TickInputs,
    Value,
    functions::{self, math, numeric_conversion},
};

pub type Args = Stack<16>;

pub struct Interpreter {}

#[derive(Clone, Debug, PartialEq)]
pub enum Interpretation {
    Cell(Atom),
    /// A Sequence value leaving evaluation intact.
    ///
    /// No Source-parseable Function returns a Sequence yet — Range, Reverse,
    /// and Concatenate arrive with issues 02 and 03 — so nothing reaches this
    /// variant from Source text today. It exists now because the whole point
    /// of the Sequence value is that it can cross Function evaluation and
    /// leave it without first becoming Source writes; adding it later would
    /// mean the consumer had already been written as though it could not.
    Sequence(Sequence),
    Play(PlayCommand),
}

///
/// What one evaluation has to work with: the operands it has resolved so far,
/// and the explicit inputs ADR 0012 supplies alongside the Source Snapshot.
///
/// A Function reaching for the Tick or its anchor takes `&mut Context` exactly
/// as an arithmetic Function does today, so a Tick-reading Function is a new
/// arm in `execute` rather than a new evaluation path.
///
pub struct Context {
    pub stack: Args,
    /// The explicit inputs ADR 0012 supplies alongside the Source Snapshot.
    ///
    /// `dead_code` reads an unread field as one to delete, and here that
    /// conclusion is wrong rather than merely early: this field is the input
    /// side of the seam, and its value is already chosen by the Playback
    /// Engine and threaded through every caller. Deleting it would delete the
    /// threading, not an unused field. Clock, Delay, and Euclidean read the
    /// Tick and Random also reads the anchor. `expect` rather than `allow`, so
    /// the first Function to read the field turns this attribute into the
    /// error that deletes it.
    #[expect(
        dead_code,
        reason = "an unread input is not an unused one: this is the seam its consumers read"
    )]
    pub inputs: TickInputs,
}

impl Context {
    pub fn new(inputs: TickInputs) -> Self {
        Self {
            stack: Args::new(),
            inputs,
        }
    }
}

impl Interpreter {
    ///
    /// Evaluates one Expression's Atoms against `inputs`.
    ///
    /// `inputs` is the whole of what evaluation knows beyond the Atoms
    /// themselves: nothing here reads a clock, a static, or a thread-local, so
    /// the same Atoms and the same inputs answer the same way every time.
    ///
    #[inline(always)]
    pub fn execute(atoms: &Atoms, inputs: TickInputs) -> Result<Interpretation, Error> {
        let mut ctx = Context::new(inputs);

        for (index, atom) in atoms.iter().enumerate().rev() {
            // info!("atoms: {:?}", atoms);
            // info!("stack: {:?}", stack);
            // Every Function answers a language Value, so a Function that returns
            // a Sequence needs an arm here and nothing else: the push below already
            // carries whichever shape the Value holds.
            let value = match atom {
                // A Terminal Output Function performs an effect and answers
                // with no language value, so the only place it can stand is
                // the one place nothing consumes an answer: the Expression
                // root, which the Interpreter reaches last. Rejecting every
                // other index here leaves each terminal arm below free to
                // assume it is the root.
                Atom::Function(fun) if fun.is_terminal() && index != 0 => {
                    return Err(InterpretationError::NestedTerminalFunction.into());
                }
                Atom::Function(fun) => match fun {
                    Function::AbsoluteDifference => math::absolute_difference(&mut ctx)?,
                    Function::Add => math::add(&mut ctx)?,
                    Function::ConvertToNote => numeric_conversion::to_note(&mut ctx)?,
                    Function::ConvertToNumber => numeric_conversion::to_number(&mut ctx)?,
                    Function::Divide => math::divide(&mut ctx)?,
                    Function::Equality => math::equality(&mut ctx)?,
                    Function::Maximum => math::maximum(&mut ctx)?,
                    Function::Minimum => math::minimum(&mut ctx)?,
                    Function::Modulo => math::modulo(&mut ctx)?,
                    Function::Multiply => math::multiply(&mut ctx)?,
                    Function::Subtract => math::subtract(&mut ctx)?,
                    Function::Play => {
                        return Ok(Interpretation::Play(functions::play(&mut ctx)?));
                    }
                },
                atom => (*atom).into(),
            };
            ctx.stack.push(value);
        }

        // A non-terminal Expression leaves one language value on the stack.
        // Source decides where that value belongs when it builds the Tick Plan.
        // An empty stack is the absence marker, not a Sequence of no Atoms.
        Ok(match ctx.stack.pop_value() {
            None => Interpretation::Cell(Atom::Empty),
            Some(Value::Atom(atom)) => Interpretation::Cell(atom),
            Some(Value::Sequence(sequence)) => Interpretation::Sequence(sequence),
        })
    }
}

#[cfg(test)]
mod test {

    use crate::{
        Anchor, ArgumentError, Atom, Error, Function, InterpretationError, Parser, Tick,
        TickInputs, TypeError, interpreter::Interpreter, trace,
    };
    use tracing::info;

    ///
    /// The explicit inputs for a test that is about neither time nor Position:
    /// the first Tick of a Playback run, at the Grid origin.
    ///
    fn inputs() -> TickInputs {
        TickInputs::new(Tick::ZERO, Anchor::new(0, 0))
    }

    fn interpret(exp: String) -> Atom {
        let mut exp = exp.clone();
        let parser = Parser::from(&mut exp);
        let parsed = parser.try_parse().unwrap();

        info!("Parsed: {:?}", parsed);

        match Interpreter::execute(&parsed, inputs()).unwrap() {
            super::Interpretation::Cell(atom) => atom,
            other => panic!("expected a Cell result, found {other:?}"),
        }
    }

    fn interpret_stack(exp: Vec<Atom>) -> Result<Atom, Error> {
        let atoms = exp.into_iter().collect();
        Interpreter::execute(&atoms, inputs()).map(|result| match result {
            super::Interpretation::Cell(atom) => atom,
            other => panic!("expected a Cell result, found {other:?}"),
        })
    }

    #[test]
    fn test_add_function() {
        trace();

        let s = String::from(".+0102");
        let result = interpret(s);

        let expected = Atom::Number(3);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_add_wraps_over_the_byte_domain() {
        trace();

        // 0xFF + 0xFF wraps modulo 256.
        let s = String::from(".+FFFF");
        let result = interpret(s);
        assert_eq!(result, Atom::Number(254));

        // Exact boundary: 0xFF + 0x01 wraps to zero.
        let s = String::from(".+FF01");
        let result = interpret(s);
        assert_eq!(result, Atom::Number(0));

        // Just below the boundary still computes normally
        let s = String::from(".+FE01");
        let result = interpret(s);
        assert_eq!(result, Atom::Number(255));

        let s = String::from(".+FD01");
        let result = interpret(s);
        assert_eq!(result, Atom::Number(254));
    }

    #[test]
    fn test_multiply_wraps_over_the_byte_domain() {
        trace();

        // 0x99 * 0x99 == 23409, whose low byte is 0x71.
        let s = String::from(".x9999");
        let result = interpret(s);
        assert_eq!(result, Atom::Number(113));

        let s = String::from(".xFFFF");
        let result = interpret(s);
        assert_eq!(result, Atom::Number(1));

        // Exact boundary: 0x10 * 0x10 wraps to zero.
        let s = String::from(".x1010");
        let result = interpret(s);
        assert_eq!(result, Atom::Number(0));

        // Largest exactly-representable product is computed, not saturated
        let s = String::from(".x0F11");
        let result = interpret(s);
        assert_eq!(result, Atom::Number(255));

        let s = String::from(".x0F10");
        let result = interpret(s);
        assert_eq!(result, Atom::Number(240));
    }

    #[test]
    fn test_sub_function() {
        trace();

        let s = String::from(".-0201");
        let result = interpret(s);

        let expected = Atom::Number(1);
        assert_eq!(result, expected);

        let s = String::from(".-0102");
        let result = interpret(s);

        let expected = Atom::Number(255);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_multiply_function() {
        trace();

        let s = String::from(".x0201");
        let result = interpret(s);

        let expected = Atom::Number(2);
        assert_eq!(result, expected);

        let s = String::from(".x0002");
        let result = interpret(s);

        let expected = Atom::Number(0);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_divide() {
        trace();

        let s = String::from("./0402");
        let result = interpret(s);

        let expected = Atom::Number(2);
        assert_eq!(result, expected);

        let mut exp = String::from("./0100");
        let parsed = Parser::from(&mut exp).try_parse().unwrap();
        assert!(matches!(
            Interpreter::execute(&parsed, inputs()),
            Err(Error::Interpretation(InterpretationError::DivisionByZero))
        ));
    }

    #[test]
    fn test_recursive() {
        trace();

        let s = String::from(".+.+0101.-0A05");
        let result = interpret(s);

        let expected = Atom::Number(7);
        assert_eq!(result, expected);
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

        let stack = vec![
            Atom::Function(Function::Add),
            Atom::Number(1),
            Atom::Char('v'),
            Atom::Char('t'),
            Atom::Char('h'),
            Atom::Char('a'),
        ];

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
        // Char and Bang reach the stack from Source text and from an Equality
        // answer respectively, so neither may coerce into a Number either.
        for function in [
            Function::AbsoluteDifference,
            Function::Equality,
            Function::Maximum,
            Function::Minimum,
            Function::Modulo,
        ] {
            for operand in [Atom::Char('z'), Atom::Bang, Atom::Empty] {
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
    fn direct_and_nested_arithmetic_report_the_same_missing_operand_diagnostic() {
        for atoms in [
            vec![Atom::Function(Function::Add), Atom::Number(1)],
            vec![
                Atom::Function(Function::Add),
                Atom::Function(Function::Add),
                Atom::Number(1),
                Atom::Number(2),
            ],
        ] {
            assert!(matches!(
                interpret_stack(atoms),
                Err(Error::Argument(ArgumentError::Arity {
                    expected: 2,
                    found: 1,
                }))
            ));
        }
    }

    #[test]
    fn direct_and_nested_arithmetic_report_the_same_operand_type_diagnostic() {
        let note = Atom::Note(crate::Note::try_from(60).unwrap());
        for atoms in [
            vec![Atom::Function(Function::Add), note, Atom::Number(1)],
            vec![
                Atom::Function(Function::Add),
                Atom::Function(Function::ConvertToNote),
                Atom::Number(60),
                Atom::Number(1),
            ],
        ] {
            assert!(matches!(
                interpret_stack(atoms),
                Err(Error::Type(TypeError::Number(found))) if found == "C4"
            ));
        }
    }

    #[test]
    fn direct_play_evaluation_enforces_each_operand_type() {
        let note = Atom::Note(crate::Note::try_from(60).unwrap());
        for atoms in [
            vec![Function::Play.into(), note, Atom::Number(0x7F), note],
            vec![Function::Play.into(), Atom::Number(0), note, note],
            vec![
                Function::Play.into(),
                Atom::Number(0),
                Atom::Number(0x7F),
                Atom::Number(60),
            ],
            vec![
                Function::Play.into(),
                Atom::Number(0),
                Atom::Number(0x7F),
                Atom::Char('C'),
            ],
        ] {
            let atoms = atoms.into_iter().collect();
            assert!(matches!(
                Interpreter::execute(&atoms, inputs()),
                Err(Error::Type(_))
            ));
        }
    }

    #[test]
    fn every_terminal_function_is_invalid_where_a_value_is_required() {
        // The guard reads the Function's own classification, so a terminal
        // spelling added by a later issue is nested-invalid the day it exists.
        for function in Function::ALL.iter().copied().filter(|f| f.is_terminal()) {
            let atoms = vec![
                Atom::Function(Function::Add),
                Atom::Function(function),
                Atom::Number(1),
            ]
            .into_iter()
            .collect();

            assert!(
                matches!(
                    Interpreter::execute(&atoms, inputs()),
                    Err(Error::Interpretation(
                        InterpretationError::NestedTerminalFunction
                    ))
                ),
                "{function:?}"
            );
        }
    }

    #[test]
    fn explicit_numeric_conversions_have_fixed_result_types() {
        for value in 0..=0x7F {
            assert_eq!(
                interpret_stack(vec![
                    Atom::Function(Function::ConvertToNumber),
                    Atom::Number(value),
                ])
                .unwrap(),
                Atom::Number(value)
            );
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
    fn conversions_are_idempotent_through_nested_source_expressions() {
        assert_eq!(interpret(".v.vC4".to_owned()), Atom::Number(60));
        assert_eq!(
            interpret(".^.^3C".to_owned()),
            Atom::Note(crate::Note::try_from(60).unwrap())
        );
    }

    #[test]
    fn conversion_source_literals_use_the_monomorphic_operand_type() {
        assert_eq!(interpret(".vA0".to_owned()), Atom::Number(21));

        let mut source = ".^C4".to_owned();
        let atoms = Parser::from(&mut source).try_parse().unwrap();
        assert!(matches!(
            Interpreter::execute(&atoms, inputs()),
            Err(Error::Interpretation(InterpretationError::NoteConversion(
                0xC4
            )))
        ));

        assert_eq!(interpret(".v.^3C".to_owned()), Atom::Number(60));
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
        assert_eq!(interpret(".|050A".to_owned()), Atom::Number(5));
        assert_eq!(interpret(".%0A03".to_owned()), Atom::Number(1));
        assert_eq!(interpret(".<0A03".to_owned()), Atom::Number(3));
        assert_eq!(interpret(".>0A03".to_owned()), Atom::Number(10));
        assert_eq!(interpret(".=0A0A".to_owned()), Atom::Bang);

        // Nested operands resolve before the outer Function sees them
        assert_eq!(interpret(".<.+0102.%0A03".to_owned()), Atom::Number(1));
        assert_eq!(interpret(".|.>0A03.<0A03".to_owned()), Atom::Number(7));
    }

    #[test]
    fn equality_composes_with_nested_arithmetic_on_both_answers() {
        // The Bang answer stands where a value stands, and the absent answer is
        // absent everywhere: nesting it as an operand diagnoses rather than
        // silently reading as a Number.
        assert_eq!(interpret(".=.+010203".to_owned()), Atom::Bang);
        assert_eq!(interpret(".=.+010204".to_owned()), Atom::Empty);

        let mut source = ".+.=010203".to_owned();
        let atoms = Parser::from(&mut source).try_parse().unwrap();
        assert!(matches!(
            Interpreter::execute(&atoms, inputs()),
            Err(Error::Type(TypeError::Number(found))) if found == "_"
        ));
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
}
