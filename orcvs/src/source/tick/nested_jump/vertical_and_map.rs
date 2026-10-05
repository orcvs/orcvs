//! See the parent module.
//!
//! Nested `&^` and `&v`, Tick after Tick, and what the Language Map shows of
//! every nested Jump before any Tick runs: the Output Portal Reservations
//! Source Paint highlights and the Map's own diagnostics.

use lang::Tick;

use super::super::{Cells, Lookup, Portal, computations};
use super::{Observed, observe};
use crate::grid::Grid;
use crate::source::Source;
use crate::source::language_map::LanguageMap;

///
/// The same Grid and diagnostics after each of `ticks` Ticks.
///
fn steady(rows: &[&str], diagnostics: &[(usize, usize, &str)], ticks: usize) -> Vec<Observed> {
    (0..ticks)
        .map(|_| Observed {
            rows: rows.iter().map(|row| (*row).to_owned()).collect(),
            diagnostics: diagnostics
                .iter()
                .map(|&(column, row, message)| (column, row, message.to_owned()))
                .collect(),
        })
        .collect()
}

///
/// `rows`, each padded to the Grid's width, with blank rows filling the rest.
///
fn padded(grid: Grid, rows: &[&str]) -> String {
    let width = grid.columns();
    let text: String = rows.iter().map(|row| format!("{row:width$}")).collect();
    format!("{text:count$}", count = grid.count())
}

///
/// One row of `#` and `.` per Grid row, `#` where `cells` holds.
///
fn picture(grid: Grid, cells: &[bool]) -> Vec<String> {
    cells
        .chunks(grid.columns())
        .map(|row| row.iter().map(|&on| if on { '#' } else { '.' }).collect())
        .collect()
}

///
/// What the performer sees before a Tick: the Output Portal Reservations
/// Source Paint highlights, and each Map diagnostic with the Grid column and
/// row of its first Cell.
///
fn before_a_tick(grid: Grid, rows: &[&str]) -> (Vec<String>, Vec<(usize, usize, String)>) {
    let text = padded(grid, rows);
    let map = LanguageMap::build(grid, Cells::of(text.as_bytes()));
    let width = grid.columns();
    let diagnostics = map
        .diagnostics()
        .map(|diagnostic| {
            let start = diagnostic.start();
            (start % width, start / width, diagnostic.message.clone())
        })
        .collect();
    (picture(grid, &map.output_portal_cells()), diagnostics)
}

///
/// The Cells the Tick scheduler reserves for every computation's write sites,
/// read from its own `Lookup`.
///
fn scheduled(grid: Grid, rows: &[&str]) -> Vec<String> {
    let text = padded(grid, rows);
    let map = LanguageMap::build(grid, Cells::of(text.as_bytes()));
    let (nodes, _diagnostics) = computations(grid, &map);
    let lookup = Lookup::new(grid, nodes, &map);
    let mut cells = vec![false; grid.count()];
    for node in lookup.nodes() {
        for output in node
            .portal_access
            .write_sites()
            .iter()
            .filter_map(|output| output.as_ref().ok())
        {
            if let Some(span) = Portal::at(grid, *output).reservation() {
                for index in span.range() {
                    cells[index] = true;
                }
            }
        }
    }
    picture(grid, &cells)
}

///
/// How many Play Commands each of `ticks` Ticks delivered.
///
fn plays(grid: Grid, rows: &[&str], ticks: u64) -> Vec<usize> {
    let mut source = Source::new(grid);
    for (index, byte) in padded(grid, rows).bytes().enumerate() {
        source
            .set(
                grid.cell_index(index).expect("inside the Grid"),
                &char::from(byte).to_string(),
            )
            .unwrap();
    }
    (0..ticks)
        .map(|tick| source.execute(Tick::new(tick)).play_commands.len())
        .collect()
}

///
/// A bare two-Cell value outside any Expression, which the Map reports
/// four times: two unknown Function spellings and two invalid characters.
///
fn stray_value(column: usize, row: usize, value: &str) -> Vec<(usize, usize, String)> {
    let first = &value[..1];
    let second = &value[1..];
    vec![
        (column, row, format!("unknown function \"{value}\"")),
        (column + 1, row, format!("unknown function \"{second} \"")),
        (
            column,
            row,
            format!("invalid Language Unit character '{first}'"),
        ),
        (
            column + 1,
            row,
            format!("invalid Language Unit character '{second}'"),
        ),
    ]
}

