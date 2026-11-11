//! A MIDI output whose operand Cells are empty sends nothing and reports
//! nothing, as Orca's `:` returns when a port it needs is empty. The output
//! sees only the Cells: written empty in Source, emptied by a Portal write, or
//! returned empty by a nested Function are the same two Cells. One empty Cell
//! beside a written one is still partial input, and a value Function still
//! refuses an empty operand.
//!
//! Equality at (0, 0) Bangs the output at (2, 1) on every Tick.

use std::ops::Range;

use lang::{PlayCommand, Tick};

use super::observed::{Observed, diagnostic, first, first_tick, observed, source_of};
use crate::grid::Grid;
use crate::source::Source;

/// Diagnostics by anchor column, row and message, as [`Observed`] holds them.
type Diagnostics = Vec<(usize, usize, String)>;

/// Every MIDI output, with an operand list each Turn sounds.
const OUTPUTS: [(&str, &[&str]); 7] = [
    ("!>", &["01", "7F", "C4"]),
    ("!~", &["01", "7F", "C4", "04"]),
    ("!%", &["01", "7F", "C4", "04"]),
    ("!:", &["01", "7F", "C4", "M7", "CL", "00"]),
    ("!#", &["01", "7F", "C4", "M7", "CL", "00", "04"]),
    ("!c", &["01", "07", "40"]),
    ("!b", &["01", "00", "40"]),
];

/// The Grid every case runs on: wide enough for a Timed Chord Play with a
/// nested operand, and deep enough for the empty row a nested Read selects.
fn grid() -> Grid {
    Grid::with_shape(40, 4)
}

/// Row 1 for `spelling` with `operands`, anchored where the Equality Bangs.
fn row(spelling: &str, operands: &[&str]) -> String {
    format!("  {spelling}{}", operands.concat())
}

/// `operands` with operand `index` replaced by `cells`.
fn with<'a>(operands: &[&'a str], index: usize, cells: &'a str) -> Vec<&'a str> {
    let mut operands = operands.to_vec();
    operands[index] = cells;
    operands
}

/// Row 1 for `spelling` with operand `index` replaced by `cells`.
fn row_with(spelling: &str, operands: &[&str], index: usize, cells: &str) -> String {
    row(spelling, &with(operands, index, cells))
}

