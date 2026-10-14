//! The scheduling inputs of a Language Map revision, borrowed from its rows.
//!
//! A schedule is shared by every revision whose scheduling inputs are equal,
//! so schedule construction must read nothing that equality does not compare.
//! This module is where both come from: construction reads a revision through
//! [`ScheduleInputs`] and nothing else, and reuse compares rows through the
//! [`PartialEq`] of what this module projects. Each projection's accessors
//! read only the facts its `PartialEq` compares: a fact construction needs is
//! added as an accessor here and to that `PartialEq` together, never read
//! from the borrowed Parser structure anywhere else.
//!
//! The projection borrows the Parser structure each row already holds. It
//! stores nothing and allocates nothing: an Operand Literal's value, a Token,
//! a Parser diagnostic and the Source bytes are never part of it.

use std::ops::Range;
use std::sync::Arc;

use lang::{Atom, Expression, Function};

use crate::grid::Position;
use crate::source::language_map::{DerivedRow, LanguageUnit, Span};

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

    fn rows(self) -> impl Iterator<Item = RowInputs<'a>> {
        self.rows.iter().map(|row| RowInputs::of(row))
    }

    /// Every Expression a schedule computes, in row and then Source order.
    pub(super) fn expressions(self) -> impl Iterator<Item = ScheduledExpression<'a>> {
        self.rows().flat_map(RowInputs::expressions)
    }

    /// Every Language Unit, in row and then anchor order.
    pub(super) fn units(self) -> impl Iterator<Item = &'a LanguageUnit> {
        self.rows().flat_map(RowInputs::units)
    }
}

///
/// What a schedule reads of one row: every Language Unit, and the Expressions
/// a schedule computes.
///
/// A Language Unit is read whole. Its anchor and Span decide the occupancy a
/// Halt target is classified by, and its kind is compared too, so an edit
/// that changes only what kind of unit occupies some Cells orders a new
/// schedule. A unit records no value.
///
#[derive(Clone, Copy)]
pub(in crate::source) struct RowInputs<'a> {
    row: &'a DerivedRow,
}

impl<'a> RowInputs<'a> {
    pub(in crate::source) fn of(row: &'a DerivedRow) -> Self {
        Self { row }
    }

    fn units(self) -> &'a [LanguageUnit] {
        self.row.units()
    }

    fn expressions(self) -> impl Iterator<Item = ScheduledExpression<'a>> {
        self.row.scheduled()
    }
}

impl PartialEq for RowInputs<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.units() == other.units() && self.expressions().eq(other.expressions())
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

/// Compares every fact an accessor above reads. An accessor reading more of
/// `expression` than [`Self::entries`] does widens this comparison with it.
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
    /// It bound an Atom that is not a Function.
    Value,
    /// It bound this Function.
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
