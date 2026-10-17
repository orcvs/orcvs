//! Bang activation: a Bang activates the roots aligned with it and no further,
//! and a Bang inside an operand activates nothing.

use crate::grid::Grid;
use crate::source::language_map::LanguageMap;
use crate::source::{Cells, MidiChannel, Note, PlayCommand, Velocity};

use super::Tick;
use super::fixture::{carried_source, planned, snapshot, tick_by_tick};
use super::observed::raw;

#[test]
fn a_bang_activates_its_aligned_neighbours_and_no_further_root() {
    // A Bang activates four aligned cardinal anchors — north, south, west,
    // east — and scheduling asks about those four Positions rather than
    // testing every root against every Bang. What that has to keep is
    // the boundary: the root one row away performs, and the root two rows
    // away is untouched by the same Bang.
    //
    // The Bang is delivered to (0, 2) so the roots around it are ordinary
    // Terminal Output Expressions rather than an arrangement bent to reach
    // a downward Portal.
    let grid = Grid::with_shape(16, 6);
    let bytes = snapshot(
        grid,
        &["", "!>007FC4", "", "!>007FC5", "!>007FC6", ".=0101"],
    );
    let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));
    let destinations = [(
        grid.index(grid.position(0, 5).unwrap()),
        vec![grid.position(0, 2).unwrap()],
    )]
    .into_iter()
    .collect();

    let (plan, _) = super::plan_carrying(
        grid,
        Cells::of(bytes.as_bytes()),
        &map,
        Tick::ZERO,
        &destinations,
    );

    // North of the Bang, then south. The root at row 4 is two rows from
    // the Bang and never performs.
    assert_eq!(
        plan.play_commands,
        vec![raw(0, 0x7F, 60), raw(0, 0x7F, 72)],
        "diagnostics: {:?}",
        plan.diagnostics
    );
}

#[test]
fn one_bang_activates_every_aligned_timed_play_root_as_one_chord() {
    // A chord is several Timed Play roots that one Bang activates, each
    // answering its own Play Command. Equality writes `**` below itself,
    // and two Copies carry it east to (8, 1), where its cardinal anchors
    // hold three Timed Play roots: north (8, 0), east (10, 1) and south
    // (8, 2). The roots take their Turns in anchor order once the Bang
    // has settled, so the chord reads C4, E4, G4 on every Tick.
    let grid = Grid::with_shape(20, 3);
    let rows = [
        ".=0101  !~017FC404",
        "  =>  =>  !~017FE404",
        "        !~017FG404",
    ];
    let timed = |note: u8| PlayCommand::Timed {
        channel: MidiChannel::try_from(0x01).expect("a MIDI channel"),
        velocity: Velocity::try_from(0x7F).expect("a MIDI data byte"),
        note: Note::try_from(note).expect("a MIDI note"),
        length: crate::source::Length::from(0x04),
    };

    let (plans, grids, _) = tick_by_tick(grid, &rows, 2);

    assert_eq!(
        grids[0][1], "**=>**=>**!~017FE404",
        "the Bang was carried to (8, 1)"
    );
    for plan in &plans {
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(plan.play_commands, vec![timed(60), timed(64), timed(67)]);
    }
}

#[test]
fn fixed_bang_destinations_respect_alignment_and_operand_contact() {
    let grid = Grid::with_shape(16, 6);
    // One terminal at (4, 2). A Bang two Cells east of it is in its
    // channel operand, so cardinal alignment alone cannot activate it.
    for (column, row, performs) in [
        (4, 1, true),
        (4, 3, true),
        (2, 2, true),
        (6, 2, false),
        (3, 1, false),
        (5, 1, false),
        (1, 2, false),
        (3, 2, false),
        (15, 1, false),
    ] {
        let (plan, _) = carried_source(
            grid,
            &["", "", "    !>007FC4", "", "", ".=0101"],
            &[(80, row * 16 + column)],
        );
        let expected = if performs {
            vec![raw(0, 0x7F, 60)]
        } else {
            vec![]
        };
        assert_eq!(
            plan.play_commands, expected,
            "destination ({column}, {row}): {:?}",
            plan.diagnostics
        );
        if column == 15 {
            // A destination whose pair leaves the row is refused at the
            // Portal, and a terminal wide enough to sound cannot be
            // cardinally aligned with one, so this case cannot separate
            // `dependencies::reserved_at`'s row-fit guard from `Portal::admit`'s
            // refusal.
            // `competing_writers_preserve_an_independent_rejected_destination_diagnostic`
            // is what holds that guard.
            assert!(plan.writes.is_empty());
            assert_eq!(plan.diagnostics.len(), 1);
            assert!(plan.diagnostics[0].message.contains("crosses the row edge"));
        } else {
            assert_eq!(plan.writes.len(), 2);
            assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        }
    }
}

