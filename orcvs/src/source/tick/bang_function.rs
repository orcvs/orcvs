//! The Bang Function, standalone `**`, written as a performer sees it Tick
//! after Tick.
//!
//! A standalone `**` is a root-only Function with no operands that takes its
//! Turn with nothing delivered to it. Its Turn activates the roots aligned
//! with its Span and clears its two Cells. Every other outcome here is an
//! existing rule applied to that root: a producer's write over it is ordered
//! first and suppresses it, a Halt locks it, a reader of its Cells waits for
//! it, and a mover meets it in Source order. Nothing about a Bang is
//! remembered between Ticks, so whether a `**` fires depends on the Source
//! Snapshot and the Tick alone.

use lang::Tick;

use super::observed::{
    MISSED_BANG, Observed, diagnostic, observe_at, observed, quiet, raw, raw_c4, rows_of, source_of,
};
use crate::grid::Grid;
use crate::source::{Source, TickPlan};

/// Writes `text` into `source` from column `x` of row `y`, one Cell at a
/// time, as an editor does.
fn type_at(source: &mut Source, x: usize, y: usize, text: &str) {
    let grid = source.grid();
    for (offset, character) in text.chars().enumerate() {
        let position = grid.position(x + offset, y).expect("inside the Grid");
        source
            .set(grid.index(position), &character.to_string())
            .unwrap();
    }
}

/// Runs one Tick of `source` and answers its Tick Plan with what it left.
fn tick(source: &mut Source, tick: u64) -> (TickPlan, Observed) {
    let plan = source.execute(Tick::new(tick));
    let seen = observed(source, &plan);
    (plan, seen)
}

#[test]
fn a_halt_above_its_activating_bang_misses_it_and_the_bang_fires() {
    // E1. The Halt's lock edge orders it before the `**`, and the `**`'s
    // Bang onto the Halt is the edge the lock already covers. The Halt takes
    // its Turn inert, the `**` fires and plays Raw Play, and its Bang reaches
    // the Halt after its Turn.
    let mut source = source_of(Grid::with_shape(8, 3), &["*!", "**", "!>007FC4"]);
    let (plan, seen) = tick(&mut source, 0);
    assert_eq!(plan.play_commands, [raw_c4()]);
    assert_eq!(seen.rows, ["*!      ", "        ", "!>007FC4"]);
    assert_eq!(seen.diagnostics, [diagnostic(0, 0, MISSED_BANG)]);
    assert!(plan.locks.is_empty(), "{:?}", plan.locks);

    let (plan, seen) = tick(&mut source, 1);
    assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
    quiet(&seen);
}

#[test]
fn a_halt_activated_independently_locks_the_bang_until_its_activator_goes_quiet() {
    // E2. The Equality's Bang activates the Halt, which locks the `**` below
    // it every Tick: the `**` stands and nothing plays.
    let grid = Grid::with_shape(10, 4);
    let rows = [".=0101", "  *!", "  **", "  !>007FC4"];
    let mut source = source_of(grid, &rows);
    for at in 0..2 {
        let (plan, seen) = tick(&mut source, at);
        assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
        assert_eq!(
            seen.rows,
            [".=0101    ", "***!      ", "  **      ", "  !>007FC4"]
        );
        assert_eq!(plan.locks, [grid.position(2, 2).expect("inside the Grid")]);
        quiet(&seen);
    }

    // A quiet Equality clears its Output Portal, so the Halt is inert, the
    // `**` fires, and its Bang reaches the Halt after its Turn.
    let mut source = source_of(grid, &[".=0102", "  *!", "  **", "  !>007FC4"]);
    let (plan, seen) = tick(&mut source, 0);
    assert_eq!(plan.play_commands, [raw_c4()]);
    assert_eq!(
        seen.rows,
        [".=0102    ", "  *!      ", "          ", "  !>007FC4"]
    );
    assert_eq!(seen.diagnostics, [diagnostic(2, 1, MISSED_BANG)]);
    let (plan, seen) = tick(&mut source, 1);
    assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
    quiet(&seen);
}

#[test]
fn a_halt_whose_activator_depends_on_its_target_is_a_cycle() {
    // E3. The `**` is read by `=>`, which feeds `=^`, which feeds `=<`, whose
    // Bang could activate the Halt that locks the `**`. That loop is not the
    // one edge a lock covers, so every Expression on it stops, the `**`
    // stands, and Raw Play waits on it.
    let rows = ["    =<", "  *!  =^", "  **=>", "  !>007FC4"];
    let seen = observe_at(Grid::with_shape(10, 4), &rows, [0]).remove(0);
    assert_eq!(
        seen.rows,
        ["    =<    ", "  *!  =^  ", "  **=>    ", "  !>007FC4"]
    );
    assert_eq!(
        seen.diagnostics[0],
        diagnostic(4, 0, "same-Tick dependency cycle")
    );
    assert!(
        seen.diagnostics
            .contains(&diagnostic(2, 3, "waiting on a same-Tick dependency cycle")),
        "{:?}",
        seen.diagnostics
    );
}

