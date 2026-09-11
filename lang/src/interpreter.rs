use crate::{
    Atom, Error, Function, InterpretationError, Performance, Sequence, SourceEffect, Stack,
    TickInputs, Value,
    functions::{self, math, numeric_conversion, tick},
};

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
/// A Function reaching for the Tick or its anchor takes `&mut Context` exactly
/// as an arithmetic Function does today, so a Tick-reading Function is a new
/// arm in `execute` rather than a new evaluation path.
///
pub struct Context {
    pub stack: Stack,
    /// The explicit inputs ADR 0012 supplies alongside the Source Snapshot.
    ///
    /// The seam has consumers now: Clock, Delay, and Euclidean each read the
    /// Tick from here, which is what makes an absolute Tick an input to
    /// interpretation rather than something a Function goes looking for. The
    /// anchor is still read by nothing — ADR 0013's Random is the Function it
    /// is there for — but it travels in the same struct, so it arrives the day
    /// that Function is declared rather than needing the threading rebuilt.
    pub inputs: TickInputs,
}

impl Context {
    pub fn new(inputs: TickInputs, stack_limit: usize) -> Self {
        Self {
            stack: Stack::new(stack_limit),
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
    pub fn execute(atoms: &[Atom], inputs: TickInputs) -> Result<Interpretation, Error> {
        // No Atom raises the stack depth by more than one: literals push one
        // value, and Functions pop their operands before producing one value.
        // The actual Atom count therefore bounds this Expression's peak depth.
        Self::execute_context(atoms, Context::new(inputs, atoms.len()))
    }

    /// Evaluates one Function with already resolved, typed inputs. Literal
    /// decoding and nested ownership belong to the caller; evaluation retains
    /// the same type, domain, absence and Sequence rules as `execute`.
    ///
    /// ```
    /// use lang::{Anchor, Atom, Function, Interpretation, Interpreter, Sequence, Tick, TickInputs, Value};
    /// let sequence = Sequence::new([Atom::Number(5), Atom::Number(9)]).unwrap();
    /// let answer = Interpreter::execute_function(
    ///     Function::Subtract, &[Value::Sequence(sequence), Value::Atom(Atom::Number(2))],
    ///     TickInputs::new(Tick::ZERO, Anchor::new(0, 0)),
    /// ).unwrap();
    /// assert_eq!(answer, Interpretation::Sequence(
    ///     Sequence::new([Atom::Number(3), Atom::Number(7)]).unwrap()));
    /// ```
    pub fn execute_function(
        function: Function,
        operands: &[Value],
        inputs: TickInputs,
    ) -> Result<Interpretation, Error> {
        let expected = function.signature().len();
        if operands.len() != expected {
            return Err(crate::ArgumentError::Arity {
                expected,
                found: operands.len(),
            }
            .into());
        }
        let mut ctx = Context::new(inputs, operands.len() + 1);
        for operand in operands.iter().rev() {
            ctx.stack.push(operand.clone())?;
        }
        Self::execute_context(&[Atom::Function(function)], ctx)
    }

    fn execute_context(atoms: &[Atom], mut ctx: Context) -> Result<Interpretation, Error> {
        for (index, atom) in atoms.iter().enumerate().rev() {
            // info!("atoms: {:?}", atoms);
            // info!("stack: {:?}", stack);
            // Every Function answers a language Value, so a Function that returns
            // a Sequence needs an arm here and nothing else: the push below already
            // carries whichever shape the Value holds.
            let value = match atom {
                // A Function that answers an effect rather than a value can
                // stand in only one place: the one place nothing consumes an
                // answer, which is the Expression root the Interpreter reaches
                // last. The guard asks the Function's declared kind and not
                // which effect it performs, so the Source-writing Functions of
                // ADR 0004 are nested-invalid the day they are declared.
                // Rejecting every other index here leaves each effect arm below
                // free to assume it is the root.
                Atom::Function(fun) if !fun.answers_value() && index != 0 => {
                    return Err(InterpretationError::NestedEffectFunction.into());
                }
                Atom::Function(fun) => match fun {
                    Function::AbsoluteDifference => math::absolute_difference(&mut ctx)?,
                    Function::Add => math::add(&mut ctx)?,
                    Function::Clock => tick::clock(&mut ctx)?,
                    Function::ConvertToNote => numeric_conversion::to_note(&mut ctx)?,
                    Function::ConvertToNumber => numeric_conversion::to_number(&mut ctx)?,
                    Function::Delay => tick::delay(&mut ctx)?,
                    Function::Divide => math::divide(&mut ctx)?,
                    Function::Equality => math::equality(&mut ctx)?,
                    Function::Euclidean => tick::euclidean(&mut ctx)?,
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
                    // The Source-writing Functions take no operand and read no
                    // Context: the whole of the effect is declared in the
                    // table, so the arm reads the declaration rather than
                    // repeating eight offsets and two bundles here. A Function
                    // whose kind carries no Source write cannot reach this arm,
                    // which is what `expect` states.
                    //
                    // One arm for both groups, and it is not a case the two
                    // share by coincidence: they differ in what they declare
                    // and not in what interpreting them does, which is read the
                    // declaration. ADR 0029's asymmetry lives in the activation
                    // column and the bundle, and both are settled before this.
                    Function::DirectionalBangEast
                    | Function::DirectionalBangNorth
                    | Function::DirectionalBangSouth
                    | Function::DirectionalBangWest
                    | Function::SelfBangingEast
                    | Function::SelfBangingNorth
                    | Function::SelfBangingSouth
                    | Function::SelfBangingWest => {
                        return Ok(Interpretation::Source(
                            fun.source_effect()
                                .expect("a Source-writing Function declares a Source write"),
                        ));
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
        Anchor, ArgumentError, Atom, Error, Function, Interpretation, InterpretationError,
        MidiChannel, Note, Parser, Performance, PlayCommand, Tick, TickInputs, Token, TypeError,
        Velocity, interpreter::Interpreter, trace,
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
        Interpreter::execute(&exp, inputs()).map(|result| match result {
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
            assert!(matches!(
                Interpreter::execute(&atoms, inputs()),
                Err(Error::Type(_))
            ));
        }
    }

    #[test]
    fn every_effect_function_is_invalid_where_a_value_is_required() {
        // The guard reads the Function's own classification and not which
        // effect it performs, so a Function declared with any effect kind by a
        // later issue is nested-invalid the day it exists.
        for function in Function::ALL.iter().copied().filter(|f| !f.answers_value()) {
            let atoms = [
                Atom::Function(Function::Add),
                Atom::Function(function),
                Atom::Number(1),
            ];

            assert!(
                matches!(
                    Interpreter::execute(&atoms, inputs()),
                    Err(Error::Interpretation(
                        InterpretationError::NestedEffectFunction
                    ))
                ),
                "{function:?}"
            );
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
    fn a_long_addition_chain_evaluates_all_of_its_operands() {
        let mut source = format!("{}{}", ".+".repeat(64), "01".repeat(65));
        let atoms = Parser::from(&mut source).try_parse().unwrap();

        assert_eq!(
            Interpreter::execute(&atoms, inputs()).unwrap(),
            Interpretation::Cell(Atom::Number(65)),
        );
    }

    #[test]
    fn a_play_expression_with_seventeen_pending_values_evaluates() {
        // The sixteen Number literals stand above the Note before the first
        // addition consumes any, reproducing the former sixteen-slot panic.
        let mut source =
            "!>.+.+.+.+.+.+.+.+.+.+.+.+.+.+01010101010101010101010101010101C4".to_owned();
        let atoms = Parser::from(&mut source).try_parse().unwrap();

        // The chain sums fifteen of the sixteen Operand Literals into the
        // channel, leaving the sixteenth as the velocity.
        assert_eq!(
            Interpreter::execute(&atoms, inputs()).unwrap(),
            Interpretation::Play(Performance::One(PlayCommand::Raw {
                channel: MidiChannel::try_from(0x0F).unwrap(),
                velocity: Velocity::try_from(0x01).unwrap(),
                note: Note::try_from(60).unwrap(),
            }))
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
    pub(super) fn peak_depth(atoms: &[Atom]) -> usize {
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
    fn nested_chains_under_every_root_do_not_exhaust_the_operand_stack() {
        // A test budget, not a language limit. Chains put every Function ahead
        // of its literals, making the reverse walk hold them all at once.
        const CHAIN_LENGTH: usize = 64;
        let binary: Vec<Function> = Function::ALL
            .iter()
            .copied()
            .filter(|function| function.answers_value() && function.signature().len() == 2)
            .collect();

        let widest = Function::ALL
            .iter()
            .map(|function| function.signature().len())
            .max()
            .expect("the definitions declare at least one Function");
        let mut deepest_walk_reached = 0;

        // A root declaring no operand has no position for a chain to stand in,
        // and appending one spells trailing content the Parser rejects rather
        // than a deeper walk. The Self-Banging and Directional Bang Functions
        // are the rows this skips; every other row still carries the chain.
        for root in Function::ALL
            .iter()
            .copied()
            .filter(|root| !root.takes_no_operand())
        {
            for link in binary.iter().copied() {
                for chain in 1..=CHAIN_LENGTH {
                    // The chain stands in the first operand, which the
                    // right-to-left walk reaches last and so with the most
                    // already on the stack.
                    let mut source = root.spelling().to_owned();
                    source.push_str(&link.spelling().repeat(chain));
                    source.push_str(&literal(Token::Number).repeat(chain + 1));
                    for token in root.signature().iter().skip(1) {
                        source.push_str(literal(*token));
                    }

                    let atoms = Parser::from(&mut source).try_parse().unwrap();
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

        assert_eq!(deepest_walk_reached, CHAIN_LENGTH + widest);
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

/// Generated Expression shapes complement the deterministic deep chains:
/// every generated source parses, and its Atom count suffices for evaluation.
/// Type and domain errors are legitimate evaluation outcomes; stack exhaustion
/// is not.
///
/// The `cfg` matches the `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]`
/// table that declares proptest, so a WASM build never sees the dependency.
///
#[cfg(all(test, not(target_arch = "wasm32")))]
mod property {
    use super::test::peak_depth;
    use crate::{
        Anchor, Error, Function, InterpretationError, Interpreter, Parser, Tick, TickInputs, Token,
        midi_number_to_note,
    };
    use proptest::collection::vec;
    use proptest::prelude::*;
    use proptest::sample::select;
    use proptest::test_runner::{Config, TestRunner};
    use std::cell::Cell;

    /// Generation budgets only; neither constrains accepted Source.
    const NESTING: u32 = 3;
    const CHAIN_LENGTH: usize = 64;

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
            .filter(|function| function.answers_value())
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
    /// Mix short chains and longer ones within the test's generation budget.
    fn chain_source() -> BoxedStrategy<String> {
        // A zero-link chain is a Number literal, which would be invalid in a
        // Note slot. The separate literal strategy already respects slot types.
        prop_oneof![4 => 1usize..4, 1 => 1usize..=CHAIN_LENGTH]
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

    /// Source text for one whole Expression, including terminal roots.
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
    /// The direct runner also checks that the generated cases reach a depth
    /// requiring nested Expressions.
    ///
    #[test]
    fn evaluating_every_expression_the_parser_accepts_returns_rather_than_panicking() {
        // Naming the source file is what `proptest!` would have done. The
        // persistence layer derives a regression file's name from it, and
        // declines to write one when it is unset, so driving the runner
        // directly to count the cases means saying where this property lives.
        let config = Config {
            source_file: Some(file!()),
            ..Config::default()
        };
        let deepest_walk = Cell::new(0usize);

        TestRunner::new(config)
            .run(&expression_source(), |source| {
                let mut source = source;
                let parsed = Parser::from(&mut source).try_parse();
                prop_assert!(parsed.is_ok(), "{source:?} failed to parse: {parsed:?}");
                let atoms = parsed.unwrap();
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
