//! Halt: an active Halt locks the complete root one row south before it can
//! run, and an inert or suppressed one locks nothing.

use crate::grid::Grid;
use crate::source::Cells;
use crate::source::language_map::LanguageMap;

use super::fixture::{anchors, messages, snapshot, tick_by_tick, turns};
use super::{Tick, execution::ComputationState};

#[test]
fn halt_does_not_supply_cells_read_by_a_copy() {
    // Halt targets the empty Cells west of East Copy. Treating that lock
    // as a write closes a false cycle through both other Copies and Equality.
    let (plans, grids, _) = tick_by_tick(
        Grid::with_shape(8, 3),
        &[".=0101=<", "  *!  =^", "    =>  "],
        1,
    );
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
    assert_eq!(grids[0], [".=0101  ", "***!  =^", "    =>  "]);
    assert!(plans[0].locks.is_empty());
    assert!(plans[0].play_commands.is_empty());
}

#[test]
fn an_inactive_or_suppressed_halt_does_not_diagnose_its_target() {
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 2), &["  *!    ", "  ||    "], 1);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
    assert!(plans[0].locks.is_empty());
    assert_eq!(grids[0], ["  *!    ", "  ||    "]);

    let (plans, grids, source) = tick_by_tick(
        Grid::with_shape(8, 4),
        &[".=0101  ", "  *!    ", "  *!    ", "  ||    "],
        1,
    );
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
    assert_eq!(plans[0].locks, vec![source.grid().position(2, 2).unwrap()]);
    assert_eq!(grids[0], [".=0101  ", "***!    ", "  *!    ", "  ||    "]);
}

#[test]
fn halt_locks_nested_computations_before_they_can_diagnose() {
    // The nested Divide would diagnose its zero divisor if it took a Turn.
    let (plans, grids, source) = tick_by_tick(
        Grid::with_shape(12, 4),
        &[
            ".=0101      ",
            "  *!        ",
            "  .+./010001",
            "            ",
        ],
        1,
    );
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
    assert_eq!(plans[0].locks, vec![source.grid().position(2, 2).unwrap()]);
    assert_eq!(
        grids[0],
        [
            ".=0101      ",
            "***!        ",
            "  .+./010001",
            "            "
        ]
    );
}

#[test]
fn an_active_halt_locks_the_complete_root_one_row_south() {
    // CONTEXT.md: when Halt is active it "establishes a dependency that
    // locks the Expression root directly south before that root can
    // execute." Equality Bangs every Tick; its result sits two columns west
    // of `*!`, which is the horizontal activation geometry. The Add
    // one row south of Halt is intrinsically active, so the lock is the
    // only reason it contributes no `07` and its Source is unchanged.
    let (plans, grids, source) = tick_by_tick(
        Grid::with_shape(8, 4),
        &[".=0101  ", "  *!    ", "  .+0304", "        "],
        1,
    );

    assert_eq!(grids[0], [".=0101  ", "***!    ", "  .+0304", "        "]);
    assert_eq!(
        plans[0].locks,
        vec![source.grid().position(2, 2).expect("inside the Grid")]
    );
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
    assert!(plans[0].play_commands.is_empty());
}