#[test]
fn a_nested_vertical_jump_copies_what_it_reads_and_returns_it() {
    // A value: the Jump writes `05` through its Output Portal and returns it,
    // so the parent writes `06` south. The same every Tick.
    let rows = ["  05    ", ".+&^01  ", "0605    ", "        "];
    assert_eq!(
        observe(Grid::with_shape(8, 4), &["", ".+&^01", "  05", ""], 3),
        steady(&rows, &[], 3)
    );
    let rows = ["  05    ", ".+&v01  ", "0605    ", "        "];
    assert_eq!(
        observe(Grid::with_shape(8, 4), &["  05", ".+&v01", "", ""], 3),
        steady(&rows, &[], 3)
    );

    // Empty: the Jump clears its destination and returns nothing, and the
    // parent diagnoses at its own anchor every Tick.
    let returned_nothing = [(
        0,
        1,
        "nested computation at column 2, row 1 returned nothing",
    )];
    for rows in [
        ["        ", ".+&^01  ", "        ", "        "],
        ["        ", ".+&v01  ", "        ", "        "],
    ] {
        let source: Vec<&str> = rows.iter().map(|row| row.trim_end()).collect();
        assert_eq!(
            observe(Grid::with_shape(8, 4), &source, 3),
            steady(&rows, &returned_nothing, 3)
        );
    }
}

#[test]
fn a_nested_vertical_jump_copies_another_expressions_spelling() {
    // `&^` copies the `.+` spelling north and returns it; the parent reads it
    // as its Number operand and diagnoses. From the next Tick the copy is a
    // root whose Output Portal is the Jump, and the Jump's is the copy: a
    // same-Tick cycle diagnosed at the copy, and the parent says nothing.
    let observed = observe(Grid::with_shape(10, 4), &["", ".+&^01", "  .+0102", ""], 3);
    let rows = ["  .+      ", ".+&^01    ", "  .+0102  ", "  03      "];
    let mut expected = steady(&rows, &[(0, 1, "expected a number, found \".+\"")], 1);
    expected.extend(steady(&rows, &[(2, 0, "same-Tick dependency cycle")], 2));
    assert_eq!(observed, expected);

    // `&v` copies `!>` south. The copy's operands are unwritten, so it is
    // pending and nothing reports it.
    let rows = ["  !>007FC4", ".+&v01    ", "  !>      ", "          "];
    assert_eq!(
        observe(
            Grid::with_shape(10, 4),
            &["  !>007FC4", ".+&v01", "", ""],
            3
        ),
        steady(&rows, &[(0, 1, "expected a number, found \"!>\"")], 3)
    );
    assert_eq!(before_a_tick(Grid::with_shape(10, 4), &rows).1, []);
}

#[test]
fn a_nested_vertical_jump_overwrites_another_expression() {
    // Onto an operand: `07` replaces `05`, and that root reads `0407` in the
    // same Tick.
    let rows = ["  .+0407  ", "  0B.+&^01", "    0807  ", "          "];
    assert_eq!(
        observe(
            Grid::with_shape(10, 4),
            &["  .+0405", "    .+&^01", "      07", ""],
            3
        ),
        steady(&rows, &[], 3)
    );
    let rows = ["      07  ", "    .+&v01", "  .+0807  ", "  0F      "];
    assert_eq!(
        observe(
            Grid::with_shape(10, 4),
            &["      07", "    .+&v01", "  .+0405", ""],
            3
        ),
        steady(&rows, &[], 3)
    );

    // Onto a spelling: `07` replaces `&<` before it takes a Turn, and `.+`
    // below loses its Function to a value that is not an Expression.
    let rows = ["  0705    ", ".+&^01    ", "0807      ", "          "];
    assert_eq!(
        observe(
            Grid::with_shape(10, 4),
            &["  &<05", ".+&^01", "  07", ""],
            3
        ),
        steady(&rows, &[], 3)
    );
    let rows = ["  07      ", ".+&v01    ", "08070405  ", "          "];
    assert_eq!(
        observe(
            Grid::with_shape(10, 4),
            &["  07", ".+&v01", "  .+0405", ""],
            3
        ),
        steady(&rows, &[], 3)
    );

    // Empty input clears the operand under the Jump, which leaves that root
    // pending. The parent still diagnoses the Return the Jump does not give.
    let observed = observe(
        Grid::with_shape(10, 4),
        &["", "    .+&v01", "  .+0405", ""],
        3,
    );
    let rows = ["          ", "    .+&v01", "  .+04    ", "          "];
    let returned_nothing = (
        4,
        1,
        "nested computation at column 6, row 1 returned nothing",
    );
    assert_eq!(observed, steady(&rows, &[returned_nothing], 3));
    assert_eq!(before_a_tick(Grid::with_shape(10, 4), &rows).1, []);

    // A root writing onto the nested Jump's spelling wins: the Jump never
    // runs, and the parent adds the value that replaced it.
    let rows = ["  .+0102  ", ".+0301    ", "0405      ", "          "];
    assert_eq!(
        observe(
            Grid::with_shape(10, 4),
            &["  .+0102", ".+&v01", "  05", ""],
            3
        ),
        steady(&rows, &[], 3)
    );
}

