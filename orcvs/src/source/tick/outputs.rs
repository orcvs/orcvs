//! Output admission at the row edge and beside standalone Source: a destination
//! that does not fit costs its own Expression the Turn, not the Tick.

use crate::grid::Grid;
use crate::source::Cells;
use crate::source::language_map::LanguageMap;

use super::Tick;
use super::fixture::{carried_source, cell, planned, rows_of, seeded_source, snapshot};
use super::observed::raw;

#[test]
fn outputs_beside_and_over_standalone_source_are_admitted() {
    let (plan, source) = carried_source(Grid::with_shape(8, 3), &[".=0101", "  0102", ""], &[]);
    assert_eq!(&source.snapshot()[8..14], "**0102");
    assert_eq!(planned(&plan), vec![(8, '*'), (9, '*')]);
    assert!(plan.diagnostics.is_empty());
    assert_eq!(source.language_map().bang_functions().count(), 1);
    let (plan, source) =
        carried_source(Grid::with_shape(10, 3), &["    .=0101", "  0102", ""], &[]);
    assert_eq!(&source.snapshot()[10..16], "  01**");
    assert_eq!(planned(&plan), vec![(14, '*'), (15, '*')]);
    assert!(plan.diagnostics.is_empty());
}

#[test]
fn a_destination_at_the_row_edge_costs_one_expression_its_turn_not_the_tick() {
    // A complete destination must fit even when the stated Portal is at
    // the final Cell, independently of other computations.
    let grid = Grid::with_shape(16, 4);
    let bytes = snapshot(grid, &[".+0102", "", ".+0304", ""]);
    let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));
    let destinations = [(
        grid.cell_index(0).unwrap(),
        vec![grid.position(15, 1).expect("inside the Grid")],
    )]
    .into_iter()
    .collect();

    let (plan, _, _) = super::plan_carrying(
        grid,
        Cells::of(bytes.as_bytes()),
        &map,
        Tick::ZERO,
        &destinations,
    );

    assert_eq!(
        planned(&plan),
        vec![(48, '0'), (49, '7')],
        "the other root still plays: {:?}",
        plan.diagnostics
    );
    assert!(
        plan.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("crosses the row edge")),
        "the Expression at the row edge keeps its own diagnostic: {:?}",
        plan.diagnostics
    );
}

#[test]
fn overlapping_outputs_beside_standalone_source_both_contribute_cells() {
    let (plan, source) = carried_source(
        Grid::with_shape(16, 3),
        &[".+0102 .+0304", "0102", ""],
        &[(0, 20), (7, 21)],
    );
    assert_eq!(planned(&plan), vec![(20, '0'), (21, '0'), (22, '7')]);
    assert_eq!(&source.snapshot()[16..23], "0102007");
    assert!(plan.diagnostics.is_empty());
}

#[test]
fn source_after_an_expression_is_the_next_expressions_and_costs_the_tick_nothing() {
    // An Expression ends where its arity does, so the `Z` after `.+0102` is
    // the Source the next parse reads rather than evidence against the
    // Addition: the Addition takes its turn, and the Tick has nothing to
    // report about either.
    //
    // A fifth row so the Addition has somewhere to put its result: what
    // this test is about is that the Addition takes a turn at all.
    let grid = Grid::with_shape(16, 5);
    let bytes = snapshot(grid, &[".=0101", "", "!>007FC4", ".+0102Z", ""]);
    let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));

    let (plan, _, _) = super::plan(grid, Cells::of(bytes.as_bytes()), &map, Tick::ZERO);

    assert_eq!(plan.play_commands, vec![raw(0, 0x7F, 60)]);
    // The Bang the Equality writes, and the Addition's own `03` below it.
    assert_eq!(
        planned(&plan),
        vec![(16, '*'), (17, '*'), (64, '0'), (65, '3')]
    );
    assert!(
        plan.diagnostics.is_empty(),
        "the Tick reported something about Source it does not own: {:?}",
        plan.diagnostics
    );
}

#[test]
fn a_row_edge_fragment_diagnoses_its_own_expression_and_leaves_the_tick_playing() {
    // A row-edge fragment is local syntax failure, not a graph error.
    let grid = Grid::with_shape(16, 4);
    let bytes = snapshot(grid, &[".=0101", "", "!>007FC4", "            .+01"]);
    let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));

    let (plan, _, _) = super::plan(grid, Cells::of(bytes.as_bytes()), &map, Tick::ZERO);

    assert_eq!(plan.play_commands, vec![raw(0, 0x7F, 60)]);
    assert!(
        plan.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "Expression layout crosses the row edge"),
        "the off-row Expression keeps its own diagnostic: {:?}",
        plan.diagnostics
    );
}

