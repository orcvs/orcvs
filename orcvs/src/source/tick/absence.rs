//! Unwritten operands and the Clear: what an empty pair blocks, and which
//! Functions clear their Output Portal when they do not Bang.

use crate::grid::Grid;

use super::Tick;
use super::fixture::{rows_of, tick_by_tick};
use super::observed::raw;

#[test]
fn an_unwritten_operand_does_not_hide_an_earlier_malformed_one() {
    // `G4` spells no Number. A Portal writes it into the first operand
    // while the second is unwritten: the Addition is invalid, and the
    // first operand it cannot decode is the fault it reports. With the
    // second operand written, the same `G4` diagnoses the same way.
    let grid = Grid::with_shape(8, 3);
    let tick = |rows: &[&str]| super::observed::observe_at(grid, rows, [0]).remove(0);
    for rows in [&[".+    ", "  =^", "  G4"][..], &[".+  01", "  =^", "  G4"]] {
        let decoded = tick(rows);
        assert_eq!(&decoded.rows[0][2..4], "G4", "{rows:?}");
        assert_eq!(
            decoded.diagnostics,
            [(0, 0, "expected a number, found \"G4\"".to_string())],
            "{rows:?}"
        );
    }
}

#[test]
fn an_unwritten_operand_in_the_source_blocks_its_function_as_the_map_reports() {
    // The second operand is unwritten in the Source and nothing writes
    // it, so the Language Map diagnoses the Expression and the Tick does
    // not report it again. The nested `.^43` still answers and writes
    // `G4` south of itself; the Addition it returns to writes nothing.
    let grid = Grid::with_shape(8, 3);
    let mut source = super::observed::source_of(grid, &[".+.^43  ", "xx"]);
    assert!(
        source
            .language_map()
            .diagnostics()
            .any(|diagnostic| diagnostic.start() == 0)
    );

    let plan = source.execute(Tick::ZERO);

    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(rows_of(grid, &source)[1], "xxG4    ");
}

#[test]
fn a_nested_function_invalid_on_an_unwritten_slot_fails_its_parent() {
    // The Copy north copies the empty Cells below it over `.^`'s operand,
    // so `.^` is invalid at its Turn and gives the Addition no operand.
    // Both are diagnosed.
    let grid = Grid::with_shape(8, 4);
    let observed = super::observed::observe_at(grid, &[".+01.^43", "      =^"], [0]).remove(0);
    assert_eq!(observed.rows[0], ".+01.^  ");
    assert_eq!(
        observed.diagnostics,
        [
            (4, 0, "expected a number, found \"  \"".to_string()),
            (
                0,
                0,
                "nested computation at column 4, row 0 returned nothing".to_string()
            ),
        ]
    );
}

#[test]
fn a_nested_equality_that_does_not_bang_leaves_its_parent_operand_unwritten() {
    // Equality with unequal operands clears its own Output Portal, one row
    // south of its anchor, as a root Equality that does not Bang does, so
    // the Addition's operand is unwritten and the Addition is diagnosed,
    // naming the Equality that cleared it.
    let grid = Grid::with_shape(10, 3);
    let observed = super::observed::observe_at(grid, &[".+.=010201", "  C4"], [0]).remove(0);
    assert_eq!(observed.rows[1], "          ");
    assert_eq!(
        observed.diagnostics,
        [(
            0,
            0,
            "nested computation at column 2, row 0 cleared its operand".to_string()
        )]
    );
}

#[test]
fn an_unequal_root_equality_clears_the_pair_south_and_an_equal_one_bangs_there() {
    // Equality answers only Bang or a Clear, so it writes its answer
    // through its Output Portal on every Turn: `**` when its
    // operands are equal, which activates the root aligned south of it,
    // and two empty Cells when they are not.
    let grid = Grid::with_shape(8, 3);
    let unequal = super::observed::first(grid, &[".=0102", "C4", "!>007FC4"]);
    super::observed::quiet(&unequal);
    assert_eq!(unequal.rows, [".=0102  ", "        ", "!>007FC4"]);

    let (plans, grids, _) = tick_by_tick(grid, &[".=0101", "C4", "!>007FC4"], 1);
    assert_eq!(grids[0], [".=0101  ", "**      ", "!>007FC4"]);
    assert_eq!(plans[0].play_commands, [raw(0, 0x7F, 60)]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn delay_and_euclidean_clear_their_output_portal_on_every_tick_they_do_not_bang() {
    // Each Tick starts from Source that holds `C4` in the Output Portal,
    // so a Tick that left it alone would show it. `~*0201` Bangs on every
    // second Tick, and `~%0304` places onsets at steps 0, 2 and 3.
    let grid = Grid::with_shape(8, 2);
    for (function, bangs) in [
        ("~*0201", [true, false, true, false]),
        ("~%0304", [true, false, true, true]),
    ] {
        for (tick, bangs) in (0..).zip(bangs) {
            let observed = super::observed::observe_at(grid, &[function, "C4"], [tick]).remove(0);
            super::observed::quiet(&observed);
            let expected = if bangs { "**      " } else { "        " };
            assert_eq!(observed.rows[1], expected, "{function} at Tick {tick}");
        }
    }
}

#[test]
fn nested_delay_and_euclidean_clear_their_output_portal_on_every_tick_they_do_not_bang() {
    // Nested as the Addition's left operand, each Bang producer's Output
    // Portal is the pair one row south of its own anchor, which holds `C4`
    // at the start of every Tick. `~*0201` Bangs on every second Tick and
    // `~%0304` on steps 0, 2 and 3, so on Tick 1 neither Bangs.
    let grid = Grid::with_shape(10, 2);
    for function in ["~*0201", "~%0304"] {
        let row = format!(".+{function}01");
        let observed = super::observed::observe_at(grid, &[&row, "  C4"], [1]).remove(0);
        assert_eq!(observed.rows[1], "          ", "{function}");
    }
}

#[test]
fn a_bang_written_into_a_copied_operand_fires_once_freed() {
    // Equality's `**` plays the Raw Play aligned east of it, but the `.+`
    // copied in the same Tick makes the `**` that Addition's operand by
    // the commit. An edit then removes the Equality, the Copy and the
    // `.+` without reaching the `**`: it is a root again, so the next
    // Tick fires it as a Bang Function and plays the Raw Play once more.
    let grid = Grid::with_shape(14, 4);
    let mut source = super::observed::source_of(grid, &["  .=0101", "    !>007FC4", "=^", ".+"]);
    let first = source.execute(Tick::ZERO);
    assert_eq!(first.play_commands, [raw(0, 0x7F, 60)]);
    assert_eq!(super::observed::rows_of(&source)[1], ".+**!>007FC4  ");

    for idx in [2, 3, 4, 5, 6, 7, 14, 15, 28, 29, 42, 43] {
        source
            .set(grid.cell_index(idx).expect("inside the Grid"), " ")
            .unwrap();
    }
    assert_eq!(super::observed::rows_of(&source)[1], "  **!>007FC4  ");

    let freed = source.execute(Tick::new(1));
    assert_eq!(freed.play_commands, [raw(0, 0x7F, 60)]);
    assert_eq!(super::observed::rows_of(&source)[1], "    !>007FC4  ");

    let after = source.execute(Tick::new(2));
    assert!(after.play_commands.is_empty(), "{:?}", after.play_commands);
}
