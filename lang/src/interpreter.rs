use crate::{
    Atom, Atoms, EXP_LEN, Error, Function, InterpretationError, PlayCommand, Sequence, Stack,
    TickInputs, Value,
    functions::{self, math, numeric_conversion},
};

/// The Operand Stack one Expression evaluates against, sized by the parser's
/// own bound.
///
/// `EXP_LEN` is derived rather than chosen: no Atom raises the depth by more
/// than one, so the peak depth of a walk can never exceed the Atom count, and
/// the parser already refuses an Expression of more than `EXP_LEN` Atoms with
/// `SyntaxError::ExpressionTooLong`. A literal Atom pushes one value. A
/// Function pushes one value only after popping the operands its signature
/// declares, so its net change is never positive for any declared arity, and
/// even a nullary Function would raise the depth by one like a literal. Nothing
/// an arity change can do invalidates that, which is why widening a signature
/// needs no new proof here.
pub type Args = Stack<EXP_LEN>;

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
            ctx.stack.push(value)?;
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
        Anchor, ArgumentError, Atom, EXP_LEN, Error, Function, Interpretation, InterpretationError,
        MidiChannel, Note, Parser, PlayCommand, Tick, TickInputs, Token, TypeError, Velocity,
        interpreter::Interpreter, trace,
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
    fn an_expression_at_the_parser_bound_evaluates_without_exhausting_the_operand_stack() {
        // The witness for the bound `Args` declares. These 64 Cells parse to
        // exactly `EXP_LEN` Atoms, and the sixteen Operand Literals stand above
        // the Note before the first `.+` consumes any of them, so the walk
        // reaches a depth of seventeen. A stack sized by an arity rather than
        // by `EXP_LEN` cannot hold that.
        let mut source =
            "!>.+.+.+.+.+.+.+.+.+.+.+.+.+.+01010101010101010101010101010101C4".to_owned();
        let atoms = Parser::from(&mut source).try_parse().unwrap();
        assert_eq!(atoms.len(), EXP_LEN);

        // The chain sums fifteen of the sixteen Operand Literals into the
        // channel, leaving the sixteenth as the velocity.
        assert_eq!(
            Interpreter::execute(&atoms, inputs()).unwrap(),
            Interpretation::Play(PlayCommand::Raw {
                channel: MidiChannel::try_from(0x0F).unwrap(),
                velocity: Velocity::try_from(0x01).unwrap(),
                note: Note::try_from(60).unwrap(),
            })
        );
    }

    #[test]
    fn every_chain_the_parser_accepts_evaluates_without_exhausting_the_operand_stack() {
        // The reproduction above is one point on this boundary; this is the
        // whole of it. A left-leaning chain is the shape that grows the Operand
        // Stack, because prefix order puts every Function ahead of every
        // operand and the walk therefore pushes all of a chain's literals
        // before its innermost Function consumes one. Every chain length is
        // enumerated under every root, so the widest root reaches the deepest
        // walk a Source can write, and a root Function added later is covered
        // by its own definition.
        fn literal(token: Token) -> &'static str {
            match token {
                Token::Number => "01",
                Token::Note => "C4",
                other => panic!("no operand is declared as {other:?}"),
            }
        }

        let mut reached_the_parser_bound = false;

        for function in Function::ALL.iter().copied() {
            for chain in 0..EXP_LEN {
                // The chain stands in the first operand, which the
                // right-to-left walk reaches last and so with the most already
                // on the stack.
                let mut source = function.spelling().to_owned();
                source.push_str(&".+".repeat(chain));
                source.push_str(&"01".repeat(chain + 1));
                for token in function.signature().iter().skip(1) {
                    source.push_str(literal(*token));
                }

                let Ok(atoms) = Parser::from(&mut source).try_parse() else {
                    // Refusing an over-long Expression is the parser's half of
                    // the bound, and an acceptable outcome here.
                    continue;
                };

                reached_the_parser_bound |= atoms.len() == EXP_LEN;

                // Any diagnostic but one is an acceptable answer: an operand
                // may be mistyped or out of its domain. Exhaustion is the one
                // the bound rules out, and a panic fails the test outright.
                assert!(
                    !matches!(
                        Interpreter::execute(&atoms, inputs()),
                        Err(Error::Interpretation(
                            InterpretationError::OperandStackExhausted { .. }
                        ))
                    ),
                    "{function:?} over a chain of {chain} exhausted the Operand Stack",
                );
            }
        }

        assert!(
            reached_the_parser_bound,
            "no enumerated chain reached the parser's own bound, so nothing here tested it",
        );
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

