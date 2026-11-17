//! A Chord Play driven from Source: a Clock steps nested Reads and Tracks
//! through its operands Tick after Tick, and a Portal-written operand means
//! what a Return means.
//!
//! Equality at (0, 0) Bangs the Play at (2, 1) on every Tick, with these
//! exceptions: the tracker, sequencer and chord tracker tests Bang each Play
//! with an Every `~*0101` north of it, and the Portal-written operand test
//! puts Equality at (0, 1) and the Play at (2, 2), beneath the Read on row 1
//! that writes the operand.

use lang::{PlayCommand, Tick};

use super::observed::{first_tick, quiet_plan, raw, rows_of, source_of, timed};
use crate::grid::Grid;
use crate::source::{Source, file};

/// The Play Commands Tick `tick` of `source` plans, asserting it diagnoses
/// nothing.
fn played(source: &mut Source, tick: u64) -> Vec<PlayCommand> {
    let plan = source.execute(Tick::new(tick));
    quiet_plan(&plan);
    plan.play_commands
}

/// The Play Commands of `count` consecutive Ticks of `rows`, each of them
/// diagnosing nothing.
fn run(grid: Grid, rows: &[&str], count: u64) -> Vec<Vec<PlayCommand>> {
    let mut source = source_of(grid, rows);
    (0..count).map(|tick| played(&mut source, tick)).collect()
}

#[test]
fn a_clock_steps_nested_reads_through_all_six_operands_after_channel() {
    // Each operand is a 12-Cell `&v.+02~.0102`, so every Read's `n` slot, on
    // row 1, is two rows north of its column's step 0 on row 3. The Comment
    // keeps the steps from being read as Expressions.
    let rows = [
        ".=0101",
        "  !#01&v.+02~.0102&v.+02~.0102&v.+02~.0102&v.+02~.0102&v.+02~.0102&v.+02~.0102",
        "",
        "      ||7F          F4          mi          CL          00          03||",
        "      ||40          C4          M7          OP          01          02||",
    ];

    let step_0 = vec![
        timed(1, 0x7F, 0x41, 3),
        timed(1, 0x7F, 0x44, 3),
        timed(1, 0x7F, 0x48, 3),
    ];
    // C4 M7 with `OP` and one inversion is E4 B4 G5 C6.
    let step_1 = vec![
        timed(1, 0x40, 0x40, 2),
        timed(1, 0x40, 0x47, 2),
        timed(1, 0x40, 0x4F, 2),
        timed(1, 0x40, 0x54, 2),
    ];
    let mut source = source_of(Grid::with_shape(80, 5), &rows);
    assert_eq!(played(&mut source, 0), step_0);
    assert_eq!(played(&mut source, 1), step_1);

    // Row 2 holds each Function's Output Portal write: the Read's answer
    // at its anchor, the Add's sum and the Clock's count over theirs.
    assert_eq!(
        rows_of(&source)[2].trim_end(),
        "      4003  01    C403  01    M703  01    OP03  01    0103  01    0203  01",
    );

    assert_eq!(played(&mut source, 2), step_0, "the Clock wraps to step 0");
}

/// Every voice of the shipped tracker moved to the nested form, and how many
/// voices moved: its Add and Clock leave the row above the Play for the
/// Read's `n` operand, the Read keeps its column, and the pattern rows are
/// untouched.
fn nested_tracker(shipped: &str) -> (Vec<String>, usize) {
    let mut rows: Vec<String> = shipped.lines().map(str::to_owned).collect();
    let mut voices = 0;
    for y in (0..rows.len())
        .filter(|&y| rows[y].contains("~*0101"))
        .collect::<Vec<_>>()
    {
        while let Some(add) = rows[y].find(".+") {
            let functions = rows[y][add..add + 10].to_owned();
            rows[y].replace_range(add..add + 10, &" ".repeat(10));
            let play = &mut rows[y + 1];
            assert_eq!(&play[add - 2..add + 4], "&v  01", "row {y}, column {add}");
            // The nested form takes eight more Cells east of the Play, so
            // they must hold nothing it would overwrite.
            let end = play.len().min(add + 12);
            assert!(
                play[add + 4..end].bytes().all(|cell| cell == b' '),
                "row {}, columns {}..{end}: {:?}",
                y + 1,
                add + 4,
                &play[add + 4..end]
            );
            play.replace_range(add..end, &format!("{functions}01"));
            voices += 1;
        }
    }
    (rows, voices)
}

