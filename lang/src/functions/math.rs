use crate::{Atom, Error, InterpretationError, interpreter::Context, stack::NumberValue};

#[inline(always)]
pub fn add(ctx: &mut Context) -> Result<Atom, Error> {
    let NumberValue(arg_1) = ctx.stack.try_pop(2, 0)?;
    let NumberValue(arg_2) = ctx.stack.try_pop(2, 1)?;
    Ok(add_impl(arg_1, arg_2))
}

#[inline(always)]
fn add_impl(a: u8, b: u8) -> Atom {
    let res = a.wrapping_add(b);
    Atom::Number(res)
}

#[inline(always)]
pub fn divide(ctx: &mut Context) -> Result<Atom, Error> {
    let NumberValue(arg_1) = ctx.stack.try_pop(2, 0)?;
    let NumberValue(arg_2) = ctx.stack.try_pop(2, 1)?;

    divide_impl(arg_1, arg_2)
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
    let NumberValue(arg_1) = ctx.stack.try_pop(2, 0)?;
    let NumberValue(arg_2) = ctx.stack.try_pop(2, 1)?;
    Ok(multiply_impl(arg_1, arg_2))
}

#[inline(always)]
fn multiply_impl(a: u8, b: u8) -> Atom {
    let res = a.wrapping_mul(b);
    Atom::Number(res)
}

#[inline(always)]
pub fn subtract(ctx: &mut Context) -> Result<Atom, Error> {
    let NumberValue(arg_1) = ctx.stack.try_pop(2, 0)?;
    let NumberValue(arg_2) = ctx.stack.try_pop(2, 1)?;
    Ok(subtract_impl(arg_1, arg_2))
}

#[inline(always)]
fn subtract_impl(a: u8, b: u8) -> Atom {
    let res = a.wrapping_sub(b);
    Atom::Number(res)
}
