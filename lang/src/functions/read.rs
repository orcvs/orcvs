use super::copy::copy;
use crate::{
    Atom, Direction, Error,
    atom::operands::{AbsoluteRead, ReadEast, ReadNorth, ReadSouth, ReadWest},
    interpreter::Context,
    stack::Operands,
};

/// A Read: `&^ n`, `&v n`, `&< n`, `&> n` or `&$ column row`.
///
/// The Language Unit at the pair its operands select, `n` Portals from the
/// `n` operand in a directional Read's direction or at the absolute Read's
/// Position, read as a Copy reads its Input Portal. The Turn supplies
/// the Cells of that pair, so evaluation binds the operands and answers what a
/// Copy would.
pub fn read<O: Operands>(ctx: &mut Context) -> Result<Atom, Error> {
    ctx.stack.extract::<O>()?;
    copy(ctx, O::FUNCTION)
}

/// The distance a directional Read in `direction` counts, from the operands
/// its Turn resolved.
///
/// Operands outside their domain diagnose as they would at evaluation.
pub(crate) fn read_distance(direction: Direction, operands: &[Atom]) -> Result<u8, Error> {
    match direction {
        Direction::North => bound(operands, |ReadNorth { n }| n),
        Direction::South => bound(operands, |ReadSouth { n }| n),
        Direction::East => bound(operands, |ReadEast { n }| n),
        Direction::West => bound(operands, |ReadWest { n }| n),
    }
}

/// The Position the absolute Read `&$ column row` addresses, from the
/// operands its Turn resolved, as `(column, row)`.
///
/// Operands outside their domain diagnose as they would at evaluation.
pub(crate) fn read_position(operands: &[Atom]) -> Result<(u8, u8), Error> {
    bound(operands, |AbsoluteRead { column, row }| (column, row))
}

/// Checks and binds `operands` as `O`'s, answering what `read` takes from
/// them.
fn bound<O: Operands, T>(operands: &[Atom], read: fn(O) -> T) -> Result<T, Error> {
    O::check(operands)?;
    O::bind(operands).map(read)
}

#[cfg(test)]
mod test {
    use crate::{
        Anchor, Atom, Direction, Function, FunctionInputs, Interpretation, InterpretationError,
        Interpreter, Note, PortalSource, Tick, TickInputs,
    };

    const READS: [(Function, Direction); 4] = [
        (Function::ReadNorth, Direction::North),
        (Function::ReadSouth, Direction::South),
        (Function::ReadEast, Direction::East),
        (Function::ReadWest, Direction::West),
    ];

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

    #[test]
    fn a_read_counts_the_distance_its_n_operand_states() {
        let note = Atom::Note(Note::try_from(60).unwrap());
        for (_, direction) in READS {
            for n in [0, 1, 0xFF] {
                assert_eq!(
                    super::read_distance(direction, &[Atom::Number(n)]).unwrap(),
                    n,
                    "{direction:?}"
                );
            }
            assert!(
                matches!(
                    super::read_distance(direction, &[note]),
                    Err(crate::Error::Type(_))
                ),
                "{direction:?}"
            );
        }
    }

    #[test]
    fn an_absolute_read_addresses_column_then_row() {
        let position =
            |column, row| super::read_position(&[Atom::Number(column), Atom::Number(row)]);
        assert_eq!(position(0, 0).unwrap(), (0, 0));
        assert_eq!(position(0x3A, 0x18).unwrap(), (0x3A, 0x18));
        assert_eq!(position(0xFF, 0xFF).unwrap(), (0xFF, 0xFF));
        let note = Atom::Note(Note::try_from(60).unwrap());
        assert!(matches!(
            super::read_position(&[Atom::Number(0), note]),
            Err(crate::Error::Type(_))
        ));
        assert_eq!(
            evaluate(
                Function::AbsoluteRead,
                [Atom::Number(0), Atom::Number(0)],
                Some("G4")
            )
            .unwrap(),
            Interpretation::Cell(Atom::Note(Note::try_from(67).unwrap()))
        );
    }

    #[test]
    fn a_read_answers_what_a_copy_answers_for_the_same_cells() {
        let copied: Vec<(&str, Interpretation)> = ["D4", "01", "**", "  ", ".+", "&<"]
            .into_iter()
            .map(|cells| {
                let copy = evaluate(Function::CopyEast, [], Some(cells)).unwrap();
                (cells, copy)
            })
            .collect();
        for (function, _) in READS {
            let read = |cells| evaluate(function, [Atom::Number(1)], cells);
            for (cells, copy) in &copied {
                assert_eq!(read(Some(cells)).unwrap(), *copy, "{function:?} {cells:?}");
            }
            assert_eq!(
                read(Some("G4")).unwrap(),
                Interpretation::Cell(Atom::Note(Note::try_from(67).unwrap()))
            );
            for cells in [None, Some("xx"), Some("D")] {
                assert!(
                    matches!(
                        read(cells),
                        Err(crate::Error::Interpretation(
                            InterpretationError::CopyInput { function: diagnosed }
                        )) if diagnosed == function
                    ),
                    "{function:?} {cells:?}"
                );
            }
        }
    }
}