#[test]
fn a_bang_in_the_source_is_gone_before_a_vertical_jump_reads_it() {
    // A `**` the performer typed is not a Bang the Jump can relay: the Tick
    // clears it, the Jump reads empty and clears its destination. Nested,
    // the parent diagnoses the missing Return; a root says nothing. Either
    // way an aligned `.=` loses its spelling.
    let returned_nothing = [(
        0,
        1,
        "nested computation at column 2, row 1 returned nothing",
    )];
    let rows = ["        ", ".+&^01  ", "        ", "        "];
    assert_eq!(
        observe(Grid::with_shape(8, 4), &["", ".+&^01", "  **", ""], 3),
        steady(&rows, &returned_nothing, 3)
    );
    let rows = ["          ", ".+&v01    ", "    0101  ", "          "];
    assert_eq!(
        observe(
            Grid::with_shape(10, 4),
            &["  **", ".+&v01", "  .=0101", ""],
            3
        ),
        steady(&rows, &returned_nothing, 3)
    );
    let rows = ["          ", "  &v      ", "    0101  ", "          "];
    assert_eq!(
        observe(
            Grid::with_shape(10, 4),
            &["  **", "  &v", "  .=0101", ""],
            3
        ),
        steady(&rows, &[], 3)
    );
}

#[test]
fn a_nested_vertical_jump_relays_a_same_tick_bang_to_an_aligned_root() {
    // `.=0101` Bangs into the Jump's Input Portal; the Jump activates `!>`
    // without overwriting it, so it plays every Tick, as from a root Jump.
    // The Return is `**`, which the parent cannot read as a Number.
    let grid = Grid::with_shape(10, 4);
    let nested = ["  .=0101", "", ".+&v01", "  !>007FC4"];
    let rows = ["  .=0101  ", "  **      ", ".+&v01    ", "  !>007FC4"];
    assert_eq!(
        observe(grid, &nested, 3),
        steady(&rows, &[(0, 2, "expected a number, found \"**\"")], 3)
    );
    assert_eq!(plays(grid, &nested, 3), [1, 1, 1]);
    let root = ["  .=0101", "", "  &v", "  !>007FC4"];
    let rows = ["  .=0101  ", "  **      ", "  &v      ", "  !>007FC4"];
    assert_eq!(observe(grid, &root, 3), steady(&rows, &[], 3));
    assert_eq!(plays(grid, &root, 3), [1, 1, 1]);

    // North: `&<` carries the Bang west into the Jump's Input Portal.
    let grid = Grid::with_shape(12, 3);
    let nested = ["  !>007FC4", ".+&^01.=0101", "    &<"];
    let rows = ["  !>007FC4  ", ".+&^01.=0101", "  **&<**    "];
    assert_eq!(
        observe(grid, &nested, 3),
        steady(&rows, &[(0, 1, "expected a number, found \"**\"")], 3)
    );
    assert_eq!(plays(grid, &nested, 3), [1, 1, 1]);
}

#[test]
fn a_south_jump_off_the_grid_diagnoses_at_its_own_anchor() {
    // Nothing is highlighted: the Output Portal leaves no room for a Cell
    // pair. During the Tick both writes are refused, and each diagnostic
    // anchors at the Function whose write it refuses, nested or root.
    let grid = Grid::with_shape(8, 2);
    let blank = vec!["........".to_owned(); 2];
    assert_eq!(
        before_a_tick(grid, &["  07", ".+&v01"]),
        (blank.clone(), stray_value(2, 0, "07"))
    );
    assert_eq!(
        observe(grid, &["  07", ".+&v01"], 3),
        steady(
            &["  07    ", ".+&v01  "],
            &[
                (2, 1, "result \"07\" falls below the Source"),
                (0, 1, "result \"08\" falls below the Source"),
            ],
            3
        )
    );
    assert_eq!(before_a_tick(grid, &["  07", "  &v"]).0, blank);
    assert_eq!(
        observe(grid, &["  07", "  &v"], 3),
        steady(
            &["  07    ", "  &v    "],
            &[(2, 1, "result \"07\" falls below the Source")],
            3
        )
    );
}

