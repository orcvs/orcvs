//! Declared Portal inputs, resolved from working Source by the caller at Turn.
//!
//! Cell operands live on the Operand Stack; Portal inputs are the other half of
//! what a Function reads at Turn. [`FunctionInputs`] is the one bundle the
//! Evaluator receives beside resolved cell operands.

use crate::{
    Atom, Error, Function, InputPortal, InterpretationError, PairSelection, PortalAddress,
    PortalCoords, Stack, TickInputs, Token,
    expression::DEFAULT_TOKEN_LEN,
    functions::{
        copy::{copied_from, copied_to},
        read::{distance, position},
        track::track_pair,
        write::lane_pair,
    },
    stack::Operands,
};

impl InputPortal {
    ///
    /// Where this Input Portal stands for its Function's Turn.
    ///
    /// A static Input Portal is the offset it declares and reads neither
    /// argument. A dynamic one is the pair its [`PairSelection`] selects,
    /// which [`PairSelection::resolve`] describes.
    ///
    /// # Errors
    ///
    /// As [`PairSelection::resolve`].
    ///
    pub fn resolve(
        self,
        operands: &[Atom],
        operand_columns: usize,
    ) -> Result<PortalAddress, Error> {
        match self {
            Self::Static(coords) => Ok(PortalAddress::Offset(coords)),
            Self::Dynamic(selection) => selection.resolve(operands, operand_columns),
        }
    }
}

impl PairSelection {
    ///
    /// Where the selected pair stands for its Function's Turn.
    ///
    /// `operands` are the values the Turn resolved, and `operand_columns` is
    /// how many columns east of the anchor the Function and its operands
    /// occupy, nested operands included: `orcvs` knows how they lie in the
    /// Grid. The rule takes the values of only the address operands that
    /// lead the signature, and the directional and absolute rules count the
    /// operands against the signature of the Read or Write they name: a
    /// Write's `value` after them counts toward that number, and its value is
    /// never read. Track's pair counts from `operand_columns`, Push's lane
    /// from the pair under its anchor, a directional rule's from its `n`
    /// operand's slot, the pair after its spelling, whatever that operand's
    /// width, and the absolute rules' are the Positions their operands name,
    /// which `orcvs` finds in its Grid. The absolute Copy's two rules each read
    /// their own half of its four operands.
    ///
    /// # Errors
    ///
    /// The address operands diagnose as evaluation does: a count other than
    /// the signature's is its arity, an operand that is not a Number, such as
    /// a Note given as Track's `count` or as a Read's `n`, is an
    /// [`Error::Type`], and Track's or Push's `count` of `00` is a wrap by
    /// zero. Track's pair can lie too far east to represent, which is
    /// a Portal outside any Grid and diagnoses as Track's partial or invalid
    /// input, as a Portal past the row edge does. Any other pair is unchecked
    /// here: a pair placed outside the Grid diagnoses when `orcvs` resolves
    /// it.
    ///
    pub fn resolve(
        self,
        operands: &[Atom],
        operand_columns: usize,
    ) -> Result<PortalAddress, Error> {
        match self {
            Self::Position(function) => {
                let (column, row) = position(function, operands)?;
                Ok(PortalAddress::Position { column, row })
            }
            Self::SourcePosition => {
                let (column, row) = copied_from(operands)?;
                Ok(PortalAddress::Position { column, row })
            }
            Self::DestinationPosition => {
                let (column, row) = copied_to(operands)?;
                Ok(PortalAddress::Position { column, row })
            }
            Self::IndexModuloCount => {
                let pair = track_pair(operands)?;
                let columns = operand_columns
                    .checked_add(usize::from(pair) * DEFAULT_TOKEN_LEN)
                    .and_then(|columns| i16::try_from(columns).ok())
                    .ok_or(InterpretationError::PartialInput {
                        function: Function::Track,
                    })?;
                Ok(PortalAddress::Offset(PortalCoords { columns, rows: 0 }))
            }
            Self::Lane => {
                // At most 254 pairs east of the anchor, every offset fits.
                let pair = i16::from(lane_pair(operands)?);
                Ok(PortalAddress::Offset(PortalCoords {
                    columns: pair * DEFAULT_TOKEN_LEN as i16,
                    rows: 1,
                }))
            }
            Self::Distance(direction, function) => {
                // The slot is the pair after the spelling, and at most 255
                // steps from it every offset fits.
                let n = i16::from(distance(function, operands)?);
                let step = direction.step();
                Ok(PortalAddress::Offset(PortalCoords {
                    columns: DEFAULT_TOKEN_LEN as i16 + n * step.columns,
                    rows: n * step.rows,
                }))
            }
        }
    }
}

