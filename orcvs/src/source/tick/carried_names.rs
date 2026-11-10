//! Interval Set and Voicing names carried by the Functions that move a
//! Language Unit, Tick by Tick. Each name is two Cells, and whichever Function
//! moves them leaves them exactly as written.

use super::observed::{first, quiet};
use crate::grid::Grid;

/// One name of each shape: a chord, a scale, the names that also spell a
/// Number or a Note, and the Voicings.
const NAMES: [&str; 8] = ["M7", "MA", "69", "13", "a9", "CL", "RL", "D2"];

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
    for partial in ["M ", " 7", "C ", " L"] {
        let tick = first(Grid::with_shape(6, 1), &[&format!("{partial}=>xx")]);
        assert_eq!(
            tick.rows,
            [format!("{partial}=>xx")],
            "{partial:?} wrote over the Output Portal"
        );
        assert_eq!(
            tick.diagnostics.len(),
            1,
            "{partial:?}: {:?}",
            tick.diagnostics
        );
    }
}
