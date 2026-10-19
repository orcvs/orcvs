//! Tick Portals: every Portal access one Tick's Turns made, in the order they
//! made them.
//!
//! The record is an observation of a committed Tick, not a Portal: a Portal is
//! a local of the Turn that resolved it, and what survives here is what that
//! Turn did with it. Execution is the only writer, so each entry is built from
//! the facts the Turn held as it acted — the schedule's anchor and the
//! Function the Turn was running — and never recovered from the Source the
//! Tick committed, which the Tick may have rewritten.

use lang::{Function, Tick};

use super::{Portal, PortalError, Span};
use crate::grid::Position;

///
/// The ordered, immutable record of every Portal access one Tick made.
///
/// Order is the order execution made the accesses: Turns in the order they
/// settled, and within a Turn its read first. A Turn that waited for a writer
/// recorded nothing until it was retaken, and a Turn refused before
/// interpretation records nothing at all.
///
#[derive(Clone, Debug, PartialEq)]
pub struct TickPortals {
    tick: Tick,
    uses: Vec<PortalUse>,
}

impl TickPortals {
    /// The record a Tick at `tick` starts with: no access yet.
    pub(in crate::source) fn new(tick: Tick) -> Self {
        Self {
            tick,
            uses: Vec::new(),
        }
    }

    /// Appends the next access of this Tick.
    pub(super) fn push(&mut self, owner: Owner, event: PortalEvent) {
        self.uses.push(PortalUse {
            anchor: owner.anchor,
            function: owner.function,
            event,
        });
    }

    /// The Tick this record observed.
    pub fn tick(&self) -> Tick {
        self.tick
    }

    /// Every access, in the order execution made them.
    pub fn uses(&self) -> &[PortalUse] {
        &self.uses
    }
}

/// The Function whose Turn made an access, as execution held it.
#[derive(Clone, Copy)]
pub(super) struct Owner {
    pub(super) anchor: Position,
    pub(super) function: Function,
}

///
/// One Portal access: whose Turn made it, and what happened.
///
#[derive(Clone, Debug, PartialEq)]
pub struct PortalUse {
    anchor: Position,
    function: Function,
    event: PortalEvent,
}

impl PortalUse {
    /// The anchor of the Function whose Turn made this access.
    pub fn anchor(&self) -> Position {
        self.anchor
    }

    /// The Function that Turn executed, after any Function Replacement the
    /// Tick made before it.
    pub fn function(&self) -> Function {
        self.function
    }

    /// What the access was, and how it ended.
    pub fn event(&self) -> &PortalEvent {
        &self.event
    }
}

///
/// What one Portal access was, and how it ended.
///
#[derive(Clone, Debug, PartialEq)]
pub enum PortalEvent {
    /// The Input Portal handed to the Interpreter for a Turn that got past
    /// the writer wait. A resolved site says Cells were read there, not that
    /// they held a value the Function could bind: that is the Turn's
    /// diagnostic.
    Read {
        origin: PortalOrigin,
        site: Result<InputSpan, PortalError>,
    },
}

/// Whether a Portal sits where its Function declares it, or where its
/// operands chose it this Turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortalOrigin {
    /// At a fixed offset the Function declares.
    Declared,
    /// Where the Function's operands selected it this Turn.
    Selected,
}

///
/// The Cells a Turn's Input Portal read.
///
/// Built only from the Input Portal a Turn resolved, so no other Span can
/// stand for one.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputSpan(Span);

impl InputSpan {
    /// The `width` Cells the Input Portal `portal` reads, or why the row edge
    /// cuts them short.
    pub(super) fn read(portal: Portal, width: usize) -> Result<Self, PortalError> {
        portal.span(width).map(Self)
    }

    /// The Cells read.
    pub fn span(&self) -> Span {
        self.0
    }
}
