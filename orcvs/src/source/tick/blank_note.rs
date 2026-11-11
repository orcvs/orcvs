//! A Play's note is optional by type: a blank note on a Raw, Timed or
//! Monophonic Play sends no Play Command and reports nothing, whether it was
//! written blank in Source or a Portal write emptied it. A half-typed or
//! mistyped note is still diagnosed, and every other operand still binds, so
//! a blank or out-of-range channel or length is diagnosed on a blank step. A
//! blank or a Clear in any other operand of any Function is invalid input.
//! Either way the outcome belongs to the Play alone: every other Expression
//! takes its Turn as before.
//!
//! Equality at (0, 0) Bangs the Play at (2, 1) on every Tick.

use std::ops::Range;

use lang::{PlayCommand, Tick};

use super::observed::{Observed, diagnostic, first, observed, source_of};
use crate::grid::Grid;

/// Diagnostics by anchor column, row and message, as [`Observed`] holds them.
type Diagnostics = Vec<(usize, usize, String)>;

/// Each Tick in `ticks` of `rows` on a `grid`: its Play Commands, the Tick
/// observed, and the Language Map's diagnostics on row 1 read after it.
///
/// The Language Map is what reports a refusal deferred to a later Tick: a
/// Portal write lands in Source after its Tick, and the Map is rebuilt from
/// what the Tick left.
fn executed(
    grid: Grid,
    rows: &[&str],
    ticks: Range<u64>,
) -> Vec<(Vec<PlayCommand>, Observed, Diagnostics)> {
    let mut source = source_of(grid, rows);
    ticks
        .map(|tick| {
            let plan = source.execute(Tick::new(tick));
            let mapped = source
                .language_map()
                .diagnostics()
                .filter(|d| d.anchor().y() == 1)
                .map(|d| diagnostic(d.anchor().x(), d.anchor().y(), &d.message))
                .collect();
            (plan.play_commands.clone(), observed(&source, &plan), mapped)
        })
        .collect()
}

/// The Play Commands and diagnostics of the first Tick of `play` on row 1,
/// beneath the Equality that Bangs it.
fn banged(play: &str) -> (Vec<PlayCommand>, Diagnostics) {
    let (commands, tick, _) = executed(Grid::with_shape(16, 3), &[".=0101", play], 0..1).remove(0);
    (commands, tick.diagnostics)
}

/// Every Language Map diagnostic for `rows`, with its message.
fn mapped_of(grid: Grid, rows: &[&str]) -> Diagnostics {
    source_of(grid, rows)
        .language_map()
        .diagnostics()
        .map(|d| diagnostic(d.anchor().x(), d.anchor().y(), &d.message))
        .collect()
}

/// The Language Map's diagnostics for `rows`.
fn mapped(grid: Grid, rows: &[&str]) -> Vec<(usize, usize)> {
    source_of(grid, rows)
        .language_map()
        .diagnostics()
        .map(|d| (d.anchor().x(), d.anchor().y()))
        .collect()
}

/// The Play Commands and diagnostics of the first Tick of `rows`, and the
/// Source's row 1 after it.
fn run(rows: &[&str]) -> (Vec<PlayCommand>, Diagnostics, String) {
    let (commands, mut tick, _) = executed(Grid::with_shape(18, 3), rows, 0..1).remove(0);
    (commands, tick.diagnostics, tick.rows.swap_remove(1))
}

/// The diagnostic a Play at (2, 1) gives when the nested Function at
/// `column` on row 1 cleared one of its operands.
fn cleared(column: usize) -> (usize, usize, String) {
    diagnostic(
        2,
        1,
        &format!("nested computation at column {column}, row 1 cleared its operand"),
    )
}

#[test]
fn a_raw_play_whose_note_a_nested_read_of_blank_cells_cleared_plays_nothing() {
    let (commands, diagnostics) = banged("  !>017F&>01C4");
    assert_eq!(commands.len(), 1, "{diagnostics:?}");
    assert!(diagnostics.is_empty(), "{diagnostics:?}");

    assert_eq!(banged("  !>017F&>01  "), (vec![], vec![]));
}

