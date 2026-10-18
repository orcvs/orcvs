//! Self-Banging Functions and Directional Bang emissions: how a mover advances,
//! is blocked or bangs where it stops, and how emissions and moves contest Cells
//! in Source order.

use crate::grid::Grid;

use super::fixture::{messages, tick_by_tick};
use super::observed::raw;

#[test]
fn a_self_banging_function_advances_its_whole_span_by_one_cell() {
    // The move, in all four directions, each from the middle of a Grid that
    // admits it. The vertical pair shares no Cell with the Span it left;
    // the horizontal pair overlaps it by one, and the clear still covers
    // the complete old Span while later-write-wins commits the Cell the two
    // writes share. Carving the clear around that Cell and testing both
    // Cells a horizontal move lands on are the two symmetrical bugs this
    // states the answer to.
    let column = Grid::with_shape(6, 3);
    let (_, north, _) = tick_by_tick(column, &["", "^^", ""], 1);
    assert_eq!(north[0], ["^^    ", "      ", "      "]);

    let (_, south, _) = tick_by_tick(column, &["", "vv", ""], 1);
    assert_eq!(south[0], ["      ", "      ", "vv    "]);

    let row = Grid::with_shape(6, 1);
    let (_, east, _) = tick_by_tick(row, &["  >>  "], 1);
    assert_eq!(east[0], ["   >> "]);

    let (_, west, _) = tick_by_tick(row, &["  <<  "], 1);
    assert_eq!(west[0], [" <<   "]);
}

#[test]
fn a_self_banging_function_moves_once_per_tick_and_bangs_where_it_stops() {
    // Intrinsic activation is a turn every Tick and not a one-off: the
    // Function is in the Snapshot each time, so it moves each time. The
    // last Tick is the edge, which is the other half — an out-of-Grid move
    // costs it the Span it stood in and leaves `**` there.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(6, 1), &["  >>  "], 3);

    assert_eq!(grids[0], ["   >> "]);
    assert_eq!(grids[1], ["    >>"]);
    assert_eq!(grids[2], ["    **"]);
    for plan in &plans {
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }
}

#[test]
fn two_moves_that_want_the_same_cells_each_bang_in_their_own_span() {
    // Two movers approaching each other close the gap by two Cells a Tick,
    // so its parity never changes and there are two cases. An odd gap ends
    // at one Cell and is `an_odd_gap_leaves_the_earlier_mover_the_cell_and_
    // the_later_one_bangs`. An even gap ends
    // flush, and that is this test — each wants the two Cells the other
    // stands in, and each takes the refusal a mover blocked by anything
    // else takes: it replaces its current Span with `**`.
    //
    // No rule here is new. `>>` holds the earlier Turn by Source order,
    // finds `<` in the Cell it would enter, and reports in its own Span.
    // `<<` then finds that fresh `*` and does the same. Each `**` is a Bang
    // Function on the next Tick, which fires and clears it, so the pair
    // is gone.
    //
    // What this pins is the schedule. Each mover reserves Cells the other
    // stands in, which is an edge each way and an order no sort satisfies.
    // `dependencies::derive` drops one of them: keeping both makes a cycle that
    // stops both movers, and neither moves again.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 1), &[">>  <<  "], 3);

    assert_eq!(grids[0], [" >><<   "]);
    assert_eq!(grids[1], [" ****   "]);
    assert_eq!(grids[2], ["        "]);
    for plan in &plans {
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }

    // The vertical pair, which reaches the same refusal through a different
    // geometry: a vertical destination shares no Cell with the Span it
    // leaves, so each mover tests the whole of the other's Span rather than
    // the one Cell a horizontal move enters.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(4, 2), &["vv", "^^"], 2);

    assert_eq!(grids[0], ["**  ", "**  "]);
    assert_eq!(grids[1], ["    ", "    "]);
    for plan in &plans {
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }
}

#[test]
fn a_mover_contacts_the_whole_mover_that_entered_vacated_cells() {
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 3), &["", "  <<<<", "   ^^"], 1);
    assert_eq!(grids[0], ["        ", " <<<<   ", "   **   "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn a_mover_still_diagnoses_partial_contact_with_a_moved_mover() {
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 3), &["", "  <<<<", "  ^^"], 1);
    assert_eq!(grids[0], ["        ", " <<<<   ", "  **    "]);
    assert_eq!(messages(&plans[0]), ["^^ contacts part of a Language Unit"]);
}

