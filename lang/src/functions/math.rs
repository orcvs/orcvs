use crate::{
    Atom, Error, InterpretationError,
    atom::operands::{
        AbsoluteDifference, Add, Divide, Equality, Maximum, Minimum, Modulo, Multiply, Subtract,
    },
    interpreter::Context,
};

// Each body states what its operation is for the operands `Stack::apply` or
// `Stack::predicate` binds, and nothing about checking them: every operand has
// passed its type check before a body runs.
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
pub fn absolute_difference(ctx: &mut Context) -> Result<Atom, Error> {
    ctx.stack
        .apply(|AbsoluteDifference { left, right }: AbsoluteDifference| {
            Ok(Atom::Number(left.abs_diff(right)))
        })
}

#[inline(always)]
pub fn add(ctx: &mut Context) -> Result<Atom, Error> {
    ctx.stack
        .apply(|Add { left, right }: Add| Ok(Atom::Number(left.wrapping_add(right))))
}

#[inline(always)]
pub fn divide(ctx: &mut Context) -> Result<Atom, Error> {
    ctx.stack
        .apply(|Divide { left, right }: Divide| match right {
            0 => Err(InterpretationError::DivisionByZero.into()),
            right => Ok(Atom::Number(left / right)),
        })
}

/// Equality: `.= left right`.
///
/// A predicate that answers a pulse rather than a truth value:
/// equal operands produce one Bang, and unequal operands produce `Atom::Empty`,
/// which is the Interpreter's "no result write" signal. Answering a
/// Number for the unequal case would put a Cell meaning "false" into the
/// Source, where the next Tick would read it as an ordinary operand.
#[inline(always)]
pub fn equality(ctx: &mut Context) -> Result<Atom, Error> {
    ctx.stack
        .predicate(|Equality { left, right }: Equality| left == right)
}

/// Maximum: `.> left right`.
#[inline(always)]
pub fn maximum(ctx: &mut Context) -> Result<Atom, Error> {
    ctx.stack
        .apply(|Maximum { left, right }: Maximum| Ok(Atom::Number(left.max(right))))
}

/// Minimum: `.< left right`.
#[inline(always)]
pub fn minimum(ctx: &mut Context) -> Result<Atom, Error> {
    ctx.stack
        .apply(|Minimum { left, right }: Minimum| Ok(Atom::Number(left.min(right))))
}

/// Modulo: `.% left right`.
///
/// A zero divisor has no remainder to name, so this diagnoses and produces no
/// Atom rather than inventing one, exactly as Division does. The diagnostic is
/// its own so the Source learns which Function it wrote.
#[inline(always)]
pub fn modulo(ctx: &mut Context) -> Result<Atom, Error> {
    ctx.stack
        .apply(|Modulo { left, right }: Modulo| match right {
            0 => Err(InterpretationError::ModuloByZero.into()),
            right => Ok(Atom::Number(left % right)),
        })
}

#[inline(always)]
pub fn multiply(ctx: &mut Context) -> Result<Atom, Error> {
    ctx.stack
        .apply(|Multiply { left, right }: Multiply| Ok(Atom::Number(left.wrapping_mul(right))))
}

#[inline(always)]
pub fn subtract(ctx: &mut Context) -> Result<Atom, Error> {
    ctx.stack
        .apply(|Subtract { left, right }: Subtract| Ok(Atom::Number(left.wrapping_sub(right))))
}

#[cfg(test)]
mod test {
    use crate::{
        Anchor, Atom, Error, Function, Interpretation, InterpretationError, Interpreter, Tick,
        TickInputs,
    };

    /// What a Function should answer for one operand pair, in signature order.
    type Reference = fn(u8, u8) -> Result<Atom, InterpretationError>;

    /// Exercises Function dispatch with resolved operands in signature order.
    fn evaluate(function: Function, left: Atom, right: Atom) -> Result<Interpretation, Error> {
        Interpreter::execute_function(
            function,
            [left, right],
            TickInputs::new(Tick::ZERO, Anchor::new(0, 0)).into(),
        )
    }

    #[test]
    fn arithmetic_evaluation_is_wired_to_each_function() {
        // The stack seam is tested where it lives; this is the claim that the
        // Functions the Source can write are actually wired to it, which a test
        // of `Stack::apply` alone cannot make.
        assert_eq!(
            evaluate(Function::Add, Atom::Number(0x10), Atom::Number(2)).unwrap(),
            Interpretation::Cell(Atom::Number(0x12))
        );
        assert!(matches!(
            evaluate(Function::Divide, Atom::Number(0x10), Atom::Number(0)),
            Err(Error::Interpretation(InterpretationError::DivisionByZero))
        ));
    }

    #[test]
    fn equality_answers_a_bang_or_the_absence_marker() {
        // The unequal answer is the absence marker rather than a Number, so the
        // Source never gains a Cell meaning "false".
        assert_eq!(
            evaluate(Function::Equality, Atom::Number(1), Atom::Number(1)).unwrap(),
            Interpretation::Cell(Atom::Bang)
        );
        assert_eq!(
            evaluate(Function::Equality, Atom::Number(1), Atom::Number(2)).unwrap(),
            Interpretation::Cell(Atom::Empty)
        );
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
        let cases: [(Function, Reference); 3] = [
            (Function::Subtract, |left, right| {
                Ok(Atom::Number(left.wrapping_sub(right)))
            }),
            (Function::Divide, |left, right| match right {
                0 => Err(InterpretationError::DivisionByZero),
                right => Ok(Atom::Number(left / right)),
            }),
            (Function::Modulo, |left, right| match right {
                0 => Err(InterpretationError::ModuloByZero),
                right => Ok(Atom::Number(left % right)),
            }),
        ];

        for (function, reference) in cases {
            for left in 0..=u8::MAX {
                for right in 0..=u8::MAX {
                    match (
                        evaluate(function, Atom::Number(left), Atom::Number(right)),
                        reference(left, right),
                    ) {
                        (Ok(answer), Ok(expected)) => {
                            assert_eq!(
                                answer,
                                Interpretation::Cell(expected),
                                "{left:02X} {right:02X}"
                            );
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