#[test]
fn an_inert_halt_does_not_lock_and_the_south_root_runs() {
    // "When active" is load-bearing: an unactivated `*!` establishes no
    // lock. The Add one row south is intrinsically active, so it writes
    // `07` the way it would if Halt were absent.
    let (plans, grids, _) = tick_by_tick(
        Grid::with_shape(8, 3),
        &["*!      ", ".+0304  ", "        "],
        1,
    );

    assert_eq!(grids[0], ["*!      ", ".+0304  ", "07      "]);
    assert!(plans[0].locks.is_empty(), "{:?}", plans[0].locks);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn a_bang_its_producer_clears_does_not_activate_halt() {
    // Delay Bangs on Tick 0 (a multiple of its cycle) and not on Tick 1,
    // so Tick 0 locks the Add, and on Tick 1 the Delay's clear suppresses
    // the `**` it left: Halt is inert and the Add runs.
    let (plans, grids, source) = tick_by_tick(
        Grid::with_shape(8, 4),
        &["~*0201  ", "  *!    ", "  .+0304", "        "],
        2,
    );

    assert_eq!(grids[0], ["~*0201  ", "***!    ", "  .+0304", "        "]);
    assert_eq!(grids[1], ["~*0201  ", "  *!    ", "  .+0304", "  07    "]);
    assert_eq!(
        plans[0].locks,
        vec![source.grid().position(2, 2).expect("inside the Grid")]
    );
    assert!(plans[1].locks.is_empty(), "{:?}", plans[1].locks);
    for plan in &plans {
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }
}

#[test]
fn a_typed_bang_activates_halt_once() {
    // A typed `**` two columns west of Halt fires at the start of Tick 0:
    // Halt joins the Tick before any Turn and locks the Add below it. The
    // `**` is cleared, so Tick 1 leaves Halt inert and the Add runs.
    let (plans, grids, source) = tick_by_tick(
        Grid::with_shape(8, 3),
        &["***!    ", "  .+0304", "        "],
        2,
    );

    assert_eq!(grids[0], ["  *!    ", "  .+0304", "        "]);
    assert_eq!(grids[1], ["  *!    ", "  .+0304", "  07    "]);
    assert_eq!(
        plans[0].locks,
        vec![source.grid().position(2, 1).expect("inside the Grid")]
    );
    assert!(plans[1].locks.is_empty(), "{:?}", plans[1].locks);
    for plan in &plans {
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }
}

#[test]
fn an_empty_halt_target_is_a_noop() {
    // Off the last row, and an empty row that exists: neither diagnoses
    // and neither invents a lock.
    let last_row = tick_by_tick(Grid::with_shape(8, 2), &[".=0101  ", "  *!    "], 1);
    assert_eq!(last_row.1[0], [".=0101  ", "***!    "]);
    assert!(last_row.0[0].locks.is_empty(), "{:?}", last_row.0[0].locks);
    assert!(
        last_row.0[0].diagnostics.is_empty(),
        "{:?}",
        last_row.0[0].diagnostics
    );

    let empty_row = tick_by_tick(
        Grid::with_shape(8, 3),
        &[".=0101  ", "  *!    ", "        "],
        1,
    );
    assert_eq!(empty_row.1[0], [".=0101  ", "***!    ", "        "]);
    assert!(
        empty_row.0[0].locks.is_empty(),
        "{:?}",
        empty_row.0[0].locks
    );
    assert!(
        empty_row.0[0].diagnostics.is_empty(),
        "{:?}",
        empty_row.0[0].diagnostics
    );
}

#[test]
fn an_occupied_non_root_halt_target_diagnoses() {
    // A Comment and an operand Cell are each occupied and neither is a
    // root anchored one row south. Halt diagnoses and invents no lock —
    // the Add whose operand sits south of Halt still writes. A `**` there
    // is the Bang Function, a root, which `an_active_halt_locks_a_bang_function`
    // locks.
    let comment = tick_by_tick(
        Grid::with_shape(8, 3),
        &[".=0101  ", "  *!    ", "  ||    "],
        1,
    );
    assert_eq!(comment.1[0], [".=0101  ", "***!    ", "  ||    "]);
    assert!(comment.0[0].locks.is_empty(), "{:?}", comment.0[0].locks);
    assert_eq!(
        messages(&comment.0[0]),
        vec!["*! target is not an Expression root"]
    );

    let operand = tick_by_tick(
        Grid::with_shape(8, 4),
        &[".=0101  ", "  *!    ", ".+0304  ", "        "],
        1,
    );
    assert_eq!(
        operand.1[0],
        [".=0101  ", "***!    ", ".+0304  ", "07      "]
    );
    assert!(operand.0[0].locks.is_empty(), "{:?}", operand.0[0].locks);
    assert_eq!(
        messages(&operand.0[0]),
        vec!["*! target is not an Expression root"]
    );
}

#[test]
fn a_halt_aimed_at_comment_text_locks_nothing_and_does_not_diagnose() {
    // A Comment occupies only its delimiters, so its text south of the Halt
    // is ordinary Cells, blank or holding glyphs, and nothing occupies them.
    for text in ["||      ", "||C3    "] {
        let (plans, grids, _) =
            tick_by_tick(Grid::with_shape(8, 3), &[".=0101  ", "  *!    ", text], 1);
        assert_eq!(grids[0], [".=0101  ", "***!    ", text]);
        assert!(plans[0].locks.is_empty(), "{text}: {:?}", plans[0].locks);
        assert!(
            plans[0].diagnostics.is_empty(),
            "{text}: {:?}",
            plans[0].diagnostics
        );
    }
}

#[test]
fn an_active_halt_locks_a_bang_function() {
    // The `**` south of the Halt is a root, so the Halt locks it: it
    // stands, takes no Turn, and activates nothing.
    let (plans, grids, source) = tick_by_tick(
        Grid::with_shape(10, 4),
        &[".=0101", "  *!", "  **", "  !>007FC4"],
        2,
    );
    for (plan, grid) in plans.iter().zip(&grids) {
        assert_eq!(grid[2], "  **      ");
        assert_eq!(
            plan.locks,
            vec![source.grid().position(2, 2).expect("inside the Grid")]
        );
        assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }
}

#[test]
fn a_suppressed_halt_does_not_lock_its_own_target() {
    // Two-row Halt stack: A locks B, and B therefore does not lock the
    // Add below it. The Add writes `07`.
    let (plans, grids, source) = tick_by_tick(
        Grid::with_shape(8, 5),
        &[".=0101  ", "  *!    ", "  *!    ", "  .+0304", "        "],
        1,
    );

    assert_eq!(
        grids[0],
        [".=0101  ", "***!    ", "  *!    ", "  .+0304", "  07    "]
    );
    assert_eq!(
        plans[0].locks,
        vec![source.grid().position(2, 2).expect("inside the Grid")]
    );
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn halt_precedes_its_south_root_by_a_lock_edge() {
    // Halt is always above its target, so Position alone would put it
    // first once it is ready. The edge is what keeps the target from
    // running while Halt is still waiting on activation: Equality and
    // the Add are both ready at the start of the Tick, Equality is first
    // by Position, and without the lock the Add would take the next Turn
    // before Halt became ready. An independent Add on Halt's row still
    // runs — it is not the south root.
    //
    // A Halt whose Position is after its target cannot be spelled: the
    // target is one row south. A Bang producer after both Halt and that
    // target also cannot activate Halt under the cardinal geometry — the
    // default Portal is one row south, so a producer after the target
    // writes a Bang that cannot touch Halt. The lock that would reach an
    // already-executed root is therefore inexpressible, and
    // `a_late_halt_lock_is_refused_and_the_tick_continues` builds it at
    // the execution seam instead.
    let (plans, grids, source) = tick_by_tick(
        Grid::with_shape(14, 4),
        &[
            ".=0101  .+0901",
            "  *!          ",
            "  .+0304      ",
            "              ",
        ],
        1,
    );

    assert_eq!(
        grids[0],
        [
            ".=0101  .+0901",
            "***!    0A    ",
            "  .+0304      ",
            "              "
        ]
    );
    assert_eq!(
        plans[0].locks,
        vec![source.grid().position(2, 2).expect("inside the Grid")]
    );
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );

    let grid = Grid::with_shape(8, 4);
    let bytes = snapshot(grid, &[".=0101  ", "  *!    ", "  .+0304", "        "]);
    let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));
    let (_, _, states) = super::plan(grid, Cells::of(bytes.as_bytes()), &map, Tick::ZERO);
    let order = turns(&states);
    // Parser preorder: Equality, Halt, Add.
    assert_eq!(order, vec![Some(0), Some(1), Some(2)]);
    assert!(
        order[1] < order[2],
        "Halt's Turn precedes the south root: {order:?}"
    );
}