#[test]
fn a_halted_producer_writes_nothing_and_its_last_bang_fires_once_more() {
    // E4. A locked Equality takes no Turn and writes no clear, so the `**`
    // it left fires at its own Turn.
    let grid = Grid::with_shape(10, 4);
    let mut source = source_of(grid, &["", "  .=0101", "", "  !>007FC4"]);
    let (plan, seen) = tick(&mut source, 0);
    assert_eq!(plan.play_commands, [raw_c4()]);
    assert_eq!(seen.rows[2], "  **      ");

    type_at(&mut source, 0, 0, "***!");
    let (plan, seen) = tick(&mut source, 1);
    assert_eq!(plan.play_commands, [raw_c4()]);
    assert_eq!(
        seen.rows,
        ["  *!      ", "  .=0101  ", "          ", "  !>007FC4"]
    );
    assert_eq!(plan.locks, [grid.position(2, 1).expect("inside the Grid")]);
    quiet(&seen);

    let (plan, seen) = tick(&mut source, 2);
    assert_eq!(plan.play_commands, [raw_c4()]);
    assert_eq!(seen.rows[2], "  **      ");
    assert!(plan.locks.is_empty(), "{:?}", plan.locks);
    quiet(&seen);
}

#[test]
fn an_invalid_producer_writes_nothing_and_its_last_bang_fires_once_more() {
    // E5. An Equality whose operand is erased is invalid and writes nothing,
    // so the `**` it left fires once.
    let mut source = source_of(Grid::with_shape(8, 3), &[".=0101", "", "!>007FC4"]);
    let (plan, _) = tick(&mut source, 0);
    assert_eq!(plan.play_commands, [raw_c4()]);

    type_at(&mut source, 4, 0, "  ");
    let (plan, seen) = tick(&mut source, 1);
    assert_eq!(plan.play_commands, [raw_c4()]);
    assert_eq!(seen.rows, [".=01    ", "        ", "!>007FC4"]);

    let (plan, seen) = tick(&mut source, 2);
    assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
    assert_eq!(seen.rows, [".=01    ", "        ", "!>007FC4"]);
}

#[test]
fn a_mover_meets_a_bang_in_source_order() {
    // E6. `<<` follows the `**` in Source order and enters the Cell it
    // vacated.
    let mut source = source_of(Grid::with_shape(10, 2), &["**<<", "!>007FC4"]);
    let (plan, seen) = tick(&mut source, 0);
    assert_eq!(plan.play_commands, [raw_c4()]);
    assert_eq!(seen.rows, [" <<       ", "!>007FC4  "]);
    quiet(&seen);

    // `>>` precedes it, finds its Cells occupied and bangs, and the `**` then
    // fires. The `**` the mover left fires on the next Tick with no root
    // aligned with it.
    let mut source = source_of(Grid::with_shape(10, 2), &[">>**", "  !>007FC4"]);
    let (plan, seen) = tick(&mut source, 0);
    assert_eq!(plan.play_commands, [raw_c4()]);
    assert_eq!(seen.rows, ["**        ", "  !>007FC4"]);
    quiet(&seen);
    let (plan, seen) = tick(&mut source, 1);
    assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
    assert_eq!(seen.rows, ["          ", "  !>007FC4"]);
    quiet(&seen);
}

#[test]
fn a_reader_of_a_bang_waits_for_it_and_sees_what_its_turn_left() {
    // E7. The Copy reads the `**`'s Cells after its Turn: cleared, so it
    // copies blanks and nothing plays.
    let mut source = source_of(Grid::with_shape(8, 4), &["**", "=v", "", "!>007FC4"]);
    let (plan, seen) = tick(&mut source, 0);
    assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
    assert_eq!(seen.rows, ["        ", "=v      ", "        ", "!>007FC4"]);
    quiet(&seen);

    // Locked, the `**` stands, and the Copy carries it to Raw Play every Tick.
    let rows = [".=0101", "  *!", "  **", "  =v", "", "  !>007FC4"];
    let mut source = source_of(Grid::with_shape(10, 6), &rows);
    for at in 0..3 {
        let (plan, seen) = tick(&mut source, at);
        assert_eq!(plan.play_commands, [raw_c4()], "Tick {at}");
        assert_eq!(seen.rows[2], "  **      ");
        assert_eq!(seen.rows[4], "  **      ");
        quiet(&seen);
    }
}

