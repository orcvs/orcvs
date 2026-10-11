//! Root-only counterparts of the nested Copy layouts, and the spatial layout
//! each nested form is shorthand for, run side by side so a test states
//! where nesting and placement agree and where they part.
//!
//! An Increment `~+0110` stands apart from the Copies in several layouts as a
//! witness: it counts one per Tick, whatever the Copies beside it do, because
//! nothing they stop reaches it.

use super::{Observed, observe};
use crate::grid::Grid;

///
/// One expected Tick: `rows` padded to `width`, and each diagnostic's column,
/// row and message.
///
fn seen(width: usize, rows: &[&str], diagnostics: &[(usize, usize, &str)]) -> Observed {
    Observed {
        rows: rows.iter().map(|row| format!("{row:width$}")).collect(),
        diagnostics: diagnostics
            .iter()
            .map(|&(column, row, message)| (column, row, message.to_string()))
            .collect(),
    }
}

#[test]
fn a_root_copy_overwrites_whatever_its_output_portal_lands_on() {
    // Onto another Expression's Function spelling: the Copy writes before the
    // Addition's Turn, so the Addition is replaced, takes no Turn and never
    // writes over `.+0304` below it. No diagnostic says it was replaced.
    let spelling = ["01=>010102", "    .+0304", "    07"];
    assert_eq!(
        observe(
            Grid::with_shape(10, 3),
            &["01=>.+0102", "    .+0304", ""],
            2
        ),
        vec![seen(10, &spelling, &[]), seen(10, &spelling, &[])]
    );

    // Onto an operand: the Copy runs first and the Addition adds what it wrote.
    let operand = [".+0105=<05", "06"];
    assert_eq!(
        observe(Grid::with_shape(10, 2), &[".+0102=<05", ""], 2),
        vec![seen(10, &operand, &[]), seen(10, &operand, &[])]
    );

    // Onto empty Cells.
    assert_eq!(
        observe(Grid::with_shape(8, 1), &["01=>"], 2),
        vec![seen(8, &["01=>01"], &[]), seen(8, &["01=>01"], &[])]
    );

    // Onto a root that waits for Bang: a value is an ordinary overwrite, so
    // Raw Play's spelling is replaced like any other.
    assert_eq!(
        observe(Grid::with_shape(12, 1), &["01=>!>007FC4"], 2),
        vec![
            seen(12, &["01=>01007FC4"], &[]),
            seen(12, &["01=>01007FC4"], &[]),
        ]
    );
}

#[test]
fn a_root_copy_writes_a_same_tick_bang_over_a_root_and_reads_a_typed_bang_as_empty_once_it_fires() {
    // A root Copy answers a value, so it takes its Turn with no Bang. The
    // Bang that Equality writes this Tick is written over the Addition's
    // spelling, so the Addition answers nothing.
    let written = [".=0101", "**=>**0102", ""];
    assert_eq!(
        observe(Grid::with_shape(10, 3), &[".=0101", "  =>.+0102", ""], 2),
        vec![seen(10, &written, &[]), seen(10, &written, &[])]
    );

    // A `**` typed into the Input Portal fires at the start of the Tick and
    // is cleared before the Copy's Turn: it activates the `*v` aligned south
    // of it, which emits `vv`, and the Copy copies empty Cells over the
    // Addition's spelling without a word.
    assert_eq!(
        observe(Grid::with_shape(10, 3), &["**=>.+0102", "*v", ""], 1),
        vec![seen(10, &["  =>  0102", "*v", "vv"], &[])]
    );
}

#[test]
fn a_root_only_cycle_stops_only_its_own_expressions() {
    // The Addition writes south over `=^`, so it goes before the Copy; the
    // Copy copies the empty Cells below it over the Addition's spelling, so it
    // goes before the Addition. No order exists for those two, and the
    // Increment beside them, which depends on neither, counts every Tick.
    let rows = [".+0102", "=^    ~+0110", ""];
    let counting = |count: &str| {
        seen(
            14,
            &[".+0102", "=^    ~+0110", &format!("      {count}")],
            &[(0, 0, "same-Tick dependency cycle")],
        )
    };
    assert_eq!(
        observe(Grid::with_shape(14, 3), &rows, 3),
        vec![counting("01"), counting("02"), counting("03")]
    );

    // Without the Copy the Addition publishes too.
    assert_eq!(
        observe(Grid::with_shape(14, 3), &[".+0102", "      ~+0110", ""], 3),
        vec![
            seen(14, &[".+0102", "03    ~+0110", "      01"], &[]),
            seen(14, &[".+0102", "03    ~+0110", "      02"], &[]),
            seen(14, &[".+0102", "03    ~+0110", "      03"], &[]),
        ]
    );
}

