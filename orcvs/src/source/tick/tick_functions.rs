//! The Tick Functions: Clock, Delay, Euclidean, Increment, Interpolation and
//! Random, read across Ticks and Anchors.

use crate::grid::Grid;
use crate::source::Cells;
use crate::source::language_map::LanguageMap;

use super::Tick;
use super::fixture::{
    carried_source, cell, interpreted, messages, planned, rows_of, seeded_source, snapshot,
    tick_by_tick,
};
use super::observed::raw;

#[test]
fn the_interpreter_is_handed_the_shared_tick_and_each_roots_own_anchor() {
    // The explicit Tick inputs are only as good as something watching the
    // thread from the Playback Engine to `Interpreter::execute_function`.
    // Severing it — passing a fixed Tick or a fixed anchor at the call site
    // instead of this root's own — fails here rather than passing
    // unnoticed.
    //
    // The expected anchors are literals, so the assertion cannot be
    // satisfied by the code under test, and they are asymmetric so a
    // transposed column and row is visible. The Tick is not `Tick::ZERO`,
    // so a hardcoded first Tick is visible too.
    //
    // The Tick half is also asserted through results by the test below,
    // which watches what three Tick Functions write rather than what they
    // were handed. These roots read neither input, so both halves are read
    // off the execution states here.
    let grid = Grid::with_shape(20, 3);
    let bytes = snapshot(grid, &[".+0102 .+0304", "          .-0504", ""]);
    let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));
    let tick = Tick::new(11);

    let (_, states) = super::plan(grid, Cells::of(bytes.as_bytes()), &map, tick);
    let interpreted = interpreted(&states);

    assert_eq!(interpreted.len(), 3, "each of the three roots is evaluated");
    assert!(
        interpreted.iter().all(|inputs| inputs.tick() == tick),
        "one Tick is given to the whole Source Snapshot"
    );
    assert_eq!(
        interpreted
            .iter()
            .map(|inputs| (inputs.anchor().column(), inputs.anchor().row()))
            .collect::<Vec<_>>(),
        vec![(0, 0), (7, 0), (10, 1)],
        "each root is told its own anchor"
    );
}

#[test]
fn the_tick_functions_answer_about_the_absolute_tick_they_are_planned_at() {
    // The Tick threaded from the Playback Engine changes what a Source
    // Snapshot writes, so severing it fails here rather than only in the
    // seam test above, which watches the inputs rather than the answers.
    //
    // One Grid, three roots, two Ticks. Each root's result lands in the
    // Cell pair directly south of its anchor, so the expected writes are
    // literal Cell indices and characters rather than anything the code
    // under test could satisfy by agreeing with itself.
    let grid = Grid::with_shape(24, 2);
    let bytes = snapshot(grid, &["~.0304 ~*0202 ~%0304", ""]);
    let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));

    // Tick 1. The Clock is still in its first step of three, so it writes
    // `00`; the Delay's cycle is 2 * 2 = 4 Ticks and 1 is not a multiple of
    // it; the Euclidean's `X.XX` over four steps has no onset at step 1.
    // Both answer a Clear and clear their pair.
    let (plan, _) = super::plan(grid, Cells::of(bytes.as_bytes()), &map, Tick::new(1));
    let plan = plan.into_plan();

    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(
        planned(&plan),
        vec![
            (24, '0'),
            (25, '0'),
            (31, ' '),
            (32, ' '),
            (38, ' '),
            (39, ' ')
        ]
    );

    // Tick 4. The Clock has counted one whole step of three; the Delay is
    // on a multiple of its cycle; and step 0 of `X.XX` is an onset. A
    // hardcoded first Tick would write `00` here and a hardcoded Tick of
    // its own would move all three at once, so the pair of assertions is
    // what makes this about the Tick rather than about the formulas.
    let (plan, _) = super::plan(grid, Cells::of(bytes.as_bytes()), &map, Tick::new(4));
    let plan = plan.into_plan();

    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(
        planned(&plan),
        vec![
            (24, '0'),
            (25, '1'),
            (31, '*'),
            (32, '*'),
            (38, '*'),
            (39, '*')
        ]
    );
}

#[test]
fn a_tick_function_with_no_cycle_diagnoses_and_writes_nothing() {
    // A cycle with a zero factor is refused rather than invented, and the
    // diagnostic has to reach the Source through the ordinary
    // Tick Plan: it names the Function's spelling and the operand role, so
    // the console can say which of the two Cell pairs to edit.
    let grid = Grid::with_shape(16, 2);
    let bytes = snapshot(grid, &["~*0300 ~%0400", ""]);
    let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));

    let (plan, _) = super::plan(grid, Cells::of(bytes.as_bytes()), &map, Tick::new(3));
    let plan = plan.into_plan();

    assert!(plan.writes.is_empty(), "{:?}", plan.writes);
    assert_eq!(
        plan.diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.clone())
            .collect::<Vec<_>>(),
        vec![
            "~* cannot count a cycle with a zero modulus".to_string(),
            "~% cannot count a cycle with a zero step count".to_string(),
        ]
    );
}

