//! The fixture the topic test modules drive Ticks through: Sources seeded from
//! stated rows, Ticks run with carried or stated Portal destinations, and the
//! readings of the plans they commit.

use crate::grid::{CellIndex, Grid};
use crate::source::{Diagnostic, TickPlan};
use lang::Atom;

use super::{Effect, Encoding, Portal, Tick, execution::ComputationState};

///
/// Builds one Source Snapshot from `rows`, padded to the Grid's width.
///
pub(super) fn snapshot(grid: Grid, rows: &[&str]) -> String {
    let width = grid.count() / rows.len();
    rows.iter().map(|row| format!("{row:width$}")).collect()
}

///
/// One Source built from `rows`, its Cells set one at a time as an editor
/// sets them.
///
/// Shared by every helper that runs a Tick against stated rows, so that
/// comparing two of them compares what they do differently and never two
/// ways of building a Source.
///
pub(super) fn seeded_source(grid: Grid, rows: &[&str]) -> crate::source::Source {
    let mut source = crate::source::Source::new(grid);
    for (index, byte) in snapshot(grid, rows).bytes().enumerate() {
        source
            .set(cell(grid, index), &char::from(byte).to_string())
            .unwrap();
    }
    source
}

///
/// Runs one Tick against `rows` with the Portal destinations `outputs`
/// states carried on its schedule, and commits its plan.
///
/// The plan-only half of the carried route, for the tests that state
/// their destinations through a fixture rather than calling
/// [`super::plan_carrying`] themselves.
///
pub(super) fn carried_source(
    grid: Grid,
    rows: &[&str],
    outputs: &[(usize, usize)],
) -> (TickPlan, crate::source::Source) {
    let (plan, _, source) = carried_tick(grid, rows, outputs);
    (plan, source)
}

///
/// [`carried_source`], with the inputs the Interpreter received for each
/// computation it ran for.
///
/// Two helpers rather than one because the two questions are asked by
/// different tests: most of this module asks only what a Tick planned, and
/// binding a fact they never read would cost every one of them a line.
///
pub(super) fn carried_tick(
    grid: Grid,
    rows: &[&str],
    outputs: &[(usize, usize)],
) -> (TickPlan, Vec<lang::TickInputs>, crate::source::Source) {
    let mut source = seeded_source(grid, rows);
    let (plan, states) = source.execute_carrying(Tick::ZERO, &carried_destinations(grid, outputs));
    (plan, interpreted(&states), source)
}

///
/// The explicit inputs the Interpreter received during one Tick, one entry
/// per call rather than per computation.
///
/// The three questions this module asks of an execution — how many
/// computations were interpreted, which inputs each of them received, and
/// how many times each of them ran — read off the Tick's own output rather
/// than off a record kept beside the thread it ran on.
///
/// Per call and not per computation because the third question is the one
/// with no other witness: a computation that runs twice is handed the same
/// inputs both times and writes the same value both times, so neither the
/// state's own slot nor the Source can tell it from one that ran once. A
/// state repeated [`ComputationState::interpretations`] times restores that
/// multiplicity.
///
/// The entries are grouped in the order the schedule holds its
/// computations, which is the order they were parsed rather than the order
/// their Turns were taken. One test compares that order — the one naming
/// three roots no dependency orders against each other — and reads it as
/// which computations ran rather than as evidence of Turn order.
///
pub(super) fn interpreted(states: &[ComputationState]) -> Vec<lang::TickInputs> {
    let mut calls = Vec::new();
    for state in states {
        if let Some(inputs) = state.interpreted() {
            calls.extend(std::iter::repeat_n(inputs, state.interpretations()));
        }
    }
    calls
}

///
/// Which Turn each computation took during one Tick, or `None` for a
/// computation its order holds no Turn for.
///
/// One entry per computation, in the order the schedule holds them — the
/// order they were parsed, which is the order [`interpreted`] groups its
/// entries by and not the order the Turns were taken. An ordering claim is
/// therefore read as "this computation took the earlier Turn" rather than
/// off a position in a list, and a test that asserts the whole answer at
/// once states an order no reordering of two computations survives.
///
pub(super) fn turns(states: &[ComputationState]) -> Vec<Option<usize>> {
    states.iter().map(ComputationState::turn).collect()
}

///
/// The fixed Portal destinations a fixture states, in the terms a carried
/// schedule takes them: one destination list per anchor, keyed by the
/// anchor's own Cell.
///
pub(super) fn carried_destinations(
    grid: Grid,
    outputs: &[(usize, usize)],
) -> std::collections::BTreeMap<CellIndex, Vec<crate::grid::Position>> {
    let mut carried: std::collections::BTreeMap<_, Vec<_>> = std::collections::BTreeMap::new();
    for (anchor, output) in outputs {
        carried
            .entry(cell(grid, *anchor))
            .or_default()
            .push(grid.position_at(cell(grid, *output)));
    }
    carried
}

///
/// The committed Source as one string per Grid row.
///
/// A Self-Banging Function is specified by where its Cells are, so a test
/// of one states a Grid and compares a Grid. Reading the same fact off a
/// list of Cell indices states the arithmetic instead of the geometry, and
/// an expectation nobody can picture is one nobody can check.
///
pub(super) fn rows_of(grid: Grid, source: &crate::source::Source) -> Vec<String> {
    source
        .snapshot()
        .into_bytes()
        .chunks(grid.columns())
        .map(|row| String::from_utf8(row.to_vec()).expect("ASCII Source"))
        .collect()
}