#[test]
fn a_writer_over_a_typed_bang_suppresses_it() {
    // E8. The Addition's write is ordered before the `**` it covers and
    // suppresses it.
    let mut source = source_of(Grid::with_shape(8, 3), &[".+0102", "**", "!>007FC4"]);
    let (plan, seen) = tick(&mut source, 0);
    assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
    assert_eq!(seen.rows, [".+0102  ", "03      ", "!>007FC4"]);
    quiet(&seen);

    // A quiet Equality's clear suppresses it the same way.
    let mut source = source_of(Grid::with_shape(8, 3), &[".=0102", "**", "!>007FC4"]);
    let (plan, seen) = tick(&mut source, 0);
    assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
    assert_eq!(seen.rows, [".=0102  ", "        ", "!>007FC4"]);
    quiet(&seen);
}

#[test]
fn a_directional_bang_function_blocked_on_its_first_step_alternates() {
    // E9. `vv` is blocked by `00` and writes `**`, which re-activates `*v`
    // on the next Tick; `*v` emits into the Cells the `**` vacated.
    let rows: Vec<_> = observe_at(Grid::with_shape(4, 3), &["***v", "", "  00"], 0..5)
        .into_iter()
        .map(|tick| tick.rows)
        .collect();
    assert_eq!(rows[0], ["  *v", "  vv", "  00"]);
    assert_eq!(rows[1], ["  *v", "  **", "  00"]);
    assert_eq!(rows[2], ["  *v", "  vv", "  00"]);
    assert_eq!(rows[3], ["  *v", "  **", "  00"]);
    assert_eq!(rows[4], ["  *v", "  vv", "  00"]);
}

#[test]
fn a_typed_bang_plays_once_on_the_next_tick_and_is_cleared() {
    let mut source = source_of(Grid::with_shape(8, 2), &["**", "!>007FC4"]);
    let (plan, seen) = tick(&mut source, 0);
    assert_eq!(plan.play_commands, [raw_c4()]);
    assert_eq!(seen.rows, ["        ", "!>007FC4"]);
    let (plan, _) = tick(&mut source, 1);
    assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
}

#[test]
fn a_banging_producer_plays_in_its_tick_and_its_clear_suppresses_its_bang() {
    // Banging every Tick, the Equality plays once per Tick: each new `**`
    // is written over the last, which never fires.
    let mut source = source_of(Grid::with_shape(8, 3), &[".=0101", "", "!>007FC4"]);
    for at in 0..3 {
        let (plan, seen) = tick(&mut source, at);
        assert_eq!(plan.play_commands, [raw_c4()], "Tick {at}");
        assert_eq!(seen.rows[1], "**      ");
        quiet(&seen);
    }

    // Going quiet, its clear suppresses the `**` it left.
    type_at(&mut source, 5, 0, "2");
    let (plan, seen) = tick(&mut source, 3);
    assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
    assert_eq!(seen.rows[1], "        ");
    quiet(&seen);
}

#[test]
fn a_copy_chain_carries_a_bang_in_one_tick_and_plays_once() {
    // Each Copy writes over the `**` the last Tick left in its Output
    // Portal, so only the Bang carried this Tick plays.
    let rows = [".=0101", "", "=v", "", "=v", "", "!>007FC4"];
    let mut source = source_of(Grid::with_shape(8, 7), &rows);
    for at in 0..2 {
        let (plan, seen) = tick(&mut source, at);
        assert_eq!(plan.play_commands, [raw_c4()], "Tick {at}");
        assert_eq!(seen.rows[5], "**      ");
        quiet(&seen);
    }

    type_at(&mut source, 5, 0, "2");
    let (plan, seen) = tick(&mut source, 2);
    assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
    assert_eq!(seen.rows[5], "        ");
    quiet(&seen);
}

#[test]
fn a_lone_star_typed_beside_a_written_bang_fires_nothing_and_survives() {
    // Typed west of the Equality's `**`, the `*` makes the parser read a
    // `**` one Cell west and a stray `*`. The Equality's write covers part of
    // that `**` and suppresses it, so the Raw Play below it never plays and
    // the typed `*` stays where it was typed.
    let mut source = source_of(Grid::with_shape(10, 3), &["  .=0101", "", " !>007FC4"]);
    let (plan, _) = tick(&mut source, 0);
    assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);

    type_at(&mut source, 1, 1, "*");
    for at in 1..4 {
        let (plan, seen) = tick(&mut source, at);
        assert!(
            plan.play_commands.is_empty(),
            "Tick {at}: {:?}",
            plan.play_commands
        );
        assert_eq!(seen.rows[1], " ***      ", "Tick {at}");
    }
}

