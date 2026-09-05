use crate::{
    Atom, Atoms, EXP_LEN, Error, Function, InterpretationError, PlayCommand, Sequence, Stack,
    TickInputs, Value,
    functions::{self, math, numeric_conversion},
};

/// The Operand Stack one Expression evaluates against.
///
/// `EXP_LEN` is chosen; this size is derived from it. No Atom raises the depth
/// by more than one — a literal Atom pushes one value, and a Function pushes
/// one value only after popping the operands its signature declares — so the
/// peak depth of a walk can never exceed the Atom count. A Function of one
/// operand or more therefore has a net change that is never positive, and a
/// nullary Function would raise the depth by one exactly as a literal does;
/// neither can raise it by more, which is why a Function of any arity, added
/// later, needs no new proof here.
///
/// The Atom count is bounded by the type rather than by the caller. `Atoms` is
/// an `ArrayVec<Atom, EXP_LEN>`, so `Expression` cannot record more than that
/// and `Parser` diagnoses the attempt as `SyntaxError::ExpressionTooLong`; and
/// because [`Interpreter::execute`] takes `&Atoms`, a caller who assembles
/// Atoms without going through the parser at all is held to the same bound.
pub type Args = Stack<EXP_LEN>;

pub struct Interpreter {}

#[derive(Clone, Debug, PartialEq)]
pub enum Interpretation {
    Cell(Atom),
    /// A Sequence value leaving evaluation intact.
    ///
    /// The Atomic Functions broadcast over a Sequence operand and answer one,
    /// but no Source-parseable Function produces the operand yet: the
    /// structural Sequence Functions arrive with issue 03 and the Range
    /// Functions with issue 05, so nothing reaches this variant from Source
    /// text today. It exists now because the whole point of the Sequence value
    /// is that it can cross Function evaluation and leave it without first
    /// becoming Source writes; adding it later would mean the consumer had
    /// already been written as though it could not.
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
                    Function::RawPlay => {
                        return Ok(Interpretation::Play(functions::raw_play(&mut ctx)?));
                    }
                    Function::TimedPlay => {
                        return Ok(Interpretation::Play(functions::timed_play(&mut ctx)?));
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

    /// The Source spelling of one Operand Literal of the type `token` names.
    fn literal(token: Token) -> &'static str {
        match token {
            Token::Number => "01",
            Token::Note => "C4",
            other => panic!("no operand is declared as {other:?}"),
        }
    }

    /// The peak Operand Stack depth a complete walk of `atoms` reaches.
    ///
    /// This models the machine rather than measuring it, because the depth a
    /// walk reaches is not something the Evaluator reports. The model is ADR
    /// 0028's rule restated once, where a test can read it: the walk runs last
    /// Atom to first, a literal pushes one value, and a Function pops the
    /// operands its signature declares and pushes one. Where the Evaluator
    /// would stop early with a diagnostic the model keeps walking, so its
    /// answer is an upper bound on what such an Expression actually reached.
    /// The shapes whose depth is asserted below reach their peak while the
    /// literals are still being pushed, before any Function has run, so for
    /// those the model and the machine agree exactly.
    pub(super) fn peak_depth(atoms: &crate::Atoms) -> usize {
        let mut depth: usize = 0;
        let mut peak: usize = 0;

        for atom in atoms.iter().rev() {
            if let Atom::Function(function) = atom {
                let arity = function.signature().len();
                if depth < arity {
                    // Too few operands: the Evaluator diagnoses here and the
                    // walk has already peaked.
                    break;
                }
                depth -= arity;
            }
            depth += 1;
            peak = peak.max(depth);
        }

        peak
    }