#[test]
fn a_pulse_activates_an_aligned_root_only_on_the_ticks_it_bangs() {
    // Delay and Euclidean declare `can_emit_bang`, and that declaration is
    // what `dependencies::derive` reads to decide which roots can
    // supply activation. Nothing else asserts the edge is built:
    // the tests above watch the two Cells a pulse writes, which a Function
    // that Banged into no activation edge would still satisfy while the
    // neighbouring terminal fell silent with no diagnostic anywhere.
    //
    // So the assertion is the play, not the write, and it is made at two
    // Ticks per Function. A hardcoded edge would fire the terminal at both
    // and a missing one at neither, so the pair is what makes this about
    // the pulse rather than about scheduling in general.
    for (spelling, banging, silent) in [("~*0202", 4, 1), ("~%0304", 4, 1)] {
        let grid = Grid::with_shape(16, 6);
        let bytes = snapshot(grid, &["", "!>007FC4", "", "", "", spelling]);
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
            Tick::new(banging),
            &destinations,
        );
        let plan = plan.into_plan();

        assert_eq!(
            plan.play_commands,
            vec![raw(0, 0x7F, 60)],
            "{spelling} at Tick {banging}, diagnostics: {:?}",
            plan.diagnostics
        );

        let (plan, _) = super::plan_carrying(
            grid,
            Cells::of(bytes.as_bytes()),
            &map,
            Tick::new(silent),
            &destinations,
        );
        let plan = plan.into_plan();

        assert!(
            plan.play_commands.is_empty(),
            "{spelling} at Tick {silent} played {:?}",
            plan.play_commands
        );
    }
}

#[test]
fn an_empty_portal_initialises_increment_and_interpolation_as_zero() {
    // The first previous: two space Cells are Number `00`, so the
    // first Tick of an unused Portal is the step itself for Increment and
    // the first step toward the target for Interpolation. The expected
    // writes are literal Cells, so a body that treated empty as occupied
    // would diagnose here rather than write.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(6, 2), &["~+0104", ""], 1);

    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
    assert_eq!(grids[0], ["~+0104", "01    "]);

    let (plans, grids, _) = tick_by_tick(Grid::with_shape(6, 2), &["~>0210", ""], 1);

    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
    assert_eq!(grids[0], ["~>0210", "02    "]);
}

#[test]
fn increment_and_interpolation_advance_across_two_ticks() {
    // Cross-Tick state is the Portal Cells and nothing else: the second
    // Tick reads what the first wrote. Increment walks `01 02` from empty;
    // Interpolation walks `04 08` toward `10`. A hidden counter that
    // ignored the Portal would still pass the first Tick and fail the
    // second once the Cells were live-edited, which the next test does.
    let (_, grids, _) = tick_by_tick(Grid::with_shape(6, 2), &["~+0104", ""], 2);
    assert_eq!(grids[0], ["~+0104", "01    "]);
    assert_eq!(grids[1], ["~+0104", "02    "]);

    let (_, grids, _) = tick_by_tick(Grid::with_shape(6, 2), &["~>0410", ""], 2);
    assert_eq!(grids[0], ["~>0410", "04    "]);
    assert_eq!(grids[1], ["~>0410", "08    "]);
}

#[test]
fn increment_reads_a_live_edited_previous_from_working_source() {
    // Live Editing is a Source Snapshot change, not a Function-local
    // memory: after Tick 0 writes `01`, the Portal is set to `05` and
    // Tick 1 must wrap from that Number. A hidden previous that remembered
    // `01` would write `02` here.
    let grid = Grid::with_shape(6, 2);
    let mut source = seeded_source(grid, &["~+0108", ""]);
    source.execute(Tick::ZERO);
    assert_eq!(rows_of(grid, &source)[1], "01    ");

    source.set(cell(grid, 6), "0").unwrap();
    source.set(cell(grid, 7), "5").unwrap();
    source.execute(Tick::new(1));

    assert_eq!(rows_of(grid, &source)[1], "06    ");
}

#[test]
fn increment_and_interpolation_keep_their_state_across_an_unwritten_operand() {
    // Their state is the Cells they wrote. On the Tick an operand is
    // unwritten they are invalid and write nothing, so those Cells stand,
    // and once the operand is written again they continue from them.
    for (function, before, after) in [("~+0108", "01", "02"), ("~>0410", "04", "08")] {
        let grid = Grid::with_shape(6, 2);
        let mut source = seeded_source(grid, &[function, ""]);
        source.execute(Tick::ZERO);
        assert_eq!(rows_of(grid, &source)[1], format!("{before}    "));

        source.set(cell(grid, 2), " ").unwrap();
        source.set(cell(grid, 3), " ").unwrap();
        assert!(
            source
                .language_map()
                .diagnostics()
                .any(|diagnostic| diagnostic.start() == 0),
            "{function}"
        );
        source.execute(Tick::new(1));
        assert_eq!(
            rows_of(grid, &source)[1],
            format!("{before}    "),
            "{function}"
        );

        source.set(cell(grid, 2), &function[2..3]).unwrap();
        source.set(cell(grid, 3), &function[3..4]).unwrap();
        source.execute(Tick::new(2));
        assert_eq!(
            rows_of(grid, &source)[1],
            format!("{after}    "),
            "{function}"
        );
    }
}