#[test]
fn the_cycle_diagnostic_anchors_at_a_computation_on_the_cycle() {
    // The cycle is the Addition and the `=^` below it. The `=^` on row 1
    // reads the Addition's spelling, which the cycle writes, so it waits on
    // the cycle without being part of it. It comes first in Source order, and
    // the cycle diagnostic still anchors at the Addition; the `=^` is
    // diagnosed at its own root as waiting.
    let downstream = ["", "=^", ".+0102", "=^"];
    assert_eq!(
        observe(Grid::with_shape(14, 4), &downstream, 1),
        vec![seen(
            14,
            &downstream,
            &[
                (0, 2, "same-Tick dependency cycle"),
                (0, 1, "waiting on a same-Tick dependency cycle"),
            ]
        )]
    );

    let alone = ["", "", ".+0102", "=^"];
    assert_eq!(
        observe(Grid::with_shape(14, 4), &alone, 1),
        vec![seen(14, &alone, &[(0, 2, "same-Tick dependency cycle")])]
    );
}

#[test]
fn a_nested_east_copy_agrees_with_its_spatial_equivalent_on_the_first_tick_only() {
    // `=>` nested in a parent copies the parent's spelling east over the
    // parent's second operand and Returns that spelling to the first. The
    // spatial equivalent places both deliveries with roots: `=v` writes the
    // spelling into the first slot and `=^` writes it over the second.
    for parent in [".+", ".x"] {
        let nested = format!("{parent}=>01");
        let rewritten = format!("{parent}=>{parent}");
        let found = format!("expected a number, found \"{parent}\"");

        // Tick 0: the parent refuses the spelling as a Number. From Tick 1
        // the Source reads `{parent}=>{parent}`, whose second `{parent}` has
        // blank operands, so the parent answers blank: nothing is written
        // and nothing is said, while the witness counts on.
        let nested_rows =
            |count: &'static str| [rewritten.as_str(), "", "", "", "", "", "~+0110", count];
        assert_eq!(
            observe(
                Grid::with_shape(14, 8),
                &[&nested, "", "", "", "", "", "~+0110", ""],
                3
            ),
            vec![
                seen(14, &nested_rows("01"), &[(0, 0, &found)]),
                seen(14, &nested_rows("02"), &[]),
                seen(14, &nested_rows("03"), &[]),
            ]
        );

        // Tick 0 matches the nested form, with the spelling in the slot where
        // the nested form keeps `=>`. From Tick 1 the slot's spelling parses
        // as a nested Function whose south write covers `=^`, while `=^`
        // covers that spelling: a cycle that stops those Expressions while the
        // witness counts on.
        let source = format!("  {parent}");
        let slot = format!("{parent}  01");
        let below = format!("    {parent}");
        let placed = format!("{parent}{parent}{parent}");
        let spatial_rows = |count: &'static str| {
            [
                source.clone(),
                "  =v".to_owned(),
                placed.clone(),
                "    =^".to_owned(),
                below.clone(),
                String::new(),
                "~+0110".to_owned(),
                count.to_owned(),
            ]
        };
        assert_eq!(
            observe(
                Grid::with_shape(14, 8),
                &[&source, "  =v", &slot, "    =^", &below, "", "~+0110", ""],
                3
            ),
            vec![
                seen(
                    14,
                    &spatial_rows("01").each_ref().map(String::as_str),
                    &[(0, 2, &found)]
                ),
                seen(
                    14,
                    &spatial_rows("02").each_ref().map(String::as_str),
                    &[(4, 2, "same-Tick dependency cycle")]
                ),
                seen(
                    14,
                    &spatial_rows("03").each_ref().map(String::as_str),
                    &[(4, 2, "same-Tick dependency cycle")]
                ),
            ]
        );
    }
}

#[test]
fn a_nested_west_copy_stops_its_expression_where_its_spatial_equivalent_replaces_the_parent() {
    // `=<` nested in a parent copies `01` west over the parent's own spelling
    // and Returns `01` to the first operand. A nested Function's write onto
    // its own root is a same-Tick self-dependency, so that Expression stops
    // every Tick, diagnosed at the Copy, while the witness counts on.
    for parent in [".+", ".x"] {
        let nested = format!("{parent}=<01");
        let stopped = |count: &'static str| {
            seen(
                14,
                &["", "", nested.as_str(), "", "~+0110", count],
                &[(2, 2, "same-Tick dependency cycle")],
            )
        };
        assert_eq!(
            observe(
                Grid::with_shape(14, 6),
                &["", "", nested.as_str(), "", "~+0110", ""],
                2
            ),
            vec![stopped("01"), stopped("02")]
        );

        // Placed with roots, the same two deliveries replace the parent
        // before its Turn: the parent is silently gone and the witness counts.
        let slot = format!("{parent}  01");
        let spatial_rows = |count: &'static str| ["0101", "=v=v", "010101", "", "~+0110", count];
        assert_eq!(
            observe(
                Grid::with_shape(14, 6),
                &["0101", "=v=v", &slot, "", "~+0110", ""],
                2
            ),
            vec![
                seen(14, &spatial_rows("01"), &[]),
                seen(14, &spatial_rows("02"), &[]),
            ]
        );
    }
}
