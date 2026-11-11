//! A Play's note is optional by type: a blank note on a Raw, Timed or
//! Monophonic Play sends no Play Command and reports nothing, whether it was
//! written blank in Source or a Portal write emptied it. A half-typed or
//! mistyped note is still diagnosed, and every other operand still binds, so
//! a blank or out-of-range channel or length is diagnosed on a blank step. A
//! nested Function that answers a Clear in the note leaves it blank too. A
//! blank or a nested Clear in any other operand of any Function, such as a
//! Play's channel or any operand of Control Change or Pitch Bend, is invalid
//! input. Either way the outcome belongs to the Play alone: every other
//! Expression takes its Turn as before.
//!
//! Equality at (0, 0) Bangs the Play at (2, 1) on every Tick, unless a test
//! writes a Bang beside it instead.

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
fn a_play_whose_note_a_nested_read_of_blank_cells_cleared_plays_nothing_and_reports_nothing() {
    // Each `&>` selects the pair two Cells east of its operand, past the
    // length where there is one, which holds `C4` in the sounding row and
    // nothing in the cleared one. Each `&v01` selects the blank pair south of
    // its operand.
    let grid = Grid::with_shape(18, 3);
    for (sounding, cleared_notes) in [
        ("  !>017F&>01C4", ["  !>017F&>01  ", "  !>017F&v01"]),
        ("  !~017F&>0204C4", ["  !~017F&>0204  ", "  !~017F&v0104"]),
        ("  !%017F&>0204C4", ["  !%017F&>0204  ", "  !%017F&v0104"]),
    ] {
        let (commands, diagnostics) = banged(sounding);
        assert_eq!(commands.len(), 1, "{sounding:?}: {diagnostics:?}");
        assert!(diagnostics.is_empty(), "{sounding:?}: {diagnostics:?}");

        for play in cleared_notes {
            let ticks = executed(grid, &[".=0101", play], 0..3);
            for (tick, (commands, observed, mapped)) in ticks.into_iter().enumerate() {
                assert_eq!(commands, vec![], "{play:?} Tick {tick}");
                assert_eq!(observed.diagnostics, vec![], "{play:?} Tick {tick}");
                assert_eq!(mapped, vec![], "{play:?} Tick {tick}");
            }
        }
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
    // A `&v01` reads the blank pair south of its operand. A blank note beside
    // it excuses nothing.
    for (play, child) in [
        ("  !~&v017FC404", 4),
        ("  !~01&v01C404", 6),
        ("  !~017FC4&v01", 10),
        ("  !%&v017FC404", 4),
        ("  !%01&v01C404", 6),
        ("  !%017FC4&v01", 10),
        ("  !>&v017FC4", 4),
        ("  !>01&v01C4", 6),
        ("  !~&v017F  04", 4),
        ("  !%01&v01  04", 6),
        ("  !>&v017F  ", 4),
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
        "  !~017F&>0204  .+0102",
        "  !%017F&>0204  .+0102",
        "  !>017FC .+0102",
    ] {
        let tick = first(Grid::with_shape(24, 3), &[".=0101", play]);
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

/// Every Bang-driven MIDI output, with an operand list each Turn sounds, and
/// the index of the operand that admits a blank: a Play's note, and nothing
/// on Control Change or Pitch Bend.
const OUTPUTS: [(&str, &[&str], Option<usize>); 5] = [
    ("!>", &["01", "7F", "C4"], Some(2)),
    ("!~", &["01", "7F", "C4", "04"], Some(2)),
    ("!%", &["01", "7F", "C4", "04"], Some(2)),
    ("!c", &["01", "07", "40"], None),
    ("!b", &["01", "00", "40"], None),
];

#[test]
fn a_nested_clear_under_a_bang_driven_output_binds_to_absent_only_in_a_plays_note() {
    // Each operand in turn is a `&v01`, which reads the blank pair south of
    // it and answers a Clear. The output is activated by the Equality above
    // it, or by a Bang written beside it.
    let grid = Grid::with_shape(24, 4);
    for (spelling, operands, admitted) in OUTPUTS {
        for (above, beside) in [(".=0101", "  "), ("", "**")] {
            let first_tick = |written: &str| {
                let row = format!("{beside}{spelling}{written}");
                let (commands, tick, _) = executed(grid, &[above, &row], 0..1).remove(0);
                (row, commands, tick.diagnostics)
            };
            let (row, commands, diagnostics) = first_tick(&operands.concat());
            assert_eq!(commands.len(), 1, "{row:?}: {diagnostics:?}");
            assert_eq!(diagnostics, vec![], "{row:?}");

            for index in 0..operands.len() {
                let mut nested = operands.to_vec();
                nested[index] = "&v01";
                let (row, commands, diagnostics) = first_tick(&nested.concat());
                let expected = if admitted == Some(index) {
                    vec![]
                } else {
                    vec![cleared(2 + spelling.len() + 2 * index)]
                };
                assert_eq!(commands, vec![], "{row:?}");
                assert_eq!(diagnostics, expected, "{row:?}");
            }
        }
    }
}
