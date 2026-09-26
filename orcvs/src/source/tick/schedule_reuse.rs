//! Tests of the schedule a Language Map shares with every revision holding
//! the same scheduling inputs.
//!
//! Each change a scheduling input can undergo is made between two Ticks, and
//! the Tick planned through the shared schedule is compared with one planned
//! against a schedule ordered afresh from the same Map: the Tick Plan, its
//! diagnostics, and which Turn every computation took. Where a change is one
//! a stale schedule would plan differently, the test says so, so a key too
//! coarse to see that change fails here rather than in a pattern.

use lang::Tick;

use super::execution::{self, ComputationState};
use super::{plan, plan_unshared};
use crate::grid::{CellIndex, Grid};
use crate::source::{CellContent, CellWrite, Cells, LanguageMap, Source, TickPlan};

fn cell(grid: Grid, idx: usize) -> CellIndex {
    grid.cell_index(idx).expect("inside the Grid")
}

/// A Source holding `rows`, each padded to the Grid's width, written as one
/// revision.
fn source_of(grid: Grid, rows: &[&str]) -> Source {
    let mut source = Source::new(grid);
    write_rows(&mut source, rows);
    source
}

/// Writes every Cell of `rows`, spaces included, as one revision: a row
/// whose bytes are unchanged is still written and re-derived.
fn write_rows(source: &mut Source, rows: &[&str]) {
    let grid = source.grid();
    let width = grid.columns();
    let text: String = rows.iter().map(|row| format!("{row:width$}")).collect();
    assert_eq!(text.len(), grid.count(), "the rows fill the Grid");
    let writes: Vec<_> = text
        .bytes()
        .enumerate()
        .map(|(idx, byte)| CellWrite {
            cell: cell(grid, idx),
            content: CellContent::new(byte).expect("printable test Source"),
        })
        .collect();
    source.write_cells(&writes);
}

fn turns(states: &[ComputationState]) -> Vec<Option<usize>> {
    states.iter().map(ComputationState::turn).collect()
}

///
/// Plans one Tick through the Map's shared schedule, asserts it agrees with
/// the Tick planned against a schedule ordered afresh from the same Map, and
/// commits it.
///
fn agreeing_tick(source: &mut Source, tick: u64) -> TickPlan {
    let grid = source.grid();
    let map = source.shared_language_map();
    let bytes = source.snapshot();
    let (shared, shared_states) = plan(grid, Cells::of(bytes.as_bytes()), &map, Tick::new(tick));
    let (fresh, fresh_states) =
        plan_unshared(grid, Cells::of(bytes.as_bytes()), &map, Tick::new(tick));
    assert_eq!(
        shared, fresh,
        "the shared schedule planned Tick {tick} differently"
    );
    assert_eq!(
        turns(&shared_states),
        turns(&fresh_states),
        "the shared schedule ordered Tick {tick} differently"
    );
    source.commit_tick(&shared);
    shared
}

///
/// The Tick `source` plans against the schedule `earlier` ordered, which is
/// what a key that missed the change between the two revisions would plan.
///
fn stale_tick(source: &Source, earlier: &LanguageMap, tick: u64) -> TickPlan {
    let grid = source.grid();
    let bytes = source.snapshot();
    let current = source.shared_language_map();
    match earlier.schedule_cache().schedule(grid, earlier) {
        Ok(schedule) => {
            execution::execute(
                grid,
                Cells::of(bytes.as_bytes()),
                &current,
                Tick::new(tick),
                schedule,
            )
            .0
        }
        Err(diagnostics) => super::unscheduled(diagnostics.clone()).0,
    }
}

/// Writes `text` from Column `x` of Row `y` onward, for each edit, as one
/// revision: only the rows those Cells are in are re-derived.
fn write_at(source: &mut Source, edits: &[(usize, usize, &str)]) {
    let grid = source.grid();
    let writes: Vec<_> = edits
        .iter()
        .flat_map(|&(x, y, text)| {
            text.bytes()
                .enumerate()
                .map(move |(offset, byte)| CellWrite {
                    cell: grid.index(grid.position(x + offset, y).expect("inside the Grid")),
                    content: CellContent::new(byte).expect("printable test Source"),
                })
        })
        .collect();
    source.write_cells(&writes);
}

///
/// Runs one Tick against `rows`, makes `edits`, and runs the next Tick,
/// requiring both to agree with fresh planning. Answers whether the revision
/// after the edits shares the schedule the one before them held, the second
/// Tick's plan, and the plan the earlier schedule would have given it.
///
fn edited_between_ticks(
    grid: Grid,
    rows: &[&str],
    edits: &[(usize, usize, &str)],
) -> (bool, TickPlan, TickPlan) {
    let mut source = source_of(grid, rows);
    agreeing_tick(&mut source, 0);
    let earlier = source.shared_language_map();
    write_at(&mut source, edits);
    let shared = source
        .language_map()
        .schedule_cache()
        .is_shared_with(earlier.schedule_cache());
    let stale = stale_tick(&source, &earlier, 1);
    let planned = agreeing_tick(&mut source, 1);
    (shared, planned, stale)
}

