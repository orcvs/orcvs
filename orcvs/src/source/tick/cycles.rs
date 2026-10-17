//! Same-Tick cycles: each is diagnosed once at a computation on it, and stops
//! only the Expressions it reaches.

use crate::grid::Grid;

use super::fixture::{anchors, carried_source, carried_tick, messages, tick_by_tick};

#[test]
fn a_cycle_is_diagnosed_at_a_computation_on_it() {
    // The nested `=<` writes over its parent's anchor, which covers the
    // Copy itself, so the Copy is the computation the cycle runs through.
    // The parent only waits on it, and is not where the cycle is.
    let (plans, _, _) = tick_by_tick(Grid::with_shape(6, 1), &[".+=<01"], 1);
    assert_eq!(anchors(&plans[0]), [(2, 0, "same-Tick dependency cycle")]);
}

#[test]
fn an_inactive_root_a_cycle_writes_over_is_not_reported_as_waiting() {
    // The cycle writes over the `00` the Note Function reads, but nothing
    // activates the Note Function this Tick, so it would take no Turn
    // with the cycle gone either, and has nothing to wait for.
    let (plans, _, _) = tick_by_tick(Grid::with_shape(8, 3), &["  .+=<01", "!>007FC4", ""], 1);
    assert_eq!(anchors(&plans[0]), [(4, 0, "same-Tick dependency cycle")]);
}

#[test]
fn each_separate_cycle_is_diagnosed_once() {
    // Two Additions, each with a nested `=<` writing over it: neither
    // cycle reaches the other, so each is diagnosed at its own Copy.
    let (plans, _, _) = tick_by_tick(Grid::with_shape(14, 1), &[".+=<01  .+=<01"], 1);
    assert_eq!(
        anchors(&plans[0]),
        [
            (2, 0, "same-Tick dependency cycle"),
            (10, 0, "same-Tick dependency cycle"),
        ]
    );
}

#[test]
fn a_cycle_leaves_an_independent_expression_publishing() {
    // ADR 0065: the cycle stops its own Expression, and the Addition beside
    // it, which depends on nothing there, writes `03` every Tick.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(14, 2), &[".+=<01  .+0102"], 3);
    for (plan, rows) in plans.iter().zip(&grids) {
        assert_eq!(rows, &[".+=<01  .+0102", "        03    "]);
        assert_eq!(messages(plan), ["same-Tick dependency cycle"]);
    }
}

#[test]
fn an_expression_a_cycle_stops_is_diagnosed_at_its_root_as_waiting() {
    // The lower Addition reads `00`, which the upper one writes over, so
    // it waits on the cycle the `=<` runs through and holds no part of it.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 3), &["  .+=<01", ".+0001", ""], 1);
    assert_eq!(grids[0], ["  .+=<01", ".+0001  ", "        "]);
    assert_eq!(
        anchors(&plans[0]),
        [
            (4, 0, "same-Tick dependency cycle"),
            (0, 1, "waiting on a same-Tick dependency cycle"),
        ]
    );
}

#[test]
fn a_copy_that_closes_a_same_tick_cycle_is_diagnosed() {
    // Increment reads and writes one row south. A Copy that copies
    // that Cell pair back onto Increment's operand closes a cycle.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(6, 2), &["~+0104", "=^"], 1);
    assert_eq!(grids[0], ["~+0104", "=^    "]);
    assert!(
        plans[0]
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("same-Tick dependency cycle")),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn live_cycles_stop_only_the_expressions_they_reach() {
    // ADR 0065. The first two rows form a cycle, then a self-dependency
    // beside an Addition that writes over `!>`. Either way `.=` is
    // independent: its Bang lands above `!>`, which plays only where no
    // write covers it.
    for (outputs, writes, plays) in [
        (vec![(0, 18), (16, 2), (32, 48)], vec![(48, "**")], 1),
        (
            vec![(0, 2), (16, 64), (32, 48)],
            vec![(48, "**"), (64, "01")],
            0,
        ),
    ] {
        let (plan, _, source) = carried_tick(
            Grid::with_shape(16, 5),
            &[".+0001", ".+0001", ".=0101", "", "!>007FC4"],
            &outputs,
        );
        for (start, spelling) in writes {
            assert_eq!(&source.snapshot()[start..start + 2], spelling);
        }
        assert_eq!(&source.snapshot()[2..6], "0001");
        assert_eq!(plan.play_commands.len(), plays);
        assert_eq!(messages(&plan), ["same-Tick dependency cycle"]);
    }
    // A nested computation on the cycle stops its whole Expression.
    let (plan, _) = carried_source(
        Grid::with_shape(16, 3),
        &[".+02.x0304", ".+0001", ""],
        &[(0, 18), (4, 18), (16, 6)],
    );
    assert!(plan.writes.is_empty());
    assert_eq!(messages(&plan), ["same-Tick dependency cycle"]);
}