#[test]
fn source_paint_marks_each_nested_jumps_output_portal_before_a_tick() {
    // Each nested Jump reserves the Cells its direction names, as a root
    // would, beside the parent's own south pair. The Map diagnoses none of
    // these layouts, so a highlight over the Expression's own operand or
    // spelling is the only pre-Tick sign of a self-overwrite.
    let grid = Grid::with_shape(10, 3);
    for (row, highlighted) in [
        (".+&>01", "....##...."),
        (".+&<01", "##........"),
        (".+01&>", "......##.."),
        (".+01&<", "..##......"),
    ] {
        assert_eq!(
            before_a_tick(grid, &["", row, ""]),
            (
                vec![
                    "..........".to_owned(),
                    highlighted.to_owned(),
                    "##........".to_owned(),
                ],
                vec![]
            ),
            "{row:?}"
        );
    }
    let grid = Grid::with_shape(8, 3);
    assert_eq!(
        before_a_tick(grid, &["", ".+&^01", ""]),
        (
            vec!["..##....".into(), "........".into(), "##......".into()],
            vec![]
        )
    );
    assert_eq!(
        before_a_tick(grid, &["", ".+&v01", ""]),
        (
            vec!["........".into(), "........".into(), "####....".into()],
            vec![]
        )
    );
    // A root Jump, and a nested value Function, for comparison. The bare
    // value a root Jump reads is itself a Map diagnostic.
    assert_eq!(
        before_a_tick(grid, &["", "  &^", "  01"]),
        (
            vec!["..##....".into(), "........".into(), "........".into()],
            stray_value(2, 2, "01")
        )
    );
    assert_eq!(
        before_a_tick(Grid::with_shape(10, 3), &["", ".+.+010203", ""]),
        (
            vec![
                "..........".into(),
                "..........".into(),
                "####......".into()
            ],
            vec![]
        )
    );

    // What the self-overlapping layouts then do. `&>` copies the parent's
    // spelling over its own operand: one Tick diagnostic, then a nested `.+`
    // whose operands are unwritten, so the row is pending and nothing reports
    // it. `&<` writes the
    // parent's spelling while the parent reads it: a cycle every Tick, and a
    // Map that reports nothing.
    let grid = Grid::with_shape(10, 3);
    let rows = ["          ", ".+&>.+    ", "          "];
    let mut expected = steady(&rows, &[(0, 1, "expected a number, found \".+\"")], 1);
    expected.extend(steady(&rows, &[], 2));
    assert_eq!(observe(grid, &["", ".+&>01", ""], 3), expected);
    assert_eq!(before_a_tick(grid, &rows).1, []);
    let rows = ["          ", ".+&<01    ", "          "];
    assert_eq!(
        observe(grid, &["", ".+&<01", ""], 3),
        steady(&rows, &[(2, 1, "same-Tick dependency cycle")], 3)
    );
}

#[test]
fn the_map_reserves_what_the_scheduler_reserves_for_nested_jumps() {
    // The highlight and the write reservations that order a Tick are the
    // same Cells for every Jump direction, nested or root, including a
    // nested Jump whose Output Portal is its own Expression or leaves the
    // Grid.
    for (grid, rows) in [
        (Grid::with_shape(10, 3), &["", ".+&>01", ""][..]),
        (Grid::with_shape(10, 3), &["", ".+&<01", ""]),
        (Grid::with_shape(10, 3), &["", ".+01&>", ""]),
        (Grid::with_shape(10, 3), &["", ".+01&<", ""]),
        (Grid::with_shape(10, 3), &["", "01&>", ""]),
        (Grid::with_shape(10, 4), &["", ".+&^01", "  .+0102", ""]),
        (Grid::with_shape(10, 4), &["  &<05", ".+&^01", "  07", ""]),
        (Grid::with_shape(10, 4), &["  .+0102", ".+&v01", "  05", ""]),
        (
            Grid::with_shape(10, 4),
            &["  .=0101", "", ".+&v01", "  !>007FC4"],
        ),
        (
            Grid::with_shape(12, 3),
            &["  !>007FC4", ".+&^01.=0101", "    &<"],
        ),
        (Grid::with_shape(8, 2), &["  07", ".+&v01"]),
        (Grid::with_shape(8, 2), &[".+&^01", "  07"]),
    ] {
        assert_eq!(
            before_a_tick(grid, rows).0,
            scheduled(grid, rows),
            "{rows:?}"
        );
    }
}