#[test]
fn a_timed_or_monophonic_play_whose_note_a_nested_read_of_blank_cells_cleared_plays_nothing() {
    // Each Read selects the pair two Cells east of its operand, which holds
    // `C4` in the first row of each pair and nothing in the second.
    for (sounding, cleared_note) in [
        ("  !~017F&>0204C4", "  !~017F&>0204  "),
        ("  !%017F&>0204C4", "  !%017F&>0204  "),
    ] {
        let (commands, diagnostics) = banged(sounding);
        assert_eq!(commands.len(), 1, "{sounding:?}: {diagnostics:?}");
        assert!(diagnostics.is_empty(), "{sounding:?}: {diagnostics:?}");

        assert_eq!(banged(cleared_note), (vec![], vec![]), "{cleared_note:?}");
    }
}

#[test]
fn a_raw_play_with_a_note_written_blank_in_source_plays_nothing_and_reports_nothing() {
    let grid = Grid::with_shape(16, 3);
    for play in ["  !>017F  ", "  !>017F  .+0102"] {
        let ticks = executed(grid, &[".=0101", play], 0..3);
        for (tick, (commands, observed, mapped)) in ticks.into_iter().enumerate() {
            assert_eq!(commands, vec![], "{play:?} Tick {tick}");
            assert_eq!(observed.diagnostics, vec![], "{play:?} Tick {tick}");
            assert_eq!(mapped, vec![], "{play:?} Tick {tick}");
        }
        assert_eq!(mapped_of(grid, &[".=0101", play]), vec![], "{play:?}");
    }
}

#[test]
fn a_timed_or_monophonic_play_with_a_note_written_blank_in_source_plays_nothing_and_reports_nothing()
 {
    let grid = Grid::with_shape(18, 3);
    for play in [
        "  !~017F  04",
        "  !%017F  04",
        "  !~017F  04.+0102",
        "  !%017F  04.+0102",
    ] {
        let ticks = executed(grid, &[".=0101", play], 0..3);
        for (tick, (commands, observed, mapped)) in ticks.into_iter().enumerate() {
            assert_eq!(commands, vec![], "{play:?} Tick {tick}");
            assert_eq!(observed.diagnostics, vec![], "{play:?} Tick {tick}");
            assert_eq!(mapped, vec![], "{play:?} Tick {tick}");
        }
        assert_eq!(mapped_of(grid, &[".=0101", play]), vec![], "{play:?}");
    }
}

#[test]
fn a_raw_play_whose_note_a_portal_write_emptied_plays_nothing_and_reports_nothing() {
    // Track writes the pair it selects over the note slot: `D4` when the pair
    // holds it, two blank Cells when it is empty.
    let grid = Grid::with_shape(18, 3);
    let play = "  !>017FC4";
    let (commands, diagnostics, _) = run(&[".=0101  &t0001D4", play]);
    assert_eq!(commands.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics, vec![]);

    let rows = [".=0101  &t0001  ", play];
    let ticks = executed(grid, &rows, 0..2);
    let (_, observed, _) = &ticks[0];
    assert_eq!(&observed.rows[1][8..10], "  ", "Track emptied the slot");
    for (tick, (commands, observed, mapped)) in ticks.into_iter().enumerate() {
        assert_eq!(commands, vec![], "Tick {tick}");
        assert_eq!(observed.diagnostics, vec![], "Tick {tick}");
        assert_eq!(mapped, vec![], "Tick {tick}");
    }
}

