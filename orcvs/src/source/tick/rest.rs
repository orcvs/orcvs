//! A Play whose note or Chord Play whose root is blank rests, however it came to
//! be blank: its Turn sends no Play Command and reports nothing. Written blank in
//! Source, emptied by a Portal write and cleared by a nested Function are one
//! case. A blank root also excuses a Chord Play's other blank or cleared
//! operands. A blank anywhere else, and any Clear elsewhere, leaves its Play or
//! Function invalid.
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
/// The Language Map is what can tell a rest from a refusal deferred to a later
/// Tick: a Portal write lands in Source after its Tick, and the Map is rebuilt
/// from what the Tick left.
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

/// The Play Commands and diagnostics of each of the first two Ticks of
/// `rows`, with the Language Map's diagnostics on row 1 read after each.
fn two_ticks(rows: &[&str]) -> Vec<(Vec<PlayCommand>, Diagnostics, Diagnostics)> {
    executed(Grid::with_shape(18, 3), rows, 0..2)
        .into_iter()
        .map(|(commands, tick, mapped)| (commands, tick.diagnostics, mapped))
        .collect()
}

#[test]
fn every_play_rests_on_a_note_written_blank_in_source() {
    for play in ["  !>017F", "  !~017F  04", "  !%017F  04"] {
        let grid = Grid::with_shape(16, 3);
        assert_eq!(banged(play), (vec![], vec![]), "{play:?}");
        assert_eq!(mapped(grid, &[".=0101", play]), vec![], "{play:?}");
    }
}

#[test]
fn every_play_rests_on_a_note_a_portal_write_emptied() {
    // Track writes the pair it selects over the note slot: `D4` when the pair
    // holds it, two blank Cells when it is empty.
    for play in ["  !>017FC4", "  !~017FC404", "  !%017FC404"] {
        let (commands, diagnostics, _) = run(&[".=0101  &t0001D4", play]);
        assert_eq!(commands.len(), 1, "{play:?}: {diagnostics:?}");
        assert_eq!(diagnostics, vec![], "{play:?}");

        let (commands, diagnostics, row) = run(&[".=0101  &t0001  ", play]);
        assert_eq!(commands, vec![], "{play:?}");
        assert_eq!(diagnostics, vec![], "{play:?}");
        assert_eq!(&row[8..10], "  ", "{play:?}: Track emptied the slot");

        // Once the slot is blank in Source, the next Tick and the Language
        // Map rebuilt from it rest too, rather than refusing the blank note.
        assert_eq!(
            two_ticks(&[".=0101  &t0001  ", play]),
            vec![(vec![], vec![], vec![]); 2],
            "{play:?}"
        );
    }
}

