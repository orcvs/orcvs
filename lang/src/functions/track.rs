use super::copy::copy;
use crate::{
    Atom, Error, Function, InterpretationError, atom::operands::Track, interpreter::Context,
    stack::Operands,
};

/// Track: `&t index count`.
///
/// The Language Unit at the pair its operands select, read as a Copy reads its
/// Input Portal. The Turn supplies the Cells of that pair, so evaluation binds
/// the operands, refuses a zero `count`, and answers what a Copy would.
pub fn track(ctx: &mut Context) -> Result<Atom, Error> {
    let Track { index, count } = ctx.stack.extract::<Track>()?;
    selected_pair(Function::Track, index, count)?;
    copy(ctx, Function::Track)
}

/// The pair Track selects from the operands its Turn resolved, counted from
/// zero east of its last operand.
///
/// A `count` of zero selects no pair and diagnoses as a wrap by zero does, and
/// operands outside their domain diagnose as they would at evaluation.
pub(crate) fn track_pair(operands: &[Atom]) -> Result<u8, Error> {
    <Track as Operands>::check(operands)
        .and_then(|()| <Track as Operands>::bind(operands))
        .and_then(|Track { index, count }| selected_pair(Function::Track, index, count))
}

/// The pair `index % count` that `function`'s operands select, Track's or
/// Push's. A `count` of zero selects no pair and diagnoses as a wrap by zero.
pub(crate) fn selected_pair(function: Function, index: u8, count: u8) -> Result<u8, Error> {
    if count == 0 {
        return Err(InterpretationError::ZeroWrap {
            function,
            role: "count",
        }
        .into());
    }
    Ok(index % count)
}

#[cfg(test)]
mod test {
    use crate::{
        Anchor, Atom, Function, FunctionInputs, Interpretation, InterpretationError, Interpreter,
        Note, PortalSource, Tick, TickInputs,
    };

    /// Evaluates `function` over `operands` with `cells` at its Input Portal,
    /// as the Turn hands them over once it has resolved them.
    fn evaluate<const N: usize>(
        function: Function,
        operands: [Atom; N],
        cells: Option<&str>,
    ) -> Result<Interpretation, crate::Error> {
        Interpreter::execute_function(
            function,
            operands,
            FunctionInputs::with_portal_source(
                TickInputs::new(Tick::ZERO, Anchor::new(0, 0)),
                PortalSource::from_cells(cells),
            ),
        )
    }

    /// Evaluates Track over `index` and `count` with `cells` at the pair the
    /// Turn found them to select.
    fn track(index: u8, count: u8, cells: Option<&str>) -> Result<Interpretation, crate::Error> {
        evaluate(
            Function::Track,
            [Atom::Number(index), Atom::Number(count)],
            cells,
        )
    }

    #[test]
    fn track_selects_index_modulo_count() {
        let pair =
            |index, count| super::track_pair(&[Atom::Number(index), Atom::Number(count)]).unwrap();
        assert_eq!(pair(1, 3), 1);
        assert_eq!(pair(5, 3), 2);
        assert_eq!(pair(0xFF, 0xFF), 0);
    }

    #[test]
    fn a_zero_count_selects_no_pair() {
        let zero = || {
            crate::Error::from(InterpretationError::ZeroWrap {
                function: Function::Track,
                role: "count",
            })
            .to_string()
        };
        let selected = super::track_pair(&[Atom::Number(1), Atom::Number(0)]);
        assert_eq!(selected.unwrap_err().to_string(), zero());
        assert_eq!(track(1, 0, Some("D4")).unwrap_err().to_string(), zero());
    }

    #[test]
    fn track_answers_what_a_copy_answers_for_the_same_cells() {
        for cells in ["D4", "01", "**", "  ", ".+"] {
            assert_eq!(
                track(1, 3, Some(cells)).unwrap(),
                evaluate(Function::CopyEast, [], Some(cells)).unwrap(),
                "{cells:?}"
            );
        }
        // Read as a Copy reads it: Number before Note, so `D4` is `0xD4`
        // and `G4`, which spells no Number, is the Note.
        assert_eq!(
            track(1, 3, Some("D4")).unwrap(),
            Interpretation::Cell(Atom::Number(0xD4))
        );
        assert_eq!(
            track(1, 3, Some("G4")).unwrap(),
            Interpretation::Cell(Atom::Note(Note::try_from(67).unwrap()))
        );
        assert!(matches!(
            track(1, 3, Some("xx")),
            Err(crate::Error::Interpretation(
                InterpretationError::CopyInput {
                    function: Function::Track
                }
            ))
        ));
    }
}