/// The operand-Bang fixture after Tick 0: Equality's `**` played Raw Play,
/// and the Addition the Copy wrote beside it makes that `**` its operand.
fn bang_in_an_operand() -> Source {
    let grid = Grid::with_shape(14, 4);
    let mut source = source_of(grid, &["  .=0101", "    !>007FC4", "=^", ".+"]);
    let first = source.execute(Tick::ZERO);
    assert_eq!(first.play_commands, [raw_c4()]);
    assert_eq!(rows_of(&source)[1], ".+**!>007FC4  ");
    // The Equality stops writing `**`; the Addition still holds it.
    type_at(&mut source, 2, 0, "      ");
    source
}

/// Frees the `**` the Addition held: the Copy, its input and the Addition go.
fn free_the_operand_bang(source: &mut Source) {
    type_at(source, 0, 1, "  ");
    type_at(source, 0, 2, "  ");
    type_at(source, 0, 3, "  ");
    assert_eq!(rows_of(source)[1], "  **!>007FC4  ");
}

#[test]
fn a_bang_in_an_operand_fires_exactly_once_when_freed() {
    for quiet_ticks in [0, 1, 3] {
        let mut source = bang_in_an_operand();
        for at in 1..=quiet_ticks {
            let (plan, _) = tick(&mut source, at);
            assert!(
                plan.play_commands.is_empty(),
                "quiet Tick {at}: {:?}",
                plan.play_commands
            );
            assert_eq!(rows_of(&source)[1], ".+**!>007FC4  ");
        }
        free_the_operand_bang(&mut source);
        let (plan, seen) = tick(&mut source, quiet_ticks + 1);
        assert_eq!(plan.play_commands, [raw_c4()], "after {quiet_ticks}");
        assert_eq!(seen.rows[1], "    !>007FC4  ");
        let (plan, _) = tick(&mut source, quiet_ticks + 2);
        assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
    }
}

#[test]
fn a_removed_writer_leaves_its_last_bang_to_fire_once_more() {
    let mut source = source_of(Grid::with_shape(8, 3), &[".=0101", "", "!>007FC4"]);
    let (plan, _) = tick(&mut source, 0);
    assert_eq!(plan.play_commands, [raw_c4()]);

    type_at(&mut source, 0, 0, "      ");
    let (plan, seen) = tick(&mut source, 1);
    assert_eq!(plan.play_commands, [raw_c4()]);
    assert_eq!(seen.rows, ["        ", "        ", "!>007FC4"]);
    let (plan, _) = tick(&mut source, 2);
    assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
}

#[test]
fn a_write_moved_after_it_banged_leaves_its_old_bang_to_fire_once_more() {
    // The old `**` at column 0 fires and plays the Raw Play below it; the
    // new one lands at column 4 with no root aligned with it.
    let mut source = source_of(Grid::with_shape(8, 3), &["@$0001**", "", "!>007FC4"]);
    let (plan, seen) = tick(&mut source, 0);
    assert_eq!(plan.play_commands, [raw_c4()]);
    assert_eq!(seen.rows[1], "**      ");

    type_at(&mut source, 2, 0, "04");
    let (plan, seen) = tick(&mut source, 1);
    assert_eq!(plan.play_commands, [raw_c4()]);
    assert_eq!(seen.rows[1], "    **  ");
    let (plan, seen) = tick(&mut source, 2);
    assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
    assert_eq!(seen.rows[1], "    **  ");
}

#[test]
fn a_partial_overwrite_suppresses_the_whole_bang_and_the_stray_star_is_no_bang() {
    let mut source = source_of(Grid::with_shape(8, 3), &[" .+0102", "**", "!>007FC4"]);
    for at in 0..2 {
        let (plan, seen) = tick(&mut source, at);
        assert!(
            plan.play_commands.is_empty(),
            "Tick {at}: {:?}",
            plan.play_commands
        );
        assert_eq!(seen.rows[1], "*03     ", "Tick {at}");
    }
}

#[test]
fn a_typed_bang_activates_an_emitter_that_emits_into_the_cells_it_vacated() {
    let mut source = source_of(Grid::with_shape(8, 3), &["*v", "**", "!>007FC4"]);
    let (plan, seen) = tick(&mut source, 0);
    assert_eq!(plan.play_commands, [raw_c4()]);
    assert_eq!(seen.rows, ["*v      ", "vv      ", "!>007FC4"]);
    quiet(&seen);
}