/// One Portal input declaration, beside the Function's cell operands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PortalInput {
    token: Token,
    role: &'static str,
}

impl PortalInput {
    pub(crate) const fn number(role: &'static str) -> Self {
        Self {
            token: Token::Number,
            role,
        }
    }

    pub const fn token(self) -> Token {
        self.token
    }
    pub const fn role(self) -> &'static str {
        self.role
    }
}

/// Working Source at a Portal, borrowed for one Function evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PortalSource<'a> {
    cells: Option<&'a str>,
}

impl<'a> PortalSource<'a> {
    /// No Portal Cells were supplied.
    #[inline]
    #[must_use]
    pub const fn none() -> Self {
        Self { cells: None }
    }

    /// The Cells at the Function's Portal, when they resolved.
    #[inline]
    #[must_use]
    pub const fn from_cells(cells: Option<&'a str>) -> Self {
        Self { cells }
    }

    /// The Cells at the Function's Portal.
    #[inline]
    #[must_use]
    pub fn cells(self) -> Option<&'a str> {
        self.cells
    }
}

/// Everything one Function evaluation reads beyond its cell operands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FunctionInputs<'a> {
    tick: TickInputs,
    portal_source: PortalSource<'a>,
}

impl<'a> FunctionInputs<'a> {
    /// Playback Tick and anchor only; no Portal Cells.
    #[inline]
    #[must_use]
    pub fn new(tick: TickInputs) -> Self {
        Self {
            tick,
            portal_source: PortalSource::none(),
        }
    }

    /// Tick, anchor, and working Source at the Function's Portal.
    #[inline]
    #[must_use]
    pub const fn with_portal_source(tick: TickInputs, portal_source: PortalSource<'a>) -> Self {
        Self {
            tick,
            portal_source,
        }
    }

    /// The absolute Tick this evaluation belongs to.
    #[inline]
    #[must_use]
    pub fn tick(self) -> crate::Tick {
        self.tick.tick()
    }

    /// The anchor Position of the root being evaluated.
    #[inline]
    #[must_use]
    pub fn anchor(self) -> crate::Anchor {
        self.tick.anchor()
    }

    /// Working Source at the Function's Portal, supplied by the Turn.
    #[inline]
    #[must_use]
    pub const fn portal_source(self) -> PortalSource<'a> {
        self.portal_source
    }
}

impl From<TickInputs> for FunctionInputs<'_> {
    #[inline]
    fn from(tick: TickInputs) -> Self {
        Self::new(tick)
    }
}

/// Implemented by the declaration table only for Functions with a Portal input.
pub(crate) trait PortalOperands: Operands {
    const PORTAL: PortalInput;
}

/// A Number decoded from a declared Portal input. Its private field prevents
/// formulas from accepting unchecked Cells.
pub(crate) struct PortalNumber(u8);

