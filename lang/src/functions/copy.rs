use crate::{
    Atom, Error, Function, InterpretationError,
    atom::{note_atom_from_spelling, number_atom_from_spelling},
    expression::DEFAULT_TOKEN_LEN,
    interpreter::Context,
};

/// The Language Unit at a Copy's input Portal.
///
/// The Turn supplies Cells only after the Portal has classified them as one
/// complete aligned unit. Empty and Bang are values; any other admitted
/// two-Cell unit is the Atom those Cells spell. Alignment and partial Spans
/// never reach here. A missing or invalid
/// spelling, or Cells that are not exactly one two-Cell unit, is diagnosed
/// rather than delivered as a write: blank Cells of another width are not an
/// Empty unit.
pub fn copy(ctx: &mut Context, function: Function) -> Result<Atom, Error> {
    let cells = ctx
        .inputs
        .portal_source()
        .cells()
        .filter(|cells| cells.len() == DEFAULT_TOKEN_LEN)
        .ok_or(InterpretationError::CopyInput { function })?;
    if cells.bytes().all(|cell| cell == b' ') {
        return Ok(Atom::Empty);
    }
    if cells == "**" {
        return Ok(Atom::Bang);
    }
    Ok(copied_atom(cells).ok_or(InterpretationError::CopyInput { function })?)
}

/// The Atom two Cells spell, read as a Function, then a Number, then a Note.
///
/// Each reading borrows the Cells and builds nothing on refusal: the only
/// error a Copy reports is `CopyInput`, so an error built by a reading the
/// Cells fail would be discarded unread.
pub(crate) fn copied_atom(cells: &str) -> Option<Atom> {
    Function::from_spelling(cells)
        .map(Atom::Function)
        .or_else(|| number_atom_from_spelling(cells))
        .or_else(|| note_atom_from_spelling(cells))
}

#[cfg(test)]
mod test {
    use super::copy;
    use crate::{
        Anchor, Atom, Function, FunctionInputs, InterpretationError, PortalSource, Tick,
        TickInputs, interpreter::Context,
    };

    // Decode admitted Portal Cells. Alignment and partial Spans are
    // classified at the Portal, not here.
    fn evaluate(function: Function, cells: Option<&str>) -> Result<Atom, crate::Error> {
        let mut ctx = Context::new(
            FunctionInputs::with_portal_source(
                TickInputs::new(Tick::ZERO, Anchor::new(0, 0)),
                PortalSource::from_cells(cells),
            ),
            1,
        );
        copy(&mut ctx, function)
    }

    #[test]
    fn empty_input_answers_the_absence_marker() {
        assert_eq!(
            evaluate(Function::CopyEast, Some("  ")).unwrap(),
            Atom::Empty
        );
    }

    #[test]
    fn bang_input_answers_bang() {
        assert_eq!(
            evaluate(Function::CopyNorth, Some("**")).unwrap(),
            Atom::Bang
        );
    }

    #[test]
    fn a_number_unit_answers_that_number() {
        assert_eq!(
            evaluate(Function::CopyWest, Some("01")).unwrap(),
            Atom::Number(1)
        );
    }

    #[test]
    fn a_missing_or_invalid_site_diagnoses() {
        assert!(matches!(
            evaluate(Function::CopySouth, None),
            Err(crate::Error::Interpretation(
                InterpretationError::CopyInput {
                    function: Function::CopySouth
                }
            ))
        ));
        assert!(matches!(
            evaluate(Function::CopyEast, Some("xx")),
            Err(crate::Error::Interpretation(
                InterpretationError::CopyInput {
                    function: Function::CopyEast
                }
            ))
        ));
    }

    #[test]
    fn a_portal_that_is_not_one_two_cell_unit_diagnoses() {
        // Blank Cells of the wrong width are not an Empty unit, and a wider
        // run whose digits still parse is not a Number: only exactly two
        // Cells are a Language Unit a Copy can copy.
        for cells in ["", " ", "   ", "    ", "1", "001", "0001", "****"] {
            assert!(
                matches!(
                    evaluate(Function::CopyEast, Some(cells)),
                    Err(crate::Error::Interpretation(
                        InterpretationError::CopyInput {
                            function: Function::CopyEast
                        }
                    ))
                ),
                "{cells:?} was read as a Language Unit",
            );
        }
    }
}