#[test]
fn a_timed_or_monophonic_play_whose_note_a_portal_write_emptied_plays_nothing_and_reports_nothing()
{
    // Track writes the pair it selects over the note slot: `D4` when the pair
    // holds it, two blank Cells when it is empty.
    let grid = Grid::with_shape(18, 3);
    for play in ["  !~017FC404", "  !%017FC404"] {
        let (commands, diagnostics, _) = run(&[".=0101  &t0001D4", play]);
        assert_eq!(commands.len(), 1, "{play:?}: {diagnostics:?}");
        assert_eq!(diagnostics, vec![], "{play:?}");

        let ticks = executed(grid, &[".=0101  &t0001  ", play], 0..2);
        let (_, observed, _) = &ticks[0];
        assert_eq!(
            &observed.rows[1][8..10],
            "  ",
            "{play:?}: Track emptied the slot"
        );
        for (tick, (commands, observed, mapped)) in ticks.into_iter().enumerate() {
            assert_eq!(commands, vec![], "{play:?} Tick {tick}");
            assert_eq!(observed.diagnostics, vec![], "{play:?} Tick {tick}");
            assert_eq!(mapped, vec![], "{play:?} Tick {tick}");
        }
    }
}

#[test]
fn a_blank_note_a_portal_fills_plays_the_note_written() {
    // The same Play is silent with Track's pair empty, and sounds what Track
    // writes the moment the pair holds a Note.
    for play in ["  !>017F", "  !~017F  04", "  !%017F  04"] {
        let (commands, diagnostics, row) = run(&[".=0101  &t0001D4", play]);
        assert_eq!(commands.len(), 1, "{play:?}: {diagnostics:?}");
        assert_eq!(diagnostics, vec![], "{play:?}");
        assert_eq!(&row[8..10], "D4", "{play:?}");
    }
}

#[test]
fn a_cleared_note_leaves_the_clear_under_the_nested_read() {
    // The Read still takes its Turn and clears its Output Portal, and the
    // Play it returns into plays nothing.
    let (commands, tick, _) = executed(
        Grid::with_shape(16, 3),
        &[".=0101", "  !~017F&>0204", "        xx"],
        0..1,
    )
    .remove(0);
    assert_eq!(tick.rows[2], "                ");
    assert_eq!(commands, vec![]);
    assert_eq!(tick.diagnostics, vec![]);
}

#[test]
fn a_raw_play_with_a_blank_note_still_diagnoses_its_channel_and_velocity() {
    let grid = Grid::with_shape(16, 3);
    // A blank channel or velocity is refused from Source, beside a blank note
    // as beside any other.
    for play in ["  !>  7F  ", "  !>01    "] {
        assert_eq!(banged(play), (vec![], vec![]), "{play:?}");
        assert_eq!(mapped(grid, &[".=0101", play]), vec![(2, 1)], "{play:?}");
    }
    // A channel or velocity outside its domain is refused at the Turn, on a
    // blank step as on a sounding one.
    for (play, message) in [
        ("  !>107F  ", "MIDI channel 10 is outside the range 00–0F"),
        ("  !>0180  ", "MIDI velocity 80 is outside the range 00–7F"),
    ] {
        assert_eq!(
            banged(play),
            (vec![], vec![diagnostic(2, 1, message)]),
            "{play:?}"
        );
        assert_eq!(mapped(grid, &[".=0101", play]), vec![], "{play:?}");
    }
}

#[test]
fn a_timed_or_monophonic_play_with_a_blank_note_still_diagnoses_its_other_operands() {
    let grid = Grid::with_shape(18, 3);
    for kind in ["!~", "!%"] {
        // A blank length, a length that is no Number, or a blank channel is
        // refused from Source, beside a blank note as beside a written one.
        // Every byte is a length, so no written length is out of range.
        for operands in ["017F    ", "017F  ZZ", "  7F  04"] {
            let play = format!("  {kind}{operands}");
            assert_eq!(banged(&play), (vec![], vec![]), "{play:?}");
            assert!(
                mapped(grid, &[".=0101", &play]).contains(&(2, 1)),
                "{play:?}"
            );
        }
        // A channel or velocity outside its domain is refused at the Turn.
        for (operands, message) in [
            ("107F  04", "MIDI channel 10 is outside the range 00–0F"),
            ("0180  04", "MIDI velocity 80 is outside the range 00–7F"),
        ] {
            let play = format!("  {kind}{operands}");
            assert_eq!(
                banged(&play),
                (vec![], vec![diagnostic(2, 1, message)]),
                "{play:?}"
            );
        }
        // A nested Clear in the length leaves the Play invalid.
        let play = format!("  {kind}017F  &v01");
        assert_eq!(banged(&play), (vec![], vec![cleared(10)]), "{play:?}");
    }
}