///
/// Runs `ticks` consecutive Ticks against `rows` through the production
/// path, answering the Grid after each one.
///
/// [`crate::source::Source::execute`] and not [`carried_source`]: a
/// Self-Banging Function resolves its own Portals from the offset it
/// declares, so a fixture that carried destinations for it would be
/// testing the fixture.
///
pub(super) fn tick_by_tick(
    grid: Grid,
    rows: &[&str],
    ticks: u64,
) -> (Vec<TickPlan>, Vec<Vec<String>>, crate::source::Source) {
    let mut source = seeded_source(grid, rows);
    let mut plans = Vec::new();
    let mut grids = Vec::new();
    for tick in 0..ticks {
        plans.push(source.execute(Tick::new(tick)));
        grids.push(rows_of(grid, &source));
    }
    (plans, grids, source)
}

///
/// The messages one Tick Plan diagnosed, so a test states what was said
/// rather than how many things were.
///
pub(super) fn messages(plan: &TickPlan) -> Vec<&str> {
    plan.diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect()
}

///
/// Where each of one Tick Plan's diagnostics sends a reader, with what it
/// says there.
///
pub(super) fn anchors(plan: &TickPlan) -> Vec<(usize, usize, &str)> {
    plan.diagnostics
        .iter()
        .map(|diagnostic| {
            let anchor = diagnostic.anchor();
            (anchor.x(), anchor.y(), diagnostic.message.as_str())
        })
        .collect()
}

///
/// Runs one Tick in which the computations at `answers` state the value
/// they answer rather than computing one, and commits its plan.
///
/// A test chooses answers independently of what its producers can compute,
/// as the [`stated`](super::execution::stated) module describes. The values
/// are delivered by `stated::plan_with_answers`, one call below the
/// planning entry point, so the Source, its destinations, the schedule and
/// every other computation's Turn are the production ones and nothing in
/// the shipped module compiles differently to admit the answer. The plan is
/// committed through `Source::commit_tick`, which is the commit a Tick gets
/// however it was planned, rather than through edits that imitate it.
///
pub(super) fn stated_source(
    grid: Grid,
    rows: &[&str],
    outputs: &[(usize, usize)],
    answers: &[(usize, Atom)],
) -> (TickPlan, crate::source::Source) {
    let (plan, _, source) = stated_tick(grid, rows, outputs, answers);
    (plan, source)
}

///
/// [`stated_source`], with what each computation's Turn did. Split from it
/// for the reason [`carried_tick`] gives.
///
/// The states themselves rather than one fact read off them, because two
/// facts are read off them here: [`interpreted`] asks what the Interpreter
/// was handed and [`turns`] asks which Turn each computation took, and a
/// fixture that answered one of them could not answer the other.
///
pub(super) fn stated_tick(
    grid: Grid,
    rows: &[&str],
    outputs: &[(usize, usize)],
    answers: &[(usize, Atom)],
) -> (TickPlan, Vec<ComputationState>, crate::source::Source) {
    let mut source = seeded_source(grid, rows);
    let answers: Vec<_> = answers
        .iter()
        .map(|(anchor, atom)| (cell(grid, *anchor), *atom))
        .collect();
    let bytes = source.snapshot();
    let (plan, portals, states) = super::execution::stated::plan_with_answers(
        grid,
        bytes.as_bytes(),
        &source.shared_language_map(),
        Tick::ZERO,
        &carried_destinations(grid, outputs),
        &answers,
    );
    source.commit_tick(&plan, portals);
    (plan, states, source)
}

///
/// The index `grid` mints for `idx`, so a test states an expected planned
/// write in the same terms a Tick Plan carries.
///
pub(super) fn cell(grid: Grid, idx: usize) -> CellIndex {
    grid.cell_index(idx).expect("inside the Grid")
}

///
/// A complete write of `content` starting at `idx`. One Effect covers the
/// whole run, which is the shape a producer emits, and it is admitted by a
/// Portal because that is the only way one comes into being.
///
pub(super) fn write(grid: Grid, idx: usize, content: &str) -> Effect {
    let destination = grid.position_at(grid.cell_index(idx).expect("inside the Grid"));
    Effect::Write(
        Portal::at(grid, destination)
            .admit(&Encoding::literal(content).expect("printable test content"))
            .expect("the encoding fits its row"),
    )
}

///
/// The Cells `plan` writes, as plain numbers and characters, so an
/// expected Source row reads as one.
///
pub(super) fn planned(plan: &TickPlan) -> Vec<(usize, char)> {
    plan.writes
        .iter()
        .map(|write| (write.cell.get(), write.content.as_char()))
        .collect()
}

pub(super) fn diagnostic(grid: Grid, start: usize, end: usize, message: &str) -> Effect {
    let cell = |idx: usize| grid.cell_index(idx).expect("inside the Grid");
    Effect::Diagnose(Diagnostic::for_range(
        grid,
        cell(start),
        cell(end),
        message.to_string(),
    ))
}
