//! Interval Set and Voicing names carried by the Functions that move a
//! Language Unit, Tick by Tick. Each name is two Cells, and whichever Function
//! moves them leaves them exactly as written.

use super::observed::{diagnostic, first, quiet, source_of};
use crate::grid::Grid;

/// One name of each shape: a chord, a scale, the names that also spell a
/// Number or a Note, and the Voicings.
const NAMES: [&str; 10] = ["M7", "io", "69", "13", "07", "d7", "a9", "CL", "RL", "D2"];

/// Half-written Interval Set and Voicing names: one Cell of a name and a
/// blank.
const PARTIALS: [&str; 4] = ["M ", " 7", "C ", " L"];

#[test]
fn a_copy_relays_each_name_unchanged() {
    for name in NAMES {
        let tick = first(Grid::with_shape(6, 1), &[&format!("{name}=>")]);
        assert_eq!(tick.rows, [format!("{name}=>{name}")], "{name}");
        quiet(&tick);
    }
}

#[test]
fn a_read_answers_each_name_unchanged() {
    for name in NAMES {
        let row = format!("&>01{name}");
        let tick = first(Grid::with_shape(6, 2), &[&row]);
        assert_eq!(tick.rows, [row, format!("{name:6}")], "{name}");
        quiet(&tick);
    }
}

#[test]
fn a_track_answers_each_name_it_selects_unchanged() {
    for name in NAMES {
        let row = format!("&t0102xx{name}");
        let tick = first(Grid::with_shape(10, 2), &[&row]);
        assert_eq!(tick.rows, [row, format!("{name:10}")], "{name}");
        quiet(&tick);
    }
}

#[test]
fn a_write_carries_each_name_to_its_pair_unchanged() {
    for name in NAMES {
        let row = format!("@$0601{name}");
        let tick = first(Grid::with_shape(8, 2), &[&row]);
        assert_eq!(tick.rows, [row, format!("      {name}")], "{name}");
        quiet(&tick);
    }
}

#[test]
fn a_push_carries_each_name_to_its_pair_unchanged() {
    for name in NAMES {
        let row = format!("@t0102{name}");
        let tick = first(Grid::with_shape(8, 2), &[&row, "xxxxxxxx"]);
        assert_eq!(tick.rows, [row, format!("xx{name}xxxx")], "{name}");
        quiet(&tick);
    }
}

#[test]
fn a_half_written_name_is_still_diagnosed_rather_than_carried() {
    for partial in PARTIALS {
        let tick = first(Grid::with_shape(6, 1), &[&format!("{partial}=>xx")]);
        assert_eq!(
            tick.rows,
            [format!("{partial}=>xx")],
            "{partial:?} wrote over the Output Portal"
        );
        assert_eq!(
            tick.diagnostics,
            [diagnostic(2, 0, "=> has partial or invalid input")],
            "{partial:?} is refused by the Copy's reading"
        );
    }
}

#[test]
fn a_read_or_track_diagnoses_a_half_written_name_and_writes_nothing() {
    for partial in PARTIALS {
        for (row, width, spelling) in [
            (format!("&>01{partial}"), 6, "&>"),
            (format!("&t0102xx{partial}"), 10, "&t"),
        ] {
            let below = "x".repeat(width);
            let tick = first(Grid::with_shape(width, 2), &[&row, &below]);
            assert_eq!(tick.rows, [row.clone(), below], "{row:?} wrote its answer");
            assert!(
                tick.diagnostics.contains(&diagnostic(
                    0,
                    0,
                    &format!("{spelling} has partial or invalid input")
                )),
                "{row:?}: {:?}",
                tick.diagnostics
            );
        }
    }
}

#[test]
fn a_write_or_push_refuses_a_half_written_name_and_writes_nothing() {
    // The Language Map diagnoses the half-written value operand, and the Tick
    // writes nothing and does not report it again.
    for partial in PARTIALS {
        for row in [format!("@$0601{partial}"), format!("@t0102{partial}")] {
            let rows = [row.as_str(), "xxxxxxxx"];
            let source = source_of(Grid::with_shape(8, 2), &rows);
            assert!(
                source
                    .language_map()
                    .diagnostics()
                    .any(|diagnostic| diagnostic.start() == 0),
                "{row:?} is not diagnosed by the Language Map"
            );
            let tick = first(Grid::with_shape(8, 2), &rows);
            assert_eq!(tick.rows, rows, "{row:?} wrote its value");
            quiet(&tick);
        }
    }
}
