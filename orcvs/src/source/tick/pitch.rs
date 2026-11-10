//! Pitch Functions in Source: Up and Down write the Note they answer through
//! their Output Portal as a root and nested, and a nested one in a Play's
//! note operand plays the Note it answers.

use crate::grid::Grid;

use super::fixture::{messages, tick_by_tick};
use super::observed::raw;

#[test]
fn up_and_down_write_the_transposed_note_through_their_output_portal() {
    // Each Tick writes the same answer again, so the result stands.
    for (source, written) in [("#+C40C", "C5    "), ("#-C40C", "C3    ")] {
        let (plans, grids, _) = tick_by_tick(Grid::with_shape(6, 2), &[source, ""], 2);
        for (plan, grid) in plans.iter().zip(&grids) {
            assert_eq!(*grid, [source, written], "{source}");
            assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        }
    }
}

#[test]
fn a_nested_transposition_writes_its_own_answer_and_returns_it() {
    // The nested Up writes `C5` under its own anchor and returns it to
    // Convert to Number, which writes the Number `48` under its anchor.
    // Nested in Down's note operand, the same Return is transposed again.
    for (source, written) in [(".v#+C40C", "48C5    "), ("#-#+C40C07", "F4C5      ")] {
        let width = source.len();
        let (plans, grids, _) = tick_by_tick(Grid::with_shape(width, 2), &[source, ""], 1);
        assert_eq!(grids[0], [source, written], "{source}");
        assert!(
            plans[0].diagnostics.is_empty(),
            "{:?}",
            plans[0].diagnostics
        );
    }
}

#[test]
fn an_answer_outside_the_note_range_writes_nothing_and_diagnoses() {
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(6, 2), &["#+G901", ""], 1);
    assert_eq!(grids[0], ["#+G901", "      "]);
    assert_eq!(
        messages(&plans[0]),
        ["#+ answers 80, outside the Note range 00–7F"]
    );

    let (plans, grids, _) = tick_by_tick(Grid::with_shape(6, 2), &["#-C/01", ""], 1);
    assert_eq!(grids[0], ["#-C/01", "      "]);
    assert_eq!(
        messages(&plans[0]),
        ["#- answers -01, outside the Note range 00–7F"]
    );
}

#[test]
fn a_play_with_a_nested_transposition_in_its_note_plays_the_transposed_note() {
    // Equality's Bang lands on row 1 and activates the Play aligned south
    // of it. The nested Pitch Function writes its answer under its own
    // anchor, and the Play sounds that Note.
    for (play, note, written) in [
        ("!>007F#+C40C", 0x48, "      C5    "),
        ("!>007F#-C40C", 0x30, "      C3    "),
    ] {
        let (plans, grids, _) = tick_by_tick(Grid::with_shape(12, 4), &[".=0101", "", play, ""], 1);
        assert_eq!(plans[0].play_commands, [raw(0, 0x7F, note)], "{play}");
        assert_eq!(grids[0][3], written, "{play}");
        assert!(
            plans[0].diagnostics.is_empty(),
            "{:?}",
            plans[0].diagnostics
        );
    }
}

#[test]
fn a_play_whose_nested_transposition_leaves_the_note_range_sends_nothing() {
    let (plans, grids, _) = tick_by_tick(
        Grid::with_shape(12, 4),
        &[".=0101", "", "!>007F#+G901", ""],
        1,
    );
    assert!(plans[0].play_commands.is_empty());
    assert_eq!(grids[0][3], "            ");
    assert!(
        messages(&plans[0]).contains(&"#+ answers 80, outside the Note range 00–7F"),
        "{:?}",
        plans[0].diagnostics
    );
}
