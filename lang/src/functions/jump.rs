use crate::{
    Atom, Error, Function, InterpretationError, Value, expression::DEFAULT_TOKEN_LEN,
    interpreter::Context, to_atom_note, to_atom_num,
};

/// The Language Unit at a Jump's input Portal.
///
/// The Turn supplies Cells only after the Portal has classified them as one
/// complete aligned unit. Empty and Bang are values; any other admitted
/// two-Cell unit is the Atom those Cells spell. Alignment, Sequence
/// membership, and partial Spans never reach here. A missing or invalid
/// spelling, or Cells that are not exactly one two-Cell unit, is diagnosed
/// rather than delivered as a write: blank Cells of another width are not an
/// Empty unit.
pub fn jump(ctx: &mut Context, function: Function) -> Result<Value, Error> {
    let cells = ctx
        .inputs
        .portal_source()
        .cells()
        .filter(|cells| cells.len() == DEFAULT_TOKEN_LEN)
        .ok_or(InterpretationError::JumpInput { function })?;
    if cells.bytes().all(|cell| cell == b' ') {
        return Ok(Atom::Empty.into());
    }
    if cells == "**" {
        return Ok(Atom::Bang.into());
    }
    Ok(copied_atom(cells)
        .ok_or(InterpretationError::JumpInput { function })?
        .into())
}

fn copied_atom(cells: &str) -> Option<Atom> {
    if let Ok(function) = Function::try_from(cells) {
        return Some(Atom::Function(function));
    }
    if let Ok(atom) = to_atom_num(cells) {
        return Some(atom);
    }
    if let Ok(atom) = to_atom_note(cells) {
        return Some(atom);
    }
    None
}

#[cfg(test)]
mod test {
    use super::jump;
    use crate::{
        Anchor, Atom, Function, FunctionInputs, InterpretationError, PortalSource, Tick,
        TickInputs, interpreter::Context,
    };

    // Decode admitted Portal Cells. Alignment, Sequence membership, and
    // partial Spans are classified at the Portal, not here.
    fn evaluate(function: Function, cells: Option<&str>) -> Result<crate::Value, crate::Error> {
        let mut ctx = Context::new(
            FunctionInputs::with_portal_source(
                TickInputs::new(Tick::ZERO, Anchor::new(0, 0)),
                PortalSource::from_cells(cells),
            ),
            1,
        );
        jump(&mut ctx, function)
    }

    #[test]
    fn empty_input_answers_the_absence_marker() {
        assert_eq!(
            evaluate(Function::JumpEast, Some("  ")).unwrap(),
            Atom::Empty.into()
        );
    }

    #[test]
    fn bang_input_answers_bang() {
        assert_eq!(
            evaluate(Function::JumpNorth, Some("**")).unwrap(),
            Atom::Bang.into()
        );
    }

    #[test]
    fn a_number_unit_answers_that_number() {
        assert_eq!(
            evaluate(Function::JumpWest, Some("01")).unwrap(),
            Atom::Number(1).into()
        );
    }

    #[test]
    fn a_missing_or_invalid_site_diagnoses() {
        assert!(matches!(
            evaluate(Function::JumpSouth, None),
            Err(crate::Error::Interpretation(
                InterpretationError::JumpInput {
                    function: Function::JumpSouth
                }
            ))
        ));
        assert!(matches!(
            evaluate(Function::JumpEast, Some("xx")),
            Err(crate::Error::Interpretation(
                InterpretationError::JumpInput {
                    function: Function::JumpEast
                }
            ))
        ));
    }

    #[test]
    fn a_portal_that_is_not_one_two_cell_unit_diagnoses() {
        // Blank Cells of the wrong width are not an Empty unit, and a wider
        // run whose digits still parse is not a Number: only exactly two
        // Cells are a Language Unit a Jump can copy.
        for cells in ["", " ", "   ", "    ", "1", "001", "0001", "****"] {
            assert!(
                matches!(
                    evaluate(Function::JumpEast, Some(cells)),
                    Err(crate::Error::Interpretation(
                        InterpretationError::JumpInput {
                            function: Function::JumpEast
                        }
                    ))
                ),
                "{cells:?} was read as a Language Unit",
            );
        }
    }
}