#[test]
fn a_typed_bang_activates_the_roots_on_all_four_sides() {
    // North, east and south play; the operandless `*v` west of it emits.
    let mut source = source_of(
        Grid::with_shape(12, 3),
        &["  !>007FC4", "*v**!>007FC5", "  !>007FC6"],
    );
    let (plan, seen) = tick(&mut source, 0);
    let mut played = plan.play_commands.clone();
    played.sort_by_key(|command| format!("{command:?}"));
    let mut expected = vec![raw_c4(), raw(0, 0x7F, 0x48), raw(0, 0x7F, 0x54)];
    expected.sort_by_key(|command| format!("{command:?}"));
    assert_eq!(played, expected);
    assert_eq!(seen.rows, ["  !>007FC4  ", "*v  !>007FC5", "vv!>007FC6  "]);
    quiet(&seen);
}

#[test]
fn a_blocked_mover_activates_nothing_and_its_bang_plays_on_the_next_tick() {
    // Blocked by a root: the root does not play in the collision Tick.
    let mut source = source_of(Grid::with_shape(10, 1), &[">>!>007FC4"]);
    let (plan, seen) = tick(&mut source, 0);
    assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
    assert_eq!(seen.rows, ["**!>007FC4"]);
    quiet(&seen);
    let (plan, seen) = tick(&mut source, 1);
    assert_eq!(plan.play_commands, [raw_c4()]);
    assert_eq!(seen.rows, ["  !>007FC4"]);

    // Blocked by a Comment.
    let mut source = source_of(Grid::with_shape(8, 2), &[">>||", "!>007FC4"]);
    let (plan, seen) = tick(&mut source, 0);
    assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
    assert_eq!(seen.rows[0], "**||    ");
    let (plan, _) = tick(&mut source, 1);
    assert_eq!(plan.play_commands, [raw_c4()]);

    // Blocked by the Grid edge.
    let mut source = source_of(Grid::with_shape(8, 2), &["      >>", "      !>"]);
    let (_, seen) = tick(&mut source, 0);
    assert_eq!(seen.rows[0], "      **");
    let (_, seen) = tick(&mut source, 1);
    assert_eq!(seen.rows[0], "        ");
}

#[test]
fn a_mover_meets_a_bang_activated_root_in_source_order() {
    // The `**` activates the Raw Play its mover contacts. The root stays in
    // its Cells through its Turn, so whichever of the two Source order puts
    // first, the root plays and the mover is blocked and leaves `**`, which
    // plays the root again on the next Tick. The mover precedes the root in
    // the first Grid and follows it in the second.
    let cases: [(&[&str], [&str; 3]); 2] = [
        (
            &["  **", ">>!>007FC4"],
            ["          ", "**!>007FC4", "          "],
        ),
        (
            &["  **", "  !>007FC4", "  ^^"],
            ["          ", "  !>007FC4", "  **      "],
        ),
    ];
    for (rows, blocked) in cases {
        let mut source = source_of(Grid::with_shape(10, 3), rows);
        let (plan, seen) = tick(&mut source, 0);
        assert_eq!(plan.play_commands, [raw_c4()], "{rows:?}");
        assert_eq!(seen.rows, blocked, "{rows:?}");
        quiet(&seen);
        let (plan, seen) = tick(&mut source, 1);
        assert_eq!(plan.play_commands, [raw_c4()], "{rows:?}");
        assert_eq!(seen.rows, ["          ", "  !>007FC4", "          "]);
        quiet(&seen);
    }
}

#[test]
fn a_bang_in_an_increments_own_output_portal_is_a_cycle() {
    // The Increment writes over the `**` and reads the Cells it reserves,
    // as a Function writing what it reads does.
    let seen = observe_at(Grid::with_shape(8, 2), &["~+0108", "**"], [0]).remove(0);
    assert_eq!(seen.rows, ["~+0108  ", "**      "]);
    assert!(
        seen.diagnostics
            .iter()
            .any(|(_, _, message)| message == "same-Tick dependency cycle"),
        "{:?}",
        seen.diagnostics
    );
}

#[test]
fn a_bang_left_by_a_producer_a_cycle_stops_stands_and_waits() {
    // The Equality reads, through two Copies, the `**` the Increment's
    // cycle holds, so it waits on that cycle and writes nothing. The `**`
    // it left stands, a consumer of a stopped writer, every Tick.
    let rows = ["~+0108", "**=>01", "    =v", ".=0101", "**", "!>007FC4"];
    for seen in observe_at(Grid::with_shape(8, 6), &rows, 0..2) {
        assert_eq!(seen.rows[4], "**      ");
        assert!(
            seen.diagnostics
                .contains(&diagnostic(0, 4, "waiting on a same-Tick dependency cycle")),
            "{:?}",
            seen.diagnostics
        );
    }
}