#[test]
fn a_late_halt_lock_is_refused_and_the_tick_continues() {
    // Supply a broken order at the execution seam: the Add takes its Turn
    // before the Halt that locks it. Only the lock is refused, so the Add
    // has already written `07` and the Bang that activated Halt stands.
    let grid = Grid::with_shape(8, 4);
    let bytes = snapshot(grid, &[".=0101  ", "  *!    ", "  .+0304", "        "]);
    let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));
    let mut schedule = super::schedule(grid, map.schedule_inputs());
    // Parser preorder: Equality, Halt, Add.
    schedule.ordered.order = [0, 2, 1].map(super::ComputationId).to_vec();
    let (plan, _, states) = super::execution::execute(
        grid,
        Cells::of(bytes.as_bytes()),
        &map,
        Tick::ZERO,
        &schedule,
    );
    assert_eq!(turns(&states), vec![Some(0), Some(2), Some(1)]);
    let mut published = bytes.clone().into_bytes();
    for write in &plan.writes {
        published[write.cell.get()] = write.content.as_char() as u8;
    }
    let published: Vec<_> = published
        .chunks(grid.columns())
        .map(|row| std::str::from_utf8(row).unwrap())
        .collect();
    assert_eq!(published, [".=0101  ", "***!    ", "  .+0304", "  07    "]);
    assert!(plan.locks.is_empty());
    assert_eq!(
        anchors(&plan),
        [(2, 1, "spatial output reached an executed computation")]
    );
}