/// Every voice of the shipped sequencer moved to the nested form, and how
/// many voices moved. The Play grows eight Cells east, so its Comment does,
/// and the `n` that selects the first step grows by the four pairs it now
/// has to cross.
///
/// Each conversion shifts everything east of it on both rows of the voice by
/// those eight Cells, so a later voice on the same rows keeps its Every over
/// its Play. A row's voices are found before any moves and converted east to
/// west, so each column still to be converted is one no conversion has
/// shifted yet.
fn nested_sequencer(shipped: &str) -> (Vec<String>, usize) {
    let mut rows: Vec<String> = shipped.lines().map(str::to_owned).collect();
    let mut voices = 0;
    for y in 0..rows.len() {
        let adds: Vec<usize> = rows[y]
            .match_indices(".+04~.0120")
            .map(|(add, _)| add)
            .collect();
        for &add in adds.iter().rev() {
            rows[y].replace_range(add..add + 10, &" ".repeat(18));
            let play = &mut rows[y + 1];
            assert_eq!(&play[add - 2..add + 6], "&>  01  ", "row {y}, column {add}");
            play.replace_range(add..add + 6, ".+08~.012001  ");
            voices += 1;
        }
    }
    (rows, voices)
}

/// Asserts that `current` and `nested` plan the same Play Commands on each of
/// `ticks` consecutive Ticks, diagnosing nothing, and that they play something.
fn assert_plays_the_same(current: &[u8], nested: &[String], ticks: u64) {
    let mut current = file::read(current).expect("a Source File");
    let mut nested = file::read(nested.join("\n").as_bytes()).expect("a Source File");
    let mut heard = 0;
    for tick in 0..ticks {
        let (from_current, from_nested) = (
            current.execute(Tick::new(tick)),
            nested.execute(Tick::new(tick)),
        );
        quiet_plan(&from_current);
        quiet_plan(&from_nested);
        assert_eq!(
            from_nested.play_commands, from_current.play_commands,
            "Tick {tick}"
        );
        heard += from_current.play_commands.len();
    }
    assert!(heard > 0, "the patch plays");
}

#[test]
fn the_nested_tracker_voice_plays_what_its_current_voice_plays() {
    let shipped = include_str!("../../../../examples/tracker.orcvs");
    let (nested, voices) = nested_tracker(shipped);
    assert_eq!(voices, 8, "every voice of the tracker moved");
    // The first voice of the staircase, exactly as the nested form is written.
    assert!(
        nested[13].starts_with("    !~0977&v.+02~.011001"),
        "{}",
        nested[13]
    );

    assert_plays_the_same(shipped.as_bytes(), &nested, 32);
}

#[test]
fn the_nested_sequencer_voice_plays_what_its_current_voice_plays() {
    let shipped = include_str!("../../../../examples/sequencer.orcvs");
    let (nested, voices) = nested_sequencer(shipped);
    assert_eq!(voices, 10, "every voice of the sequencer moved");
    assert!(
        nested[4].starts_with("    !~0977&>.+08~.012001  ||C2"),
        "{}",
        nested[4]
    );

    assert_plays_the_same(shipped.as_bytes(), &nested, 64);
}

#[test]
fn every_sequencer_voice_sharing_a_row_moves_to_the_nested_form() {
    // The shipped sequencer's first two voices, side by side on one pair of
    // rows.
    let shipped: Vec<&str> = include_str!("../../../../examples/sequencer.orcvs")
        .lines()
        .collect();
    let width = shipped[4].len() + 2;
    let side_by_side = format!(
        "{:width$}{}\n{:width$}{}",
        shipped[3], shipped[6], shipped[4], shipped[7]
    );

    let (nested, voices) = nested_sequencer(&side_by_side);

    assert_eq!(voices, 2, "both voices on the row moved");
    let east = width + 8;
    assert!(
        nested[1].starts_with("    !~0977&>.+08~.012001  ||C2"),
        "{}",
        nested[1]
    );
    assert!(
        nested[1][east..].starts_with("    !~0977&>.+08~.012001  ||        D2"),
        "{}",
        nested[1]
    );
    assert!(nested[0][east..].starts_with("  ~*0101"), "{}", nested[0]);
    assert!(!nested.join("\n").contains(".+04~"), "{nested:?}");

    assert_plays_the_same(side_by_side.as_bytes(), &nested, 64);
}

