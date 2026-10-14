//! The scheduling inputs of a Language Map revision, borrowed from its rows.
//!
//! A schedule is shared by every revision whose scheduling inputs are equal,
//! so the facts schedule construction reads and the facts that equality
//! compares have to be the same facts. This module is where both come from:
//! construction reads a revision through [`ScheduleInputs`] and nothing else,
//! and reuse compares rows through the [`PartialEq`] of what this module
//! projects. A fact that construction starts reading is a fact equality
//! compares, because there is no second description of either.
//!
//! The projection borrows the Parser structure each row already holds. It
//! stores nothing and allocates nothing: an Operand Literal's value, a Token,
//! a Parser diagnostic and the Source bytes are never part of it.

use std::ops::Range;
use std::sync::Arc;

use lang::{Atom, Expression, Function};

use crate::grid::Position;
use crate::source::language_map::{DerivedRow, Span};

///
/// Everything a schedule reads of one Language Map revision.
///
#[derive(Clone, Copy)]
pub(in crate::source) struct ScheduleInputs<'a> {
    rows: &'a [Arc<DerivedRow>],
}

impl<'a> ScheduleInputs<'a> {
    pub(in crate::source) fn new(rows: &'a [Arc<DerivedRow>]) -> Self {
        Self { rows }
    }

    /// Every Expression a schedule computes, in row and then Source order.
    pub(super) fn expressions(self) -> impl Iterator<Item = ScheduledExpression<'a>> {
        self.rows.iter().flat_map(|row| row.scheduled())
    }
}

///
/// One Expression a schedule computes: those with a leading Function.
///
/// Its scheduling facts are its Span, its leading Function and where that
/// Function stands, and for every positioned entry its Cells, its parent and
/// how it parsed. The Function identities decide Portal access, activation and
/// reservations, so every Portal relationship; the Cells decide claims and the
/// row edge diagnostic; whether an entry parsed decides syntax blocking.
///
#[derive(Clone, Copy)]
pub(in crate::source) struct ScheduledExpression<'a> {
    span: Span,
    leading: (Position, Function),
    expression: &'a Expression,
}

impl<'a> ScheduledExpression<'a> {
    ///
    /// The scheduling facts of the Expression `expression` parsed as, which
    /// `span` covers and `leading` begins, or none when it has no leading
    /// Function and no schedule computes it.
    ///
    pub(in crate::source) fn of(
        span: Span,
        leading: Option<(Position, Function)>,
        expression: &'a Expression,
    ) -> Option<Self> {
        Some(Self {
            span,
            leading: leading?,
            expression,
        })
    }

    pub(super) fn span(self) -> Span {
        self.span
    }

    /// Every positioned entry, in Parser preorder.
    pub(super) fn entries(self) -> impl Iterator<Item = ScheduledEntry> + 'a {
        self.expression.positioned().map(|entry| ScheduledEntry {
            cells: entry.cells.clone(),
            parent: entry.parent,
            parsed: Parsed::of(entry.atom.as_ref()),
        })
    }
}

impl PartialEq for ScheduledExpression<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.span == other.span
            && self.leading == other.leading
            && self.entries().eq(other.entries())
    }
}

///
/// One positioned entry as a schedule reads it: its Cells, the index of its
/// parent entry within the same Expression, and how it parsed.
///
#[derive(Clone, PartialEq)]
pub(super) struct ScheduledEntry {
    pub(super) cells: Range<usize>,
    pub(super) parent: Option<usize>,
    pub(super) parsed: Parsed,
}

/// How one positioned entry parsed, without any value it parsed to.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Parsed {
    /// Its Cells bound no Atom.
    Unparsed,
    /// It bound an Atom that is not a Function: an Operand Literal or a Bang.
    Value,
    Function(Function),
}

impl Parsed {
    fn of(atom: Option<&Atom>) -> Self {
        match atom {
            None => Self::Unparsed,
            Some(Atom::Function(function)) => Self::Function(*function),
            Some(_) => Self::Value,
        }
    }
}
