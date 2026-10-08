//! See the parent module.
//!
//! Each test states a horizontal Jump nested in a parent Expression. `&>`
//! reads the two Cells west of its anchor and writes the two Cells east of
//! them; `&<` reads east and writes west. Those Cells are the parent's
//! spelling, its sibling operands, or the Cells just past the Expression.

use lang::Token;

use super::super::observed;
use super::{Observed, observe, pending_on_a_number};
use crate::grid::Grid;

///
/// One observed Tick, spelled as the rows and the diagnostics a performer
/// reads.
///
fn tick(rows: &[&str], diagnostics: &[(usize, usize, &str)]) -> Observed {
    Observed {
        rows: rows.iter().map(ToString::to_string).collect(),
        diagnostics: diagnostics
            .iter()
            .map(|&(column, row, message)| (column, row, message.to_string()))
            .collect(),
        pending: vec![None; diagnostics.len()],
    }
}

///
/// Three Ticks that each observe the same Grid and diagnostics.
///
fn every_tick(rows: &[&str], diagnostics: &[(usize, usize, &str)]) -> Vec<Observed> {
    (0..3).map(|_| tick(rows, diagnostics)).collect()
}

///
/// Three Ticks whose first observes `first` and whose next two observe `then`.
///
fn settles(first: Observed, rows: &[&str], diagnostics: &[(usize, usize, &str)]) -> Vec<Observed> {
    let mut ticks = vec![first];
    ticks.extend((0..2).map(|_| tick(rows, diagnostics)));
    ticks
}

#[test]
fn an_east_jump_in_the_first_operand_copies_the_parent_spelling_over_the_next_operand() {
    // The Jump reads the parent's own spelling and writes it over the operand
    // beside it, then Returns that spelling, which the parent cannot read as a
    // Number. From Tick 1 the copied spelling parses as a Function without
    // operands, and what is diagnosed depends on how many Cells lie east of it.
    assert_eq!(
        observe(Grid::with_shape(6, 2), &[".+&>01"], 3),
        settles(
            tick(
                &[".+&>.+", "      "],
                &[(0, 0, "expected a number, found \".+\"")],
            ),
            &[".+&>.+", "      "],
            &[(4, 0, "Expression layout crosses the row edge")],
        ),
    );
    assert_eq!(
        observe(Grid::with_shape(10, 2), &[".+&>01"], 3),
        settles(
            tick(
                &[".+&>.+    ", "          "],
                &[(0, 0, "expected a number, found \".+\"")],
            ),
            &[".+&>.+    ", "          "],
            &[],
        ),
    );
    assert_eq!(
        observe(Grid::with_shape(10, 2), &["~?&>0109"], 3),
        settles(
            tick(
                &["~?&>~?09  ", "          "],
                &[(0, 0, "expected a number, found \"~?\"")],
            ),
            &["~?&>~?09  ", "          "],
            &[
                (0, 0, "Expression layout crosses the row edge"),
                (4, 0, "Expression layout crosses the row edge"),
            ],
        ),
    );
}

#[test]
fn an_east_jump_in_a_middle_or_last_operand_copies_the_operand_before_it_east() {
    // The Jump copies the value west of it into the next operand, or into the
    // Cells past the Expression's end whatever they hold, and Returns the same
    // value. The parent reads both and publishes south every Tick.
    assert_eq!(
        observe(Grid::with_shape(10, 2), &["~?01&>09"], 3),
        every_tick(&["~?01&>01  ", "01        "], &[]),
    );
    assert_eq!(
        observe(Grid::with_shape(10, 2), &["~?0109&>"], 3),
        every_tick(&["~?0109&>09", "09        "], &[]),
    );
    assert_eq!(
        observe(Grid::with_shape(10, 2), &[".+01&>ab"], 3),
        every_tick(&[".+01&>01  ", "02        "], &[]),
    );
    assert_eq!(
        observe(Grid::with_shape(10, 2), &[".+01&>  ab"], 3),
        every_tick(&[".+01&>01ab", "02        "], &[]),
    );
}

