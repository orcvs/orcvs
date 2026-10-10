use super::copy::copy;
use crate::{
    Atom, Error, Function,
    atom::operands::{
        AbsoluteRead, AbsoluteWrite, ReadEast, ReadNorth, ReadSouth, ReadWest, WriteEast,
        WriteNorth, WriteSouth, WriteWest,
    },
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

/// The distance the directional Read or Write `function` counts, from the
/// operands its Turn resolved.
///
/// The operands are checked against `function`'s own signature, so they
/// diagnose as they would at its evaluation: a Write's `value` follows its
/// `n` and is never read.
pub(crate) fn distance(function: Function, operands: &[Atom]) -> Result<u8, Error> {
    match function {
        Function::ReadEast => bound(operands, |ReadEast { n }| n),
        Function::ReadNorth => bound(operands, |ReadNorth { n }| n),
        Function::ReadSouth => bound(operands, |ReadSouth { n }| n),
        Function::ReadWest => bound(operands, |ReadWest { n }| n),
        Function::WriteEast => bound(operands, |WriteEast { n, .. }| n),
        Function::WriteNorth => bound(operands, |WriteNorth { n, .. }| n),
        Function::WriteSouth => bound(operands, |WriteSouth { n, .. }| n),
        Function::WriteWest => bound(operands, |WriteWest { n, .. }| n),
        // Only `Function::input_portal` and `Function::dynamic_output_portal`
        // build a distance selection, and each names one of the Functions
        // above.
        _ => unreachable!("{function:?} counts no distance"),
    }
}

/// The Position the absolute Read `&$ column row` or the absolute Write
/// `@$ column row value` that `function` names addresses, from the operands
/// its Turn resolved, as `(column, row)`.
///
/// The operands are checked against `function`'s own signature, so they
/// diagnose as they would at its evaluation: a Write's `value` follows the
/// Position and is never read.
pub(crate) fn position(function: Function, operands: &[Atom]) -> Result<(u8, u8), Error> {
    match function {
        Function::AbsoluteRead => bound(operands, |AbsoluteRead { column, row }| (column, row)),
        Function::AbsoluteWrite => {
            bound(operands, |AbsoluteWrite { column, row, .. }| (column, row))
        }
        // Only `Function::input_portal` and `Function::dynamic_output_portal`
        // build a Position selection, and each names one of the Functions
        // above.
        _ => unreachable!("{function:?} addresses no Position"),
    }
}

/// Checks and binds `operands` as `O`'s, answering what `read` takes from
/// them.
pub(super) fn bound<O: Operands, T>(operands: &[Atom], read: fn(O) -> T) -> Result<T, Error> {
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
        for (function, direction) in READS {
            for n in [0, 1, 0xFF] {
                assert_eq!(
                    super::distance(function, &[Atom::Number(n)]).unwrap(),
                    n,
                    "{direction:?}"
                );
            }
            assert!(
                matches!(
                    super::distance(function, &[note]),
                    Err(crate::Error::Type(_))
                ),
                "{direction:?}"
            );
        }
    }

    #[test]
    fn an_absolute_read_addresses_column_then_row() {
        let position = |column, row| {
            super::position(
                Function::AbsoluteRead,
                &[Atom::Number(column), Atom::Number(row)],
            )
        };
        assert_eq!(position(0, 0).unwrap(), (0, 0));
        assert_eq!(position(0x3A, 0x18).unwrap(), (0x3A, 0x18));
        assert_eq!(position(0xFF, 0xFF).unwrap(), (0xFF, 0xFF));
        let note = Atom::Note(Note::try_from(60).unwrap());
        assert!(matches!(
            super::position(Function::AbsoluteRead, &[Atom::Number(0), note]),
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
                            InterpretationError::PartialInput { function: diagnosed }
                        )) if diagnosed == function
                    ),
                    "{function:?} {cells:?}"
                );
            }
        }
    }
}