#[test]
fn changing_a_function_orders_a_new_schedule() {
    // The same Expression shape and the same Cells, and a different Function:
    // what the root computes is read from the schedule's computation.
    let (shared, planned, stale) = edited_between_ticks(
        Grid::with_shape(8, 2),
        &[".+0304  ", "        "],
        &[(1, 0, "x")],
    );
    assert!(!shared);
    assert_ne!(planned, stale);
}

#[test]
fn changing_a_portal_relationship_orders_a_new_schedule() {
    // The Equality's Bang lands one row south of it, directly north of the
    // Terminal Output root, which ADR 0006 activates. Moving that root two
    // columns east leaves it no cardinal neighbour of the Bang, so no Bang
    // delivers to it: the same Functions, related differently through the
    // Portal.
    let (shared, planned, stale) = edited_between_ticks(
        Grid::with_shape(12, 3),
        &[".=0101      ", "            ", "!>010AC4    "],
        &[(0, 2, "  !>010AC4  ")],
    );
    assert!(!shared);
    assert!(planned.play_commands.is_empty(), "{planned:?}");
    assert_ne!(planned, stale);
}

#[test]
fn changing_occupancy_alone_orders_a_new_schedule() {
    // No computation changes. A Comment written into the empty Cells Halt
    // names turns its target from empty into an occupied non-root, which
    // Halt diagnoses; the schedule classified that target, so a key that
    // compared only Expressions would plan the empty target again.
    let (shared, planned, stale) = edited_between_ticks(
        Grid::with_shape(8, 3),
        &[".=0101  ", "  *!    ", "        "],
        &[(2, 2, "||")],
    );
    assert!(!shared);
    assert!(!planned.diagnostics.is_empty(), "{planned:?}");
    assert_ne!(planned, stale);
}

#[test]
fn changing_operand_syntax_orders_a_new_schedule() {
    // An operand that no longer parses blocks the Function's Turn. The Cells
    // the Expression claims are the same; whether its operands parse is what
    // changed, and the Language Unit they formed goes with it.
    let (shared, planned, stale) = edited_between_ticks(
        Grid::with_shape(8, 2),
        &[".+0304  ", "        "],
        &[(4, 0, "XY")],
    );
    assert!(!shared);
    assert!(planned.writes.is_empty(), "{planned:?}");
    assert_ne!(planned, stale);
}

#[test]
fn an_expression_cut_by_the_row_edge_orders_a_new_schedule() {
    // The row edge diagnostic is the schedule's own, stated before any Turn.
    let (shared, planned, _) = edited_between_ticks(
        Grid::with_shape(8, 2),
        &["  .+0304", "        "],
        &[(0, 0, "   .+030")],
    );
    assert!(!shared);
    assert!(
        planned
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "Expression layout crosses the row edge"),
        "{planned:?}"
    );
}

#[test]
fn a_commit_that_writes_nothing_keeps_the_schedule() {
    let grid = Grid::with_shape(8, 2);
    let mut source = source_of(grid, &[".+0304  ", "        "]);
    agreeing_tick(&mut source, 0);
    let earlier = source.shared_language_map();
    assert!(earlier.schedule_cache().is_filled());

    source.write_cells(&[]);

    let cache = source.language_map().schedule_cache();
    assert!(cache.is_shared_with(earlier.schedule_cache()));
    assert!(cache.is_filled(), "the schedule is not ordered again");
    agreeing_tick(&mut source, 1);
}

#[test]
fn rewriting_identical_bytes_keeps_the_schedule() {
    // Every row is written with the bytes it already holds, so every row is
    // re-derived and the Map is a new revision with a new identity. The
    // scheduling inputs are the same. The first Tick writes the Bang that
    // activates the Halt, which is an occupancy change; the second rewrites
    // that Bang where it stands and plans against the settled inputs.
    let grid = Grid::with_shape(8, 3);
    let mut source = source_of(grid, &[".=0101  ", "  *!    ", "  .+0304"]);
    agreeing_tick(&mut source, 0);
    agreeing_tick(&mut source, 1);
    let earlier = source.shared_language_map();
    assert!(earlier.schedule_cache().is_filled());
    let held = source.snapshot();
    let rows: Vec<_> = held.as_bytes().chunks(grid.columns()).collect();
    assert_eq!(
        rows[1], b"***!    ",
        "the Equality's Bang activated the Halt"
    );

    write_rows(
        &mut source,
        &rows
            .iter()
            .map(|row| std::str::from_utf8(row).expect("ASCII Source"))
            .collect::<Vec<_>>(),
    );

    assert!(!std::ptr::eq(source.language_map(), &*earlier));
    let cache = source.language_map().schedule_cache();
    assert!(cache.is_shared_with(earlier.schedule_cache()));
    assert!(cache.is_filled(), "the schedule is not ordered again");
    agreeing_tick(&mut source, 2);
}

