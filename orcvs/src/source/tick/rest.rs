//! A Play whose note a nested Function cleared rests: its Turn sends no Play
//! Command and reports nothing. Every other Clear, in another operand of a Play
//! or in any operand of another Function, leaves its parent invalid.
//!
//! Equality at (0, 0) Bangs the Play at (2, 1) on every Tick.

use lang::{PlayCommand, Tick};

use super::observed::{diagnostic, first, source_of};
use crate::grid::Grid;

/// The Play Commands and diagnostics of the first Tick of `play` on row 1,
/// beneath the Equality that Bangs it.
fn banged(play: &str) -> (Vec<PlayCommand>, Vec<(usize, usize, String)>) {
    let mut source = source_of(Grid::with_shape(16, 3), &[".=0101", play]);
    let plan = source.execute(Tick::ZERO);
    let diagnostics = plan
        .diagnostics
        .iter()
        .map(|d| (d.anchor().x(), d.anchor().y(), d.message.clone()))
        .collect();
    (plan.play_commands, diagnostics)
}

#[test]
fn every_play_rests_on_a_note_a_nested_read_of_blank_cells_cleared() {
    // Each Read selects the pair two Cells east of its operand, which holds
    // `C4` in the first row of each pair and nothing in the second.
    for (sounding, resting) in [
        ("  !>017F&>01C4", "  !>017F&>01  "),
        ("  !~017F&>0204C4", "  !~017F&>0204  "),
        ("  !%017F&>0204C4", "  !%017F&>0204  "),
    ] {
        let (commands, diagnostics) = banged(sounding);
        assert_eq!(commands.len(), 1, "{sounding:?}: {diagnostics:?}");
        assert!(diagnostics.is_empty(), "{sounding:?}: {diagnostics:?}");

        assert_eq!(banged(resting), (vec![], vec![]), "{resting:?}");
    }
}

#[test]
fn a_rest_writes_the_clear_under_the_nested_read() {
    // The Read still takes its Turn and clears its Output Portal; only the
    // Play it returns into has nothing to perform.
    let tick = first(
        Grid::with_shape(16, 3),
        &[".=0101", "  !~017F&>0204", "        xx"],
    );
    assert_eq!(tick.rows[2], "                ");
    assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
}

#[test]
fn a_play_whose_channel_velocity_or_length_was_cleared_is_invalid() {
    // A `&v01` reads the blank pair south of its operand.
    for (play, child) in [
        ("  !~&v017FC404", 4),
        ("  !~01&v01C404", 6),
        ("  !~017FC4&v01", 10),
        ("  !>&v017FC4", 4),
    ] {
        assert_eq!(
            banged(play),
            (
                vec![],
                vec![diagnostic(
                    2,
                    1,
                    &format!("nested computation at column {child}, row 1 cleared its operand")
                )]
            ),
            "{play:?}"
        );
    }
}

#[test]
fn a_rest_needs_every_other_operand_to_decode() {
    // The note rests, but the length beside it was cleared too.
    assert_eq!(
        banged("  !~017F&v01&v01"),
        (
            vec![],
            vec![diagnostic(
                2,
                1,
                "nested computation at column 12, row 1 cleared its operand"
            )]
        )
    );
}

#[test]
fn a_note_operand_of_a_value_function_still_refuses_a_clear() {
    // Note to Number reads a Note too, but answers a value rather than
    // sounding one, so a Clear leaves it nothing to convert.
    let tick = first(Grid::with_shape(10, 2), &[".v&>01  "]);
    assert_eq!(
        tick.diagnostics,
        [diagnostic(
            0,
            0,
            "nested computation at column 2, row 0 cleared its operand"
        )]
    );
}

#[test]
fn a_partly_written_note_is_not_a_rest() {
    // A half-typed Note read into the slot is partial input and not a Clear:
    // the Read refuses it, so the Play has no Return and says so.
    assert_eq!(
        banged("  !~017F&>0204C "),
        (
            vec![],
            vec![
                diagnostic(8, 1, "&> has partial or invalid input"),
                diagnostic(
                    2,
                    1,
                    "nested computation at column 8, row 1 returned nothing"
                ),
            ]
        )
    );
    // Written at the slot itself, the Parser reports it before any Turn.
    let source = source_of(Grid::with_shape(16, 3), &[".=0101", "  !~017FC 04"]);
    assert!(source.language_map().diagnostics().next().is_some());
}
