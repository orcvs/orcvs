use crate::{interpreter::Context, Atom, Error};

#[inline(always)]
pub fn add(ctx: &mut Context) -> Result<Atom, Error> {
    let arg_1 = ctx.stack.try_pop(2, 0)?;
    let arg_2 = ctx.stack.try_pop(2, 1)?;
    Ok(add_impl(arg_1, arg_2))
}

#[inline(always)]
fn add_impl(a: u8, b: u8) -> Atom {
    // Numbers are a single byte, so sums saturate at 255 rather than overflow
    let res: u8 = a.saturating_add(b);
    Atom::Number(res)
}

#[inline(always)]
pub fn divide(ctx: &mut Context) -> Result<Atom, Error> {
    let arg_1 = ctx.stack.try_pop(2, 0)?;
    let arg_2 = ctx.stack.try_pop(2, 1)?;

    Ok(divide_impl(arg_1, arg_2))
}

#[inline(always)]
fn divide_impl(a: u8, b: u8) -> Atom {
    // Divide by zero is zero, which is terribly incorrect
    if b == 0 {
        return Atom::Number(0);
    }
    let res = a / b;
    Atom::Number(res)
}

#[inline(always)]
pub fn multiply(ctx: &mut Context) -> Result<Atom, Error> {
    let arg_1 = ctx.stack.try_pop(2, 0)?;
    let arg_2 = ctx.stack.try_pop(2, 1)?;
    Ok(multiply_impl(arg_1, arg_2))
}

#[inline(always)]
fn multiply_impl(a: u8, b: u8) -> Atom {
    // Numbers are a single byte, so products saturate at 255 rather than overflow
    let res = a.saturating_mul(b);
    Atom::Number(res)
}

#[inline(always)]
pub fn subtract(ctx: &mut Context) -> Result<Atom, Error> {
    let arg_1 = ctx.stack.try_pop(2, 0)?;
    let arg_2 = ctx.stack.try_pop(2, 1)?;
    Ok(subtract_impl(arg_1, arg_2))
}

#[inline(always)]
fn subtract_impl(a: u8, b: u8) -> Atom {
    // No negative numbers
    if a < b {
        return Atom::Number(0);
    }
    let res = a - b;
    Atom::Number(res)
}