#[test]
fn a_copy_carrying_a_function_spelling_onto_a_bang_is_a_refused_replacement() {
    let seen = observe_at(Grid::with_shape(4, 3), &["*v", "=v", "**"], [0]).remove(0);
    assert_eq!(seen.rows, ["*v  ", "=v  ", "    "]);
    assert!(
        seen.diagnostics
            .iter()
            .any(|(x, y, message)| (*x, *y) == (0, 1)
                && message.starts_with("Function replacement changes")),
        "{:?}",
        seen.diagnostics
    );
}

#[test]
fn identical_snapshots_at_the_same_tick_plan_the_same_tick() {
    // A Source holding the `**` an Equality wrote, and a fresh Source typed
    // with the same Cells, plan the same Tick: the `**` is suppressed by the
    // Equality's next write in both.
    let rows = [".=0101", "", "!>007FC4"];
    let mut live = source_of(Grid::with_shape(8, 3), &rows);
    live.execute(Tick::ZERO);
    let mut typed = source_of(Grid::with_shape(8, 3), &[".=0101", "**", "!>007FC4"]);
    assert_eq!(rows_of(&live), rows_of(&typed));
    let live_plan = live.execute(Tick::new(1));
    let typed_plan = typed.execute(Tick::new(1));
    assert_eq!(live_plan.play_commands, typed_plan.play_commands);
    let writes = |plan: &TickPlan| {
        plan.writes
            .iter()
            .map(|write| (write.cell.get(), write.content))
            .collect::<Vec<_>>()
    };
    assert_eq!(writes(&live_plan), writes(&typed_plan));
    assert_eq!(rows_of(&live), rows_of(&typed));
}

// Ticket 02: a producer that takes no Turn writes nothing, so the `**` it
// left on the Tick before fires once at its own Turn.

/// The three Functions that answer only Bang or a clear, each
/// spelled to Bang on every Tick until an edit says otherwise.
const BANGING: [&str; 3] = [".=0101", "~*0101", "~%0101"];

/// The Play Commands of Ticks 0 and 1 of `rows`, with `edit` typed between
/// them, and what Tick 1 left.
fn bang_then(rows: &[&str], edits: &[(usize, usize, &str)]) -> (usize, usize, Observed) {
    let width = rows.iter().map(|row| row.len()).max().unwrap_or(0) + 2;
    let mut source = source_of(Grid::with_shape(width, rows.len()), rows);
    let (first, _) = tick(&mut source, 0);
    for (x, y, text) in edits {
        type_at(&mut source, *x, *y, text);
    }
    let (second, seen) = tick(&mut source, 1);
    (first.play_commands.len(), second.play_commands.len(), seen)
}

#[test]
fn a_producer_that_takes_no_turn_leaves_its_last_bang_to_fire_once() {
    for producer in BANGING {
        let direct = ["", &format!("  {producer}"), "", "  !>007FC4"];
        let nested = ["", &format!("  .+{producer}01"), "", "    !>007FC4"];
        let mut cases = vec![
            ("Halt-locked", &direct, vec![(0, 0, "***!")]),
            ("a blanked operand", &direct, vec![(6, 1, "  ")]),
            ("nested in a locked root", &nested, vec![(0, 0, "***!")]),
        ];
        if producer != ".=0101" {
            // A zero modulus or a zero step count fails evaluation. Equality
            // answers every pair of values, so it has no such case.
            cases.push(("failing evaluation", &direct, vec![(6, 1, "00")]));
        }
        for (case, rows, edits) in cases {
            let (first, second, seen) = bang_then(&rows[..], &edits);
            assert_eq!((first, second), (1, 1), "{producer}, {case}");
            let output = if rows == &nested { 4 } else { 2 };
            assert_eq!(
                &seen.rows[2][output..output + 2],
                "  ",
                "{producer}, {case}"
            );
        }
    }
}

#[test]
fn a_producer_answering_a_clear_suppresses_its_last_bang() {
    for producer in BANGING {
        // `.=0102` is unequal; `~*0102` and `~%0102` Bang on even Ticks.
        let rows = ["", &format!("  {producer}"), "", "  !>007FC4"];
        let (first, second, seen) = bang_then(&rows, &[(6, 1, "02")]);
        assert_eq!((first, second), (1, 0), "{producer}");
        assert_eq!(seen.rows[2].trim(), "", "{producer}");
        quiet(&seen);
    }
}