#[test]
fn a_bang_half_inside_an_operand_does_not_activate_an_aligned_terminal() {
    // Operand contact is a fact about the whole reserved pair, so a
    // destination whose first Cell is free still belongs to an operand when
    // its second Cell lands in one. Row 2 holds `.=` at columns 3 and 4
    // with its first operand at columns 5 and 6; the destination (4, 2)
    // covers columns 4 and 5, so only its second Cell is in that operand.
    // The terminal directly below is cardinally aligned and must stay
    // silent anyway -- narrowing the contact test to the anchor Cell alone
    // would sound it.
    let (plan, _) = carried_source(
        Grid::with_shape(16, 6),
        &["", "", "   .=0101", "    !>007FC4", "", ".=0101"],
        &[(80, 2 * 16 + 4)],
    );
    assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(plan.writes.len(), 2);
}

#[test]
fn a_bang_touching_a_nested_function_operand_does_not_activate_a_neighbour() {
    // The destination contacts a nested Function rather than a literal.
    // It still belongs to an operand, so neither its inactive owner nor
    // the aligned terminal below it can perform.
    let (plan, source) = carried_source(
        Grid::with_shape(16, 4),
        &["    .=0101", "!>00.+0101C4", "    !>007FC5", ""],
        &[],
    );
    assert!(plan.play_commands.is_empty());
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(&source.snapshot()[16..28], "!>00**0101C4");
    assert_eq!(plan.writes.len(), 2);
}

#[test]
fn a_bang_cardinally_aligned_with_a_nested_function_anchor_activates_nothing() {
    // The other half of non-root contact. The test above puts the Bang
    // inside the operand Cells, so operand contact alone answers it. Here
    // the destination is a clear row of its own and only its southern
    // anchor lands anywhere: on the `.+` at column 4, which is a nested
    // Function and so no root. Nothing is eligible, so the terminal that
    // owns that `.+` never performs and its row is left exactly as typed.
    let (plan, source) = carried_source(
        Grid::with_shape(16, 4),
        &["", "!>00.+0101C4", "", ".=0101"],
        &[(48, 4)],
    );
    assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(&source.snapshot()[0..8], "    **  ");
    assert_eq!(&source.snapshot()[16..28], "!>00.+0101C4");
}

#[test]
fn a_bang_rejected_in_a_typed_operand_never_activates() {
    // A `**` rejected in a typed operand is still invalid syntax and
    // neither activates nor is a Bang Function. The Bang lands in
    // `.+`'s left Number slot, so it is a rejected operand spelling and not
    // an activation event, even though it is Cell-aligned with the terminal
    // root below it.
    let grid = Grid::with_shape(16, 5);
    let bytes = snapshot(grid, &["    .=0101", "  .+0102", "    !>007FC4", "", ""]);
    let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));

    let (plan, _) = super::plan(grid, Cells::of(bytes.as_bytes()), &map, Tick::ZERO);

    assert_eq!(plan.play_commands, vec![]);
    assert_eq!(planned(&plan), vec![(20, '*'), (21, '*')]);
    assert!(
        plan.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("found \"**\"")),
        "the rejected operand keeps its syntax diagnostic: {:?}",
        plan.diagnostics
    );
}

#[test]
fn a_result_written_over_a_standing_bang_joins_nothing() {
    // The Equality's write is ordered before the `**` it covers, which it
    // suppresses, so the `**` it writes is its own and nothing else.
    let grid = Grid::with_shape(16, 3);
    let bytes = snapshot(grid, &[".=0101", "**", ""]);
    let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));

    let (plan, _) = super::plan(grid, Cells::of(bytes.as_bytes()), &map, Tick::ZERO);

    assert_eq!(planned(&plan), vec![(16, '*'), (17, '*')]);
    assert_eq!(
        plan.diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>(),
        Vec::<&str>::new()
    );
}