/// One step of a pattern driving a single operand: the two Cells it holds and
/// what the Chord Play sounds with them in that operand and every other
/// operand at its [`BASE`].
struct Step {
    value: &'static str,
    channel: u8,
    velocity: u8,
    notes: &'static [u8],
    length: u8,
}

/// The operands of a Timed Chord Play at rest, each two Cells; a Raw Chord
/// Play holds every one but the last.
const BASE: [&str; 7] = ["01", "7F", "C4", "MA", "CL", "00", "03"];

const fn step(
    value: &'static str,
    channel: u8,
    velocity: u8,
    notes: &'static [u8],
    length: u8,
) -> Step {
    Step {
        value,
        channel,
        velocity,
        notes,
        length,
    }
}

const C4_M: &[u8] = &[0x3C, 0x40, 0x43];

/// Each operand of a Chord Play and the two steps a pattern drives it through.
/// `69` and `13` spell Numbers and `D2` spells a Note, so those steps prove it
/// is the receiving operand that decides how a pair is read.
const OPERANDS: [(usize, [Step; 2]); 7] = [
    (
        0,
        [step("02", 2, 0x7F, C4_M, 3), step("03", 3, 0x7F, C4_M, 3)],
    ),
    (
        1,
        [step("40", 1, 0x40, C4_M, 3), step("69", 1, 0x69, C4_M, 3)],
    ),
    (
        2,
        [
            // D2 is a Note here, MIDI 38, and a Voicing two rows down.
            step("D2", 1, 0x7F, &[0x26, 0x2A, 0x2D], 3),
            step("E4", 1, 0x7F, &[0x40, 0x44, 0x47], 3),
        ],
    ),
    (
        3,
        [
            step("69", 1, 0x7F, &[0x3C, 0x40, 0x43, 0x45, 0x4A], 3),
            step("13", 1, 0x7F, &[0x3C, 0x40, 0x43, 0x46, 0x4A, 0x51], 3),
        ],
    ),
    (
        4,
        [
            // C4 MA with its 2nd tone from the top an octave lower.
            step("D2", 1, 0x7F, &[0x34, 0x3C, 0x43], 3),
            step("RL", 1, 0x7F, &[0x40, 0x43], 3),
        ],
    ),
    (
        5,
        [
            step("01", 1, 0x7F, &[0x40, 0x43, 0x48], 3),
            step("02", 1, 0x7F, &[0x43, 0x48, 0x4C], 3),
        ],
    ),
    (
        6,
        [step("04", 1, 0x7F, C4_M, 4), step("05", 1, 0x7F, C4_M, 5)],
    ),
];

/// The Play Commands `step` sounds.
fn expected(function: &str, step: &Step) -> Vec<PlayCommand> {
    let notes = step.notes.iter();
    if function == "!#" {
        notes
            .map(|&note| timed(step.channel, step.velocity, note, step.length))
            .collect()
    } else {
        notes
            .map(|&note| raw(step.channel, step.velocity, note))
            .collect()
    }
}

/// The column of operand `slot` of a Chord Play anchored at column 2, while
/// every operand before it is a literal.
const fn operand_column(slot: usize) -> usize {
    4 + 2 * slot
}

/// The last operand index `function` declares.
fn last_operand(function: &str) -> usize {
    if function == "!#" { 6 } else { 5 }
}

/// The operands of `function` as literals, with `nested` in `slot`.
fn play_with(function: &str, slot: usize, nested: &str) -> String {
    let mut play = format!("  {function}");
    for (at, literal) in BASE[..=last_operand(function)].iter().enumerate() {
        play.push_str(if at == slot { nested } else { literal });
    }
    play
}

