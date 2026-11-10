use crate::{
    Atom, Error, Function, Interpretation, InterpretationError,
    atom::{
        interval_set_atom_from_spelling, note_atom_from_spelling, number_atom_from_spelling,
        operands::AbsoluteCopy, voicing_atom_from_spelling,
    },
    expression::DEFAULT_TOKEN_LEN,
    interpreter::Context,
    stack::Operands as _,
};

/// The Language Unit at a Copy's input Portal.
///
/// The Turn supplies Cells only after the Portal has classified them as one
/// complete aligned unit. Two empty Cells answer a Clear, which writes them,
/// and `**` answers a Bang; any other admitted two-Cell unit is the Atom
/// those Cells spell. Alignment and partial Spans never reach here. A
/// missing or invalid spelling, or Cells that are not exactly one two-Cell
/// unit, is diagnosed rather than delivered as a write: blank Cells of
/// another width are not a blank pair.
pub fn copy(ctx: &mut Context, function: Function) -> Result<Interpretation, Error> {
    let cells = ctx
        .inputs
        .portal_source()
        .cells()
        .filter(|cells| cells.len() == DEFAULT_TOKEN_LEN)
        .ok_or(InterpretationError::PartialInput { function })?;
    if cells.bytes().all(|cell| cell == b' ') {
        return Ok(Interpretation::Clear);
    }
    if cells == "**" {
        return Ok(Interpretation::Cell(Atom::Bang));
    }
    Ok(Interpretation::Cell(
        copied_atom(cells).ok_or(InterpretationError::PartialInput { function })?,
    ))
}

/// The absolute Copy: `=$ src-column src-row dst-column dst-row`.
///
/// The Language Unit at its source Position, read as a Copy reads its Input
/// Portal. The Turn supplies the Cells of that pair and writes the answer at
/// the destination Position, so evaluation binds the operands and answers
/// what a Copy would.
pub fn absolute_copy(ctx: &mut Context) -> Result<Interpretation, Error> {
    ctx.stack.extract::<AbsoluteCopy>()?;
    copy(ctx, Function::AbsoluteCopy)
}

/// The source Position the absolute Copy reads, from the operands its Turn
/// resolved, as `(column, row)`.
///
/// Operands outside their domain diagnose as they would at evaluation.
pub(crate) fn copied_from(operands: &[Atom]) -> Result<(u8, u8), Error> {
    AbsoluteCopy::checked_bind(operands).map(
        |AbsoluteCopy {
             from_column,
             from_row,
             ..
         }| (from_column, from_row),
    )
}

/// The destination Position the absolute Copy writes, from the operands its
/// Turn resolved, as `(column, row)`.
///
/// Operands outside their domain diagnose as they would at evaluation.
pub(crate) fn copied_to(operands: &[Atom]) -> Result<(u8, u8), Error> {
    AbsoluteCopy::checked_bind(operands).map(
        |AbsoluteCopy {
             to_column, to_row, ..
         }| (to_column, to_row),
    )
}

/// The Atom two Cells spell, read as a Function, then a Number, then a Note,
/// then an Interval Set, then a Voicing.
///
/// A name that also spells a Number or a Note, as `69`, `13` and `D2` do, reads
/// as that Atom here. Every reading renders back as the same two Cells, so a
/// Copy writes the same Cells whichever reading it takes. The Source Tick
/// renders a nested answer to its Cells and decodes them with the receiving
/// operand's [`Token::decode`](crate::Token::decode), so there `69` reaches an
/// Interval Set operand as an Interval Set. An Atom handed straight to
/// [`TokenKind::from_atom`](crate::operand::TokenKind::from_atom) is checked
/// as read here, with no second decode.
///
/// Each reading borrows the Cells and builds nothing on refusal: the only
/// error a Copy reports is `PartialInput`, so an error built by a reading the
/// Cells fail would be discarded unread.
pub(crate) fn copied_atom(cells: &str) -> Option<Atom> {
    Function::from_spelling(cells)
        .map(Atom::Function)
        .or_else(|| number_atom_from_spelling(cells))
        .or_else(|| note_atom_from_spelling(cells))
        .or_else(|| interval_set_atom_from_spelling(cells))
        .or_else(|| voicing_atom_from_spelling(cells))
}

#[cfg(test)]
mod test {
    use super::copy;
    use crate::{
        Anchor, Atom, Function, FunctionInputs, Interpretation, InterpretationError, Interpreter,
        IntervalSet, PortalSource, Tick, TickInputs, Voicing, interpreter::Context,
    };

