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
    Ok(absolute_difference_impl(left, right).into())
}

#[inline(always)]
fn absolute_difference_impl(a: u8, b: u8) -> Atom {
    let res = a.abs_diff(b);
    Atom::Number(res)
}

#[inline(always)]
pub fn add(ctx: &mut Context) -> Result<Value, Error> {
    let operands::Add { left, right } = ctx.stack.extract()?;
    Ok(add_impl(left, right).into())
}

#[inline(always)]
fn add_impl(a: u8, b: u8) -> Atom {
    let res = a.wrapping_add(b);
    Atom::Number(res)
}

#[inline(always)]
pub fn divide(ctx: &mut Context) -> Result<Value, Error> {
    let operands::Divide { left, right } = ctx.stack.extract()?;

    divide_impl(left, right).map(Into::into)
}

#[inline(always)]
fn divide_impl(a: u8, b: u8) -> Result<Atom, Error> {
    if b == 0 {
        return Err(InterpretationError::DivisionByZero.into());
    }
    let res = a / b;
    Ok(Atom::Number(res))
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
    Ok(equality_impl(left, right).into())
}

#[inline(always)]
fn equality_impl(a: u8, b: u8) -> Atom {
    if a == b { Atom::Bang } else { Atom::Empty }
}

/// Maximum: `.> left right`.
#[inline(always)]
pub fn maximum(ctx: &mut Context) -> Result<Value, Error> {
    let operands::Maximum { left, right } = ctx.stack.extract()?;
    Ok(maximum_impl(left, right).into())
}

#[inline(always)]
fn maximum_impl(a: u8, b: u8) -> Atom {
    let res = a.max(b);
    Atom::Number(res)
}

/// Minimum: `.< left right`.
#[inline(always)]
pub fn minimum(ctx: &mut Context) -> Result<Value, Error> {
    let operands::Minimum { left, right } = ctx.stack.extract()?;
    Ok(minimum_impl(left, right).into())
}

#[inline(always)]
fn minimum_impl(a: u8, b: u8) -> Atom {
    let res = a.min(b);
    Atom::Number(res)
}

/// Modulo: `.% left right`.
///
/// A zero divisor has no remainder to name, so this diagnoses and produces no
/// Atom rather than inventing one, exactly as Division does. The diagnostic is
/// its own so the Source learns which Function it wrote.
#[inline(always)]
pub fn modulo(ctx: &mut Context) -> Result<Value, Error> {
    let operands::Modulo { left, right } = ctx.stack.extract()?;

    modulo_impl(left, right).map(Into::into)
}

#[inline(always)]
fn modulo_impl(a: u8, b: u8) -> Result<Atom, Error> {
    if b == 0 {
        return Err(InterpretationError::ModuloByZero.into());
    }
    let res = a % b;
    Ok(Atom::Number(res))
}

#[inline(always)]
pub fn multiply(ctx: &mut Context) -> Result<Value, Error> {
    let operands::Multiply { left, right } = ctx.stack.extract()?;
    Ok(multiply_impl(left, right).into())
}

#[inline(always)]
fn multiply_impl(a: u8, b: u8) -> Atom {
    let res = a.wrapping_mul(b);
    Atom::Number(res)
}

#[inline(always)]
pub fn subtract(ctx: &mut Context) -> Result<Value, Error> {
    let operands::Subtract { left, right } = ctx.stack.extract()?;
    Ok(subtract_impl(left, right).into())
}

#[inline(always)]
fn subtract_impl(a: u8, b: u8) -> Atom {
    let res = a.wrapping_sub(b);
    Atom::Number(res)
}

#[cfg(test)]
mod test {
    use super::{divide, modulo, subtract};
    use crate::{Atom, Error, Value, interpreter::Context};

    /// Evaluates `function` against the two operands its signature names
    /// `left` and `right`, pushed so that extraction pops them in signature
    /// order.
    fn evaluate(
        function: fn(&mut Context) -> Result<Value, Error>,
        left: u8,
        right: u8,
    ) -> Result<Value, Error> {
        let mut ctx = Context::new();
        ctx.stack.push(Atom::Number(right));
        ctx.stack.push(Atom::Number(left));
        function(&mut ctx)
    }

    #[test]
    fn the_non_commutative_functions_read_left_and_right_in_signature_order() {
        // Both operands share the Number domain, so neither `Stack::extract`
        // nor the compiler can tell the two roles apart; only the answer can.
        // Every case here is asymmetric, so transposing `left` and `right`
        // changes the result instead of raising a diagnostic.
        assert_eq!(
            evaluate(subtract, 0x09, 0x03).unwrap(),
            Atom::Number(0x06).into()
        );
        assert_eq!(
            evaluate(divide, 0x09, 0x03).unwrap(),
            Atom::Number(0x03).into()
        );
        assert_eq!(
            evaluate(modulo, 0x09, 0x04).unwrap(),
            Atom::Number(0x01).into()
        );
    }
}