#[test]
fn feedback_reads_a_portal_write_from_an_earlier_turn() {
    // The carried output fixture places Add's answer at the feedback
    // input. The later Turn must read `05`, not the empty Snapshot Cells.
    for (function, expected) in [("~+0108", "06"), ("~>0208", "07")] {
        let grid = Grid::with_shape(14, 2);
        let row = format!(".+0203  {function}");
        let (plan, source) = carried_source(grid, &[&row, ""], &[(0, 22), (8, 22)]);
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(rows_of(grid, &source)[1], format!("        {expected}    "));
    }
}

#[test]
fn increment_writes_its_current_encoding_and_leaves_a_stale_tail() {
    // An ordinary result writes only its current encoding and
    // never clears a stale tail outside that Span. The Portal already
    // holds `00ABCD`; Increment writes `01` and leaves `ABCD`.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 2), &["~+0104", "00ABCD"], 1);

    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
    assert_eq!(grids[0], ["~+0104  ", "01ABCD  "]);
}

#[test]
fn a_previous_that_is_not_a_number_diagnoses_and_writes_nothing() {
    // G4 is a Note spelling that is not uppercase hex, so it cannot be
    // read as a Number; `.+` is a Function; `0X` is invalid hex; `0 `
    // is a truncated pair. Each diagnoses rather than converting, and
    // none of them writes.
    for (portal, _) in [
        ("G4    ", "Note"),
        (".+    ", "Function"),
        ("0X    ", "invalid hex"),
        ("0     ", "truncated pair"),
    ] {
        let (plans, grids, _) = tick_by_tick(Grid::with_shape(6, 2), &["~+0104", portal], 1);
        assert_eq!(
            messages(&plans[0]),
            ["~+ cannot read a previous value that is not a Number"],
            "{portal:?}"
        );
        assert!(
            plans[0].writes.is_empty(),
            "{portal:?} wrote {:?}",
            plans[0].writes
        );
        assert_eq!(grids[0][1], portal, "{portal:?} must be left as written");
    }
}

#[test]
fn a_last_row_increment_diagnoses_rather_than_clamping_its_portal() {
    // A root in the last row resolves no Portal. Clamping would invent a
    // previous of `00` and write nowhere; diagnosing names the missing
    // read instead.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(6, 1), &["~+0104"], 1);

    assert_eq!(
        messages(&plans[0]),
        ["~+ cannot read a previous value that is not a Number"]
    );
    assert!(plans[0].writes.is_empty(), "{:?}", plans[0].writes);
    assert_eq!(grids[0], ["~+0104"]);
}

#[test]
fn two_randoms_in_one_expression_write_different_numbers() {
    // Two roots, same operands, different anchors. Random's streams are a
    // function of Position, so `~?010010` at column 0 writes `02`
    // and the one at column 9 writes `00`. A body that seeded both at
    // the Expression — or at a shared origin — would write `02` twice.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(18, 2), &["~?010010 ~?010010", ""], 1);

    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
    assert_eq!(grids[0], ["~?010010 ~?010010 ", "02       00       "]);
}

#[test]
fn a_nested_random_uses_its_own_anchor_not_the_expression_root() {
    // `.+~?010010~?010010` is the case that tells root granularity from
    // per-Function anchors. Each Random is its own Computation with its
    // own `node.anchor`: the left sits at column 2 and draws `0A`, the
    // right at column 10 and draws `0F`. Their sum is `19`, and each
    // draw is written under its own anchor. Seeding both at the Add's
    // origin would draw `02` twice and write `04`.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(18, 2), &[".+~?010010~?010010", ""], 1);

    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
    assert_eq!(grids[0], [".+~?010010~?010010", "190A      0F      "]);
}

#[test]
fn moving_a_random_changes_its_stream_and_identical_inputs_reproduce_it() {
    // Column 0 draws `02`; shifting the same Function two Cells draws
    // `0A`. Planning the origin Grid a second time at the same Tick
    // writes `02` again, which is what "identical inputs reproduce"
    // means when there is no hidden activation count.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(10, 2), &["~?010010", ""], 1);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
    assert_eq!(grids[0], ["~?010010  ", "02        "]);

    let (plans, grids, _) = tick_by_tick(Grid::with_shape(10, 2), &["  ~?010010", ""], 1);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
    assert_eq!(grids[0], ["  ~?010010", "  0A      "]);

    let grid = Grid::with_shape(10, 2);
    let bytes = snapshot(grid, &["~?010010", ""]);
    let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));
    let first = super::plan(grid, Cells::of(bytes.as_bytes()), &map, Tick::ZERO);
    let second = super::plan(grid, Cells::of(bytes.as_bytes()), &map, Tick::ZERO);
    assert_eq!(first.0.plan().writes, second.0.plan().writes);
}
