pub(crate) mod math;
use crate::{
    interpreter::{Args, Context},
    Atom, Error, Function,
};

#[inline(always)]
pub fn ident(ctx: &mut Context) -> Result<Atom, Error> {
    Ok(ctx.stack.pop().into())
}

#[inline(always)]
pub fn play(ctx: &mut Context) -> Result<Atom, Error> {
    let arg_1 = ctx.stack.try_pop(3, 0)?;
    let arg_2 = ctx.stack.try_pop(3, 1)?;
    let arg_3 = ctx.stack.try_pop(3, 2)?;
    Ok(play_impl(arg_1, arg_2, arg_3))
}

#[inline(always)]
fn play_impl(c: u8, v: u8, n: u8) -> Atom {
    // info!("Play: c: {}, v: {}, n: {}", c, v, n);
    Atom::Number(0)
}
