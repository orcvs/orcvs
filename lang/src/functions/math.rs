use crate::{
    Atom, Error, InterpretationError, Value,
    atom::operands::{
        AbsoluteDifference, Add, Divide, Equality, Maximum, Minimum, Modulo, Multiply, Subtract,
    },
    interpreter::Context,
};

// Every Function here is declared Pervasive in `define_functions!`, so each
// body states what its operation is for one element and says nothing about
// Sequences: `Stack::apply` decides the one shape the operands make, hands out
// the operands for each element, and assembles the answers, and
// `Stack::predicate` does the same for the one Function that answers about all
// of them at once. A body that mapped over a Sequence itself would be a second
// broadcast mechanism, free to disagree with the first about lengths, about
// ordering, and about what a partial failure leaves behind.
//
// The operand struct is named twice in each body — once as the pattern that
// binds the roles, once as the type that tells the compiler which Function the
// closure is for — because a struct pattern alone does not resolve the generic
// `Stack::apply` is called at. Naming it is the price of the roles; indexing an
// operand slice would drop the annotation and the role names together.

/// Absolute Difference: `.| left right`.
///
/// Ordered Subtraction wraps modulo 256, so it answers a cycle position rather
/// than a distance. This Function is the distance, which is why it is separate
/// from `.-` rather than a spelling of it: `abs_diff` is symmetric and has no
/// borrow to wrap.
#[inline(always)]
pub fn absolute_difference(ctx: &mut Context) -> Result<Value, Error> {
    ctx.stack
        .apply(|AbsoluteDifference { left, right }: AbsoluteDifference| {
            Ok(Atom::Number(left.abs_diff(right)))
        })
}

#[inline(always)]
pub fn add(ctx: &mut Context) -> Result<Value, Error> {
    ctx.stack
        .apply(|Add { left, right }: Add| Ok(Atom::Number(left.wrapping_add(right))))
}

#[inline(always)]
pub fn divide(ctx: &mut Context) -> Result<Value, Error> {
    ctx.stack
        .apply(|Divide { left, right }: Divide| match right {
            0 => Err(InterpretationError::DivisionByZero.into()),
            right => Ok(Atom::Number(left / right)),
        })
}

/// Equality: `.= left right`.
///
/// A whole-value predicate that answers a pulse rather than a truth value:
/// equal operands produce one Bang, and unequal operands produce `Atom::Empty`,
/// which is already the Interpreter's "no result write" signal. Answering a
/// Number for the unequal case would put a Cell meaning "false" into the
/// Source, where the next Tick would read it as an ordinary operand.
///
/// ADR 0011 keeps that true of a comparison over Sequences: this uses ordinary
/// pervasive extension to find its pairs and still answers exactly one Atom
/// about all of them, so it goes through `predicate` rather than `apply`. A map
/// would have to write an absent element where a pair disagreed, and Sequence
/// has no such member; the aggregations that do want positions are deferred to
/// Functions of their own.
#[inline(always)]
pub fn equality(ctx: &mut Context) -> Result<Value, Error> {
    ctx.stack
        .predicate(|Equality { left, right }: Equality| left == right)
}

/// Maximum: `.> left right`.
#[inline(always)]
pub fn maximum(ctx: &mut Context) -> Result<Value, Error> {
    ctx.stack
        .apply(|Maximum { left, right }: Maximum| Ok(Atom::Number(left.max(right))))
}

/// Minimum: `.< left right`.
#[inline(always)]
pub fn minimum(ctx: &mut Context) -> Result<Value, Error> {
    ctx.stack
        .apply(|Minimum { left, right }: Minimum| Ok(Atom::Number(left.min(right))))
}

/// Modulo: `.% left right`.
///
/// A zero divisor has no remainder to name, so this diagnoses and produces no
/// Atom rather than inventing one, exactly as Division does. The diagnostic is
/// its own so the Source learns which Function it wrote.
#[inline(always)]
pub fn modulo(ctx: &mut Context) -> Result<Value, Error> {
    ctx.stack
        .apply(|Modulo { left, right }: Modulo| match right {
            0 => Err(InterpretationError::ModuloByZero.into()),
            right => Ok(Atom::Number(left % right)),
        })
}