#[test]
fn a_blank_note_a_portal_fills_plays_the_note_written() {
    // The same Play is a rest with Track's pair empty, and sounds what Track
    // writes the moment the pair holds a Note.
    for play in ["  !>017F", "  !~017F  04", "  !%017F  04"] {
        let (commands, diagnostics, row) = run(&[".=0101  &t0001D4", play]);
        assert_eq!(commands.len(), 1, "{play:?}: {diagnostics:?}");
        assert_eq!(diagnostics, vec![], "{play:?}");
        assert_eq!(&row[8..10], "D4", "{play:?}");
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
fn a_blank_note_beside_a_blank_or_undecodable_operand_is_diagnosed() {
    let grid = Grid::with_shape(16, 3);
    for play in [
        "  !>  7F",
        "  !>01  ",
        "  !~017F    ",
        "  !%  7F  04",
        "  !>ZZ7F",
        "  !>01ZZ",
        "  !~017F  ZZ",
    ] {
        assert_eq!(banged(play).0, vec![], "{play:?}");
        assert!(
            mapped(grid, &[".=0101", play]).contains(&(2, 1)),
            "{play:?}"
        );
    }
    // A nested Function that clears another operand is diagnosed even though
    // the note slot is written blank.
    assert_eq!(
        banged("  !>01&v01  "),
        (
            vec![],
            vec![diagnostic(
                2,
                1,
                "nested computation at column 6, row 1 cleared its operand"
            )]
        )
    );
    // A note a Portal emptied, beside a length the Source left blank.
    let (commands, diagnostics, _) = run(&[".=0101  &t0001  ", "  !~017FC4"]);
    assert_eq!(commands, vec![]);
    assert_eq!(
        diagnostics,
        [diagnostic(2, 1, "expected a number, found \"  \"")]
    );
}

#[test]
fn a_rest_does_not_evaluate_a_channel_outside_its_domain() {
    // `80` decodes as a Number, and only the Interpreter asks whether it is a
    // MIDI channel. A rest takes no evaluation, so it reports nothing, and the
    // same channel with a note is refused.
    assert_eq!(banged("  !>807F"), (vec![], vec![]));
    let (commands, diagnostics) = banged("  !>807FC4");
    assert_eq!(commands, vec![]);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
}

/// The Play Commands and diagnostics of the first Tick of `rows`, on a Grid
/// wide and deep enough for a Chord Play with nested operands.
fn chord_tick(rows: &[&str]) -> (Vec<PlayCommand>, Vec<(usize, usize, String)>) {
    let mut source = source_of(Grid::with_shape(40, 4), rows);
    let plan = source.execute(Tick::ZERO);
    let diagnostics = plan
        .diagnostics
        .iter()
        .map(|d| (d.anchor().x(), d.anchor().y(), d.message.clone()))
        .collect();
    (plan.play_commands, diagnostics)
}

/// The Tick of a Chord Play on row 1, beneath the Equality that Bangs it.
fn chord(play: &str) -> (Vec<PlayCommand>, Vec<(usize, usize, String)>) {
    chord_tick(&[".=0101", play])
}

/// The Language Map's diagnostics for a Chord Play on row 1.
fn chord_mapped(play: &str) -> Vec<(usize, usize)> {
    mapped(Grid::with_shape(40, 4), &[".=0101", play])
}

#[test]
fn both_chord_plays_rest_on_a_root_written_blank() {
    for play in ["  !:017F  M7CL00", "  !#017F  M7CL0004"] {
        assert_eq!(chord(play), (vec![], vec![]), "{play:?}");
        assert_eq!(chord_mapped(play), vec![], "{play:?}");
    }
}

#[test]
fn the_language_map_admits_a_chord_play_with_a_blank_root_as_a_root() {
    for play in [
        "  !:017F  M7CL00",
        "  !#017F  M7CL0004",
        "  !:01    M7CL00",
        "  !#              ",
    ] {
        let source = source_of(Grid::with_shape(40, 4), &[".=0101", play]);
        let map = source.language_map();

        assert!(
            map.expressions().any(|expression| expression
                .root()
                .is_some_and(|root| (root.x(), root.y()) == (2, 1))),
            "{play:?}"
        );
    }
}

#[test]
fn both_chord_plays_rest_on_a_root_a_portal_emptied() {
    // Track writes the pair it selects over the root: `D4` when the pair holds
    // it, two blank Cells when it is empty.
    for play in ["  !:017FC4M7CL00", "  !#017FC4M7CL0004"] {
        let (commands, diagnostics) = chord_tick(&[".=0101  &t0001D4", play]);
        assert_eq!(commands.len(), 4, "{play:?}: {diagnostics:?}");
        assert_eq!(diagnostics, vec![], "{play:?}");

        assert_eq!(
            chord_tick(&[".=0101  &t0001  ", play]),
            (vec![], vec![]),
            "{play:?}"
        );
    }
}

#[test]
fn both_chord_plays_rest_on_a_root_a_nested_read_cleared() {
    for play in ["  !:017F&v01M7CL00", "  !#017F&v01M7CL0004"] {
        assert_eq!(chord(play), (vec![], vec![]), "{play:?}");
        assert_eq!(chord_mapped(play), vec![], "{play:?}");
    }
}

#[test]
fn a_blank_root_ignores_every_other_blank_operand() {
    for play in [
        "  !:  7F  M7CL00",
        "  !:01    M7CL00",
        "  !:017F    CL00",
        "  !:017F  M7  00",
        "  !:017F  M7CL  ",
        "  !#017F  M7CL00  ",
        // A wholly empty row.
        "  !:            ",
        "  !#              ",
    ] {
        assert_eq!(chord(play), (vec![], vec![]), "{play:?}");
        assert_eq!(chord_mapped(play), vec![], "{play:?}");
    }
}

#[test]
fn a_blank_root_ignores_every_other_cleared_operand() {
    // Each `&v01` reads the blank pair south of its operand.
    for play in [
        // A root written blank.
        "  !:&v017F  M7CL00",
        "  !:01&v01  M7CL00",
        "  !:017F  &v01CL00",
        "  !:017F  M7&v0100",
        "  !:017F  M7CL&v01",
        "  !#017F  M7CL00&v01",
        // A root a nested Read cleared.
        "  !:01&v01&v01M7CL00",
        "  !:017F&v01&v01CL00",
        "  !#017F&v01M7&v0100&v01",
    ] {
        assert_eq!(chord(play), (vec![], vec![]), "{play:?}");
    }
}

#[test]
fn a_root_a_nested_read_cleared_ignores_an_operand_left_blank_in_source() {
    for play in [
        "  !:  7F&v01M7CL00",
        "  !:017F&v01  CL00",
        "  !#017F&v01M7CL00  ",
    ] {
        assert_eq!(chord(play), (vec![], vec![]), "{play:?}");
        assert_eq!(chord_mapped(play), vec![], "{play:?}");
    }
}

#[test]
fn a_root_a_portal_emptied_ignores_an_operand_a_nested_read_cleared() {
    assert_eq!(
        chord_tick(&[".=0101  &t0001  ", "  !:017FC4&v01CL00"]),
        (vec![], vec![])
    );
}

#[test]
fn a_root_written_in_source_refuses_an_operand_left_blank_until_a_portal_empties_it() {
    // The Language Map reads the root as Source holds it before the Tick, so
    // the blank operand beside it is refused there. That refusal holds the
    // Turn, which sends nothing and repeats nothing. Track leaves the root
    // blank in Source, so the Map rebuilt after the Tick, and every later
    // Tick, rest.
    for play in ["  !:017FC4  CL00", "  !#017FC4  CL0004", "  !:  7FC4M7CL00"] {
        let rows = [".=0101  &t0001  ", play];
        assert!(
            mapped(Grid::with_shape(40, 4), &rows).contains(&(2, 1)),
            "{play:?}"
        );
        for (tick, (commands, observed, mapped)) in executed(Grid::with_shape(40, 4), &rows, 0..2)
            .into_iter()
            .enumerate()
        {
            assert_eq!(commands, vec![], "{play:?} Tick {tick}");
            assert_eq!(observed.diagnostics, vec![], "{play:?} Tick {tick}");
            assert_eq!(mapped, vec![], "{play:?} Tick {tick}");
            assert_eq!(&observed.rows[1][8..10], "  ", "{play:?} Tick {tick}");
        }
    }
}

#[test]
fn a_chord_rest_does_not_evaluate_a_channel_outside_its_domain() {
    assert_eq!(chord("  !:807F  M7CL00"), (vec![], vec![]));
    let (commands, diagnostics) = chord("  !:807FC4M7CL00");
    assert_eq!(commands, vec![]);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
}

#[test]
fn a_half_written_operand_beside_a_blank_root_is_diagnosed() {
    // Written in Source, the Language Map refuses it and the Tick is silent.
    for play in [
        "  !:017F  M CL00",
        "  !:0 7F  M7CL00",
        "  !:017F  M7C 00",
        "  !#017F  M7CL000 ",
        "  !:017F  ZZCL00",
        // The rest of the row blank does not excuse it.
        "  !:017F  M     ",
    ] {
        assert_eq!(chord(play).0, vec![], "{play:?}");
        assert!(chord_mapped(play).contains(&(2, 1)), "{play:?}");
    }

    // A nested Read of a half-typed pair: the Read refuses it and the Chord
    // Play names the Read, though its root rests. A `&v02` reads the pair two
    // rows south of its operand, two Cells east of its anchor.
    let (commands, diagnostics) =
        chord_tick(&[".=0101", "  !:017F  &v02CL00", "", "            M "]);
    assert_eq!(commands, vec![]);
    assert!(
        diagnostics.iter().any(|(column, _, message)| *column == 10
            && message.ends_with("has partial or invalid input")),
        "{diagnostics:?}"
    );

    // A Portal writing one Cell of a pair across the set's second Cell.
    let (commands, diagnostics) = chord_tick(&[".=0101     &t0001M7", "  !:017F    CL00"]);
    assert_eq!(commands, vec![]);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!((diagnostics[0].0, diagnostics[0].1), (2, 1));
    assert!(diagnostics[0].2.contains("\" M\""), "{diagnostics:?}");
}

#[test]
fn a_half_written_root_is_diagnosed_by_every_route() {
    for play in ["  !:017FC M7CL00", "  !#017F CM7CL0004"] {
        assert_eq!(chord(play).0, vec![], "{play:?}");
        assert!(chord_mapped(play).contains(&(2, 1)), "{play:?}");
    }

    let (commands, diagnostics) = chord_tick(&[".=0101", "  !:017F&v02M7CL00", "", "          C "]);
    assert_eq!(commands, vec![]);
    assert!(
        diagnostics.iter().any(|(column, _, message)| *column == 8
            && message.ends_with("has partial or invalid input")),
        "{diagnostics:?}"
    );

    // A Portal writing `D` onto the root's second Cell, and `4` past it.
    let (commands, diagnostics) = chord_tick(&[".=0101   &t0001D4", "  !:017F  M7CL00"]);
    assert_eq!(commands, vec![]);
    assert_eq!(
        diagnostics,
        [diagnostic(2, 1, "expected a note, found \" D\"")]
    );
}

#[test]
fn a_blank_interval_set_with_a_root_present_is_diagnosed() {
    // Written blank in Source beside a written root.
    for play in ["  !:017FC4  CL00", "  !#017FC4  CL0004"] {
        assert_eq!(chord(play).0, vec![], "{play:?}");
        assert!(chord_mapped(play).contains(&(2, 1)), "{play:?}");
    }

    // Emptied by a Portal.
    let (commands, diagnostics) = chord_tick(&[".=0101    &t0001  ", "  !:017FC4M7CL00"]);
    assert_eq!(commands, vec![]);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!((diagnostics[0].0, diagnostics[0].1), (2, 1));

    // Cleared by a nested Read.
    let (commands, diagnostics) = chord("  !:017FC4&v01CL00");
    assert_eq!(commands, vec![]);
    assert_eq!(
        diagnostics,
        [diagnostic(
            2,
            1,
            "nested computation at column 10, row 1 cleared its operand"
        )]
    );

    // Left blank beside a nested Read that answers a root: the Turn decodes
    // what the Read answered and finds no set for it.
    let (commands, diagnostics) = chord_tick(&[".=0101", "  !:017F&v02  CL00", "", "          C4"]);
    assert_eq!(commands, vec![]);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!((diagnostics[0].0, diagnostics[0].1), (2, 1));

    // Left blank beside a root a Portal writes over blank Source: the
    // Language Map admits the row, since its root is blank there, so only the
    // Turn can find no set for the root.
    let rows = [".=0101  &t0001D4", "  !:017F    CL00"];
    assert!(!mapped(Grid::with_shape(40, 4), &rows).contains(&(2, 1)));
    let (commands, diagnostics) = chord_tick(&rows);
    assert_eq!(commands, vec![]);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!((diagnostics[0].0, diagnostics[0].1), (2, 1));
    assert!(diagnostics[0].2.contains("\"  \""), "{diagnostics:?}");
}

#[test]
fn a_blank_velocity_or_length_with_a_root_present_is_diagnosed() {
    for play in [
        "  !:  7FC4M7CL00",
        "  !:01  C4M7CL00",
        "  !:017FC4M7  00",
        "  !:017FC4M7CL  ",
        "  !#017FC4M7CL00  ",
    ] {
        assert_eq!(chord(play).0, vec![], "{play:?}");
        assert!(chord_mapped(play).contains(&(2, 1)), "{play:?}");
    }
}

#[test]
fn a_blank_note_beside_a_cleared_operand_is_diagnosed_at_every_tick() {
    // The note is blank in Source, which alone would rest the Play, but a
    // `&v01` reading the blank pair south of its operand clears the channel,
    // velocity or length beside it. In the last case a Read of blank Cells
    // clears the velocity. The Language Map has nothing to refuse, since the
    // Clear is a value only the Turn sees, so the Tick reports it each time.
    for (play, child) in [
        ("  !~&v017F  04", 4),
        ("  !~01&v01  04", 6),
        ("  !~017F  &v01", 10),
        ("  !>&v017F  ", 4),
        ("  !%01&v01  04", 6),
        ("  !~01&>03  04", 6),
    ] {
        let cleared = diagnostic(
            2,
            1,
            &format!("nested computation at column {child}, row 1 cleared its operand"),
        );
        assert_eq!(
            two_ticks(&[".=0101", play]),
            vec![(vec![], vec![cleared], vec![]); 2],
            "{play:?}"
        );
    }
}