///
/// The parser/Evaluator boundary as a property: every Expression the parser
/// accepts is one the Evaluator answers, rather than one it panics on. The
/// bound `Args` declares is what makes that true of the Operand Stack, and a
/// property is what keeps it true of Expressions nobody wrote down.
///
/// This is breadth rather than depth. The tight boundary — the one Expression
/// shape that reaches the deepest walk `EXP_LEN` Atoms admit — is narrow enough
/// that sampling would miss it, so
/// `test::every_chain_the_parser_accepts_evaluates_without_exhausting_the_operand_stack`
/// enumerates it instead, and this covers the shapes an enumeration cannot.
///
/// The `cfg` matches the `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]`
/// table that declares proptest, so a WASM build never sees the dependency.
///
#[cfg(all(test, not(target_arch = "wasm32")))]
mod property {
    use crate::{
        Anchor, EXP_LEN, Error, Function, InterpretationError, Interpreter, Parser, Tick,
        TickInputs, midi_number_to_note,
    };
    use proptest::collection::vec;
    use proptest::prelude::*;
    use proptest::sample::select;

    /// One Function applied to Source text for each of its operands.
    fn apply(function: Function, operands: &[String]) -> String {
        let mut source = function.spelling().to_owned();
        source.extend(operands.iter().map(String::as_str));
        source
    }

    /// Source text for one Operand Literal, in either type a signature names.
    fn literal_source() -> impl Strategy<Value = String> {
        prop_oneof![
            any::<u8>().prop_map(|number| format!("{number:02X}")),
            (0x00u8..=0x7F).prop_map(|note| midi_number_to_note(note).expect("a MIDI Note")),
        ]
    }

    /// Every Value Function, read from the definitions rather than listed, so a
    /// Function added later is generated the day it exists.
    fn value_functions() -> Vec<Function> {
        Function::ALL
            .iter()
            .copied()
            .filter(|function| !function.is_terminal())
            .collect()
    }

    /// The binary Value Functions, which are the ones a chain is built from.
    fn binary_value_functions() -> Vec<Function> {
        value_functions()
            .into_iter()
            .filter(|function| function.signature().len() == 2)
            .collect()
    }

    /// A left-leaning chain of binary Value Functions: `.+.+.+ a b c d`.
    ///
    /// This shape is generated deliberately rather than left to the recursion
    /// below, because it is the one that grows the Operand Stack. Prefix order
    /// puts every Function ahead of every operand, so the right-to-left walk
    /// pushes all `k + 1` literals before the innermost Function consumes any
    /// of them. A balanced tree of the same Atom count never reaches a fraction
    /// of that depth, and a uniform recursion almost never produces a chain.
    ///
    /// The chain length runs past what `EXP_LEN` admits, because an Expression
    /// the parser refuses is half of the property: the stack stays in bounds by
    /// the parser's bound, so a generator that never reached it would never
    /// test the two together.
    fn chain_source() -> impl Strategy<Value = String> {
        vec(select(binary_value_functions()), 0..EXP_LEN)
            .prop_flat_map(|functions| {
                let literals = functions.len() + 1;
                (Just(functions), vec(literal_source(), literals))
            })
            .prop_map(|(functions, literals)| {
                let mut source: String = functions.iter().map(|f| f.spelling()).collect();
                source.extend(literals.iter().map(String::as_str));
                source
            })
    }

    /// Source text for one operand: a literal, a chain, or a Value Function
    /// applied to operands generated the same way.
    fn operand_source() -> impl Strategy<Value = String> {
        let leaf = prop_oneof![2 => literal_source(), 1 => chain_source()];

        leaf.prop_recursive(8, 48, 2, move |operand| {
            select(value_functions())
                .prop_flat_map(move |function| {
                    (
                        Just(function),
                        vec(operand.clone(), function.signature().len()),
                    )
                })
                .prop_map(|(function, operands)| apply(function, &operands))
        })
    }

    /// Source text for one whole Expression. The root is any Function at all,
    /// including the terminal one, whose three operands make it the widest
    /// root a Source can write and so the deepest walk one can ask for.
    fn expression_source() -> impl Strategy<Value = String> {
        select(Function::ALL).prop_flat_map(|function| {
            vec(operand_source(), function.signature().len())
                .prop_map(move |operands| apply(function, &operands))
        })
    }

    ///
    /// The Tick inputs for a property about Atoms rather than about time or
    /// Position: the first Tick of a Playback run, at the Grid origin.
    ///
    fn inputs() -> TickInputs {
        TickInputs::new(Tick::ZERO, Anchor::new(0, 0))
    }

    proptest! {
        ///
        /// Evaluation is total over the Expressions strict parsing accepts, and
        /// never exhausts the Operand Stack. A panic fails a case outright,
        /// which is what a 64-Cell Expression used to produce; a mistyped or
        /// out-of-domain operand is an acceptable answer, and exhaustion is the
        /// one diagnostic the bound rules out.
        ///
        #[test]
        fn evaluating_every_expression_the_parser_accepts_returns_rather_than_panicking(
            source in expression_source(),
        ) {
            let mut source = source;
            let Ok(atoms) = Parser::from(&mut source).try_parse() else {
                // An over-long Expression is the parser's to refuse, and its
                // refusal is what the Operand Stack's bound rests on.
                return Ok(());
            };

            let exhausted = matches!(
                Interpreter::execute(&atoms, inputs()),
                Err(Error::Interpretation(InterpretationError::OperandStackExhausted { .. }))
            );

            prop_assert!(!exhausted, "{source:?} exhausted the Operand Stack");
        }
    }
}