#[test]
fn a_banged_play_with_an_unwritten_note_is_invalid_and_emits_nothing() {
    // The Bang gives the timing and the note slot gives what plays. With
    // no note written the Play is invalid: the Bang reaches nothing that
    // plays, and the Language Map diagnoses it.
    let grid = Grid::with_shape(16, 4);
    for (play, commands, diagnosed) in [
        ("!>007FC4", vec![raw(0, 0x7F, 60)], vec![]),
        ("!>007F", vec![], vec![(0, 2)]),
    ] {
        let bytes = snapshot(grid, &[".=0101", "", play, ""]);
        let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));

        let (plan, _, _) = super::plan(grid, Cells::of(bytes.as_bytes()), &map, Tick::ZERO);

        assert_eq!(plan.play_commands, commands, "{play:?}");
        assert!(
            plan.diagnostics.is_empty(),
            "{play:?}: {:?}",
            plan.diagnostics
        );
        assert_eq!(
            map.diagnostics()
                .map(|d| (d.anchor().x(), d.anchor().y()))
                .collect::<Vec<_>>(),
            diagnosed,
            "{play:?}"
        );
    }
}

#[test]
fn a_banged_play_whose_nested_track_copies_an_empty_pair_rests() {
    // Track stands in the note slot and copies the pair its index
    // selects. `C4` plays; the empty pair is a Clear, so the Play rests:
    // it sends nothing and is not diagnosed.
    let grid = Grid::with_shape(16, 4);
    for (play, commands) in [
        ("!>007F&t0002C4  ", vec![raw(0, 0x7F, 60)]),
        ("!>007F&t0102C4  ", vec![]),
    ] {
        let bytes = snapshot(grid, &[".=0101", "", play, ""]);
        let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));

        let (plan, _, _) = super::plan(grid, Cells::of(bytes.as_bytes()), &map, Tick::ZERO);

        assert_eq!(plan.play_commands, commands, "{play:?}");
        assert!(
            plan.diagnostics.is_empty(),
            "{play:?}: {:?}",
            plan.diagnostics
        );
    }
}

#[test]
fn a_value_function_whose_operand_a_portal_empties_leaves_its_last_note_to_replay() {
    // Track writes the pair its index selects into `.^`'s operand, and
    // `.^` writes the Note it converts into the Play's note slot. With
    // the index moved to the empty pair, Track clears `.^`'s operand, so
    // `.^` is invalid, writes nothing and is diagnosed. The `G4` it wrote
    // on the earlier Tick stays, and the next Bang plays it again.
    let grid = Grid::with_shape(18, 4);
    let mut source = seeded_source(grid, &[".=0101  &t000243  ", "      .^", "!>007F", ""]);

    let first = source.execute(Tick::ZERO);
    assert!(first.diagnostics.is_empty(), "{:?}", first.diagnostics);
    assert_eq!(first.play_commands, vec![raw(0, 0x7F, 67)]);
    assert_eq!(&rows_of(grid, &source)[2][..8], "!>007FG4");

    source.set(cell(grid, 11), "1").unwrap();
    let rest = source.execute(Tick::new(1));

    assert_eq!(&rows_of(grid, &source)[1][6..10], ".^  ");
    assert_eq!(&rows_of(grid, &source)[2][..8], "!>007FG4");
    assert_eq!(rest.play_commands, vec![raw(0, 0x7F, 67)]);
    assert_eq!(
        rest.diagnostics
            .iter()
            .map(|d| (d.anchor().x(), d.anchor().y()))
            .collect::<Vec<_>>(),
        [(6, 1)]
    );
}

#[test]
fn an_activated_consumer_uses_surviving_cells_after_supplier_failure() {
    let (plan, source) = carried_source(
        Grid::with_shape(16, 4),
        &["  .+0Z", "!>007FC4", "", ".=0101"],
        &[(48, 32)],
    );
    assert_eq!(plan.play_commands, vec![raw(0, 0x7F, 60)]);
    assert_eq!(&source.snapshot()[16..24], "!>007FC4");
    assert!(source.language_map().diagnostics().any(|d| d.start() == 2));
}
