use crate::{
    Atom, Error, Function, InterpretationError,
    atom::{note_atom_from_spelling, number_atom_from_spelling, operands::Track},
    expression::DEFAULT_TOKEN_LEN,
    interpreter::Context,
};

/// The Language Unit at a Jump's input Portal.
///
/// The Turn supplies Cells only after the Portal has classified them as one
/// complete aligned unit. Empty and Bang are values; any other admitted
/// two-Cell unit is the Atom those Cells spell. Alignment and partial Spans
/// never reach here. A missing or invalid
/// spelling, or Cells that are not exactly one two-Cell unit, is diagnosed
/// rather than delivered as a write: blank Cells of another width are not an
/// Empty unit.
pub fn jump(ctx: &mut Context, function: Function) -> Result<Atom, Error> {
    let cells = ctx
        .inputs
        .portal_source()
        .cells()
        .filter(|cells| cells.len() == DEFAULT_TOKEN_LEN)
        .ok_or(InterpretationError::JumpInput { function })?;
    if cells.bytes().all(|cell| cell == b' ') {
        return Ok(Atom::Empty);
    }
    if cells == "**" {
        return Ok(Atom::Bang);
    }
    Ok(copied_atom(cells).ok_or(InterpretationError::JumpInput { function })?)
}

/// Track: `@t index count`.
///
/// The Language Unit at the pair its operands select, read as a Jump reads its
/// Input Portal. The Turn supplies the Cells of that pair, so evaluation binds
/// the operands, refuses a zero `count`, and answers what Jump would.
pub fn track(ctx: &mut Context) -> Result<Atom, Error> {
    selected_pair(ctx.stack.extract::<Track>()?)?;
    jump(ctx, Function::Track)
}

/// The pair Track's operands select, `index % count`, counted from zero east
/// of its last operand.
pub(crate) fn selected_pair(Track { index, count }: Track) -> Result<u8, Error> {
    if count == 0 {
        return Err(InterpretationError::ZeroWrap {
            function: Function::Track,
            role: "count",
        }
        .into());
    }
    Ok(index % count)
}

/// The Atom two Cells spell, read as a Function, then a Number, then a Note.
///
/// Each reading borrows the Cells and builds nothing on refusal: the only
/// error a Jump reports is `JumpInput`, so an error built by a reading the
/// Cells fail would be discarded unread.
fn copied_atom(cells: &str) -> Option<Atom> {
    Function::from_spelling(cells)
        .map(Atom::Function)
        .or_else(|| number_atom_from_spelling(cells))
        .or_else(|| note_atom_from_spelling(cells))
}

#[cfg(test)]
mod test {
    use super::jump;
    use crate::{
        Anchor, Atom, Function, FunctionInputs, Interpretation, InterpretationError, Interpreter,
        Note, PortalSource, Tick, TickInputs, interpreter::Context,
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
        jump(&mut ctx, function)
    }

    #[test]
    fn empty_input_answers_the_absence_marker() {
        assert_eq!(
            evaluate(Function::JumpEast, Some("  ")).unwrap(),
            Atom::Empty
        );
    }

    #[test]
    fn bang_input_answers_bang() {
        assert_eq!(
            evaluate(Function::JumpNorth, Some("**")).unwrap(),
            Atom::Bang
        );
    }

    #[test]
    fn a_number_unit_answers_that_number() {
        assert_eq!(
            evaluate(Function::JumpWest, Some("01")).unwrap(),
            Atom::Number(1)
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

    /// Evaluates Track over `index` and `count` with `cells` at the pair the
    /// Turn found them to select.
    fn track(index: u8, count: u8, cells: Option<&str>) -> Result<Interpretation, crate::Error> {
        Interpreter::execute_function(
            Function::Track,
            [Atom::Number(index), Atom::Number(count)],
            FunctionInputs::with_portal_source(
                TickInputs::new(Tick::ZERO, Anchor::new(0, 0)),
                PortalSource::from_cells(cells),
            ),
        )
    }

    #[test]
    fn track_selects_index_modulo_count() {
        let pair = |index, count| {
            Function::Track
                .selected_pair(&[Atom::Number(index), Atom::Number(count)])
                .expect("Track reads after its operands")
                .unwrap()
        };
        assert_eq!(pair(1, 3), 1);
        assert_eq!(pair(5, 3), 2);
        assert_eq!(pair(0xFF, 0xFF), 0);
        assert!(Function::JumpEast.selected_pair(&[]).is_none());
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
        let selected = Function::Track
            .selected_pair(&[Atom::Number(1), Atom::Number(0)])
            .expect("Track reads after its operands");
        assert_eq!(selected.unwrap_err().to_string(), zero());
        assert_eq!(track(1, 0, Some("D4")).unwrap_err().to_string(), zero());
    }

    #[test]
    fn track_answers_what_a_jump_answers_for_the_same_cells() {
        for cells in ["D4", "01", "**", "  ", ".+"] {
            let jumped = evaluate(Function::JumpEast, Some(cells)).unwrap();
            assert_eq!(
                track(1, 3, Some(cells)).unwrap(),
                Interpretation::Cell(jumped),
                "{cells:?}"
            );
        }
        // Read as a Jump reads it: Number before Note, so `D4` is `0xD4`
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
                InterpretationError::JumpInput {
                    function: Function::Track
                }
            ))
        ));
    }
}