/// A pattern row holding `value` in `column`, framed so it is text to Read.
fn pattern(column: usize, value: &str) -> String {
    format!("{:width$}||{value}||", "", width = column - 2)
}

/// Asserts that `rows` sounds step 0, step 1 and step 0 again on three Ticks.
fn assert_steps(rows: &[String], function: &str, steps: &[Step; 2], at: &str) {
    let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
    let ticks = run(Grid::with_shape(80, 6), &rows, 3);
    assert_eq!(ticks[0], expected(function, &steps[0]), "{at}, Tick 0");
    assert_eq!(ticks[1], expected(function, &steps[1]), "{at}, Tick 1");
    assert_eq!(ticks[2], expected(function, &steps[0]), "{at}, Tick 2");
}

#[test]
fn a_clock_steps_a_nested_read_through_every_operand_of_either_chord_play() {
    for function in ["!#", "!:"] {
        for &(slot, ref steps) in &OPERANDS[..=last_operand(function)] {
            let n = operand_column(slot) + 2;
            let rows = [
                ".=0101".to_owned(),
                play_with(function, slot, "&v.+02~.0102"),
                String::new(),
                pattern(n, steps[0].value),
                pattern(n, steps[1].value),
            ];
            assert_steps(
                &rows,
                function,
                steps,
                &format!("{function} operand {slot}"),
            );
        }
    }
}

#[test]
fn a_clock_steps_a_nested_track_through_every_operand_of_either_chord_play() {
    for function in ["!#", "!:"] {
        for &(slot, ref steps) in &OPERANDS[..=last_operand(function)] {
            // Track reads the pairs east of its last operand, so its index
            // steps past the Play's remaining operands and the Comment's `||`.
            let skip = last_operand(function) - slot + 1;
            let track = format!("&t.+{skip:02X}~.0102{:02X}", skip + 2);
            let row = format!(
                "{}||{}{}||",
                play_with(function, slot, &track),
                steps[0].value,
                steps[1].value
            );
            assert_steps(
                &[".=0101".to_owned(), row],
                function,
                steps,
                &format!("{function} operand {slot}"),
            );
        }
    }
}

/// The first Tick of `rows` on the Grid every operand test here shares.
fn first(rows: &[String]) -> (Vec<PlayCommand>, Vec<(usize, usize, String)>) {
    let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
    first_tick(Grid::with_shape(80, 6), &rows)
}

