//! Declared Portal inputs, resolved from working Source by the caller at Turn.
//!
//! Cell operands live on the Operand Stack; Portal inputs are the other half of
//! what a Function reads at Turn. [`FunctionInputs`] is the one bundle the
//! Evaluator receives beside resolved cell operands.

use crate::{
    Atom, Error, Function, InputPortal, InterpretationError, PairSelection, PortalCoords, Stack,
    TickInputs, Token, expression::DEFAULT_TOKEN_LEN, functions::track::track_pair,
    stack::Operands,
};

impl InputPortal {
    ///
    /// This Input Portal's position from its Function's anchor at the Turn.
    ///
    /// `operands` are the values the Turn resolved, and `operand_columns` is
    /// how many columns east of the anchor the Function and its operands
    /// occupy, nested operands included: `orcvs` knows how they lie in the
    /// Grid. A static Input Portal is the offset it declares and reads
    /// neither. A dynamic Input Portal is the Cell pair its operands select
    /// by its [`PairSelection`], counted from `operand_columns`.
    ///
    /// # Errors
    ///
    /// A dynamic Input Portal diagnoses its operands as evaluation does,
    /// including a `count` of `00` as a wrap by zero. An offset too far east
    /// to represent is a Portal outside any Grid, and diagnoses as partial or
    /// invalid input, as a Portal past the row edge does.
    ///
    pub fn resolve(self, operands: &[Atom], operand_columns: usize) -> Result<PortalCoords, Error> {
        match self {
            Self::Static(coords) => Ok(coords),
            Self::Dynamic(selection) => {
                let (east, rows) = selection.offset(operands)?;
                let columns = operand_columns
                    .checked_add(east)
                    .and_then(|columns| i16::try_from(columns).ok())
                    .ok_or(InterpretationError::CopyInput {
                        function: selection.function(),
                    })?;
                Ok(PortalCoords {
                    columns,
                    rows: i16::from(rows),
                })
            }
        }
    }
}

impl PairSelection {
    /// The Function whose definition names this rule, which a refused
    /// resolution diagnoses.
    const fn function(self) -> Function {
        match self {
            Self::IndexModuloCount => Function::Track,
        }
    }

    /// The selected pair's offset from the end of the last operand: Cells
    /// east, and rows south.
    fn offset(self, operands: &[Atom]) -> Result<(usize, u8), Error> {
        match self {
            Self::IndexModuloCount => {
                track_pair(operands).map(|pair| (usize::from(pair) * DEFAULT_TOKEN_LEN, 0))
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
        Interpreter, Note, PortalCoords, Tick, TickInputs,
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
    fn resolve(
        function: Function,
        operands: &[Atom],
        operand_columns: usize,
    ) -> Result<PortalCoords, Error> {
        function
            .input_portal()
            .expect("the Function declares an Input Portal")
            .resolve(operands, operand_columns)
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
                    Err(Error::Interpretation(InterpretationError::CopyInput {
                        function: Function::Track
                    }))
                ),
                "{operand_columns}"
            );
        }
    }
}