    #[test]
    fn every_chain_the_parser_accepts_evaluates_without_exhausting_the_operand_stack() {
        // The reproduction above is one point on this boundary; this is the
        // whole of it. A left-leaning chain is the shape that grows the Operand
        // Stack, because prefix order puts every Function ahead of every
        // operand and the walk therefore pushes all of a chain's literals
        // before its innermost Function consumes one. Every chain length is
        // enumerated, under every root and over every binary Value Function the
        // chain can be built from, so nothing here names a spelling and a
        // Function respelled or added later is covered by its own definition.
        let binary: Vec<Function> = Function::ALL
            .iter()
            .copied()
            .filter(|function| !function.is_terminal() && function.signature().len() == 2)
            .collect();

        // The deepest walk `EXP_LEN` Atoms admit, derived rather than counted.
        // A root of arity `a` over a chain of `k` binary Functions is
        // `2k + a + 1` Atoms and peaks at `a + k`, so the longest chain the
        // bound admits is `k = (EXP_LEN - a - 1) / 2` and the peak that follows
        // from it is `(EXP_LEN + a - 1) / 2`. The widest root wins, which is
        // why the arity is read from the definitions rather than written here.
        let widest = Function::ALL
            .iter()
            .map(|function| function.signature().len())
            .max()
            .expect("the definitions declare at least one Function");
        let deepest_walk_admitted = (EXP_LEN + widest - 1) / 2;

        let mut reached_the_parser_bound = false;
        let mut deepest_walk_reached = 0;

        for root in Function::ALL.iter().copied() {
            for link in binary.iter().copied() {
                for chain in 0..EXP_LEN {
                    // The chain stands in the first operand, which the
                    // right-to-left walk reaches last and so with the most
                    // already on the stack.
                    let mut source = root.spelling().to_owned();
                    source.push_str(&link.spelling().repeat(chain));
                    source.push_str(&literal(Token::Number).repeat(chain + 1));
                    for token in root.signature().iter().skip(1) {
                        source.push_str(literal(*token));
                    }

                    let Ok(atoms) = Parser::from(&mut source).try_parse() else {
                        // Refusing an over-long Expression is the parser's half
                        // of the bound, and an acceptable outcome here.
                        continue;
                    };

                    reached_the_parser_bound |= atoms.len() == EXP_LEN;
                    deepest_walk_reached = deepest_walk_reached.max(peak_depth(&atoms));

                    // Any diagnostic but one is an acceptable answer: an
                    // operand may be mistyped or out of its domain. Exhaustion
                    // is the one the bound rules out, and a panic fails the
                    // test outright.
                    assert!(
                        !matches!(
                            Interpreter::execute(&atoms, inputs()),
                            Err(Error::Interpretation(
                                InterpretationError::OperandStackExhausted { .. }
                            ))
                        ),
                        "{root:?} over a chain of {chain} {link:?} exhausted the Operand Stack",
                    );
                }
            }
        }

        assert!(
            reached_the_parser_bound,
            "no enumerated chain reached the parser's own bound, so nothing here tested it",
        );

        // Atom count alone is too weak a guard. Several roots reach `EXP_LEN`
        // Atoms at a depth any smaller stack would still have held, so without
        // this the enumeration could lose the one case that discriminates and
        // keep passing. Pinning the depth to the deepest the definitions and
        // `EXP_LEN` jointly admit is what makes losing it a failure.
        assert_eq!(
            deepest_walk_reached, deepest_walk_admitted,
            "the enumeration reached a depth of {deepest_walk_reached}, not the \
             {deepest_walk_admitted} the definitions and EXP_LEN admit",
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
/// The enumeration in `mod test` and this module divide the boundary between
/// them. That one walks a single shape — a left-leaning chain under a root —
/// through every length there is, and pins the deepest walk `EXP_LEN` admits.
/// This one generates whole Expressions of any shape from the Function
/// definitions, which is the coverage an enumeration of one shape cannot give.
/// Both assertions below are what keep that division honest: a generator that
/// stopped producing Expressions the parser accepts, or stopped nesting them,
/// would fail rather than quietly test nothing.
///
/// What this property cannot do is catch the regression that prompted the
/// bound. Reaching a depth of seventeen needs one narrow shape — an
/// arity-three root whose first operand is a chain of exactly fourteen and
/// whose other two operands are literals — which is on the order of one draw
/// in forty thousand here, so reverting `Args` to `Stack<16>` fails the two
/// deterministic tests above and not this one. Read a pass here as evidence
/// about the breadth of shape, never about the tight boundary; that half of
/// the division belongs to the enumeration, and weakening it is not something
/// this property would report.
///
/// The `cfg` matches the `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]`
/// table that declares proptest, so a WASM build never sees the dependency.
///
#[cfg(all(test, not(target_arch = "wasm32")))]
mod property {
    use super::test::peak_depth;
    use crate::{
        Anchor, EXP_LEN, Error, Function, InterpretationError, Interpreter, Parser, Tick,
        TickInputs, Token, midi_number_to_note,
    };
    use proptest::collection::vec;
    use proptest::prelude::*;
    use proptest::sample::select;
    use proptest::test_runner::{Config, TestRunner};
    use std::cell::Cell;

    /// How far a generated Expression nests before its operands must be
    /// literals. Three levels of the widest signature already outruns
    /// `EXP_LEN`, so this is where shapes stop growing rather than a claim
    /// about the language.
    const NESTING: u32 = 3;

    /// One Function applied to Source text for each of its operands.
    fn apply(function: Function, operands: &[String]) -> String {
        let mut source = function.spelling().to_owned();
        source.extend(operands.iter().map(String::as_str));
        source
    }

    /// Source text for one Operand Literal of the type its position declares.
    ///
    /// Reading the `Token` is what keeps a generated Expression parseable. A
    /// Note in a Number position and a Number in a Note position are both
    /// refused as Source text rather than diagnosed as an operand, so a
    /// generator that ignored the declaration would spend much of its budget
    /// on cases the Evaluator never sees.
    fn literal_source(token: Token) -> BoxedStrategy<String> {
        match token {
            Token::Number => any::<u8>()
                .prop_map(|number| format!("{number:02X}"))
                .boxed(),
            Token::Note => (0x00u8..=0x7F)
                .prop_map(|note| midi_number_to_note(note).expect("a MIDI Note"))
                .boxed(),
            other => panic!("no operand is declared as {other:?}"),
        }
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

    /// A left-leaning chain of binary Value Functions.
    ///
    /// This shape is generated deliberately rather than left to the nesting
    /// below, because it is the one that grows the Operand Stack: prefix order
    /// puts every Function ahead of every operand, so the walk pushes all of a
    /// chain's literals before its innermost Function consumes one.
    ///
    /// Most chains are short, and a minority run the whole length `EXP_LEN`
    /// could admit. The mass has to straddle the bound rather than sit past it:
    /// a chain drawn uniformly from the whole range averages more Atoms than an
    /// Expression may hold before anything is built around it, so nearly every
    /// case would be refused as Source text and the property would test the
    /// parser's refusal instead of the Evaluator's answer.
    fn chain_source() -> BoxedStrategy<String> {
        prop_oneof![4 => 0usize..4, 1 => 0usize..EXP_LEN]
            .prop_flat_map(|length| {
                (
                    vec(select(binary_value_functions()), length),
                    vec(literal_source(Token::Number), length + 1),
                )
            })
            .prop_map(|(functions, literals)| {
                let mut source: String = functions.iter().map(|f| f.spelling()).collect();
                source.extend(literals.iter().map(String::as_str));
                source
            })
            .boxed()
    }

    /// Source text for one operand of the declared type: a literal, a chain, or
    /// a Value Function over operands generated the same way.
    fn operand_source(token: Token, depth: u32) -> BoxedStrategy<String> {
        if depth == 0 {
            return literal_source(token);
        }

        prop_oneof![
            5 => literal_source(token),
            2 => chain_source(),
            3 => nested_source(depth),
        ]
        .boxed()
    }

    /// A Value Function over operands of the types its signature declares.
    fn nested_source(depth: u32) -> BoxedStrategy<String> {
        select(value_functions())
            .prop_flat_map(move |function| {
                let operands: Vec<BoxedStrategy<String>> = function
                    .signature()
                    .iter()
                    .map(|token| operand_source(*token, depth - 1))
                    .collect();
                (Just(function), operands)
            })
            .prop_map(|(function, operands)| apply(function, &operands))
            .boxed()
    }

    /// Source text for one whole Expression. The root is any Function at all,
    /// including the terminal one, whose three operands make it the widest root
    /// a Source can write and so the deepest walk one can ask for.
    fn expression_source() -> BoxedStrategy<String> {
        select(Function::ALL)
            .prop_flat_map(|function| {
                let operands: Vec<BoxedStrategy<String>> = function
                    .signature()
                    .iter()
                    .map(|token| operand_source(*token, NESTING))
                    .collect();
                (Just(function), operands)
            })
            .prop_map(|(function, operands)| apply(function, &operands))
            .boxed()
    }

    ///
    /// The Tick inputs for a property about Atoms rather than about time or
    /// Position: the first Tick of a Playback run, at the Grid origin.
    ///
    fn inputs() -> TickInputs {
        TickInputs::new(Tick::ZERO, Anchor::new(0, 0))
    }

    ///
    /// Evaluation is total over the Expressions strict parsing accepts, and
    /// never exhausts the Operand Stack. A panic fails a case outright, which
    /// is what a 64-Cell Expression used to produce; a mistyped or
    /// out-of-domain operand is an acceptable answer, and exhaustion is the one
    /// diagnostic the bound rules out.
    ///
    /// The runner is driven directly rather than through `proptest!` so that
    /// what the cases reached can be counted across them and asserted at the
    /// end. A property that generates only Expressions the parser refuses
    /// passes while testing nothing, and that is the failure the two counts
    /// below exist to catch.
    ///
    #[test]
    fn evaluating_every_expression_the_parser_accepts_returns_rather_than_panicking() {
        let config = Config::default();
        let cases = config.cases as usize;
        let evaluated = Cell::new(0usize);
        let deepest_walk = Cell::new(0usize);

        TestRunner::new(config)
            .run(&expression_source(), |source| {
                let mut source = source;
                let Ok(atoms) = Parser::from(&mut source).try_parse() else {
                    // An over-long Expression is the parser's to refuse, and
                    // its refusal is what the Operand Stack's bound rests on.
                    return Ok(());
                };

                evaluated.set(evaluated.get() + 1);
                deepest_walk.set(deepest_walk.get().max(peak_depth(&atoms)));

                let exhausted = matches!(
                    Interpreter::execute(&atoms, inputs()),
                    Err(Error::Interpretation(
                        InterpretationError::OperandStackExhausted { .. }
                    ))
                );

                prop_assert!(!exhausted, "{source:?} exhausted the Operand Stack");
                Ok(())
            })
            .unwrap_or_else(|error| panic!("{error}"));

        // Most of what is generated must reach the Evaluator. The generator
        // deliberately produces Expressions past the parser's bound, so some
        // refusals are the point; a majority of them would mean the property
        // was testing the parser rather than the machine.
        assert!(
            evaluated.get() * 2 > cases,
            "only {} of {cases} generated Expressions reached the Evaluator",
            evaluated.get(),
        );

        // And what reaches it must be nested rather than flat. One Function
        // over its own operands peaks at its arity, so a depth past the widest
        // signature is the shallowest evidence that operands are themselves
        // Expressions here.
        let widest = Function::ALL
            .iter()
            .map(|function| function.signature().len())
            .max()
            .expect("the definitions declare at least one Function");

        assert!(
            deepest_walk.get() > widest,
            "the deepest generated walk reached {}, which no signature of {widest} operands \
             had to nest to produce",
            deepest_walk.get(),
        );
    }
}