/// A pair each operand cannot read, and the refusal the Chord Play raises
/// for it.
const REFUSALS: [(usize, &str, &str); 10] = [
    (0, "M7", r#"expected a number, found "M7""#),
    (1, "M7", r#"expected a number, found "M7""#),
    (2, "M7", r#"expected a note, found "M7""#),
    (3, "42", r#"expected an Interval Set, found "42""#),
    (3, "C4", r#"expected an Interval Set, found "C4""#),
    (4, "42", r#"expected a Voicing, found "42""#),
    (4, "C4", r#"expected a Voicing, found "C4""#),
    (4, "M7", r#"expected a Voicing, found "M7""#),
    (5, "M7", r#"expected a number, found "M7""#),
    (6, "CL", r#"expected a number, found "CL""#),
];

#[test]
fn an_operand_refuses_a_nested_read_of_a_pair_it_cannot_read() {
    for function in ["!#", "!:"] {
        for (slot, pair, refusal) in REFUSALS {
            if slot > last_operand(function) {
                continue;
            }
            let n = operand_column(slot) + 2;
            let rows = [
                ".=0101".to_owned(),
                play_with(function, slot, "&v02"),
                String::new(),
                pattern(n, pair),
            ];
            let at = format!("{pair} in {function} operand {slot}");

            let (commands, diagnostics) = first(&rows);

            assert_eq!(commands, vec![], "{at}");
            assert_eq!(diagnostics, vec![(2, 1, refusal.to_owned())], "{at}");
        }
    }
}

#[test]
fn the_same_cells_mean_what_each_operand_receiving_them_reads() {
    // Two absolute Reads select the one pair at (8, 3), so what differs
    // between the operands is only how each decodes it.
    let cases: [(&str, &str, Vec<PlayCommand>); 2] = [
        (
            "  !:01&$0803C4&$0803CL00",
            "69",
            // The pair is velocity 0x69 and the Interval Set C4 `69`.
            [0x3C, 0x40, 0x43, 0x45, 0x4A]
                .map(|note| raw(1, 0x69, note))
                .to_vec(),
        ),
        (
            "  !:017F&$0803MA&$080300",
            "D2",
            // The pair is the root D2, MIDI 38, and the Voicing `D2`.
            [0x1E, 0x26, 0x2D].map(|note| raw(1, 0x7F, note)).to_vec(),
        ),
    ];
    for (play, pair, expected) in cases {
        let rows = [
            ".=0101".to_owned(),
            play.to_owned(),
            String::new(),
            pattern(8, pair),
        ];

        assert_eq!(first(&rows), (expected, vec![]), "{pair}");
    }
}

#[test]
fn a_chord_tracker_steps_root_and_set_columns_through_a_progression() {
    // Two voices of the tracker's staircase, each a 36-Cell Timed Chord Play
    // with a nested Read of its root column and of its set column, the second
    // voice's staircase moved east past the first voice's Play. The row south
    // of each Play holds its nested Functions' Output Portal writes, so step 0
    // is the row `02` south of each `n` operand.
    let rows = [
        "  ~*0101                                  ~*0101",
        "    !#0877&v.+02~.0104&v.+02~.0104CL0001    !#0977&v.+02~.0104&v.+02~.0104CL0001",
        "",
        "          ||D4          m7                          D3          mi||",
        "          ||G4          07                          G3          MA||",
        "          ||C4          M7                          C3          MA||",
        "          ||A4          m7                          A3          mi||",
    ];

    let bars: [(&[u8], &[u8]); 4] = [
        // ii, V, I and vi in C, the second voice an octave down in triads.
        (&[0x3E, 0x41, 0x45, 0x48], &[0x32, 0x35, 0x39]),
        (&[0x43, 0x47, 0x4A, 0x4D], &[0x37, 0x3B, 0x3E]),
        (&[0x3C, 0x40, 0x43, 0x47], &[0x30, 0x34, 0x37]),
        (&[0x45, 0x48, 0x4C, 0x4F], &[0x39, 0x3C, 0x40]),
    ];
    // Each Function's Output Portal write, on the row south of the Play: the
    // Read's answer, the Add's sum and the Clock's count, for both operands
    // of both voices.
    let portal_rows = [
        (
            0,
            "          D402  00    m702  00                    D302  00    mi02  00",
        ),
        (
            3,
            "          A405  03    m705  03                    A305  03    mi05  03",
        ),
    ];

    let mut source = source_of(Grid::with_shape(90, 7), &rows);
    for tick in 0..5 {
        let (upper, lower) = bars[tick % 4];
        let chords: Vec<PlayCommand> = upper
            .iter()
            .map(|&note| timed(8, 0x77, note, 1))
            .chain(lower.iter().map(|&note| timed(9, 0x77, note, 1)))
            .collect();
        assert_eq!(played(&mut source, tick as u64), chords, "Tick {tick}");
        if let Some((_, row)) = portal_rows.iter().find(|&&(at, _)| at == tick) {
            assert_eq!(rows_of(&source)[2].trim_end(), *row, "Tick {tick}");
        }
    }
}

/// The value a Portal-written operand holds before its writer's first Turn,
/// which no step of [`OPERANDS`] plays.
const STALE: [(usize, &str); 4] = [(2, "G2"), (3, "di"), (4, "SH"), (5, "03")];

#[test]
fn a_portal_written_operand_means_what_a_return_means() {
    for (slot, stale) in STALE {
        let (_, steps) = &OPERANDS[slot];
        // The Read stands one row north of the operand and writes straight
        // into it, with an Add and Clock above it feeding its `n`.
        let read = operand_column(slot);
        let n = read + 2;
        let rows = [
            format!("{:n$}.+02~.0102", ""),
            format!(".=0101{:width$}&v", "", width = read - 6),
            play_with("!#", slot, stale),
            pattern(n, steps[0].value),
            pattern(n, steps[1].value),
        ];
        assert_steps(&rows, "!#", steps, &format!("operand {slot}"));
    }
}