    // Decode admitted Portal Cells. Alignment and partial Spans are
    // classified at the Portal, not here.
    fn evaluate(function: Function, cells: Option<&str>) -> Result<Interpretation, crate::Error> {
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
    fn empty_input_answers_a_clear() {
        assert_eq!(
            evaluate(Function::CopyEast, Some("  ")).unwrap(),
            Interpretation::Clear
        );
    }

    #[test]
    fn bang_input_answers_bang() {
        assert_eq!(
            evaluate(Function::CopyNorth, Some("**")).unwrap(),
            Interpretation::Cell(Atom::Bang)
        );
    }

    #[test]
    fn a_number_unit_answers_that_number() {
        assert_eq!(
            evaluate(Function::CopyWest, Some("01")).unwrap(),
            Interpretation::Cell(Atom::Number(1))
        );
    }

    #[test]
    fn every_interval_set_and_voicing_name_is_copied_as_the_cells_it_spells() {
        let names = IntervalSet::ALL
            .iter()
            .map(|set| set.cells().to_string())
            .chain(
                Voicing::ALL
                    .iter()
                    .map(|voicing| voicing.cells().to_string()),
            );
        for name in names {
            let Interpretation::Cell(atom) = evaluate(Function::CopyEast, Some(&name)).unwrap()
            else {
                panic!("{name} was not carried");
            };
            assert_eq!(atom.to_string(), name, "{name}");
        }
    }

    #[test]
    fn a_name_that_spells_nothing_else_reads_as_its_own_type() {
        assert_eq!(
            evaluate(Function::CopyEast, Some("M7")).unwrap(),
            Interpretation::Cell(Atom::IntervalSet(IntervalSet::MajorSeventh))
        );
        assert_eq!(
            evaluate(Function::CopyEast, Some("CL")).unwrap(),
            Interpretation::Cell(Atom::Voicing(Voicing::Close))
        );
    }

    #[test]
    fn partial_input_is_still_refused_with_names_in_the_reading() {
        for cells in ["M ", " 7", "CL ", "C ", " L", "m", "xx"] {
            assert!(
                matches!(
                    evaluate(Function::CopyEast, Some(cells)),
                    Err(crate::Error::Interpretation(
                        InterpretationError::PartialInput {
                            function: Function::CopyEast
                        }
                    ))
                ),
                "{cells:?} was read as a Language Unit",
            );
        }
    }

    #[test]
    fn a_missing_or_invalid_site_diagnoses() {
        assert!(matches!(
            evaluate(Function::CopySouth, None),
            Err(crate::Error::Interpretation(
                InterpretationError::PartialInput {
                    function: Function::CopySouth
                }
            ))
        ));
        assert!(matches!(
            evaluate(Function::CopyEast, Some("xx")),
            Err(crate::Error::Interpretation(
                InterpretationError::PartialInput {
                    function: Function::CopyEast
                }
            ))
        ));
    }

    #[test]
    fn an_absolute_copy_answers_what_a_directional_copy_answers_for_the_same_cells() {
        let absolute = |cells| {
            Interpreter::execute_function(
                Function::AbsoluteCopy,
                [Atom::Number(0); 4],
                FunctionInputs::with_portal_source(
                    TickInputs::new(Tick::ZERO, Anchor::new(0, 0)),
                    PortalSource::from_cells(cells),
                ),
            )
        };
        for cells in ["  ", "**", "01", "G4", ".+", "=$"] {
            assert_eq!(
                absolute(Some(cells)).unwrap(),
                evaluate(Function::CopyEast, Some(cells)).unwrap(),
                "{cells:?}"
            );
        }
        for cells in [None, Some("xx"), Some("D")] {
            assert!(
                matches!(
                    absolute(cells),
                    Err(crate::Error::Interpretation(
                        InterpretationError::PartialInput {
                            function: Function::AbsoluteCopy
                        }
                    ))
                ),
                "{cells:?}"
            );
        }
    }

    #[test]
    fn a_portal_that_is_not_one_two_cell_unit_diagnoses() {
        // Blank Cells of the wrong width are not a blank pair, and a wider
        // run whose digits still parse is not a Number: only exactly two
        // Cells are a Language Unit a Copy can copy.
        for cells in ["", " ", "   ", "    ", "1", "001", "0001", "****"] {
            assert!(
                matches!(
                    evaluate(Function::CopyEast, Some(cells)),
                    Err(crate::Error::Interpretation(
                        InterpretationError::PartialInput {
                            function: Function::CopyEast
                        }
                    ))
                ),
                "{cells:?} was read as a Language Unit",
            );
        }
    }
}
