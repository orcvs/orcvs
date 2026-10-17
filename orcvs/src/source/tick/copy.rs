//! The directional Copies: relaying one aligned Language Unit, clearing on an
//! empty input, and carrying a copied Bang into empty or occupied Source.

use crate::grid::Grid;

use super::fixture::{messages, tick_by_tick};
use super::observed::raw;

#[test]
fn a_single_east_copy_relays_one_aligned_language_unit() {
    // `=>` reads the aligned two-Cell unit at its input Portal and writes
    // that unit through its output Portal.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 1), &["01=>    "], 1);
    assert_eq!(grids[0], ["01=>01  "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn each_copy_direction_relays_one_aligned_language_unit() {
    let (_, north, _) = tick_by_tick(Grid::with_shape(2, 3), &["  ", "=^", "01"], 1);
    assert_eq!(north[0], ["01", "=^", "01"]);

    let (_, south, _) = tick_by_tick(Grid::with_shape(2, 3), &["01", "=v", "  "], 1);
    assert_eq!(south[0], ["01", "=v", "01"]);

    let (_, west, _) = tick_by_tick(Grid::with_shape(8, 1), &["    =<01"], 1);
    assert_eq!(west[0], ["  01=<01"]);
}

#[test]
fn consecutive_copies_compose_through_overlapping_portals() {
    // Each Copy's output Portal is its own displacement. Two touching
    // east Copies overlap: the first writes onto the second and suppresses
    // it, the same way any Source write covering a Function does.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(10, 1), &["01=>=>    "], 1);
    assert_eq!(grids[0], ["01=>01    "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );

    let (_, north, _) = tick_by_tick(Grid::with_shape(2, 4), &["  ", "=^", "=^", "01"], 1);
    assert_eq!(north[0], ["  ", "01", "=^", "01"]);

    let (_, south, _) = tick_by_tick(Grid::with_shape(2, 4), &["01", "=v", "=v", "  "], 1);
    assert_eq!(south[0], ["01", "=v", "01", "  "]);

    let (_, west, _) = tick_by_tick(Grid::with_shape(10, 1), &["    =<=<01"], 1);
    assert_eq!(west[0], ["    01=<01"]);
}

#[test]
fn each_copy_uses_its_own_portals() {
    // A space between two `=>` is each Function's own Portals: the first
    // writes into the gap; the second reads that write and writes past
    // itself.
    let (_, gap, _) = tick_by_tick(Grid::with_shape(12, 1), &["01=>  =>    "], 1);
    assert_eq!(gap[0], ["01=>01=>01  "]);

    // `=v` sits on `=>`'s output Portal, so the east Copy writes onto it.
    let (_, split, _) = tick_by_tick(Grid::with_shape(8, 1), &["01=>=v  "], 1);
    assert_eq!(split[0], ["01=>01  "]);
}

#[test]
fn a_copy_overwrites_an_occupied_destination() {
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(6, 1), &["01=>xx"], 1);
    assert_eq!(grids[0], ["01=>01"]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn an_empty_copy_input_clears_the_destination() {
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(6, 1), &["  =>xx"], 1);
    assert_eq!(grids[0], ["  =>  "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn a_partial_copy_input_diagnoses_and_writes_nothing() {
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(6, 1), &["0 =>xx"], 1);
    assert_eq!(grids[0], ["0 =>xx"]);
    assert_eq!(messages(&plans[0]), vec!["=> has partial or invalid input"]);
}

#[test]
fn a_copy_does_not_transport_an_incomplete_language_unit() {
    // Cells 3–4 of `.+0102` are the last Cell of `01` and the first of
    // `02`: a slice across two Language Units, not one unit to copy.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 3), &[".+0102", "   =v", ""], 1);
    assert_eq!(grids[0], [".+0102  ", "03 =v   ", "        "]);
    assert_eq!(messages(&plans[0]), vec!["=v has partial or invalid input"]);
}

#[test]
fn a_copy_copies_a_same_tick_result_as_a_language_unit() {
    // `.+0001` writes `01` on the row below this Tick, and the Copy reads
    // that Cell pair as one aligned Language Unit and relays it.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 2), &[".+0001", "  =>xx"], 1);
    assert_eq!(grids[0][0], ".+0001  ");
    assert_eq!(grids[0][1], "01=>01  ");
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn an_out_of_grid_copy_destination_diagnoses_and_writes_nothing() {
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(4, 1), &["01=>"], 1);
    assert_eq!(grids[0], ["01=>"]);
    assert_eq!(
        messages(&plans[0]),
        vec![r#"result "01" falls outside the Grid"#]
    );
}

#[test]
fn a_copied_bang_writes_into_empty_source() {
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(10, 2), &[".=0101", "  =>"], 1);
    assert_eq!(grids[0], [".=0101    ", "**=>**    "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn a_copied_bang_activates_a_neighbour_of_empty_source() {
    // `**` written into empty Source is ordinary Bang output: a MIDI
    // south of that write sounds this Tick.
    let (plans, grids, _) = tick_by_tick(
        Grid::with_shape(12, 3),
        &[".=0101", "  =>", "    !>007FC4"],
        1,
    );
    assert_eq!(grids[0][1], "**=>**      ");
    assert_eq!(plans[0].play_commands.len(), 1);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn a_copied_bang_overwrites_a_roots_anchor_and_activates_the_roots_aligned_with_it() {
    // Equality Bangs on every Tick; the Copy writes that Bang over Raw
    // Play C4's anchor, so C4 does not run, and activates Raw Play D4
    // south of it.
    let (plans, grids, _) = tick_by_tick(
        Grid::with_shape(12, 3),
        &[".=0101", "  =>!>007FC4", "    !>007FD4"],
        1,
    );
    assert_eq!(grids[0][1], "**=>**007FC4");
    assert_eq!(plans[0].play_commands, vec![raw(0, 0x7F, 62)]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn a_copied_bang_overwrites_a_root_above_the_copy() {
    // MIDI sits at the earliest Position. A North Copy below it writes
    // the Bang over that root's anchor, and the root does not run.
    let (plans, grids, _) = tick_by_tick(
        Grid::with_shape(10, 3),
        &["!>007FC4", "=^  .=0101", "  =<"],
        1,
    );
    assert_eq!(grids[0][0], "**007FC4  ");
    assert!(plans[0].play_commands.is_empty());
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn a_copied_bang_overwrites_occupied_data_and_activates_the_roots_aligned_with_it() {
    let (plans, grids, _) = tick_by_tick(
        Grid::with_shape(12, 3),
        &[".=0101", "  =>xx", "    !>007FD4"],
        1,
    );
    assert_eq!(grids[0][1], "**=>**      ");
    assert_eq!(plans[0].play_commands, vec![raw(0, 0x7F, 62)]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn a_copied_bang_leaving_the_grid_diagnoses_and_writes_nothing() {
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 2), &["  .=0101", "=<"], 1);
    assert_eq!(grids[0], ["  .=0101", "=<**    "]);
    assert_eq!(
        messages(&plans[0]),
        vec![r#"result "**" falls outside the Grid"#]
    );
}

#[test]
fn a_chain_of_copies_carries_a_bang_down_a_column_in_one_tick() {
    // `.=` Bangs into the first `=v`'s Input Portal, and each `=v`
    // writes `**` that the next reads, until the last activates Raw Play.
    let (plans, grids, _) = tick_by_tick(
        Grid::with_shape(8, 7),
        &[".=0101", "", "=v", "", "=v", "", "!>007FC4"],
        1,
    );
    assert_eq!(
        grids[0],
        [
            ".=0101  ", "**      ", "=v      ", "**      ", "=v      ", "**      ", "!>007FC4",
        ]
    );
    assert_eq!(plans[0].play_commands, vec![raw(0, 0x7F, 60)]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn a_copied_function_is_not_actionable_until_the_next_snapshot() {
    // `>>` copied this Tick is in the Grid but not in the Snapshot the
    // schedule was built from, so it first moves on the following Tick.
    // Delay Bangs once; `*>` emits `>>`; the Copy copies that spelling.
    // The copy sits at columns 8–9 on this Tick. If it were actionable
    // from the Snapshot that wrote it, it would already have moved to 9.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(16, 2), &["~*0401", "  *>  =>"], 1);
    assert_eq!(grids[0][1], "***>>>=>>>      ");
    for plan in &plans {
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }
}

#[test]
fn a_copy_below_its_consumer_reaches_it_the_same_tick() {
    // Backward routing is ordinary: the Copy is below Addition and still
    // supplies `02` in time for `03` to be written this Tick.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(6, 3), &[".+  02", "  =^", "  01"], 1);
    assert_eq!(grids[0], [".+0102", "03=^  ", "  01  "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn a_nested_copy_reads_a_same_tick_write_at_its_input_portal() {
    // Nested `=^` reads the Portal one row south and writes the unit it
    // copies one row north. The root Copy lands `01` at its input this
    // Tick; without that read span the nested Copy would run first and
    // copy empty Source.
    let (plans, grids, _) = tick_by_tick(
        Grid::with_shape(6, 5),
        &["", ".+=^01", "", "  =^", "  01"],
        1,
    );
    assert_eq!(grids[0], ["  01  ", ".+=^01", "0201  ", "  =^  ", "  01  "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );

    // On the top row its output Portal leaves the Grid. That write is
    // refused and diagnosed, and the Return still reaches the parent.
    let (plans, grids, _) =
        tick_by_tick(Grid::with_shape(6, 4), &[".+=^01", "", "  =^", "  01"], 1);
    assert_eq!(grids[0], [".+=^01", "0201  ", "  =^  ", "  01  "]);
    assert_eq!(
        messages(&plans[0]),
        ["result \"01\" falls outside the Grid"]
    );
}