#[inline(always)]
pub fn multiply(ctx: &mut Context) -> Result<Value, Error> {
    ctx.stack
        .apply(|Multiply { left, right }: Multiply| Ok(Atom::Number(left.wrapping_mul(right))))
}

#[inline(always)]
pub fn subtract(ctx: &mut Context) -> Result<Value, Error> {
    ctx.stack
        .apply(|Subtract { left, right }: Subtract| Ok(Atom::Number(left.wrapping_sub(right))))
}

#[cfg(test)]
mod test {
    use super::{add, divide, equality, modulo, subtract};
    use crate::{
        Anchor, Atom, Error, InterpretationError, Sequence, SequenceError, Tick, TickInputs, Value,
        interpreter::Context,
    };

    type Arithmetic = fn(&mut Context) -> Result<Value, Error>;

    /// What a Function should answer for one operand pair, read in signature
    /// order, naming the diagnostic where the pair has no answer.
    type Reference = fn(u8, u8) -> Result<Atom, InterpretationError>;

    /// Evaluates `function` against the two operands its signature names
    /// `left` and `right`, pushed so that extraction pops them in signature
    /// order.
    fn evaluate(function: Arithmetic, left: u8, right: u8) -> Result<Value, Error> {
        // Arithmetic reads no Tick and no Position, so the first Tick at the
        // Grid origin is as good as any other.
        let mut ctx = Context::new(TickInputs::new(Tick::ZERO, Anchor::new(0, 0)));
        ctx.stack.push(Atom::Number(right)).unwrap();
        ctx.stack.push(Atom::Number(left)).unwrap();
        function(&mut ctx)
    }

    /// Evaluates `function` against two whole language values, pushed so that
    /// the broadcast pops them in signature order.
    fn evaluate_values(
        function: Arithmetic,
        left: impl Into<Value>,
        right: impl Into<Value>,
    ) -> Result<Value, Error> {
        let mut ctx = Context::new(TickInputs::new(Tick::ZERO, Anchor::new(0, 0)));
        ctx.stack.push(right.into()).unwrap();
        ctx.stack.push(left.into()).unwrap();
        function(&mut ctx)
    }

    fn numbers(values: impl IntoIterator<Item = u8>) -> Sequence {
        Sequence::new(values.into_iter().map(Atom::Number)).unwrap()
    }

    #[test]
    fn an_arithmetic_function_body_states_one_element_and_still_broadcasts() {
        // The stack seam is tested where it lives; this is the claim that the
        // Functions the Source can write are actually wired to it, which a test
        // of `Stack::apply` alone cannot make.
        assert_eq!(
            evaluate_values(add, Atom::Number(0x10), numbers([1, 2, 3])).unwrap(),
            Value::Sequence(numbers([0x11, 0x12, 0x13]))
        );

        // And that an evaluation fault at an element other than the first still
        // discards the elements that answered.
        assert!(matches!(
            evaluate_values(divide, Atom::Number(0x10), numbers([1, 1, 0])),
            Err(Error::Interpretation(InterpretationError::DivisionByZero))
        ));
    }

    #[test]
    fn equality_answers_one_bang_only_when_every_broadcast_pair_is_equal() {
        // ADR 0011 makes `.=` a whole-value predicate: it uses ordinary
        // broadcasting to find its pairs and then answers one Atom about all of
        // them. Each case below has a shape an element-wise Function would
        // answer a Sequence for, so a map written by accident fails here rather
        // than only where the Source encodes it.
        for (left, right) in [
            (Value::from(Atom::Number(1)), Value::from(Atom::Number(1))),
            (Atom::Number(1).into(), numbers([1, 1, 1]).into()),
            (numbers([1, 1, 1]).into(), Atom::Number(1).into()),
            (numbers([1, 2, 3]).into(), numbers([1, 2, 3]).into()),
        ] {
            assert_eq!(
                evaluate_values(equality, left.clone(), right.clone()).unwrap(),
                Value::Atom(Atom::Bang),
                "{left:?} against {right:?}"
            );
        }

        // One unequal pair is enough, wherever it stands, and the answer is the
        // absence marker rather than a Sequence with a hole in it.
        for (left, right) in [
            (Value::from(Atom::Number(1)), Value::from(Atom::Number(2))),
            (Atom::Number(1).into(), numbers([1, 1, 2]).into()),
            (numbers([2, 1, 1]).into(), Atom::Number(1).into()),
            (numbers([1, 2, 3]).into(), numbers([1, 2, 4]).into()),
        ] {
            assert_eq!(
                evaluate_values(equality, left.clone(), right.clone()).unwrap(),
                Value::Atom(Atom::Empty),
                "{left:?} against {right:?}"
            );
        }
    }

