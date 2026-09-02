use crate::{Atom, Error, Function, InterpretationError, interpreter::Context};

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