#[test]
fn an_overwritten_producer_writes_no_clear_unless_the_overwrite_covers_its_bang() {
    // `.+0102` writes `03` over the Equality's anchor before its Turn, so
    // the Equality is suppressed and writes nothing, not even a clear, and
    // its `**` fires.
    let rows = ["", "  .=0101", "", "  !>007FC4"];
    let (first, second, seen) = bang_then(&rows, &[(2, 0, ".+0102")]);
    assert_eq!((first, second), (1, 1));
    assert_eq!(seen.rows[1], "  030101    ");
    assert_eq!(seen.rows[2].trim(), "");

    // Retyped as `.+0102` in place, the writer's `03` covers the `**`.
    let (first, second, seen) = bang_then(&rows, &[(2, 1, ".+0102")]);
    assert_eq!((first, second), (1, 0));
    assert_eq!(seen.rows[2], "  03        ");
}

#[test]
fn a_write_refused_outside_the_grid_suppresses_nothing() {
    let rows = ["@$0001**", "", "!>007FC4"];
    let (first, second, seen) = bang_then(&rows, &[(2, 0, "FF")]);
    assert_eq!((first, second), (1, 1));
    assert_eq!(seen.rows[1].trim(), "");
    assert_eq!(
        seen.diagnostics,
        [diagnostic(0, 0, "result \"**\" falls outside the Grid")]
    );
}

#[test]
fn a_bang_whose_producer_a_cycle_stops_stands_and_plays_once_the_cycle_is_gone() {
    // The Equality reads, through `=>` and `=v`, the `01` an Increment and
    // the `**` typed over it in its Output Portal then close a cycle over.
    let rows = ["", "01=>", "    =v", ".=0101", "", "!>007FC4"];
    let mut source = source_of(Grid::with_shape(8, 6), &rows);
    let (plan, seen) = tick(&mut source, 0);
    assert_eq!(plan.play_commands, [raw_c4()]);
    assert_eq!(seen.rows[4], "**      ");

    type_at(&mut source, 0, 0, "~+0108");
    type_at(&mut source, 0, 1, "**");
    for at in 1..3 {
        let (plan, seen) = tick(&mut source, at);
        assert!(plan.play_commands.is_empty(), "Tick {at}");
        assert_eq!(seen.rows[4], "**      ", "Tick {at}");
        assert!(
            seen.diagnostics
                .contains(&diagnostic(0, 4, "waiting on a same-Tick dependency cycle")),
            "Tick {at}: {:?}",
            seen.diagnostics
        );
    }

    // With the Increment gone, the `**` it held fires and `=>` carries the
    // blanks it left, so the Equality is invalid, writes nothing, and the
    // `**` it left fires once.
    type_at(&mut source, 0, 0, "      ");
    let (plan, seen) = tick(&mut source, 3);
    assert_eq!(plan.play_commands, [raw_c4()]);
    assert_eq!(seen.rows[4], "        ");
}

#[test]
fn a_dynamic_writer_behind_another_comes_after_its_last_bang() {
    // The first Write reads the `**`'s Cells, so the `**` goes before it, and
    // the second Write goes after the first in the writers-first chain. The
    // `**` therefore fires before the second Write rewrites it: the first
    // Write reads the Cells it cleared, an invalid value, and writes nothing.
    let rows = ["!>007FC4", "", "@$0A00&$0001", "", "@$0001**"];
    let mut source = source_of(Grid::with_shape(14, 5), &rows);
    for at in 0..3 {
        let (plan, seen) = tick(&mut source, at);
        assert_eq!(plan.play_commands, [raw_c4()], "Tick {at}");
        assert_eq!(seen.rows[0], "!>007FC4      ", "Tick {at}");
        assert_eq!(seen.rows[1], "**            ", "Tick {at}");
    }

    // With the Writes the other way round, the rewriting Write goes first and
    // its `**` over the old one suppresses it: the reading Write finds `**`.
    let rows = ["!>007FC4", "", "@$0001**", "", "@$0A00&$0001"];
    let mut source = source_of(Grid::with_shape(14, 5), &rows);
    let (_, seen) = tick(&mut source, 1);
    assert_eq!(seen.rows[0], "!>007FC4  **  ");
}

// Ticket 04: a Halt above a `**` locks it only when a Bang that executed
// activated the Halt before its Turn.