impl PortalNumber {
    pub(crate) fn bind(
        function: Function,
        input: PortalInput,
        source: PortalSource<'_>,
    ) -> Result<Self, Error> {
        let invalid = || InterpretationError::PortalInputNotNumber {
            function,
            role: input.role(),
        };
        let cells = source.cells().ok_or_else(invalid)?;
        if cells.len() == input.token().len() && cells.bytes().all(|cell| cell == b' ') {
            return Ok(Self(0));
        }
        match input.token().decode(cells) {
            Ok(Atom::Number(number)) => Ok(Self(number)),
            _ => Err(invalid().into()),
        }
    }

    pub(crate) fn number(self) -> u8 {
        self.0
    }
}

/// Pop and validate cell operands, then decode the declared Portal input.
#[inline(always)]
pub(crate) fn bind_operands<O: PortalOperands>(
    stack: &mut Stack,
    source: PortalSource<'_>,
) -> Result<(O, PortalNumber), Error> {
    let operands = stack.extract::<O>()?;
    let portal = PortalNumber::bind(O::FUNCTION, O::PORTAL, source)?;
    Ok((operands, portal))
}

#[cfg(test)]
mod test {
    use super::{PortalInput, PortalNumber, PortalSource};
    use crate::{
        Anchor, Atom, Error, Function, FunctionInputs, InputPortal, InterpretationError,
        Interpreter, Note, PortalAddress, PortalCoords, Tick, TickInputs,
    };

    const INPUT: PortalInput = PortalInput::number("previous value");

    #[test]
    fn empty_portal_cells_initialize_as_number_zero() {
        assert_eq!(
            PortalNumber::bind(
                Function::Increment,
                INPUT,
                PortalSource::from_cells(Some("  "))
            )
            .unwrap()
            .number(),
            0
        );
    }

    #[test]
    fn a_number_portal_spelling_decodes() {
        assert_eq!(
            PortalNumber::bind(
                Function::Increment,
                INPUT,
                PortalSource::from_cells(Some("0A"))
            )
            .unwrap()
            .number(),
            0x0A
        );
    }

    #[test]
    fn a_missing_portal_site_diagnoses() {
        assert!(matches!(
            PortalNumber::bind(Function::Increment, INPUT, PortalSource::none()),
            Err(Error::Interpretation(
                InterpretationError::PortalInputNotNumber {
                    function: Function::Increment,
                    role: "previous value",
                }
            ))
        ));
    }

    #[test]
    fn a_non_number_portal_spelling_diagnoses() {
        assert!(matches!(
            PortalNumber::bind(
                Function::Increment,
                INPUT,
                PortalSource::from_cells(Some("G4"))
            ),
            Err(Error::Interpretation(
                InterpretationError::PortalInputNotNumber {
                    function: Function::Increment,
                    role: "previous value",
                }
            ))
        ));
    }

    /// Resolves `function`'s declared Input Portal over `operands`, which
    /// occupy `operand_columns` columns east of its anchor.
    fn address(
        function: Function,
        operands: &[Atom],
        operand_columns: usize,
    ) -> Result<PortalAddress, Error> {
        function
            .input_portal()
            .expect("the Function declares an Input Portal")
            .resolve(operands, operand_columns)
    }

    /// The offset `function`'s declared Input Portal resolves to.
    fn resolve(
        function: Function,
        operands: &[Atom],
        operand_columns: usize,
    ) -> Result<PortalCoords, Error> {
        address(function, operands, operand_columns).map(|address| match address {
            PortalAddress::Offset(coords) => coords,
            PortalAddress::Position { .. } => panic!("{function:?} names a Position"),
        })
    }

    #[test]
    fn an_absolute_read_resolves_to_the_position_its_operands_name() {
        let read = |column, row| {
            address(
                Function::AbsoluteRead,
                &[Atom::Number(column), Atom::Number(row)],
                6,
            )
            .unwrap()
        };
        let position = |column, row| PortalAddress::Position { column, row };
        assert_eq!(read(0, 0), position(0, 0));
        assert_eq!(read(0x0B, 0x02), position(0x0B, 0x02));
        assert_eq!(read(0xFF, 0xFF), position(0xFF, 0xFF));
        let note = Atom::Note(Note::try_from(60).unwrap());
        for operands in [[note, Atom::Number(0)], [Atom::Number(0), note]] {
            assert!(
                matches!(
                    address(Function::AbsoluteRead, &operands, 6),
                    Err(Error::Type(_))
                ),
                "{operands:?}"
            );
        }
    }

