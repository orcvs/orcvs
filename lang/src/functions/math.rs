use crate::{Atom, Error, Function, InterpretationError, interpreter::Context};

/// Absolute Difference: `.| left right`.
///
/// Ordered Subtraction wraps modulo 256, so it answers a cycle position rather
/// than a distance. This Function is the distance, which is why it is separate
/// from `.-` rather than a spelling of it: `abs_diff` is symmetric and has no
/// borrow to wrap.
#[inline(always)]
pub fn absolute_difference(ctx: &mut Context) -> Result<Atom, Error> {
    let operands = ctx.stack.extract(Function::AbsoluteDifference)?;
    Ok(absolute_difference_impl(
        operands.number(0),
        operands.number(1),
    ))
}

#[inline(always)]
fn absolute_difference_impl(a: u8, b: u8) -> Atom {
    let res = a.abs_diff(b);
    Atom::Number(res)
}

#[inline(always)]
pub fn add(ctx: &mut Context) -> Result<Atom, Error> {
    let operands = ctx.stack.extract(Function::Add)?;
    Ok(add_impl(operands.number(0), operands.number(1)))
}

#[inline(always)]
fn add_impl(a: u8, b: u8) -> Atom {
    let res = a.wrapping_add(b);
    Atom::Number(res)
}

#[inline(always)]
pub fn divide(ctx: &mut Context) -> Result<Atom, Error> {
    let operands = ctx.stack.extract(Function::Divide)?;

    divide_impl(operands.number(0), operands.number(1))
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
pub fn equality(ctx: &mut Context) -> Result<Atom, Error> {
    let operands = ctx.stack.extract(Function::Equality)?;
    Ok(equality_impl(operands.number(0), operands.number(1)))
}

#[inline(always)]
fn equality_impl(a: u8, b: u8) -> Atom {
    if a == b { Atom::Bang } else { Atom::Empty }
}

/// Maximum: `.> left right`.
#[inline(always)]
pub fn maximum(ctx: &mut Context) -> Result<Atom, Error> {
    let operands = ctx.stack.extract(Function::Maximum)?;
    Ok(maximum_impl(operands.number(0), operands.number(1)))
}

#[inline(always)]
fn maximum_impl(a: u8, b: u8) -> Atom {
    let res = a.max(b);
    Atom::Number(res)
}

/// Minimum: `.< left right`.
#[inline(always)]
pub fn minimum(ctx: &mut Context) -> Result<Atom, Error> {
    let operands = ctx.stack.extract(Function::Minimum)?;
    Ok(minimum_impl(operands.number(0), operands.number(1)))
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
pub fn modulo(ctx: &mut Context) -> Result<Atom, Error> {
    let operands = ctx.stack.extract(Function::Modulo)?;

    modulo_impl(operands.number(0), operands.number(1))
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
pub fn multiply(ctx: &mut Context) -> Result<Atom, Error> {
    let operands = ctx.stack.extract(Function::Multiply)?;
    Ok(multiply_impl(operands.number(0), operands.number(1)))
}

#[inline(always)]
fn multiply_impl(a: u8, b: u8) -> Atom {
    let res = a.wrapping_mul(b);
    Atom::Number(res)
}

#[inline(always)]
pub fn subtract(ctx: &mut Context) -> Result<Atom, Error> {
    let operands = ctx.stack.extract(Function::Subtract)?;
    Ok(subtract_impl(operands.number(0), operands.number(1)))
}

#[inline(always)]
fn subtract_impl(a: u8, b: u8) -> Atom {
    let res = a.wrapping_sub(b);
    Atom::Number(res)
}
