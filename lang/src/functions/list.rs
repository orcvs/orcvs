use crate::{Error, atom::operands::Track, interpreter::Context};

/// Track: `@t index count`, followed by its List.
///
/// The position of the Item `index % count` selects, counted from zero. The
/// Item's characters are Source in the claim the Parser established, so
/// `orcvs` reads them from working Source; nothing here holds or decodes an
/// Item. Wrapping lets any index drive a List of any length, and the count
/// binds as a [`NonZeroU8`](core::num::NonZeroU8), so every index selects an
/// Item.
#[inline(always)]
pub fn track(ctx: &mut Context) -> Result<u8, Error> {
    let Track { index, count } = ctx.stack.extract::<Track>()?;
    Ok(index % count)
}

#[cfg(test)]
mod test {
    use crate::{
        Anchor, Atom, Error, Function, Interpretation, Interpreter, SyntaxError, Tick, TickInputs,
    };

    fn evaluate(index: u8, count: u8) -> Result<Interpretation, Error> {
        Interpreter::execute_function(
            Function::Track,
            [Atom::Number(index), Atom::Number(count)],
            TickInputs::new(Tick::ZERO, Anchor::new(0, 0)).into(),
        )
    }

    #[test]
    fn track_selects_the_index_wrapped_at_the_count() {
        for (index, count, item) in [(0, 2, 0), (1, 2, 1), (2, 2, 0), (7, 3, 1), (0xFF, 8, 7)] {
            assert_eq!(
                evaluate(index, count).unwrap(),
                Interpretation::Item(item),
                "{index:02X} of {count:02X}"
            );
        }
    }

    /// A direct caller can hand Track a count of `00`, which no claim holds,
    /// and is told so as the Parser tells the Source.
    #[test]
    fn track_refuses_a_count_of_00() {
        for index in [0, 1, 0xFF] {
            assert!(
                matches!(
                    evaluate(index, 0),
                    Err(Error::Syntax(SyntaxError::EmptyList))
                ),
                "{index:02X}"
            );
        }
    }
}