    #[test]
    fn a_read_handed_the_wrong_operand_count_diagnoses_its_own_arity() {
        // Evaluation would diagnose the Read's own arity, so resolution
        // diagnoses it too, whatever the Write with the same arrow declares.
        let number = Atom::Number(1);
        for function in [
            Function::ReadEast,
            Function::ReadNorth,
            Function::ReadSouth,
            Function::ReadWest,
            Function::AbsoluteRead,
        ] {
            let expected = function.signature().len();
            for found in [0, expected + 1, expected + 2] {
                let operands = vec![number; found];
                let resolved = address(function, &operands, 4);
                assert!(
                    matches!(
                        resolved,
                        Err(Error::Argument(crate::ArgumentError::Arity {
                            expected: e,
                            found: f,
                        })) if e == expected && f == found
                    ),
                    "{function:?} over {found} operands: {resolved:?}"
                );
            }
        }
    }

    #[test]
    fn a_static_input_portal_resolves_to_its_declaration() {
        for function in Function::ALL.iter().copied() {
            if let Some(InputPortal::Static(coords)) = function.input_portal() {
                for operand_columns in [0, 2, 6, usize::MAX] {
                    assert_eq!(
                        resolve(function, &[Atom::Number(1)], operand_columns).unwrap(),
                        coords,
                        "{function:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_dynamic_input_portal_resolves_to_the_pair_its_operands_select() {
        // `&t0103C4D4E4`: the operands end six columns east of the anchor and
        // select pair 1, so the Portal is `D4`, eight columns east.
        let track = |index, count, operand_columns| {
            resolve(
                Function::Track,
                &[Atom::Number(index), Atom::Number(count)],
                operand_columns,
            )
            .unwrap()
        };
        assert_eq!(
            track(1, 3, 6),
            PortalCoords {
                columns: 8,
                rows: 0
            }
        );
        assert_eq!(
            track(5, 3, 6),
            PortalCoords {
                columns: 10,
                rows: 0
            }
        );
        assert_eq!(
            track(0xFF, 0xFF, 4),
            PortalCoords {
                columns: 4,
                rows: 0
            }
        );
        assert_eq!(
            track(0xFE, 0xFF, 256),
            PortalCoords {
                columns: 764,
                rows: 0
            }
        );
    }

    #[test]
    fn a_read_resolves_to_the_pair_n_portals_from_its_operand_slot() {
        // The `n` operand's slot is the pair at column 2, after the spelling,
        // whatever the Function and its operands occupy east of it.
        let read = |function, n, operand_columns| {
            resolve(function, &[Atom::Number(n)], operand_columns).unwrap()
        };
        let at = |columns, rows| PortalCoords { columns, rows };
        for operand_columns in [4, 8, 256] {
            for function in [
                Function::ReadNorth,
                Function::ReadSouth,
                Function::ReadEast,
                Function::ReadWest,
            ] {
                assert_eq!(read(function, 0, operand_columns), at(2, 0), "{function:?}");
            }
            assert_eq!(read(Function::ReadEast, 1, operand_columns), at(4, 0));
            assert_eq!(read(Function::ReadEast, 0xFF, operand_columns), at(512, 0));
            assert_eq!(read(Function::ReadWest, 1, operand_columns), at(0, 0));
            assert_eq!(read(Function::ReadWest, 0xFF, operand_columns), at(-508, 0));
            assert_eq!(read(Function::ReadSouth, 1, operand_columns), at(2, 1));
            assert_eq!(read(Function::ReadSouth, 0xFF, operand_columns), at(2, 255));
            assert_eq!(read(Function::ReadNorth, 1, operand_columns), at(2, -1));
            assert_eq!(
                read(Function::ReadNorth, 0xFF, operand_columns),
                at(2, -255)
            );
        }
        let note = Atom::Note(Note::try_from(60).unwrap());
        assert!(matches!(
            resolve(Function::ReadEast, &[note], 4),
            Err(Error::Type(_))
        ));
    }

    #[test]
    fn a_write_selects_the_pair_the_read_with_the_same_arrow_selects() {
        // The address leads the Write's operands and its `value` follows it,
        // whatever that value is.
        let value = Atom::Note(Note::try_from(67).unwrap());
        let selected = |function: Function, operands: &[Atom], operand_columns| {
            function
                .dynamic_output_portal()
                .expect("a Write declares a dynamic Output Portal")
                .resolve(operands, operand_columns)
                .unwrap()
        };
        for (read, write) in [
            (Function::ReadEast, Function::WriteEast),
            (Function::ReadNorth, Function::WriteNorth),
            (Function::ReadSouth, Function::WriteSouth),
            (Function::ReadWest, Function::WriteWest),
        ] {
            for n in [0, 1, 2, 0xFF] {
                assert_eq!(
                    selected(write, &[Atom::Number(n), value], 6),
                    address(read, &[Atom::Number(n)], 4).unwrap(),
                    "{write:?} {n}"
                );
            }
        }
        for (column, row) in [(0, 0), (0x3A, 0x18), (0xFF, 0xFF)] {
            assert_eq!(
                selected(
                    Function::AbsoluteWrite,
                    &[Atom::Number(column), Atom::Number(row), value],
                    8
                ),
                address(
                    Function::AbsoluteRead,
                    &[Atom::Number(column), Atom::Number(row)],
                    6
                )
                .unwrap()
            );
        }
        let note = Atom::Note(Note::try_from(60).unwrap());
        assert!(matches!(
            Function::WriteEast
                .dynamic_output_portal()
                .unwrap()
                .resolve(&[note, value], 6),
            Err(Error::Type(_))
        ));
    }

    #[test]
    fn an_absolute_copy_reads_where_its_first_two_operands_point_and_writes_where_its_last_two_do()
    {
        // Each rule reads its own half of the four operands, and each half
        // addresses the Position `&$` and `@$` address with the same two.
        let copy = |operands: [u8; 4]| {
            let operands = operands.map(Atom::Number);
            let read = address(Function::AbsoluteCopy, &operands, 10).unwrap();
            let written = Function::AbsoluteCopy
                .dynamic_output_portal()
                .expect("the absolute Copy declares a dynamic Output Portal")
                .resolve(&operands, 10)
                .unwrap();
            (read, written)
        };
        let position = |column, row| PortalAddress::Position { column, row };
        assert_eq!(copy([0, 1, 6, 2]), (position(0, 1), position(6, 2)));
        assert_eq!(copy([6, 2, 0, 1]), (position(6, 2), position(0, 1)));
        assert_eq!(
            copy([0xFF, 0xFE, 0x3A, 0x18]),
            (position(0xFF, 0xFE), position(0x3A, 0x18))
        );
        assert_eq!(
            copy([0x3A, 0x18, 0, 0]).0,
            address(
                Function::AbsoluteRead,
                &[Atom::Number(0x3A), Atom::Number(0x18)],
                6
            )
            .unwrap()
        );
        // A Note in any of the four operands refuses both rules, whichever
        // half it falls in.
        let note = Atom::Note(Note::try_from(60).unwrap());
        for slot in 0..4 {
            let mut operands = [Atom::Number(0); 4];
            operands[slot] = note;
            assert!(
                matches!(
                    address(Function::AbsoluteCopy, &operands, 10),
                    Err(Error::Type(_))
                ),
                "{slot}"
            );
            assert!(
                matches!(
                    Function::AbsoluteCopy
                        .dynamic_output_portal()
                        .unwrap()
                        .resolve(&operands, 10),
                    Err(Error::Type(_))
                ),
                "{slot}"
            );
        }
    }

    #[test]
    fn push_selects_a_pair_of_the_lane_below_its_anchor() {
        // Pair 00 is the default Output Portal one row south, and `index %
        // count` steps east by pairs, wherever the operands end.
        let value = Atom::Note(Note::try_from(67).unwrap());
        let push = |index, count, operand_columns| {
            Function::Push
                .dynamic_output_portal()
                .expect("Push declares a dynamic Output Portal")
                .resolve(
                    &[Atom::Number(index), Atom::Number(count), value],
                    operand_columns,
                )
        };
        let at = |columns, rows| PortalAddress::Offset(PortalCoords { columns, rows });
        for operand_columns in [8, 12, 256] {
            assert_eq!(push(0, 3, operand_columns).unwrap(), at(0, 1));
            assert_eq!(push(1, 3, operand_columns).unwrap(), at(2, 1));
            assert_eq!(push(4, 3, operand_columns).unwrap(), at(2, 1));
            assert_eq!(push(0xFE, 0xFF, operand_columns).unwrap(), at(508, 1));
            assert_eq!(push(0xFF, 0xFF, operand_columns).unwrap(), at(0, 1));
        }
        // A zero count names Push, as Track's names Track.
        assert!(matches!(
            push(1, 0, 8),
            Err(Error::Interpretation(InterpretationError::ZeroWrap {
                function: Function::Push,
                role: "count",
            }))
        ));
        let note = Atom::Note(Note::try_from(60).unwrap());
        assert!(matches!(
            Function::Push
                .dynamic_output_portal()
                .unwrap()
                .resolve(&[note, Atom::Number(3), value], 8),
            Err(Error::Type(_))
        ));
    }

    #[test]
    fn a_dynamic_input_portal_diagnoses_its_operands_as_evaluation_does() {
        let evaluated = |operands: [Atom; 2]| {
            Interpreter::execute_function(
                Function::Track,
                operands,
                FunctionInputs::with_portal_source(
                    TickInputs::new(Tick::ZERO, Anchor::new(0, 0)),
                    PortalSource::from_cells(Some("D4")),
                ),
            )
            .unwrap_err()
            .to_string()
        };
        let note = Atom::Note(Note::try_from(60).unwrap());
        for operands in [
            [Atom::Number(1), Atom::Number(0)],
            [note, Atom::Number(3)],
            [Atom::Number(1), note],
        ] {
            assert_eq!(
                resolve(Function::Track, &operands, 6)
                    .unwrap_err()
                    .to_string(),
                evaluated(operands),
                "{operands:?}"
            );
        }
        assert!(matches!(
            resolve(Function::Track, &[Atom::Number(1), Atom::Number(0)], 6),
            Err(Error::Interpretation(InterpretationError::ZeroWrap {
                function: Function::Track,
                role: "count",
            }))
        ));
    }

    #[test]
    fn an_unrepresentable_offset_diagnoses_as_a_portal_outside_the_grid() {
        let past = |index, operand_columns| {
            resolve(
                Function::Track,
                &[Atom::Number(index), Atom::Number(3)],
                operand_columns,
            )
        };
        assert_eq!(
            past(0, usize::try_from(i16::MAX).unwrap()).unwrap(),
            PortalCoords {
                columns: i16::MAX,
                rows: 0
            }
        );
        for (index, operand_columns) in [
            (1, usize::try_from(i16::MAX).unwrap()),
            (0, usize::try_from(i16::MAX).unwrap() + 1),
            (2, usize::MAX),
        ] {
            assert!(
                matches!(
                    past(index, operand_columns),
                    Err(Error::Interpretation(InterpretationError::PartialInput {
                        function: Function::Track
                    }))
                ),
                "{operand_columns}"
            );
        }
    }
}