#[test]
fn an_active_halt_locks_a_south_root_that_would_advance_onto_it() {
    // Halt's lock withholds the south root's Turn, including a Self-Banging
    // North whose Advance names Halt's Cells. The lock edge wins: the Tick
    // is ordered, `^^` stays, and it does not become `**`.
    let (plans, grids, source) = tick_by_tick(
        Grid::with_shape(8, 3),
        &[".=0101  ", "  *!    ", "  ^^    "],
        1,
    );

    assert_eq!(grids[0], [".=0101  ", "***!    ", "  ^^    "]);
    assert_eq!(
        plans[0].locks,
        vec![source.grid().position(2, 2).expect("inside the Grid")]
    );
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn an_active_halt_locks_a_south_root_that_would_emit_onto_it() {
    // `*^` is Bang-activated, so a second Equality wakes it from the east.
    // Its north emission names Halt's Cells. The lock edge wins: the Tick
    // is ordered, `*^` stays, and it does not emit `^^`.
    let (plans, grids, source) = tick_by_tick(
        Grid::with_shape(12, 3),
        &[".=0101      ", "  *!  .=0202", "  *^        "],
        1,
    );

    assert_eq!(grids[0], [".=0101      ", "***!  .=0202", "  *^  **    "]);
    assert_eq!(
        plans[0].locks,
        vec![source.grid().position(2, 2).expect("inside the Grid")]
    );
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn an_active_halt_locks_a_south_root_that_would_copy_onto_it() {
    // `=^` is intrinsically active and overwrites occupied Cells. Its
    // output Portal is Halt. The lock wins: the Tick is ordered, Halt
    // stays, and `01` is not copied onto it.
    let (plans, grids, source) = tick_by_tick(
        Grid::with_shape(8, 4),
        &[".=0101  ", "  *!    ", "  =^    ", "  01    "],
        1,
    );

    assert_eq!(grids[0], [".=0101  ", "***!    ", "  =^    ", "  01    "]);
    assert_eq!(
        plans[0].locks,
        vec![source.grid().position(2, 2).expect("inside the Grid")]
    );
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn multiple_halts_and_activations_follow_dependency_order() {
    // Two independent Halt columns. Each Equality activates the Halt
    // two columns east; Position breaks the tie between the two
    // Equalities and then between the two Halts. Both Adds stay locked.
    let (plans, grids, source) = tick_by_tick(
        Grid::with_shape(16, 4),
        &[
            ".=0101  .=0202",
            "  *!      *!  ",
            "  .+0102  .+0304",
            "                ",
        ],
        1,
    );

    assert_eq!(
        grids[0],
        [
            ".=0101  .=0202  ",
            "***!    ***!    ",
            "  .+0102  .+0304",
            "                "
        ]
    );
    assert_eq!(
        plans[0].locks,
        vec![
            source.grid().position(2, 2).expect("inside the Grid"),
            source.grid().position(10, 2).expect("inside the Grid"),
        ]
    );
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn halt_is_interpreted_once_per_tick() {
    // Equality Bangs from the west and `<<` contacts from the east.
    // Two activation paths, one Turn: a second visit would have
    // doubled Halt's interpretation count.
    let grid = Grid::with_shape(8, 4);
    let bytes = snapshot(grid, &[".=0101  ", "  *!<<  ", "  .+0304", "        "]);
    let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));
    let (plan, _, states) = super::plan(grid, Cells::of(bytes.as_bytes()), &map, Tick::ZERO);

    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(
        plan.locks,
        vec![grid.position(2, 2).expect("inside the Grid")]
    );
    assert_eq!(
        states
            .iter()
            .map(ComputationState::interpretations)
            .collect::<Vec<_>>(),
        vec![1, 1, 1, 0],
        "Equality, Halt once, <<, locked Add never"
    );
}

#[test]
fn an_active_halt_withholds_a_terminal_root() {
    // The complete target Expression contributes no effects: a locked
    // Raw Play emits no Play Command, and its Source is unchanged.
    let (plans, grids, source) = tick_by_tick(
        Grid::with_shape(10, 3),
        &[".=0101    ", "  *!      ", "  !>007FC4"],
        1,
    );

    assert_eq!(grids[0], [".=0101    ", "***!      ", "  !>007FC4"]);
    assert_eq!(
        plans[0].locks,
        vec![source.grid().position(2, 2).expect("inside the Grid")]
    );
    assert!(
        plans[0].play_commands.is_empty(),
        "{:?}",
        plans[0].play_commands
    );
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}