#[test]
fn a_raw_play_with_a_blank_note_keeps_every_other_play_in_tick_order() {
    // Three Plays, each Banged by its own Equality: the blank one between two
    // sounding ones leaves both their commands in order.
    let rows = [
        ".=0101",
        "  !>017FD4",
        ".=0101",
        "  !>017F  ",
        ".=0101",
        "  !>017FC4",
    ];
    let (commands, observed, mapped) = executed(Grid::with_shape(12, 6), &rows, 0..1).remove(0);
    let notes: Vec<_> = commands
        .iter()
        .map(|command| match command {
            PlayCommand::Raw { note, .. } => note.value(),
            other => panic!("{other:?} is not a Raw Play"),
        })
        .collect();
    assert_eq!(notes, vec![62, 60]);
    assert_eq!(observed.diagnostics, vec![]);
    assert_eq!(mapped, vec![]);
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
        assert_eq!(banged(play), (vec![], vec![cleared(child)]), "{play:?}");
    }
}

#[test]
fn a_note_operand_of_a_value_function_refuses_a_clear() {
    // Note to Number reads a Note too, and a Clear leaves it nothing to
    // convert, as it leaves a Play nothing to sound.
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
fn a_partly_written_note_names_the_read_that_refused_it() {
    // A half-typed Note read into the slot is partial input rather than a
    // Clear: the Read refuses it, so the Play has no Return and says so.
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

#[test]
fn a_half_written_note_is_diagnosed_by_every_route_on_every_play() {
    // A nested Read of a half-typed pair: the Read refuses it and the Play
    // names the Read.
    for (play, read) in [
        ("  !>017F&>01C ", 8),
        ("  !~017F&>0204C ", 8),
        ("  !%017F&>0204C ", 8),
    ] {
        let (commands, diagnostics) = banged(play);
        assert_eq!(commands, vec![], "{play:?}");
        assert!(
            diagnostics
                .iter()
                .any(|(column, _, message)| *column == read
                    && message.ends_with("has partial or invalid input")),
            "{play:?}: {diagnostics:?}"
        );
    }
    // Written in Source, the Language Map refuses it and the Tick is silent.
    for play in ["  !>017FC ", "  !~017F C04", "  !%017FC 04"] {
        let grid = Grid::with_shape(16, 3);
        assert_eq!(banged(play).0, vec![], "{play:?}");
        assert!(
            mapped(grid, &[".=0101", play]).contains(&(2, 1)),
            "{play:?}"
        );
    }
    // A Portal writing one Cell of a pair across the slot leaves one Cell of
    // the note: `D` lands on the note's second Cell and `4` on the Cell past it.
    for play in ["  !>017F  ", "  !~017F  04", "  !%017F  04"] {
        let (commands, diagnostics, row) = run(&[".=0101   &t0001D4", play]);
        assert_eq!(commands, vec![], "{play:?}");
        assert_eq!(&row[8..10], " D", "{play:?}");
        assert_eq!(
            diagnostics,
            [diagnostic(2, 1, "expected a note, found \" D\"")],
            "{play:?}"
        );
    }
}

#[test]
fn a_play_with_a_blank_note_costs_no_other_expression_its_turn() {
    // An Addition beside the Play on its row, and a Play beside it banged by
    // the same Equality, take their Turns whichever way the note is blank.
    for play in [
        "  !>017F  .+0102",
        "  !>017F&>01  .+0102",
        "  !>017FC .+0102",
    ] {
        let tick = first(Grid::with_shape(20, 3), &[".=0101", play]);
        let addition = play.find(".+").expect("the row holds an Addition");
        assert_eq!(
            &tick.rows[2][addition..addition + 2],
            "03",
            "{play:?}: {:?}",
            tick.diagnostics
        );
        assert!(
            tick.diagnostics
                .iter()
                .all(|(column, row, _)| (*column, *row) == (2, 1) || (*column, *row) == (8, 1)),
            "{play:?}: {:?}",
            tick.diagnostics
        );
    }
}
