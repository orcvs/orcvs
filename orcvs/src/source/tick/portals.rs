//! Tick Portals: every Portal access one Tick's Turns made, in the order they
//! made them.
//!
//! The record is an observation of a committed Tick, not a Portal: a Portal is
//! a local of the Turn that resolved it, and what survives here is what that
//! Turn did with it. Execution is the only writer, so each entry is built from
//! the facts the Turn held as it acted — the schedule's anchor and the
//! Function the Turn was running — and never recovered from the Source the
//! Tick committed, which the Tick may have rewritten.

use lang::{Function, ReplacementChange, Tick};

use super::{Portal, PortalError, Span, SpanWrite};
use crate::grid::Position;

///
/// The ordered, immutable record of every Portal access one Tick made.
///
/// Order is the order execution made the accesses: Turns in the order they
/// settled, and within a Turn its read, then its writes in site order, its
/// lock, or the parts of its Source Bundle in the order they ran. A Turn that
/// waited for a writer recorded nothing until it was retaken, and a Turn
/// refused before interpretation records nothing at all.
///
#[derive(Clone, Debug, PartialEq)]
pub struct TickPortals {
    tick: Tick,
    uses: Vec<PortalUse>,
}

impl TickPortals {
    /// The record a Tick at `tick` starts with: no access yet, and room for
    /// `capacity` before it grows.
    pub(super) fn new(tick: Tick, capacity: usize) -> Self {
        Self {
            tick,
            uses: Vec::with_capacity(capacity),
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
    /// One delivery the Turn attempted at one Output Portal site: the Cells
    /// it was admitted to, or why it was refused whole. A destination that
    /// resolved for a Turn whose interpretation failed was never delivered
    /// to, so it has no entry.
    Write {
        origin: PortalOrigin,
        outcome: Result<OutputSpan, WriteRefusal>,
    },
    /// A Halt's lock at the site it declares, or why the site is outside the
    /// Grid, with what the lock did there.
    Lock {
        site: Result<Position, PortalError>,
        verdict: LockVerdict,
    },
    /// One part of a Source Bundle: the Cells it wrote, or why its
    /// destination was refused.
    Bundle {
        part: BundlePart,
        outcome: Result<OutputSpan, BundleRefusal>,
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
/// Why one delivery at an Output Portal site was refused whole.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WriteRefusal {
    /// The site gave no destination for the encoding. `at` is the Position
    /// the write was attempted from, and `None` where no Position in the
    /// Grid answers the site.
    Portal {
        error: PortalError,
        at: Option<Position>,
    },
    /// The write would replace a contacted Function with one that differs
    /// from it in a fact the schedule was ordered by.
    Replacement(ReplacementChange),
    /// The schedule ordered the write first, and it reached a computation
    /// that had already taken its Turn.
    OrderingDefect,
}

///
/// What a Halt's lock did, as the Tick decided it.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LockVerdict {
    /// The Expression root anchored here is withheld for the Tick.
    Locked(Position),
    /// The root anchored here had taken its Turn, and the lock missed it.
    Missed(Position),
    /// The schedule ordered the lock first, and the locked subtree had
    /// already taken a Turn.
    OrderingDefect,
    /// Something other than an Expression root stands at the site.
    NotARoot,
    /// The site is outside the Grid or empty, so nothing is locked.
    Unchanged,
}

///
/// Which part of a Source Bundle an entry records.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BundlePart {
    /// The producer's own Span, cleared by a move.
    Vacate,
    /// The displaced destination a move or an emission writes.
    Place,
    /// The `**` a refused move leaves in the producer's own Span.
    LeftBang,
    /// The clear a Bang Function makes of its own Span.
    Fire,
}

///
/// Why a Source Bundle's displaced destination was refused.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BundleRefusal {
    /// The destination gave no room for the spelling. `at` is the Position
    /// it was attempted from, and `None` where no Position in the Grid
    /// answers the displacement.
    Portal {
        error: PortalError,
        at: Option<Position>,
    },
    /// The destination resolved, and Cells it needed were not vacant.
    Occupied(OutputSpan),
}

///
/// The Cells a Turn's Input Portal read.
///
/// Built only from the Input Portal a Turn resolved, so no other Span can
/// stand for one, and it does not convert into an [`OutputSpan`].
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

///
/// The Cells a write a Portal admitted covers.
///
/// Built only from a write a Portal admitted, so no other Span can stand for
/// one, and it does not convert into an [`InputSpan`]. A Source Bundle's
/// destination is admitted before its vacancy is asked, so an occupied one
/// still names its Cells with one.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OutputSpan(Span);

impl OutputSpan {
    /// The Cells `write` covers.
    pub(super) fn of(write: &SpanWrite) -> Self {
        Self(write.span())
    }

    /// The Cells written.
    pub fn span(&self) -> Span {
        self.0
    }
}