#[test]
fn ticks_that_write_new_values_without_changing_an_expression_keep_the_schedule() {
    // The Clock writes a different value under itself on every Tick, which
    // re-derives that row each time. The value is an Operand Literal's
    // content, and no scheduling input reads it. The first Tick writes into
    // empty Cells, which is an occupancy change; every Tick after it is the
    // steady state.
    let grid = Grid::with_shape(8, 2);
    let mut source = source_of(grid, &["~.0110  ", "        "]);
    agreeing_tick(&mut source, 0);
    let settled = source.shared_language_map();
    let mut written = Vec::new();
    for tick in 1..6 {
        agreeing_tick(&mut source, tick);
        written.push(source.snapshot()[8..10].to_owned());
        let cache = source.language_map().schedule_cache();
        assert!(
            cache.is_shared_with(settled.schedule_cache()),
            "Tick {tick}"
        );
    }
    written.dedup();
    assert!(written.len() > 1, "the Clock wrote {written:?}");
}

#[test]
fn a_map_derived_afresh_orders_its_own_schedule() {
    let grid = Grid::with_shape(8, 2);
    let text = ".+0304          ";
    let first = LanguageMap::derive(grid, text).expect("a Source revision");
    let second = LanguageMap::derive(grid, text).expect("a Source revision");
    assert!(
        !first
            .schedule_cache()
            .is_shared_with(second.schedule_cache())
    );
}

///
/// Generated revisions, each written whole between Ticks so that unchanged
/// rows are re-derived as often as changed ones, over the agreement every
/// example above checks.
///
/// The `cfg` matches the `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]`
/// table that declares proptest, so a WASM build never sees the dependency.
///
#[cfg(not(target_arch = "wasm32"))]
mod property {
    use super::{Grid, agreeing_tick, source_of, write_rows};
    use lang::Function;
    use proptest::prelude::*;
    use proptest::sample::select;

    const COLS: usize = 10;
    const ROWS: usize = 3;

    /// One row: a few Cells of noise, a Function spelling, an Operand
    /// Literal, a Comment, or nothing, so consecutive revisions often differ
    /// in one scheduling input and keep the rest.
    fn row() -> BoxedStrategy<String> {
        prop::collection::vec(
            prop_oneof![
                3 => Just("  ".to_owned()),
                2 => select(Function::ALL).prop_map(|function| function.to_string()),
                3 => any::<u8>().prop_map(|number| format!("{number:02X}")),
                1 => Just("||".to_owned()),
                1 => proptest::char::range(' ', '~').prop_map(String::from),
            ],
            0..=COLS,
        )
        .prop_map(|fragments| {
            let mut text = fragments.concat();
            text.truncate(COLS);
            text
        })
        .boxed()
    }

    /// A revision, and a second one sharing each of its rows with even odds,
    /// so identical-byte rows and edited rows are written together.
    fn revisions() -> BoxedStrategy<Vec<Vec<String>>> {
        prop::collection::vec(prop::collection::vec(row(), ROWS), 1..=4)
            .prop_flat_map(|revisions| {
                let count = revisions.len();
                (
                    Just(revisions),
                    prop::collection::vec(prop::collection::vec(any::<bool>(), ROWS), count - 1),
                )
            })
            .prop_map(|(mut revisions, keeps)| {
                for index in 1..revisions.len() {
                    for (row, keep) in keeps[index - 1].iter().enumerate() {
                        if *keep {
                            revisions[index][row] = revisions[index - 1][row].clone();
                        }
                    }
                }
                revisions
            })
            .boxed()
    }

    fn rows(revision: &[String]) -> Vec<&str> {
        revision.iter().map(String::as_str).collect()
    }

    proptest! {
        #[test]
        fn a_shared_schedule_plans_every_tick_as_a_fresh_one_would(
            revisions in revisions(),
        ) {
            let grid = Grid::with_shape(COLS, ROWS);
            let mut source = source_of(grid, &rows(&revisions[0]));
            let mut tick = 0;
            for revision in &revisions {
                write_rows(&mut source, &rows(revision));
                // Two Ticks per revision: the second runs against whatever
                // the first wrote, which is the steady state a pattern plays in.
                for _ in 0..2 {
                    agreeing_tick(&mut source, tick);
                    tick += 1;
                }
            }
        }
    }
}