/// Each Tick in `ticks` of `rows`: its Play Commands, the Tick observed, and
/// the Language Map's diagnostics on row 1 read after it.
fn executed(rows: &[&str], ticks: Range<u64>) -> Vec<(Vec<PlayCommand>, Observed, Diagnostics)> {
    let mut source = source_of(grid(), rows);
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

/// The Play Commands and the Tick's diagnostics of the first Tick of `rows`.
fn run(rows: &[&str]) -> (Vec<PlayCommand>, Diagnostics) {
    first_tick(grid(), rows)
}

/// The first Tick of `output` on row 1, beneath the Equality that Bangs it.
fn banged(output: &str) -> (Vec<PlayCommand>, Diagnostics) {
    run(&[".=0101", output])
}

/// The Source for `output` on row 1, beneath the Equality that Bangs it.
fn source(output: &str) -> Source {
    source_of(grid(), &[".=0101", output])
}

/// The Language Map's diagnostics for `output` on row 1.
fn mapped(output: &str) -> Vec<(usize, usize)> {
    source(output)
        .language_map()
        .diagnostics()
        .map(|d| (d.anchor().x(), d.anchor().y()))
        .collect()
}

/// Whether the Language Map schedules `output` on row 1 as a root.
fn is_root(output: &str) -> bool {
    source(output)
        .language_map()
        .expressions()
        .any(|expression| {
            expression
                .root()
                .is_some_and(|root| (root.x(), root.y()) == (2, 1))
        })
}

#[test]
fn every_midi_output_sends_when_every_operand_is_written() {
    for (spelling, operands) in OUTPUTS {
        let output = row(spelling, operands);
        let (commands, diagnostics) = banged(&output);
        assert!(!commands.is_empty(), "{output:?}: {diagnostics:?}");
        assert_eq!(diagnostics, vec![], "{output:?}");
    }
}

#[test]
fn an_empty_operand_written_in_source_sends_nothing_and_reports_nothing() {
    for (spelling, operands) in OUTPUTS {
        for index in 0..operands.len() {
            let output = row_with(spelling, operands, index, "  ");
            assert_eq!(banged(&output), (vec![], vec![]), "{output:?}");
            assert_eq!(mapped(&output), vec![], "{output:?}");
            assert!(is_root(&output), "{output:?}");
        }
    }
}

#[test]
fn a_wholly_empty_row_sends_nothing_and_reports_nothing() {
    for (spelling, operands) in OUTPUTS {
        let output = row(spelling, &vec!["  "; operands.len()]);
        assert_eq!(banged(&output), (vec![], vec![]), "{output:?}");
        assert_eq!(mapped(&output), vec![], "{output:?}");
    }
}

#[test]
fn an_operand_a_nested_function_returns_empty_sends_nothing_and_reports_nothing() {
    // Each Read selects the empty row beneath it, so it returns two empty
    // Cells into the operand it stands in.
    for (spelling, operands) in OUTPUTS {
        for index in 0..operands.len() {
            let output = row_with(spelling, operands, index, "&v01");
            assert_eq!(banged(&output), (vec![], vec![]), "{output:?}");
            assert_eq!(mapped(&output), vec![], "{output:?}");
        }
    }
}

#[test]
fn an_operand_a_portal_write_emptied_sends_nothing_and_reports_nothing() {
    // A Track above the operand writes the pair it selects over it: the
    // operand's own spelling the first time, and two empty Cells the second.
    // A Bang written beside the output activates it every Tick, which leaves
    // the row above free for a Track over any operand.
    for (spelling, operands) in OUTPUTS {
        let output = format!("**{spelling}{}", operands.concat());
        for (index, operand) in operands.iter().enumerate() {
            let track = format!("{:1$}&t0001", "", 2 + spelling.len() + 2 * index);
            let (commands, diagnostics) = run(&[&format!("{track}{operand}"), &output]);
            assert!(!commands.is_empty(), "{output:?} {index}: {diagnostics:?}");
            assert_eq!(diagnostics, vec![], "{output:?} {index}");

            let emptied = executed(&[&format!("{track}  "), &output], 0..2);
            for (commands, tick, mapped) in emptied {
                assert_eq!(commands, vec![], "{output:?} {index}");
                assert_eq!(tick.diagnostics, vec![], "{output:?} {index}");
                assert_eq!(mapped, vec![], "{output:?} {index}");
            }
        }
    }
}

#[test]
fn a_half_written_operand_is_diagnosed_even_beside_an_empty_one() {
    for (spelling, operands) in OUTPUTS {
        for index in 0..operands.len() {
            let half = format!("{} ", &operands[index][..1]);
            let output = row_with(spelling, operands, index, &half);
            assert_eq!(banged(&output).0, vec![], "{output:?}");
            assert!(!mapped(&output).is_empty(), "{output:?}");

            // An empty operand anywhere else does not hide it.
            let beside = (index + 1) % operands.len();
            let output = row_with(spelling, &with(operands, index, &half), beside, "  ");
            assert_eq!(banged(&output).0, vec![], "{output:?}");
            assert!(!mapped(&output).is_empty(), "{output:?}");
        }
    }
}

#[test]
fn an_operand_that_does_not_decode_is_diagnosed_beside_an_empty_one() {
    // `xx` spells no Number, Note, Interval Set or Voicing.
    for (spelling, operands) in OUTPUTS {
        let last = operands.len() - 1;
        let output = row_with(spelling, &with(operands, last, "  "), 0, "xx");
        assert_eq!(banged(&output).0, vec![], "{output:?}");
        assert!(!mapped(&output).is_empty(), "{output:?}");
    }
}

#[test]
fn a_later_operand_that_fails_at_the_turn_is_diagnosed_after_an_empty_one() {
    // The channel is empty in Source, so the Language Map admits the row. On
    // the Turn a Track writes a Language Unit of the wrong type over the
    // third operand, or a Read returns it there from the row beneath that
    // operand. The empty channel decoded first does not hide the refusal.
    for (spelling, operands) in OUTPUTS {
        // A Number where a note or root belongs, or a Note where a Number does.
        let wrong = if matches!(spelling, "!c" | "!b") {
            "G4"
        } else {
            "60"
        };
        let column = 2 + spelling.len() + 2 * 2;
        let output = format!("**{spelling}{}", with(operands, 0, "  ").concat());
        let track = format!("{:column$}&t0001{wrong}", "");
        let (commands, diagnostics) = run(&[&track, &output]);
        assert_eq!(commands, vec![], "{output:?}");
        assert!(
            diagnostics.iter().any(|(x, y, _)| (*x, *y) == (2, 1)),
            "{output:?}: {diagnostics:?}"
        );

        let nested = format!(
            "**{spelling}{}",
            with(&with(operands, 0, "  "), 2, "&v01").concat()
        );
        let below = format!("{:1$}{wrong}", "", column + 2);
        let (commands, diagnostics) = run(&["", &nested, &below]);
        assert_eq!(commands, vec![], "{nested:?}");
        assert!(
            diagnostics.iter().any(|(x, y, _)| (*x, *y) == (2, 1)),
            "{nested:?}: {diagnostics:?}"
        );
    }
}

#[test]
fn a_nested_function_that_is_invalid_leaves_the_output_invalid() {
    // A half-typed Note read into the note: the Read refuses it, so the Play
    // has no Return and says so.
    assert_eq!(
        run(&[".=0101", "  !~017F&>0204C "]),
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
}

#[test]
fn an_empty_output_evaluates_no_operand() {
    // `80` decodes as a Number, and only the Interpreter asks whether it is a
    // MIDI channel. With the note empty nothing is evaluated, so nothing is
    // reported; with a note the channel is refused.
    assert_eq!(banged("  !>807F  "), (vec![], vec![]));
    let (commands, diagnostics) = banged("  !>807FC4");
    assert_eq!(commands, vec![]);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
}

#[test]
fn the_nested_read_still_writes_its_empty_cells() {
    // The Read takes its Turn and writes the empty Cells it copied through its
    // Output Portal; only the output it returns into has nothing to send.
    let tick = first(
        Grid::with_shape(16, 3),
        &[".=0101", "  !~017F&>0204", "        xx"],
    );
    assert_eq!(tick.rows[2], "                ");
    assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
}

#[test]
fn a_value_function_still_refuses_an_empty_operand() {
    // Only MIDI outputs send nothing on empty Cells. A value Function given
    // empty Cells, written or returned, is invalid input.
    for source in [".+01  ", "#+C4  ", ".v  "] {
        let map = source_of(Grid::with_shape(10, 2), &[source]);
        assert!(
            map.language_map().diagnostics().next().is_some(),
            "{source:?}"
        );
    }
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
