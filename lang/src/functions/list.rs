use crate::{Error, Function, InterpretationError, atom::operands::Track, interpreter::Context};

/// Track: `@t index count`, followed by its List.
///
/// The position of the Item `index % count` selects, counted from zero. The
/// Item's characters are Source in the claim the Parser established, so
/// `orcvs` reads them from working Source; nothing here holds or decodes an
/// Item. Wrapping lets any index drive a List of any length.
#[inline(always)]
pub fn track(ctx: &mut Context) -> Result<u8, Error> {
    let Track { index, count } = ctx.stack.extract::<Track>()?;
    if count == 0 {
        return Err(InterpretationError::ZeroWrap {
            function: Function::Track,
            role: "count",
        }
        .into());
    }
    Ok(index % count)
}

#[cfg(test)]
mod test {
    use crate::{
        Anchor, Atom, Error, Function, Interpretation, InterpretationError, Interpreter, Tick,
        TickInputs,
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

    #[test]
    fn track_refuses_a_zero_count() {
        assert!(matches!(
            evaluate(0, 0),
            Err(Error::Interpretation(InterpretationError::ZeroWrap {
                function: Function::Track,
                role: "count",
            }))
        ));
    }
}