#[test]
fn a_mover_diagnoses_partial_contact_with_a_blocked_movers_bang() {
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(4, 2), &["<<", " ^^"], 1);
    assert_eq!(grids[0], ["**  ", " ** "]);
    assert_eq!(messages(&plans[0]), ["^^ contacts part of a Language Unit"]);
}

#[test]
fn a_west_train_vacates_before_followers_and_preserves_other_effects() {
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(14, 2), &["  <<<<  .+0102", ""], 3);
    assert_eq!(grids[0], [" <<<<   .+0102", "        03    "]);
    assert_eq!(grids[1], ["<<<<    .+0102", "        03    "]);
    assert_eq!(grids[2], ["****    .+0102", "        03    "]);
    for plan in plans {
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }
}

#[test]
fn a_north_train_vacates_before_followers_and_bangs_at_edge() {
    let (plans, grids, _) =
        tick_by_tick(Grid::with_shape(10, 4), &["    .+0102", "", "^^", "^^"], 3);
    assert_eq!(
        grids[0],
        ["    .+0102", "^^  03    ", "^^        ", "          "]
    );
    assert_eq!(
        grids[1],
        ["^^  .+0102", "^^  03    ", "          ", "          "]
    );
    assert_eq!(
        grids[2],
        ["**  .+0102", "**  03    ", "          ", "          "]
    );
    for plan in plans {
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }
}

#[test]
fn a_converging_west_train_closes_then_bangs() {
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(20, 2), &["    >>  <<<<  .+0102", ""], 3);
    assert_eq!(grids[0], ["     >><<<<   .+0102", "              03    "]);
    assert_eq!(grids[1], ["     ******   .+0102", "              03    "]);
    assert_eq!(grids[2], ["              .+0102", "              03    "]);
    for plan in plans {
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }
}

#[test]
fn a_south_train_follower_is_blocked_before_its_leader_leaves() {
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(2, 5), &["vv", "vv", "", "", ""], 3);
    assert_eq!(grids[0], ["**", "  ", "vv", "  ", "  "]);
    assert_eq!(grids[1], ["  ", "  ", "  ", "vv", "  "]);
    assert_eq!(grids[2], ["  ", "  ", "  ", "  ", "vv"]);
    for plan in plans {
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }
}

