//! Declared Portal inputs, resolved from working Source by the caller at Turn.
//!
//! Cell operands live on the Operand Stack; Portal inputs are the other half of
//! what a Function reads at Turn. [`FunctionInputs`] is the one bundle the
//! Evaluator receives beside resolved cell operands.

use crate::{
    Atom, Error, Function, InterpretationError, Stack, TickInputs, Token, stack::Operands,
};

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
pub(crate) trait PortalOperands: Operands<Binding = crate::stack::ElementBinding> {
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
    use crate::{Error, Function, InterpretationError};

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
}