    #[test]
    fn a_comparison_with_no_pairs_is_vacuously_all_equal() {
        // An empty Sequence operand makes an operation of no elements, and a
        // predicate over nothing holds. The distinction matters because the
        // same shape makes an arithmetic Function answer the empty Sequence:
        // `.=` answers about the comparison, not about the operand.
        for (left, right) in [
            (
                Value::from(Sequence::empty()),
                Value::from(Sequence::empty()),
            ),
            (Atom::Number(1).into(), Sequence::empty().into()),
            (Sequence::empty().into(), Atom::Number(1).into()),
        ] {
            assert_eq!(
                evaluate_values(equality, left.clone(), right.clone()).unwrap(),
                Value::Atom(Atom::Bang),
                "{left:?} against {right:?}"
            );
        }
    }

    #[test]
    fn equality_diagnoses_two_non_scalar_operands_of_different_lengths() {
        // Including empty against non-empty, which is a length disagreement
        // rather than the vacuous case above: an empty Sequence repeats across
        // nothing, so there is no pairing to be vacuous about.
        for (left, right, lengths) in [
            (
                Value::from(numbers([1, 2])),
                Value::from(numbers([1, 2, 3])),
                (2, 3),
            ),
            (Sequence::empty().into(), numbers([1, 2]).into(), (0, 2)),
            (numbers([1, 2]).into(), Sequence::empty().into(), (2, 0)),
        ] {
            assert!(
                matches!(
                    evaluate_values(equality, left.clone(), right.clone()),
                    Err(Error::Sequence(SequenceError::IncompatibleLengths { left: l, right: r }))
                        if (l, r) == lengths
                ),
                "{left:?} against {right:?}"
            );
        }
    }

    #[test]
    fn the_non_commutative_functions_read_left_and_right_in_signature_order() {
        // Both operands share the Number domain, so neither `Stack::extract`
        // nor the compiler can tell the two roles apart; only the answer can.
        // The whole byte square is enumerated against a reference that names
        // `left` and `right` explicitly, so a transposition inside the
        // declaration or the body changes the answer for every asymmetric pair
        // rather than for a sampled few. The reference names which diagnostic
        // a zero divisor answers, so Modulo cannot pass by raising Division's.
        // These three are every non-commutative arithmetic Function there is:
        // the other six answer the same for either operand order, so no test
        // of theirs can observe a transposition, and only the role names in
        // the declaration say which Cell is which.
        let cases: [(Arithmetic, Reference); 3] = [
            (subtract, |left, right| {
                Ok(Atom::Number(left.wrapping_sub(right)))
            }),
            (divide, |left, right| match right {
                0 => Err(InterpretationError::DivisionByZero),
                right => Ok(Atom::Number(left / right)),
            }),
            (modulo, |left, right| match right {
                0 => Err(InterpretationError::ModuloByZero),
                right => Ok(Atom::Number(left % right)),
            }),
        ];

        for (function, reference) in cases {
            for left in 0..=u8::MAX {
                for right in 0..=u8::MAX {
                    match (evaluate(function, left, right), reference(left, right)) {
                        (Ok(answer), Ok(expected)) => {
                            assert_eq!(answer, expected.into(), "{left:02X} {right:02X}");
                        }
                        // `InterpretationError` derives no `PartialEq`, and the
                        // wording is what the Source is shown, so the rendered
                        // diagnostic is the thing worth comparing.
                        (Err(answer), Err(expected)) => {
                            assert_eq!(
                                answer.to_string(),
                                expected.to_string(),
                                "{left:02X} {right:02X}"
                            );
                        }
                        (answer, expected) => {
                            panic!("{left:02X} {right:02X}: {answer:?} is not {expected:?}")
                        }
                    }
                }
            }
        }
    }
}
