//! Declared Portal inputs, resolved from working Source by the caller at Turn.
//!
//! Cell operands live on the Operand Stack; Portal inputs are the other half of
//! what a Function reads at Turn. [`FunctionInputs`] is the one bundle the
//! Evaluator receives beside resolved cell operands.

use crate::{
    Atom, Error, Function, InterpretationError, Stack, TickInputs, Token, stack::Operands,
};

/// The site a Function reads. Grid geometry remains the Source owner's concern.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortalSite {
    OrdinaryResult,
}

/// One Portal input declaration, beside the Function's cell operands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PortalInput {
    site: PortalSite,
    token: Token,
    role: &'static str,
}

impl PortalInput {
    pub(crate) const fn ordinary_result_number(role: &'static str) -> Self {
        Self {
            site: PortalSite::OrdinaryResult,
            token: Token::Number,
            role,
        }
    }

    pub const fn site(self) -> PortalSite {
        self.site
    }
    pub const fn token(self) -> Token {
        self.token
    }
    pub const fn role(self) -> &'static str {
        self.role
    }
}

/// Spellings borrowed from working Source for declared Portal inputs.
///
/// One field per [`PortalSite`] today; extend here when `@<` and displaced
/// reads arrive rather than adding parallel parameters at the Turn seam.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PortalSpellings<'a> {
    ordinary_result: Option<&'a str>,
}

impl<'a> PortalSpellings<'a> {
    /// No Portal Cells were supplied.
    #[inline]
    #[must_use]
    pub const fn none() -> Self {
        Self {
            ordinary_result: None,
        }
    }

    /// The spelling at the ordinary result Portal, when the site resolved.
    #[inline]
    #[must_use]
    pub const fn ordinary_result(spelling: Option<&'a str>) -> Self {
        Self {
            ordinary_result: spelling,
        }
    }

    #[inline]
    #[must_use]
    pub fn spelling(self, site: PortalSite) -> Option<&'a str> {
        match site {
            PortalSite::OrdinaryResult => self.ordinary_result,
        }
    }
}

/// Everything one Function evaluation reads beyond its cell operands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FunctionInputs<'a> {
    tick: TickInputs,
    portals: PortalSpellings<'a>,
}

impl<'a> FunctionInputs<'a> {
    /// Playback Tick and anchor only; no Portal Cells.
    #[inline]
    #[must_use]
    pub fn new(tick: TickInputs) -> Self {
        Self {
            tick,
            portals: PortalSpellings::none(),
        }
    }

    /// Tick, anchor, and borrowed Portal spellings from working Source.
    #[inline]
    #[must_use]
    pub const fn with_portals(tick: TickInputs, portals: PortalSpellings<'a>) -> Self {
        Self { tick, portals }
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

    /// Borrowed Portal spellings supplied by the Turn.
    #[inline]
    #[must_use]
    pub const fn portals(self) -> PortalSpellings<'a> {
        self.portals
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
/// formulas from accepting an unchecked spelling.
pub(crate) struct PortalNumber(u8);

impl PortalNumber {
    pub(crate) fn bind(
        function: Function,
        input: PortalInput,
        spellings: PortalSpellings<'_>,
    ) -> Result<Self, Error> {
        let invalid = || InterpretationError::PortalInputNotNumber {
            function,
            role: input.role(),
        };
        let spelling = spellings.spelling(input.site()).ok_or_else(invalid)?;
        if spelling.len() == input.token().len() && spelling.bytes().all(|cell| cell == b' ') {
            return Ok(Self(0));
        }
        match input.token().decode(spelling) {
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
    spellings: PortalSpellings<'_>,
) -> Result<(O, PortalNumber), Error> {
    let operands = stack.extract::<O>()?;
    let portal = PortalNumber::bind(O::FUNCTION, O::PORTAL, spellings)?;
    Ok((operands, portal))
}

#[cfg(test)]
mod test {
    use super::{PortalInput, PortalNumber, PortalSpellings};
    use crate::{Error, Function, InterpretationError};

    const INPUT: PortalInput = PortalInput::ordinary_result_number("previous value");

    #[test]
    fn empty_portal_cells_initialize_as_number_zero() {
        assert_eq!(
            PortalNumber::bind(
                Function::Increment,
                INPUT,
                PortalSpellings::ordinary_result(Some("  "))
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
                PortalSpellings::ordinary_result(Some("0A"))
            )
            .unwrap()
            .number(),
            0x0A
        );
    }

    #[test]
    fn a_missing_portal_site_diagnoses() {
        assert!(matches!(
            PortalNumber::bind(Function::Increment, INPUT, PortalSpellings::none()),
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
                PortalSpellings::ordinary_result(Some("G4"))
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
