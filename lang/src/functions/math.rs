use crate::{Atom, Error, InterpretationError, Value, atom::operands, interpreter::Context};

/// Absolute Difference: `.| left right`.
///
/// Ordered Subtraction wraps modulo 256, so it answers a cycle position rather
/// than a distance. This Function is the distance, which is why it is separate
/// from `.-` rather than a spelling of it: `abs_diff` is symmetric and has no
/// borrow to wrap.
#[inline(always)]
pub fn absolute_difference(ctx: &mut Context) -> Result<Value, Error> {
    let operands::AbsoluteDifference { left, right } = ctx.stack.extract()?;
    Ok(Atom::Number(left.abs_diff(right)).into())
}

#[inline(always)]
pub fn add(ctx: &mut Context) -> Result<Value, Error> {
    let operands::Add { left, right } = ctx.stack.extract()?;
    Ok(Atom::Number(left.wrapping_add(right)).into())
}

#[inline(always)]
pub fn divide(ctx: &mut Context) -> Result<Value, Error> {
    let operands::Divide { left, right } = ctx.stack.extract()?;

    if right == 0 {
        return Err(InterpretationError::DivisionByZero.into());
    }
    Ok(Atom::Number(left / right).into())
}

/// Equality: `.= left right`.
///
/// A whole-value predicate that answers a pulse rather than a truth value:
/// equal operands produce one Bang, and unequal operands produce `Atom::Empty`,
/// which is already the Interpreter's "no result write" signal. Answering a
/// Number for the unequal case would put a Cell meaning "false" into the
/// Source, where the next Tick would read it as an ordinary operand.
#[inline(always)]
pub fn equality(ctx: &mut Context) -> Result<Value, Error> {
    let operands::Equality { left, right } = ctx.stack.extract()?;
    Ok(if left == right {
        Atom::Bang
    } else {
        Atom::Empty
    }
    .into())
}

/// Maximum: `.> left right`.
#[inline(always)]
pub fn maximum(ctx: &mut Context) -> Result<Value, Error> {
    let operands::Maximum { left, right } = ctx.stack.extract()?;
    Ok(Atom::Number(left.max(right)).into())
}

/// Minimum: `.< left right`.
#[inline(always)]
pub fn minimum(ctx: &mut Context) -> Result<Value, Error> {
    let operands::Minimum { left, right } = ctx.stack.extract()?;
    Ok(Atom::Number(left.min(right)).into())
}

/// Modulo: `.% left right`.
///
/// A zero divisor has no remainder to name, so this diagnoses and produces no
/// Atom rather than inventing one, exactly as Division does. The diagnostic is
/// its own so the Source learns which Function it wrote.
#[inline(always)]
pub fn modulo(ctx: &mut Context) -> Result<Value, Error> {
    let operands::Modulo { left, right } = ctx.stack.extract()?;

    if right == 0 {
        return Err(InterpretationError::ModuloByZero.into());
    }
    Ok(Atom::Number(left % right).into())
}

#[inline(always)]
pub fn multiply(ctx: &mut Context) -> Result<Value, Error> {
    let operands::Multiply { left, right } = ctx.stack.extract()?;
    Ok(Atom::Number(left.wrapping_mul(right)).into())
}

#[inline(always)]
pub fn subtract(ctx: &mut Context) -> Result<Value, Error> {
    let operands::Subtract { left, right } = ctx.stack.extract()?;
    Ok(Atom::Number(left.wrapping_sub(right)).into())
}

#[cfg(test)]
mod test {
    use super::{divide, modulo, subtract};
    use crate::{
        Anchor, Atom, Error, InterpretationError, Tick, TickInputs, Value, interpreter::Context,
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
