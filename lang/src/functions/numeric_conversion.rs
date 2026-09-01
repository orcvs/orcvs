use crate::{Atom, Error, InterpretationError, interpreter::Context, stack::NumericValue};

#[inline(always)]
pub fn to_number(ctx: &mut Context) -> Result<Atom, Error> {
    let value: NumericValue = ctx.stack.try_pop(1, 0)?;
    let number = match value {
        NumericValue::Note(value) => value.value(),
        NumericValue::Number(value) => value,
    };
    Ok(Atom::Number(number))
}

#[inline(always)]
pub fn to_note(ctx: &mut Context) -> Result<Atom, Error> {
    let value: NumericValue = ctx.stack.try_pop(1, 0)?;
    match value {
        NumericValue::Note(value) => Ok(Atom::Note(value)),
        NumericValue::Number(value @ 0x00..=0x7F) => Ok(Atom::Note(
            crate::Note::try_from(value).map_err(Error::from)?,
        )),
        NumericValue::Number(value) => Err(InterpretationError::NoteConversion(value).into()),
    }
}
