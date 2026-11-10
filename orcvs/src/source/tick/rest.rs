//! A Play whose note is fully blank rests, however it got blank: written blank
//! in Source, emptied by a Portal write, or cleared by a nested Function. Its
//! Turn sends no Play Command and nothing reports it, at the Tick or in the
//! Language Map (ADR 0104). A half-written note, and a blank or cleared
//! operand of any other kind, in a Play or in another Function, stays invalid.
//!
//! Equality at (0, 0) Bangs the Play at (2, 1) on every Tick.

use lang::{PlayCommand, Tick};

use super::observed::{diagnostic, first, source_of};
use crate::grid::Grid;

/// The Play Commands and diagnostics of the first Tick of `play` on row 1,
/// beneath the Equality that Bangs it.
fn banged(play: &str) -> (Vec<PlayCommand>, Diagnostics) {
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

/// Diagnostics as anchor column, anchor row and message.
type Diagnostics = Vec<(usize, usize, String)>;

/// What each of the first two Ticks of `rows` performs and reports: its Play
/// Commands, its Tick diagnostics, and the diagnostics the Language Map it
/// leaves behind reports on row 1, where the Play stands, each as anchor
/// column, anchor row and message. A pattern pair written on row 0 is not an
/// Expression, and the Map reports it whatever the Play does.
fn two_ticks(rows: &[&str]) -> Vec<(Vec<PlayCommand>, Diagnostics, Diagnostics)> {
    let mut source = source_of(Grid::with_shape(16, 3), rows);
    (0..2)
        .map(|tick| {
            let plan = source.execute(Tick::new(tick));
            let at =
                |d: &crate::source::Diagnostic| (d.anchor().x(), d.anchor().y(), d.message.clone());
            (
                plan.play_commands.clone(),
                plan.diagnostics.iter().map(at).collect(),
                source
                    .language_map()
                    .diagnostics()
                    .filter(|d| d.anchor().y() == 1)
                    .map(at)
                    .collect(),
            )
        })
        .collect()
}

#[test]
fn every_play_rests_on_a_note_its_source_leaves_blank() {
    for play in ["  !>017F  ", "  !~017F  04", "  !%017F  04"] {
        assert_eq!(
            two_ticks(&[".=0101", play]),
            vec![(vec![], vec![], vec![]); 2],
            "{play:?}"
        );
    }
}

#[test]
fn every_play_rests_on_a_note_a_portal_write_emptied() {
    // Track at (8, 0) copies the pair east of its `count` into the note slot
    // at (8, 1): `C4` in the first row of each pair, two empty Cells in the
    // second. The slot holds `E4` before the first Tick, so only the write
    // empties it.
    for play in ["  !>017FE4", "  !~017FE404", "  !%017FE404"] {
        for (commands, ticked, mapped) in two_ticks(&[".=0101  &t0001C4", play]) {
            assert_eq!(commands.len(), 1, "{play:?}: {ticked:?}");
            assert!(
                ticked.is_empty() && mapped.is_empty(),
                "{play:?}: {ticked:?} {mapped:?}"
            );
        }

        let rested = two_ticks(&[".=0101  &t0001  ", play]);
        assert_eq!(rested, vec![(vec![], vec![], vec![]); 2], "{play:?}");
    }
}

#[test]
fn a_half_written_note_is_diagnosed_by_every_route() {
    // Written at the slot, the Language Map reports it and nothing plays.
    for play in ["  !>017FC ", "  !~017F 404", "  !%017FC 04"] {
        for (commands, _, mapped) in two_ticks(&[".=0101", play]) {
            assert!(commands.is_empty(), "{play:?}");
            assert!(!mapped.is_empty(), "{play:?}: {mapped:?}");
        }
    }
    // Selected by Track for the slot, Track refuses the partial pair.
    let ticks = two_ticks(&[".=0101  &t0001C ", "  !~017FE404"]);
    assert!(
        ticks[0]
            .1
            .contains(&diagnostic(8, 0, "&t has partial or invalid input")),
        "{ticks:?}"
    );
}

#[test]
fn a_blank_note_beside_a_blank_or_invalid_operand_is_diagnosed_by_every_route() {
    for velocity in ["  ", "ZZ"] {
        // A note written blank.
        let play = format!("  !~01{velocity}  04");
        let ticks = two_ticks(&[".=0101", &play]);
        assert!(
            ticks
                .iter()
                .all(|(commands, _, mapped)| commands.is_empty() && !mapped.is_empty()),
            "{play:?}: {ticks:?}"
        );
        // A note a Portal write emptied.
        let play = format!("  !~01{velocity}E404");
        let ticks = two_ticks(&[".=0101  &t0001  ", &play]);
        assert!(
            ticks[0].0.is_empty() && !ticks[0].1.is_empty(),
            "{play:?}: {ticks:?}"
        );
        // A note a nested Function cleared.
        let play = format!("  !~01{velocity}&v0104");
        let ticks = two_ticks(&[".=0101", &play]);
        assert!(
            ticks[0].0.is_empty() && !(ticks[0].1.is_empty() && ticks[0].2.is_empty()),
            "{play:?}: {ticks:?}"
        );
    }
    // And a length a Portal write emptied beside a note written blank.
    let ticks = two_ticks(&[".=0101    &t0001  ", "  !~017F  04"]);
    assert!(ticks[0].0.is_empty() && !ticks[0].1.is_empty(), "{ticks:?}");
}