#[test]
fn an_east_jump_reading_a_sibling_nested_function_copies_that_siblings_operand() {
    // West of the Jump is the last operand of a sibling Addition. Each nested
    // Function writes south of its own anchor and the root writes south of
    // its own (ADR 0061), so the row below reads `05` then `03`.
    assert_eq!(
        observe(Grid::with_shape(12, 2), &[".+.+0102&>"], 3),
        every_tick(&[".+.+0102&>02", "0503        "], &[]),
    );
}

#[test]
fn a_west_jump_that_writes_over_its_parent_spelling_is_a_same_tick_cycle() {
    // As the first operand the Jump's Output Portal is the parent's anchor,
    // which the Jump's own Return feeds, so no order satisfies both and the
    // Expression is stopped, diagnosed at the Jump, whatever the Jump reads: a
    // value, or a sibling nested Function's spelling.
    assert_eq!(
        observe(Grid::with_shape(6, 2), &[".+&<01"], 3),
        every_tick(
            &[".+&<01", "      "],
            &[(2, 0, "same-Tick dependency cycle")]
        ),
    );
    assert_eq!(
        observe(Grid::with_shape(8, 2), &["~?&<0109"], 3),
        every_tick(
            &["~?&<0109", "        "],
            &[(2, 0, "same-Tick dependency cycle")],
        ),
    );
    assert_eq!(
        observe(Grid::with_shape(10, 2), &[".+&<.+0102"], 3),
        every_tick(
            &[".+&<.+0102", "          "],
            &[(2, 0, "same-Tick dependency cycle")],
        ),
    );
    assert_eq!(
        observe(Grid::with_shape(8, 2), &["  .+&<01"], 3),
        every_tick(
            &["  .+&<01", "        "],
            &[(4, 0, "same-Tick dependency cycle")],
        ),
    );
}

#[test]
fn a_west_jump_in_a_middle_or_last_operand_copies_east_cells_over_the_operand_before_it() {
    // The parent reads the operand the Jump overwrote in the same Tick, so a
    // copied value is added to itself.
    assert_eq!(
        observe(Grid::with_shape(8, 2), &["~?01&<09"], 3),
        every_tick(&["~?09&<09", "09      "], &[]),
    );
    assert_eq!(
        observe(Grid::with_shape(8, 2), &[".+01&<05"], 3),
        every_tick(&[".+05&<05", "0A      "], &[]),
    );
}

#[test]
fn a_west_jump_reading_empty_cells_past_the_expression_blanks_the_operand_before_it() {
    // Empty aligned input clears the destination, so the operand before the
    // Jump becomes unwritten and the parent is invalid: the Tick diagnoses
    // it, pending on a Number. From the next Tick the slot is unwritten in
    // the Source, so the Language Map reports it and the Tick says nothing.
    for (width, row, blanked) in [(8, ".+01&<", ".+  &<  "), (10, "~?0109&<", "~?01  &<  ")] {
        let grid = Grid::with_shape(width, 2);
        let empty = " ".repeat(width);
        let message = "expected a number, found \"  \"";
        assert_eq!(
            observe(grid, &[row], 3),
            settles(
                pending_on_a_number(vec![tick(&[blanked, &empty], &[(0, 0, message)])]).remove(0),
                &[blanked, &empty],
                &[],
            ),
            "{row:?}"
        );
        let source = observed::source_of(grid, &[blanked]);
        let map: Vec<_> = source
            .language_map()
            .diagnostics()
            .map(|diagnostic| (diagnostic.start(), diagnostic.pending()))
            .collect();
        assert_eq!(map, [(0, Some(Token::Number))], "{row:?}");
    }
}

#[test]
fn a_west_jump_reading_an_invalid_unit_past_the_expression_writes_nothing() {
    // Invalid input diagnoses and writes nothing, so the Jump Returns nothing
    // and the parent diagnoses that too: the first at the Jump, the second at
    // the root.
    assert_eq!(
        observe(Grid::with_shape(8, 2), &[".+01&<ab"], 3),
        every_tick(
            &[".+01&<ab", "        "],
            &[
                (4, 0, "&< has partial or invalid input"),
                (
                    0,
                    0,
                    "nested computation at column 4, row 0 returned nothing"
                ),
            ],
        ),
    );
}