#[test]
fn a_mover_is_blocked_by_an_earlier_overwrite() {
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 3), &[".+0102", "  <<", ""], 1);
    assert_eq!(grids[0], [".+0102  ", "03**    ", "        "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn a_movers_new_cells_can_be_overwritten_after_its_turn() {
    let (plans, grids, _) = tick_by_tick(
        Grid::with_shape(8, 5),
        &["  vv", "", "  =^", ".+0300", ""],
        1,
    );
    assert_eq!(
        grids[0],
        ["        ", "  03    ", "  =^    ", ".+0300  ", "03      "]
    );
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn a_mover_meets_the_overwrite_of_a_moved_mover_as_source_content() {
    // `vv` moves into Cells the Snapshot left empty, and the Copy then
    // overwrites its new spelling with `01`. `^^` is blocked by that `1`
    // and also enters an empty Cell past it. The overwritten mover is no
    // longer a unit there to meet part of, so the blocked move bangs
    // without diagnosing.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 3), &["    vv", "01=>", "     ^^"], 1);
    assert_eq!(grids[0], ["        ", "01=>01  ", "     ** "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn a_mover_supplies_a_claimed_operand_before_its_consumer() {
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 3), &[".+  01", "  ^^", ""], 1);
    assert_eq!(grids[0], [".+^^01  ", "        ", "        "]);
    assert_eq!(plans[0].diagnostics.len(), 1);
    assert!(
        messages(&plans[0])[0].contains("^^"),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn a_mover_supplies_an_input_portal_before_its_consumer() {
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(4, 4), &["", "=^", "", "^^"], 1);
    assert_eq!(grids[0], ["^^  ", "=^  ", "^^  ", "    "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn a_copy_reads_a_mover_that_stepped_one_cell_into_its_input_portal() {
    // The `>>` steps one Cell east into the pair `=>` reads. Its new Cells
    // overlap the Span the Snapshot parsed it at, but that Span no longer
    // holds it: the Copy reads the placed unit whole, as it does a mover
    // that entered the pair from the next row.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 1), &[" >> =>  "], 1);
    assert_eq!(grids[0], ["  >>=>>>"]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn a_copy_reads_a_dynamic_write_over_cells_a_fired_bang_cleared() {
    // The first Write reads the `**`'s Cells, so the `**` fires first and
    // clears them. The second Write then puts `C4` across the cleared Cell
    // and the one east of it, and `=v` reads that pair whole: the `**` its
    // Snapshot parsed no longer holds either Cell.
    let rows = ["", "**", " =v", "", "@$0A00&$0001", "", "@$0101C4"];
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(14, 7), &rows, 1);
    assert_eq!(
        grids[0][1..4],
        [" C4           ", " =v           ", " C4           "]
    );
    assert_eq!(
        messages(&plans[0]),
        ["nested computation at column 6, row 4 cleared its operand"]
    );
}

// The `cfg` matches the `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]`
// table that declares proptest, so a WASM build never sees the dependency.
#[cfg(not(target_arch = "wasm32"))]
mod placement_property {
    use crate::grid::Grid;
    use crate::source::TickPlan;
    use crate::source::tick::fixture::{messages, tick_by_tick};
    use proptest::prelude::*;
    use proptest::sample::select;

    /// Fails on any placement-caused ordering defect or cycle in `plans`.
    fn no_defect_or_cycle(rows: &[String], plans: &[TickPlan]) -> Result<(), TestCaseError> {
        for plan in plans {
            for message in messages(plan) {
                prop_assert!(
                    !message.contains("reached an executed computation"),
                    "{rows:?}: {message}"
                );
                prop_assert!(
                    !message.contains("same-Tick dependency cycle"),
                    "{rows:?}: {message}"
                );
            }
        }
        Ok(())
    }

    /// The anchor and spelling of a Self-Banging Function aimed at the
    /// emitter at `(5, 2)` from `side`, about `gap` Cells short of it.
    fn aimed_mover(side: usize, gap: usize) -> (usize, usize, &'static str) {
        match side {
            0 => (5, 1usize.saturating_sub(gap), "vv"),
            1 => (5, 3 + gap.min(1), "^^"),
            2 => (3 - gap, 2, ">>"),
            _ => (7 + gap, 2, "<<"),
        }
    }

    proptest! {
        #[test]
        fn placements_never_meet_an_ordering_defect_or_a_cycle(
            rows in prop::collection::vec(
                prop::collection::vec(
                    select(vec![
                        " ", "  ", "^^", "vv", "<<", ">>", "*^", "*v", "*<", "*>", "**", "*!",
                    ]),
                    0..=6,
                ),
                4,
            ),
        ) {
            // Whole spellings and single spaces give both aligned and offset
            // placements. A blocked mover and a typed `**` are what bang, so
            // an emitter or a Halt a mover contacts may be activated in the
            // Tick it is contacted, or not at all.
            let rows: Vec<String> = rows.iter().map(|row| row.concat()).collect();
            let refs: Vec<&str> = rows.iter().map(String::as_str).collect();
            let (plans, _, _) = tick_by_tick(Grid::with_shape(12, 4), &refs, 5);
            no_defect_or_cycle(&rows, &plans)?;
        }

        #[test]
        fn collision_activated_emissions_never_meet_an_ordering_defect_or_a_cycle(
            emitter in select(vec!["*^", "*v", "*<", "*>"]),
            side in 0..4usize,
            gap in 0..3usize,
            destination in select(vec!["  ", "xx", "^^", "vv", "<<", ">>"]),
        ) {
            // A mover aimed at an emitter activates it by collision, and the
            // emitter's destination is empty, static, or another mover, so
            // activated emissions both succeed and are refused.
            let mut grid = vec![vec![b' '; 12]; 5];
            let mut put = |x: usize, y: usize, spelling: &str| {
                grid[y][x..x + 2].copy_from_slice(spelling.as_bytes());
            };
            put(5, 2, emitter);
            let (x, y) = match emitter {
                "*^" => (5, 1),
                "*v" => (5, 3),
                "*<" => (3, 2),
                _ => (7, 2),
            };
            put(x, y, destination);
            let (mx, my, mover) = aimed_mover(side, gap);
            if (mx..mx + 2).all(|cell| !(x..x + 2).contains(&cell)) || my != y {
                put(mx, my, mover);
            }
            let rows: Vec<String> = grid
                .into_iter()
                .map(|row| String::from_utf8(row).expect("ASCII Source"))
                .collect();
            let refs: Vec<&str> = rows.iter().map(String::as_str).collect();
            let (plans, _, _) = tick_by_tick(Grid::with_shape(12, 5), &refs, 5);
            no_defect_or_cycle(&rows, &plans)?;
        }
    }
}

#[test]
fn an_odd_gap_leaves_the_earlier_mover_the_cell_and_the_later_one_bangs() {
    // One Cell of overlap between two movers, and Source order decides who
    // has it. `>>` takes the earlier Turn and enters the empty Cell; `<<`
    // then finds `>` where it would enter and replaces its own Span with
    // `**`. On the next Tick that `**` is a Bang Function later in Source
    // order than `>>`, which meets it still standing and is blocked in
    // turn; the `**` it leaves fires on the Tick after.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 1), &[">> <<   "], 3);

    assert_eq!(grids[0], [" >>**   "]);
    assert_eq!(grids[1], [" **     "]);
    assert_eq!(grids[2], ["        "]);
    for plan in &plans {
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }
}

#[test]
fn converging_movers_resolve_pairwise_in_source_order() {
    // Three movers settle as the two-mover rule applied Turn by Turn,
    // with no cycle among them. `>>` at Cell 0 is blocked by `>>` at Cell
    // 2, which takes the free Cell and meets `<<` flush on the next Tick.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(12, 1), &[">>>>  <<    "], 3);

    assert_eq!(grids[0], ["** >><<     "]);
    assert_eq!(grids[1], ["   ****     "]);
    assert_eq!(grids[2], ["            "]);
    for plan in &plans {
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }
}

#[test]
fn an_emission_and_a_mover_wanting_one_cell_follow_source_order() {
    // `*>` emits into Cells 4 and 5 and `<<` wants Cell 5. `*>` holds
    // the earlier Turn, so it emits, and the mover takes a blocked mover's
    // refusal. The two refusals differ because the two bundles differ: the
    // mover vacates its Span and bangs there, the Directional Bang
    // Function stands still and would diagnose.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(10, 2), &[".=0101", "  *>  <<"], 1);

    assert_eq!(grids[0], [".=0101    ", "***>>>**  "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn a_mover_entering_the_destination_first_leaves_the_emission_refused() {
    // `vv` and `*>` both want Cells 4 and 5 of the bottom row and neither
    // stands in the other's way, so Source order alone decides. `vv` holds
    // the earlier Turn and moves in; `*>` then finds its destination
    // occupied, diagnoses, and emits nothing.
    let (plans, grids, _) =
        tick_by_tick(Grid::with_shape(8, 3), &["  .=0101", "    vv", "  *>"], 1);

    assert_eq!(grids[0], ["  .=0101", "  **    ", "  *>vv  "]);
    assert_eq!(
        messages(&plans[0]),
        vec!["*> has no empty destination inside the Grid for >>"]
    );
}

#[test]
fn a_mover_flush_against_a_directional_bang_function_bangs_rather_than_costing_the_tick() {
    // The mover stands in the emission's destination and moves back into
    // the Directional Bang Function, which never vacates. The mover is
    // blocked and bangs, nothing is contested, and the Addition beside
    // them answers. The `**` it leaves activates nothing in that Tick; on
    // the next it is a Bang Function aligned with the Directional Bang
    // Function, which emits into the Cells it vacated.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(14, 2), &["  *><<  .+0102", ""], 2);
    assert_eq!(grids[0], ["  *>**  .+0102", "        03    "]);
    assert_eq!(grids[1], ["  *>>>  .+0102", "        03    "]);
    for plan in &plans {
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }

    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 1), &["  >>*<  "], 2);
    assert_eq!(grids[0], ["  ***<  "]);
    assert_eq!(grids[1], ["  <<*<  "]);
    for plan in &plans {
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }

    // The vertical pair, activated by an Equality as well: the emission
    // finds the mover's `**` and is refused. On the Tick after, that `**`
    // fires and clears before `*v`, which then emits.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 3), &[".=0101", "  *v", "  ^^"], 2);
    assert_eq!(grids[0], [".=0101  ", "***v    ", "  **    "]);
    assert_eq!(
        messages(&plans[0]),
        vec!["*v has no empty destination inside the Grid for vv"]
    );
    assert_eq!(grids[1], [".=0101  ", "***v    ", "  vv    "]);
    assert!(
        plans[1].diagnostics.is_empty(),
        "{:?}",
        plans[1].diagnostics
    );
}

#[test]
fn an_emission_enters_cells_a_mover_vacated_earlier_in_the_tick() {
    // `^^` stands in the destination of `*^` and moves away from it. It
    // holds the earlier Turn by Source order and nothing orders the
    // emitter before it, so it leaves first and `*^` emits into the Cells
    // it vacated.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(10, 3), &["", "  ^^.=0101", "  *^"], 1);

    assert_eq!(grids[0], ["  ^^      ", "  ^^.=0101", "  *^**    "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn an_emission_waits_for_no_occupant_to_leave() {
    // `vv` stands in the destination of `*v` and would move away from it,
    // but takes its Turn later in Source order. The emission finds the
    // Cells occupied at its own Turn and is refused; the mover then moves.
    let (plans, grids, _) =
        tick_by_tick(Grid::with_shape(8, 4), &[".=0101", "  *v", "  vv", ""], 1);

    assert_eq!(grids[0], [".=0101  ", "***v    ", "        ", "  vv    "]);
    assert_eq!(
        messages(&plans[0]),
        vec!["*v has no empty destination inside the Grid for vv"]
    );
}

#[test]
fn an_emission_contest_follows_the_order_its_activation_imposes() {
    // The same contenders in both Grids: `*>` at column 2 emits into
    // columns 4 and 5, and `<<` at column 6 wants column 5. Only the Bang
    // that activates `*>` moves. From above, its supplier takes the first
    // Turn, the emission is ready before `<<` and wins, and `<<` is
    // blocked.
    let grid = Grid::with_shape(10, 3);
    let (plans, grids, _) = tick_by_tick(grid, &["  **", "  *>  <<", ""], 1);
    assert_eq!(grids[0], ["          ", "  *>>>**  ", "          "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );

    // From below, the supplier follows `<<` in Source order and the
    // emitter waits for it, so `<<` enters column 5 first and the
    // emission is refused.
    let (plans, grids, _) = tick_by_tick(grid, &["", "  *>  <<", "  **"], 1);
    assert_eq!(grids[0], ["          ", "  *> <<   ", "          "]);
    assert_eq!(
        messages(&plans[0]),
        vec!["*> has no empty destination inside the Grid for >>"]
    );
}

#[test]
fn an_emission_into_a_claimed_operand_settles_before_its_consumer() {
    // `*v` emits `vv` into the empty first operand of the Addition below
    // it. The Addition waits for that supplier and reads what it wrote,
    // which is no number, so the Addition diagnoses its operand.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 3), &[".=0101", "  *v", ".+  02"], 1);

    assert_eq!(grids[0], [".=0101  ", "***v    ", ".+vv02  "]);
    assert_eq!(messages(&plans[0]), vec!["expected a number, found \"vv\""]);
}

#[test]
fn an_emission_reserving_its_activators_operand_is_a_cycle() {
    // The Equality's Bang can activate `*^`, and `*^`'s destination
    // reserves the Equality's first operand, so each waits for the other.
    // Placement adds neither edge; the value-input and activation edges
    // make the cycle that
    // .scratch/placement-semantics/issues/04-investigate-emission-activation-operand-cycles.md
    // investigates.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 3), &[".=0101", ">>*^", ""], 1);

    assert_eq!(grids[0], [".=0101  ", ">>*^    ", "        "]);
    assert!(
        messages(&plans[0]).contains(&"same-Tick dependency cycle"),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn a_blocked_pair_of_moves_leaves_the_rest_of_the_grid_running() {
    // A blocked pair costs only itself: the pair reports in its own four
    // Cells and the Addition sharing the Source with it answers `03` in
    // the same Tick.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(13, 2), &[" >><<  .+0102", ""], 1);

    assert_eq!(grids[0], [" ****  .+0102", "       03    "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn a_move_that_leaves_the_grid_replaces_its_own_span_with_bang() {
    // Every edge, each reached by the one Function that points at it. Two
    // refusals answer for the four: a displacement past the first column
    // or past the last row resolves no Portal at all, and a displacement
    // that stays inside the Grid but runs past the row edge resolves one
    // and is refused the whole write. Both have the same cost, so the Grid
    // says the same thing four times.
    let column = Grid::with_shape(4, 1);
    let (_, north, _) = tick_by_tick(column, &["^^  "], 1);
    assert_eq!(north[0], ["**  "]);

    let (_, south, _) = tick_by_tick(column, &["vv  "], 1);
    assert_eq!(south[0], ["**  "]);

    let (_, west, _) = tick_by_tick(column, &["<<  "], 1);
    assert_eq!(west[0], ["**  "]);

    let (_, east, _) = tick_by_tick(column, &["  >>"], 1);
    assert_eq!(east[0], ["  **"]);
}

#[test]
fn an_active_directional_bang_function_emits_its_self_banging_function() {
    // Each of these emits the matching root-only Self-Banging Function.
    // For a producer at `(x, y)`, north emits at `(x, y-1)`, south at
    // `(x, y+1)`, west at `(x-2, y)`, and east at `(x+2, y)`. The
    // horizontal offsets are two columns and the Self-Banging Functions'
    // are one, which is the difference a direction name would have hidden:
    // one emits outside its own Span and the other moves through its own.
    //
    // Each fixture reaches its producer through a different arm of the
    // cardinal rule, because the emission has to land somewhere
    // the Bang is not. Equality Bangs on every Tick, so the `**` it writes
    // and the emitted spelling are both in the Grid these compare.
    let tall = Grid::with_shape(8, 3);
    let (_, south, _) = tick_by_tick(tall, &[".=0101", "  *v", ""], 1);
    assert_eq!(south[0], [".=0101  ", "***v    ", "  vv    "]);

    let wide = Grid::with_shape(10, 2);
    let (_, east, _) = tick_by_tick(wide, &[".=0101", "  *>"], 1);
    assert_eq!(east[0], [".=0101    ", "***>>>    "]);

    let (_, north, _) = tick_by_tick(wide, &["    .=0101", "  *^"], 1);
    assert_eq!(north[0], ["  ^^.=0101", "  *^**    "]);

    let (_, west, _) = tick_by_tick(wide, &["    .=0101", "  *<"], 1);
    assert_eq!(west[0], ["    .=0101", "<<*<**    "]);
}

#[test]
fn an_inert_directional_bang_function_emits_nothing() {
    // The activation asymmetry, stated as the Grid that does not change.
    // `*>` and `>>` declare the same spelling at the same kind of Portal
    // and differ in where the Turn comes from; with no Bang to deliver one,
    // this Grid stands still where the `>>` test's Grid moves every Tick.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 2), &["  *>", ""], 2);

    assert_eq!(grids[0], ["  *>    ", "        "]);
    assert_eq!(grids[1], ["  *>    ", "        "]);
    for plan in &plans {
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }
}

#[test]
fn a_refused_emission_diagnoses_and_writes_no_cell() {
    // The complete initial destination must be empty and inside the Grid,
    // or the producer diagnoses and emits nothing. This is where
    // the two groups differ in what a refusal costs — a Self-Banging
    // Function replaces its own Span with `**` — and the reason is that
    // this producer is not leaving its own Cells, so it has none to say it
    // in. Both refusals are here: Cells that are not empty, and a
    // displacement that leaves the Grid.
    //
    // The diagnostic names the producer and not the Function it would have
    // emitted. The two come apart only for this group — a Self-Banging
    // Function writes its own spelling — and naming the emission alone
    // would point the author at Cells holding `>>` when the Grid contains
    // no `>>` at all. Both spellings are here because both are the author's
    // question: which Cell to fix, and what it was trying to put where.
    let occupied = Grid::with_shape(10, 2);
    let (plans, grids, _) = tick_by_tick(occupied, &[".=0101", "  *>xx"], 1);
    assert_eq!(grids[0], [".=0101    ", "***>xx    "]);
    assert_eq!(
        messages(&plans[0]),
        vec!["*> has no empty destination inside the Grid for >>"]
    );

    let edge = Grid::with_shape(10, 2);
    let (plans, grids, _) = tick_by_tick(edge, &["  .=0101", "*<"], 1);
    assert_eq!(grids[0], ["  .=0101  ", "*<**      "]);
    assert_eq!(
        messages(&plans[0]),
        vec!["*< has no empty destination inside the Grid for <<"]
    );

    // The vertical half of the same refusal, which leaves the Grid by the
    // last row rather than by the first column. It is here because the two
    // reach `Portal::displaced` through different arms of `Grid::displaced`
    // — a row that does not exist against a column that does not — and one
    // fixture proves only the arm it takes.
    let floor = Grid::with_shape(8, 2);
    let (plans, grids, _) = tick_by_tick(floor, &[".=0101", "  *v"], 1);
    assert_eq!(grids[0], [".=0101  ", "***v    "]);
    assert_eq!(
        messages(&plans[0]),
        vec!["*v has no empty destination inside the Grid for vv"]
    );
}

#[test]
fn copied_bangs_reach_emission_refusals_at_the_right_and_top_edges() {
    // A Copy's newly produced Bang activates an edge root aligned with its
    // destination. Activation must precede its Turn even when the root
    // sorts before the Copy, as the top-edge fixture does.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 2), &[".=0101", "  =>  *>"], 1);
    assert_eq!(grids[0], [".=0101  ", "**=>***>"]);
    assert_eq!(
        messages(&plans[0]),
        vec!["*> has no empty destination inside the Grid for >>"]
    );

    let (plans, grids, _) = tick_by_tick(Grid::with_shape(10, 2), &["*^  .=0101", "  =<"], 1);
    assert_eq!(grids[0], ["*^  .=0101", "**=<**    "]);
    assert_eq!(
        messages(&plans[0]),
        vec!["*^ has no empty destination inside the Grid for ^^"]
    );
}

#[test]
fn a_copied_bang_overwrites_an_edge_roots_anchor_rather_than_activating_it() {
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(6, 2), &[".=0101", "  =>*>"], 1);
    assert_eq!(grids[0], [".=0101", "**=>**"]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn an_emitted_self_banging_function_first_moves_on_the_following_tick() {
    // The whole cycle in one fixture: a Delay Bangs on Tick 0 and on no
    // Tick after it, `*>` writes `>>`, and `>>` moves once per Tick from
    // the Tick after the one that wrote it. That a generated Function first
    // receives a turn from the next Source Snapshot is the assertion on the
    // first Grid — `>>` is at Cells 2 and 3 there, not 3 and 4 —
    // and it needs no rule of its own: a schedule is built from the Source
    // Snapshot, and nothing this Tick wrote is in it.
    //
    // The Delay is what keeps the second half legible. An Equality would
    // Bang again on Tick 1, and `*>` would then be pointing at the Cells
    // its own emission had not yet vacated.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(12, 3), &["~*0401", "", "*>"], 3);

    assert_eq!(grids[0], ["~*0401      ", "**          ", "*>>>        "]);
    assert_eq!(grids[1], ["~*0401      ", "            ", "*> >>       "]);
    assert_eq!(grids[2], ["~*0401      ", "            ", "*>  >>      "]);
    for plan in &plans {
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }
}

#[test]
fn an_eastward_move_stops_at_the_row_edge_rather_than_wrapping_into_the_next_row() {
    // A row is the whole horizontal extent there is. The Cell after the
    // last column of row 0 exists in the Grid and is the first Cell of row
    // 1, so a displacement alone cannot tell the two apart; the row-edge
    // refusal is what does, and it costs the whole write. The second row
    // stays empty, which is the assertion that says so.
    let (_, grids, _) = tick_by_tick(Grid::with_shape(4, 2), &["  >>", ""], 1);

    assert_eq!(grids[0], ["  **", "    "]);
}

#[test]
fn a_westward_move_stops_at_the_row_edge_rather_than_wrapping_into_the_previous_row() {
    // The mirror, and a different refusal reaching the same cost. Eastward
    // off the last column and westward off the first both stay inside the
    // Grid when the neighbouring row exists, so `Portal::displaced`
    // resolves a destination in each case and the row-edge check is the
    // only thing that refuses it. The single-row edge tests above take the
    // other path, where no Portal resolves at all.
    let (_, grids, _) = tick_by_tick(Grid::with_shape(4, 2), &["", "<<  "], 1);

    assert_eq!(grids[0], ["    ", "**  "]);
}

#[test]
fn two_movers_reserving_each_other_each_bang_rather_than_costing_the_tick() {
    // Each of these reserves a destination that covers the other's Span,
    // so the two reservations name each other. A reservation orders Turns
    // and decides nothing else, and neither of these Turns can write where
    // the other stands: a move is admitted only into empty Cells, so mutual
    // reservation describes two blocked moves rather than two writes
    // competing for one Cell. Ordering either after the other would make
    // that pair a cycle and stop both. Both are blocked
    // by complete root contact and both bang, which is what a blocked move
    // does whatever blocked it.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(6, 1), &[">><<  "], 1);

    assert_eq!(grids[0], ["****  "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );

    // The vertical pair shares no Cell at all, so it says the same thing
    // about reservations rather than about the overlap a horizontal move
    // has with its own old Span.
    let (vertical, rows, _) = tick_by_tick(Grid::with_shape(2, 2), &["vv", "^^"], 1);

    assert_eq!(rows[0], ["**", "**"]);
    assert!(
        vertical[0].diagnostics.is_empty(),
        "{:?}",
        vertical[0].diagnostics
    );
}

#[test]
fn a_move_blocked_by_one_complete_language_unit_bangs_without_diagnosing() {
    // Complete non-root contact adds no collision diagnostic.
    // The two Cells north of `^^` are the whole of the `01` operand
    // literal, so the contact is complete and the unit is no root. The
    // Addition beside it still answers, which is what says the `**` is
    // this Function reporting its own blocked move rather than the Tick
    // going wrong around it.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 3), &[".+0102", "  ^^", ""], 1);

    assert_eq!(grids[0], [".+0102  ", "03**    ", "        "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn a_move_that_lands_across_two_language_units_diagnoses_its_misalignment() {
    // The other half of the same rule: these two Cells hold the last Cell
    // of `01` and the first Cell of `02`, so neither unit is met whole.
    // The move diagnoses and delivers nothing, and is blocked
    // exactly as a complete contact blocks it — the diagnostic is the only
    // difference, because a misalignment is the one outcome the `**` alone
    // does not tell a Source author about.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 3), &[".+0102", "   ^^", ""], 1);

    assert_eq!(grids[0], [".+0102  ", "03 **   ", "        "]);
    assert_eq!(
        messages(&plans[0]),
        vec!["^^ contacts part of a Language Unit"]
    );
}

#[test]
fn a_mover_meets_comment_text_as_ordinary_cells() {
    // Blank text is vacant, so the mover enters it.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 2), &["  vv", "||      "], 1);
    assert_eq!(grids[0], ["        ", "||vv    "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );

    // Glyphs in the text block it, and no Language Unit occupies them, so
    // the refusal makes no contact to diagnose.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 2), &["  vv", "||C3    "], 1);
    assert_eq!(grids[0], ["  **    ", "||C3    "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );

    // The introducer is the Comment's, so a pair across it and the text
    // meets part of the Comment.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 2), &[" vv", "||C3    "], 1);
    assert_eq!(grids[0], [" **     ", "||C3    "]);
    assert_eq!(messages(&plans[0]), ["vv contacts part of a Language Unit"]);
}

#[test]
fn complete_aligned_root_contact_activates_nothing_until_the_bang_it_leaves_fires() {
    // A blocked move writes `**` and delivers nothing in its Turn: the Raw
    // Play north of `^^` is inert on its own, so no Play Command is the
    // evidence. The `**` is a Bang Function on the next Tick and plays the
    // Raw Play aligned north of it then.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 2), &["!>007FC4", "^^"], 2);

    assert_eq!(grids[0], ["!>007FC4", "**      "]);
    assert!(
        plans[0].play_commands.is_empty(),
        "{:?}",
        plans[0].play_commands
    );
    assert_eq!(grids[1], ["!>007FC4", "        "]);
    assert_eq!(plans[1].play_commands, vec![raw(0, 0x7F, 60)]);
    for plan in &plans {
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }
}

#[test]
fn a_horizontally_blocked_move_plays_the_root_two_columns_away_on_the_next_tick() {
    // The same rule where the geometry differs: a horizontal move enters
    // one Cell of the root anchored two columns from the producer, because
    // every Language Unit spells as a Cell pair. The `**` the move leaves
    // has that root as its east anchor `(x+2, y)`.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(10, 2), &[">>!>007FC4", ""], 2);

    assert_eq!(grids[0], ["**!>007FC4", "          "]);
    assert!(
        plans[0].play_commands.is_empty(),
        "{:?}",
        plans[0].play_commands
    );
    assert_eq!(grids[1], ["  !>007FC4", "          "]);
    assert_eq!(plans[1].play_commands, vec![raw(0, 0x7F, 60)]);
    for plan in &plans {
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }
}

#[test]
fn the_schedule_orders_a_move_ahead_of_the_computation_it_would_land_on() {
    // Reserving the destination Portal is what puts these two Turns in an
    // order, and the order is what keeps the move honest: `>>` points at
    // the Cell `^^` stands in, so `>>` takes its Turn first and reads the
    // Source Snapshot's own answer — blocked, by a complete root, so `**`
    // and an activation `^^` already had. `^^` then moves north out of the
    // way it never had to give.
    //
    // Without that edge `^^` could move first and `>>` would find the Cell
    // empty, planning a write over Cells a computation that had already
    // run was scheduled at. That write would be refused as an ordering
    // defect, and this is the ordering that stops it arising.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(6, 2), &["", ">>^^  "], 1);

    assert_eq!(grids[0], ["  ^^  ", "**    "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn a_self_banging_function_is_not_scheduled_inside_another_expression() {
    // Root-only, enforced by the declared kind rather than by a check that
    // names these four spellings: `.+` declares two Number operands, `^^`
    // answers an effect, and the Parser refuses it where a value is
    // required. The refusal is the Expression's and is reported from the
    // Source, so the Cells are left standing, nothing moves, and the
    // Tick repeats nothing.
    let (plans, grids, source) = tick_by_tick(Grid::with_shape(8, 2), &[".+^^01", ""], 1);

    assert_eq!(grids[0], [".+^^01  ", "        "]);
    assert!(messages(&plans[0]).is_empty(), "{:?}", plans[0].diagnostics);
    assert_eq!(
        source
            .language_map()
            .diagnostics()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>(),
        ["a Function that answers an effect is valid only at the root of an Expression"]
    );
}