#[test]
fn a_halt_whose_activator_is_suppressed_or_locked_misses_the_bang_it_sits_on() {
    // `.+0102` writes over the Equality before its Turn, so the Equality is
    // suppressed and answers no Bang.
    let suppressed = [".+0102", ".=0101", "  *!", "  **", "  !>007FC4"];
    // The typed `**` activates the upper Halt, which locks the Equality.
    let locked = ["***!", "  .=0101", "    *!", "    **", "    !>007FC4"];
    for (case, rows, halt) in [
        ("suppressed", &suppressed, (2, 2)),
        ("locked", &locked, (4, 2)),
    ] {
        let mut source = source_of(Grid::with_shape(12, 5), &rows[..]);
        let (plan, seen) = tick(&mut source, 0);
        assert_eq!(plan.play_commands, [raw_c4()], "{case}");
        assert_eq!(seen.rows[3].trim(), "", "{case}");
        assert!(
            seen.diagnostics
                .contains(&diagnostic(halt.0, halt.1, MISSED_BANG)),
            "{case}: {:?}",
            seen.diagnostics
        );
    }
}

#[test]
fn a_halt_misses_a_dynamic_bang_landing_after_its_turn() {
    // The Write nested in the lower Raw Play takes its Turn once the `**`
    // has activated that Raw Play, after the Halt's, so its `**` west of the
    // Halt is missed as the `**` the Halt sits on is.
    let rows = ["", "", "    *!", "    **!>007FC4", "    !>@$0202**7FC4"];
    let mut source = source_of(Grid::with_shape(18, 5), &rows);
    let (plan, seen) = tick(&mut source, 0);
    assert_eq!(plan.play_commands, [raw_c4()]);
    assert!(plan.locks.is_empty(), "{:?}", plan.locks);
    assert_eq!(
        seen.rows[2..4],
        ["  ***!            ", "      !>007FC4    "]
    );
    assert_eq!(
        seen.diagnostics,
        [
            diagnostic(4, 2, MISSED_BANG),
            diagnostic(4, 2, MISSED_BANG),
            diagnostic(4, 4, "expected a number, found \"**\""),
        ]
    );
}

// Ticket 03: a mover meets a Bang Function in Source order.

#[test]
fn a_vertical_mover_meets_a_bang_in_source_order() {
    // `vv` precedes the `**` in Source order, finds its Cells occupied and
    // writes `**` over (2, 0); the `**` at (2, 1) then fires and plays.
    let mut source = source_of(Grid::with_shape(10, 3), &["  vv", "  **", "  !>007FC4"]);
    let (plan, seen) = tick(&mut source, 0);
    assert_eq!(plan.play_commands, [raw_c4()]);
    assert_eq!(seen.rows, ["  **      ", "          ", "  !>007FC4"]);
    quiet(&seen);
    let (plan, seen) = tick(&mut source, 1);
    assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
    assert_eq!(seen.rows[0], "          ");

    // `^^` follows it, so the `**` fires first and `^^` enters the Cells it
    // cleared.
    let mut source = source_of(Grid::with_shape(10, 3), &["  !>007FC4", "  **", "  ^^"]);
    let (plan, seen) = tick(&mut source, 0);
    assert_eq!(plan.play_commands, [raw_c4()]);
    assert_eq!(seen.rows, ["  !>007FC4", "  ^^      ", "          "]);
    quiet(&seen);
}

#[test]
fn a_writer_covering_a_bang_goes_before_a_mover_entering_its_cells() {
    // The Addition's `03` suppresses the `**` before its Turn, and a mover
    // entering those Cells after it finds `03` and is blocked.
    for (rows, blocked) in [
        ([".+0102", "**<<", "!>007FC4"], "03**      "),
        (["  .+0102", ">>**", "  !>007FC4"], "**03      "),
    ] {
        let mut source = source_of(Grid::with_shape(10, 3), &rows);
        let (plan, seen) = tick(&mut source, 0);
        assert!(plan.play_commands.is_empty(), "{rows:?}");
        assert_eq!(seen.rows[1], blocked, "{rows:?}");
        quiet(&seen);
    }
}

#[test]
fn a_locked_bang_blocks_a_mover_from_every_side_it_can_be_reached() {
    // The Halt above the `**` holds its north side, so movers reach it from
    // the east, the west and the south. Each is blocked and writes `**`,
    // and the locked `**` stands.
    for (rows, mover_row, blocked) in [
        (&[".=0101", "  *!", "  **<<"][..], 2, "  ****    "),
        (&[".=0101", "  *!", ">>**"][..], 2, "****      "),
        (&[".=0101", "  *!", "  **", "  ^^"][..], 3, "  **      "),
    ] {
        let mut source = source_of(Grid::with_shape(10, rows.len()), rows);
        let (plan, seen) = tick(&mut source, 0);
        assert!(plan.play_commands.is_empty(), "{rows:?}");
        assert_eq!(&seen.rows[2][2..4], "**", "{rows:?}");
        assert_eq!(seen.rows[mover_row], blocked, "{rows:?}");
        quiet(&seen);
    }
}
