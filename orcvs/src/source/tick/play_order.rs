//! Play Command order: fixed upward Portals schedule notes and Bangs before the
//! MIDI root, and each MIDI root executes once.

use crate::grid::Grid;
use crate::source::Cells;
use crate::source::language_map::LanguageMap;

use super::Tick;
use super::fixture::planned;
use super::observed::raw;

#[test]
fn fixed_upward_portals_schedule_note_and_bang_before_midi() {
    let grid = Grid::with_shape(16, 5);
    let rows = ["", "", "!>007FD4", "      .^3C", ".=0101"];
    let bytes = rows
        .iter()
        .map(|row| format!("{row:16}"))
        .collect::<String>();
    let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));
    let destinations = [
        (
            grid.index(grid.position(6, 3).unwrap()),
            vec![grid.position(6, 2).unwrap()],
        ),
        (
            grid.index(grid.position(0, 4).unwrap()),
            vec![grid.position(0, 3).unwrap()],
        ),
    ]
    .into_iter()
    .collect();

    let (plan, _) = super::plan_carrying(
        grid,
        Cells::of(bytes.as_bytes()),
        &map,
        Tick::ZERO,
        &destinations,
    );

    assert_eq!(plan.play_commands, vec![raw(0, 0x7F, 60)]);
    assert!(plan.diagnostics.is_empty());
    assert_eq!(
        planned(&plan),
        vec![(38, 'C'), (39, '4'), (48, '*'), (49, '*')]
    );
}

#[test]
fn two_current_bang_results_still_execute_midi_once() {
    let grid = Grid::with_shape(16, 6);
    let rows = ["", ".=0101", "", "!>007FC4", "", ".=0202"];
    let bytes = rows
        .iter()
        .map(|row| format!("{row:16}"))
        .collect::<String>();
    let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));
    let destinations = [
        (
            grid.index(grid.position(0, 1).unwrap()),
            vec![grid.position(0, 2).unwrap()],
        ),
        (
            grid.index(grid.position(0, 5).unwrap()),
            vec![grid.position(0, 4).unwrap()],
        ),
    ]
    .into_iter()
    .collect();

    let (plan, _) = super::plan_carrying(
        grid,
        Cells::of(bytes.as_bytes()),
        &map,
        Tick::ZERO,
        &destinations,
    );

    assert_eq!(plan.play_commands, vec![raw(0, 0x7F, 60)]);
    assert!(plan.diagnostics.is_empty());
}

#[test]
fn one_failed_bang_candidate_does_not_suppress_another_fresh_bang() {
    let grid = Grid::with_shape(16, 7);
    let rows = [".=0101", ".=./010001", "", "", "", "!>007FC4", ""];
    let bytes = rows
        .iter()
        .map(|row| format!("{row:16}"))
        .collect::<String>();
    let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));
    let destinations = [
        (
            grid.index(grid.position(0, 0).unwrap()),
            vec![grid.position(0, 4).unwrap()],
        ),
        (
            grid.index(grid.position(0, 1).unwrap()),
            vec![grid.position(0, 6).unwrap()],
        ),
    ]
    .into_iter()
    .collect();

    let (plan, _) = super::plan_carrying(
        grid,
        Cells::of(bytes.as_bytes()),
        &map,
        Tick::ZERO,
        &destinations,
    );

    assert_eq!(plan.play_commands, vec![raw(0, 0x7F, 60)]);
    assert!(
        plan.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "cannot divide by zero")
    );
}