#[test]
fn a_jump_nested_two_levels_deep_reads_and_writes_its_own_parents_cells() {
    // `&>` reads the inner Addition's spelling and writes it over the inner
    // operand, which the inner and then the outer Addition both diagnose. From
    // Tick 1 the copied spelling claims Cells past the row's end. `&<` writes
    // over the inner Addition's anchor, which is a cycle.
    assert_eq!(
        observe(Grid::with_shape(10, 3), &[".+.+&>0102"], 3),
        settles(
            tick(
                &[".+.+&>.+02", "          ", "          "],
                &[
                    (2, 0, "expected a number, found \".+\""),
                    (
                        0,
                        0,
                        "nested computation at column 2, row 0 returned nothing"
                    ),
                ],
            ),
            &[".+.+&>.+02", "          ", "          "],
            &[
                (0, 0, "Expression layout crosses the row edge"),
                (6, 0, "Expression layout crosses the row edge"),
            ],
        ),
    );
    assert_eq!(
        observe(Grid::with_shape(10, 3), &[".+.+&<0102"], 3),
        every_tick(
            &[".+.+&<0102", "          ", "          "],
            &[(4, 0, "same-Tick dependency cycle")],
        ),
    );
}

#[test]
fn a_nested_west_jump_cycle_leaves_unrelated_expressions_publishing() {
    // A same-Tick cycle stops only the Expressions it reaches (ADR 0065), so
    // an unrelated Addition on the same row or another row writes `03` every
    // Tick.
    assert_eq!(
        observe(Grid::with_shape(14, 2), &[".+&<01  .+0102"], 3),
        every_tick(
            &[".+&<01  .+0102", "        03    "],
            &[(2, 0, "same-Tick dependency cycle")],
        ),
    );
    assert_eq!(
        observe(Grid::with_shape(14, 4), &[".+&<01", "", "      .+0102"], 3),
        every_tick(
            &[
                ".+&<01        ",
                "              ",
                "      .+0102  ",
                "      03      ",
            ],
            &[(2, 0, "same-Tick dependency cycle")],
        ),
    );
}

#[test]
fn a_nested_east_jump_leaves_unrelated_expressions_publishing() {
    // The parent's diagnostic is confined to its own Expression, so an
    // unrelated Addition on the same row or another row writes `03` from
    // Tick 0.
    assert_eq!(
        observe(Grid::with_shape(14, 2), &[".+&>01  .+0102"], 3),
        settles(
            tick(
                &[".+&>.+  .+0102", "        03    "],
                &[(0, 0, "expected a number, found \".+\"")],
            ),
            &[".+&>.+  .+0102", "        03    "],
            &[],
        ),
    );
    assert_eq!(
        observe(Grid::with_shape(14, 4), &[".+&>01", "", "      .+0102"], 3),
        settles(
            tick(
                &[
                    ".+&>.+        ",
                    "              ",
                    "      .+0102  ",
                    "      03      ",
                ],
                &[(0, 0, "expected a number, found \".+\"")],
            ),
            &[
                ".+&>.+        ",
                "              ",
                "      .+0102  ",
                "      03      ",
            ],
            &[],
        ),
    );
}

#[test]
fn a_nested_jump_portal_past_the_grid_edge_is_diagnosed_at_the_jump() {
    // An east Output Portal past the Grid diagnoses the Jump's result, yet the
    // Jump still Returns it and the parent publishes `02`. A west Input
    // Portal past the Grid is invalid input: nothing is written and the
    // parent publishes nothing. The Jump's own diagnostic anchors at the Jump;
    // the parent's missing Return anchors at the root.
    assert_eq!(
        observe(Grid::with_shape(6, 2), &[".+01&>"], 3),
        every_tick(
            &[".+01&>", "02    "],
            &[(4, 0, "result \"01\" falls outside the Grid")],
        ),
    );
    assert_eq!(
        observe(Grid::with_shape(6, 2), &[".+01&<"], 3),
        every_tick(
            &[".+01&<", "      "],
            &[
                (4, 0, "&< has partial or invalid input"),
                (
                    0,
                    0,
                    "nested computation at column 4, row 0 returned nothing"
                ),
            ],
        ),
    );
}
