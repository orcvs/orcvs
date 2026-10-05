//! Track through the Source Tick interface: a Clock-driven index, the claim a
//! literal count establishes for the whole Tick, same-Tick writers of the
//! index and the Items, blank Items, nesting, and malformed or truncated
//! Lists.

use lang::{MidiChannel, Note, PlayCommand, Tick, Velocity};

use crate::grid::{CellIndex, Grid};
use crate::source::{Source, TickPlan};

fn cell(grid: Grid, idx: usize) -> CellIndex {
    grid.cell_index(idx).expect("inside the Grid")
}

/// A Source holding `rows`, each padded to the Grid's width, its Cells set
/// one at a time as an editor sets them.
fn seeded(grid: Grid, rows: &[&str]) -> Source {
    let width = grid.columns();
    let text: String = rows.iter().map(|row| format!("{row:width$}")).collect();
    assert_eq!(text.len(), grid.count(), "the rows fill the Grid");
    let mut source = Source::new(grid);
    for (index, byte) in text.bytes().enumerate() {
        source
            .set(cell(grid, index), &char::from(byte).to_string())
            .unwrap();
    }
    source
}

/// Writes `text` from Column `x` of Row `y` onward, as a performer types it.
fn type_at(source: &mut Source, x: usize, y: usize, text: &str) {
    let grid = source.grid();
    for (offset, character) in text.chars().enumerate() {
        let position = grid.position(x + offset, y).expect("inside the Grid");
        source
            .set(grid.index(position), &character.to_string())
            .unwrap();
    }
}

/// The committed Source as one string per Grid row.
fn rows_of(source: &Source) -> Vec<String> {
    let columns = source.grid().columns();
    source
        .snapshot()
        .into_bytes()
        .chunks(columns)
        .map(|row| String::from_utf8(row.to_vec()).expect("ASCII Source"))
        .collect()
}

/// What one Tick Plan diagnosed.
fn messages(plan: &TickPlan) -> Vec<&str> {
    plan.diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect()
}

/// What the revision's Parser and Language Map diagnose.
fn parse_messages(source: &Source) -> Vec<String> {
    source
        .language_map()
        .diagnostics()
        .map(|diagnostic| diagnostic.message.clone())
        .collect()
}

/// One Timed Play Command on channel `00` at velocity `64` for four Ticks.
fn timed(note: u8) -> PlayCommand {
    PlayCommand::Timed {
        channel: MidiChannel::try_from(0).expect("a MIDI channel"),
        velocity: Velocity::try_from(0x64).expect("a MIDI data byte"),
        note: Note::try_from(note).expect("a MIDI note"),
        length: crate::source::Length::from(4),
    }
}

///
/// A Clock above Track's index drives selection from the first Tick: the
/// Clock's write reaches the index before Track reads it, so Tick zero selects
/// Item zero although the stored index names Item two. Each index holds for
/// the Clock's rate, the index wraps at Track's count, and selection carries
/// on past Tick 255.
///
#[test]
fn a_clock_driven_track_selects_holds_wraps_and_runs_past_tick_255() {
    let grid = Grid::with_shape(12, 3);
    let mut source = seeded(grid, &["  ~.0210", "@t0203C4D4E4", ""]);
    let items = ["C4", "D4", "E4"];
    for tick in (0..12).chain(250..262) {
        let plan = source.execute(Tick::new(tick));
        assert!(
            plan.diagnostics.is_empty(),
            "Tick {tick}: {:?}",
            plan.diagnostics
        );
        let index = (tick / 2) % 0x10;
        let rows = rows_of(&source);
        assert_eq!(&rows[1][2..4], format!("{index:02X}"), "Tick {tick}");
        assert_eq!(
            &rows[2][..2],
            items[usize::try_from(index % 3).unwrap()],
            "Tick {tick}"
        );
    }
}

///
/// A same-Tick write to the count changes nothing until the next Tick parses
/// it: the claim, the wrap and the zero check all keep the count the Tick
/// began with.
///
/// The index stays `02` throughout. Growing the count from two to three
/// still selects Item zero in the Tick of the write, because `02 % 02` is
/// zero; reading the new count early would select the third Item, which the
/// established claim does not hold. The next Tick claims three Items and
/// selects the third. Shrinking from three to two does the reverse, and a
/// count written to zero still selects in the Tick of the write and claims
/// nothing in the next, which writes nothing and is diagnosed by the Parser.
///
#[test]
fn a_same_tick_count_write_takes_effect_on_the_next_tick() {
    for (initial, written, now, next) in [
        ("02", "03", "C4", Some("E4")),
        ("03", "02", "E4", Some("C4")),
        ("03", "00", "E4", None),
    ] {
        let grid = Grid::with_shape(12, 3);
        // The Subtraction above the count writes `written` into it.
        let writer = format!("    .-{:02X}00", u8::from_str_radix(written, 16).unwrap());
        let mut source = seeded(grid, &[&writer, &format!("@t02{initial}C4D4E4"), ""]);
        // Whether the claim reaches the third Item's Cells.
        let claimed = |source: &Source| {
            let position = source.grid().position(10, 1).expect("inside the Grid");
            source.language_map().token_at(position) == Some(lang::Token::Item)
        };
        assert_eq!(claimed(&source), initial == "03", "{initial}");

        let plan = source.execute(Tick::new(0));
        assert!(
            plan.diagnostics.is_empty(),
            "{initial}: {:?}",
            plan.diagnostics
        );
        let rows = rows_of(&source);
        assert_eq!(&rows[1][4..6], written, "{initial}");
        assert_eq!(&rows[2][..2], now, "{initial} then {written}");

        // The write is Source now, so the next Tick parses the new count.
        assert_eq!(claimed(&source), written == "03", "{written}");
        // The previous answer is cleared so the next Tick's write, or its
        // absence, is what the row shows.
        type_at(&mut source, 0, 2, "  ");
        let plan = source.execute(Tick::new(1));
        assert!(
            plan.diagnostics.is_empty(),
            "{written}: {:?}",
            plan.diagnostics
        );
        match next {
            Some(item) => assert_eq!(&rows_of(&source)[2][..2], item, "{written}"),
            None => {
                assert!(plan.writes.iter().all(|write| write.cell.get() >= 12));
                assert_eq!(&rows_of(&source)[2][..2], "  ");
                assert!(
                    parse_messages(&source)
                        .iter()
                        .any(|message| message == "a List count of 00 holds no Item"),
                    "{:?}",
                    parse_messages(&source)
                );
            }
        }
    }
}

///
/// Writers of the index and of the Items reach Track in the Tick they write,
/// whether they stand before Track in Grid order (a value root on the row
/// above, writing south) or after it (a North Jump on the row below, copying
/// north). Track reads after every one of them.
///
#[test]
fn index_and_item_writers_before_and_after_track_reach_it_in_the_same_tick() {
    for (case, rows, selected) in [
        ("no writer", ["", "@t0003C4D4E4", "", ""], "C4"),
        ("index before", ["  .+0101", "@t0003C4D4E4", "", ""], "E4"),
        ("index after", ["", "@t0003C4D4E4", "  &^", "  02"], "E4"),
        (
            "item before",
            ["      .+0101", "@t0003C4D4E4", "", ""],
            "02",
        ),
        (
            "item after",
            ["", "@t0003C4D4E4", "      &^", "      F4"],
            "F4",
        ),
        (
            "index after, its item before",
            ["        .+0101", "@t0003C4D4E4", "  &^", "  01"],
            "02",
        ),
        (
            "index before, its item after",
            ["  .+0101", "@t0003C4D4E4", "          &^", "          F4"],
            "F4",
        ),
    ] {
        let grid = Grid::with_shape(16, 4);
        let mut source = seeded(grid, &rows);
        let plan = source.execute(Tick::new(0));
        assert!(
            plan.diagnostics.is_empty(),
            "{case}: {:?}",
            plan.diagnostics
        );
        assert_eq!(&rows_of(&source)[2][..2], selected, "{case}");
    }
}

///
/// Item writes follow the ordinary surviving-Source rules. Two writers that
/// each reach part of the selected Item compose Cell-wise in the order they
/// take their Turns, and a supplier that fails, or answers the Absence Marker,
/// writes nothing, so Track reads the Cells that survive.
///
#[test]
fn partial_competing_failed_and_absent_item_writers_leave_track_the_surviving_cells() {
    for (case, rows, selected, diagnosed) in [
        // `0E` lands on the Item's second Cell and the next Item's first.
        (
            "partial",
            ["       .+0E00", "@t0003C4D4E4", "", ""],
            "C0",
            vec![],
        ),
        // The Addition writes `02`; the Jump, taking the later Turn, writes
        // `F4` from the Item's second Cell on.
        (
            "competing",
            ["      .+0101", "@t0003C4D4E4", "       &^", "       F4"],
            "0F",
            vec![],
        ),
        (
            "failed",
            ["      ./0100", "@t0003C4D4E4", "", ""],
            "C4",
            vec!["cannot divide by zero"],
        ),
        (
            "absent",
            ["      .=0102", "@t0003C4D4E4", "", ""],
            "C4",
            vec![],
        ),
    ] {
        let grid = Grid::with_shape(16, 4);
        let mut source = seeded(grid, &rows);
        let plan = source.execute(Tick::new(0));
        assert_eq!(messages(&plan), diagnosed, "{case}");
        assert_eq!(&rows_of(&source)[2][..2], selected, "{case}");
    }
}

///
/// Index writes follow the same surviving-Source rules as Item writes, from
/// either side of Track in Grid order. A value root above writes south and a
/// North Jump below copies north; each partial write reaches the index's
/// second Cell and the count's first, which the established claim ignores.
/// Competing writers compose Cell-wise in Turn order, and a failed or absent
/// supplier leaves the stored index `01`.
///
#[test]
fn partial_competing_failed_and_absent_index_writers_leave_track_the_surviving_cells() {
    for (case, rows, selected, diagnosed) in [
        (
            "partial before",
            ["   .+1010", "@t0103C4D4E4", "", ""],
            "E4",
            vec![],
        ),
        (
            "partial after",
            ["", "@t0103C4D4E4", "   &^", "   20"],
            "E4",
            vec![],
        ),
        // The Addition writes `02`; the Jump, taking the later Turn, writes
        // `00` from the index's second Cell on.
        (
            "competing",
            ["  .+0101", "@t0103C4D4E4", "   &^", "   00"],
            "C4",
            vec![],
        ),
        (
            "failed before",
            ["  ./0100", "@t0103C4D4E4", "", ""],
            "D4",
            vec!["cannot divide by zero"],
        ),
        (
            "failed after",
            ["", "@t0103C4D4E4", "  &^", "  ##"],
            "D4",
            vec!["&^ has partial or invalid input"],
        ),
        (
            "absent before",
            ["  .=0102", "@t0103C4D4E4", "", ""],
            "D4",
            vec![],
        ),
    ] {
        let grid = Grid::with_shape(16, 4);
        let mut source = seeded(grid, &rows);
        let plan = source.execute(Tick::new(0));
        assert_eq!(messages(&plan), diagnosed, "{case}");
        assert_eq!(&rows_of(&source)[2][..2], selected, "{case}");
    }
}

///
/// A cycle through Track's declared Item reads stops Track, the Expression on
/// the cycle with it and the Expression that reads Track's write, and nothing
/// else (ADR 0065): the Equality's Bang still plays and the independent
/// Addition still writes.
///
/// The first Addition writes the Item Track does not select and Track writes
/// that Addition's operand. Every Item of the established claim is a read, so
/// the pair is a cycle whichever Item the index selects. Track also writes the
/// third Addition's operand, so that Addition waits on the cycle.
///
#[test]
fn a_cycle_through_an_unselected_item_stops_only_track_and_its_dependants() {
    let grid = Grid::with_shape(24, 4);
    let rows = [
        "@t0002C4D4  .=0101",
        "              !>007FC4",
        ".+0101  .+0102  .+0001",
        "",
    ];
    let mut source = seeded(grid, &rows);
    // Track writes the first and third Additions' first operands, and the
    // first Addition writes Track's second Item.
    let carried: std::collections::BTreeMap<_, _> = [
        (
            cell(grid, 0),
            vec![grid.position(2, 2).unwrap(), grid.position(18, 2).unwrap()],
        ),
        (cell(grid, 48), vec![grid.position(8, 0).unwrap()]),
    ]
    .into_iter()
    .collect();
    let (plan, _) = source.execute_carrying(Tick::new(0), &carried);

    let anchored: Vec<_> = plan
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.anchor().x(),
                diagnostic.anchor().y(),
                diagnostic.message.as_str(),
            )
        })
        .collect();
    assert_eq!(
        anchored,
        [
            (0, 0, "same-Tick dependency cycle"),
            (16, 2, "waiting on a same-Tick dependency cycle"),
        ]
    );
    assert_eq!(
        rows_of(&source),
        [
            "@t0002C4D4  .=0101      ",
            "            **!>007FC4  ",
            ".+0101  .+0102  .+0001  ",
            "        03              ",
        ]
    );
    assert_eq!(plan.play_commands.len(), 1, "{:?}", plan.play_commands);

    // Without the carried Portals the same Source has no cycle, and every
    // Expression publishes.
    let plan = source.execute(Tick::new(0));
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(plan.play_commands.len(), 1);
    assert_eq!(&rows_of(&source)[3][..2], "02");
}

///
/// Track's Output Portal is Timed Play's note slot, which holds the `E4` a
/// previous Tick left there, and the Equality's Bang activates the Play.
/// Selecting `C4` replaces it and plays C4. Selecting the blank Item writes two
/// spaces, which clears the slot, so the Play emits nothing and nothing is
/// diagnosed: the rest is deliberate. An Equality in Track's place answers the
/// Absence Marker instead, which writes nothing, so the Play replays `E4`: a
/// no-write absence is not how a clear is represented.
///
#[test]
fn a_blank_item_clears_timed_plays_note_where_the_absence_marker_leaves_it() {
    for (case, producer, note, expected) in [
        ("occupied Item", "@t0002C4  ", "C4", vec![timed(60)]),
        ("blank Item", "@t0102C4  ", "  ", vec![]),
        ("Absence Marker", ".=0102    ", "E4", vec![timed(64)]),
    ] {
        let grid = Grid::with_shape(18, 2);
        let mut source = seeded(grid, &[&format!(".=0101  {producer}"), "  !~0064E404"]);
        let plan = source.execute(Tick::new(0));
        assert_eq!(
            rows_of(&source)[1],
            format!("**!~0064{note}04      "),
            "{case}"
        );
        assert_eq!(plan.play_commands, expected, "{case}");
        assert!(
            plan.diagnostics.is_empty(),
            "{case}: {:?}",
            plan.diagnostics
        );
    }
}

///
/// A nested Track returns its selection to its parent and writes it through
/// its own Output Portal too. Selecting a blank Item clears Track's south
/// Cells and returns blank, so a value parent answers blank and clears its
/// own Output Portal rather than keeping what it held, and a Timed Play parent
/// emits nothing.
///
#[test]
fn a_nested_track_returns_its_item_and_a_blank_item_makes_its_parent_answer_blank() {
    // `C4` read by the Addition's Number operand is the Number `C4`.
    for (case, row, south) in [
        ("occupied", ".+@t0002C4  01", "C5C4"),
        ("blank", ".+@t0102C4  01", "    "),
    ] {
        let grid = Grid::with_shape(14, 2);
        let mut source = seeded(grid, &[row, "XXYY"]);
        let plan = source.execute(Tick::new(0));
        assert_eq!(&rows_of(&source)[1][..4], south, "{case}");
        assert!(
            plan.diagnostics.is_empty(),
            "{case}: {:?}",
            plan.diagnostics
        );
    }

    // A Timed Play whose note a nested Track supplies, activated from the
    // west. Track stands at Column 8 and writes south of itself.
    for (index, south, expected) in [("00", "C4", vec![timed(60)]), ("03", "  ", vec![])] {
        let grid = Grid::with_shape(32, 3);
        let row = format!("  !~0064@t{index}08C4D4E4  G4C5  E404");
        let mut source = seeded(grid, &[".=0101", &row, "        E4"]);
        let plan = source.execute(Tick::new(0));
        assert_eq!(&rows_of(&source)[2][8..10], south, "{index}");
        assert_eq!(plan.play_commands, expected, "{index}");
        assert!(
            plan.diagnostics.is_empty(),
            "{index}: {:?}",
            plan.diagnostics
        );
    }
}

///
/// Items are copied whole and never decoded where they stand, so malformed
/// data diagnoses at the operand that receives it: Timed Play's note slot
/// through a Portal, or a parent's operand as a Return. A malformed Item that
/// is not selected costs nothing, and a partly empty Item is malformed rather
/// than blank.
///
#[test]
fn malformed_selected_data_diagnoses_at_its_receiving_operand() {
    for (case, track, note, expected, diagnosed) in [
        (
            "selected",
            "@t0002ZZC4",
            "ZZ",
            vec![],
            vec!["expected a note, found \"ZZ\""],
        ),
        ("unselected", "@t0102ZZC4", "C4", vec![timed(60)], vec![]),
        (
            "partly empty",
            "@t0002C C4",
            "C ",
            vec![],
            vec!["expected a note, found \"C \""],
        ),
    ] {
        let grid = Grid::with_shape(18, 2);
        let mut source = seeded(grid, &[&format!(".=0101  {track}"), "  !~0064E404"]);
        // The Parser reports nothing about any Item.
        assert!(
            parse_messages(&source).is_empty(),
            "{case}: {:?}",
            parse_messages(&source)
        );
        let plan = source.execute(Tick::new(0));
        assert_eq!(&rows_of(&source)[1][8..10], note, "{case}");
        assert_eq!(plan.play_commands, expected, "{case}");
        assert_eq!(messages(&plan), diagnosed, "{case}");
        // The diagnostic belongs to the Play that read the Cells.
        for diagnostic in &plan.diagnostics {
            assert_eq!(
                (diagnostic.anchor().x(), diagnostic.anchor().y()),
                (2, 1),
                "{case}"
            );
        }
    }

    // Nested, the parent's Note operand receives the Return and diagnoses;
    // Track still writes what it copied.
    let grid = Grid::with_shape(10, 2);
    let mut source = seeded(grid, &[".v@t0001ZZ", ""]);
    let plan = source.execute(Tick::new(0));
    assert_eq!(messages(&plan), ["expected a note, found \"ZZ\""]);
    assert_eq!(
        (
            plan.diagnostics[0].anchor().x(),
            plan.diagnostics[0].anchor().y()
        ),
        (0, 0)
    );
    assert_eq!(rows_of(&source)[1], "  ZZ      ");
}

///
/// A single-Item List selects its one Item for every index, and an all-blank
/// List answers blank for every index. Function spellings, a Comment
/// introducer and a Bang inside the claim are data: Track copies them, and the
/// root after the claim is still read and evaluated.
///
#[test]
fn single_item_all_blank_and_function_like_lists() {
    for index in ["00", "01", "FF"] {
        let grid = Grid::with_shape(8, 2);
        let mut source = seeded(grid, &[&format!("@t{index}01C4"), "XX"]);
        source.execute(Tick::new(0));
        assert_eq!(&rows_of(&source)[1][..2], "C4", "{index}");

        let grid = Grid::with_shape(12, 2);
        let mut source = seeded(grid, &[&format!("@t{index}03      "), "XX"]);
        let plan = source.execute(Tick::new(0));
        assert_eq!(&rows_of(&source)[1][..2], "  ", "{index}");
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }

    for (index, copied) in [("00", ".+"), ("01", "||"), ("02", "**")] {
        let grid = Grid::with_shape(18, 2);
        let mut source = seeded(grid, &[&format!("@t{index}03.+||**.+0102"), ""]);
        assert!(
            parse_messages(&source).is_empty(),
            "{index}: {:?}",
            parse_messages(&source)
        );
        let plan = source.execute(Tick::new(0));
        assert!(
            plan.diagnostics.is_empty(),
            "{index}: {:?}",
            plan.diagnostics
        );
        let rows = rows_of(&source);
        assert_eq!(&rows[1][..2], copied, "{index}");
        // The Addition after the claim wrote its answer.
        assert_eq!(&rows[1][12..14], "03", "{index}");
    }
}

///
/// Copied characters are an ordinary Cell write, never an answered Bang or
/// Function, so copied `**` never activates a root. Beside a Bang-activated
/// root it activates nothing in the Tick it is copied, where an answered Bang
/// there activates the root, and the `**` it leaves is Bang display, which the
/// next Tick does not read as an activation either. Over a root's anchor,
/// where a Jump's answered Bang activates that root, Track's `**` covers the
/// spelling and suppresses the Expression. A copied Function spelling over a
/// running Function's anchor suppresses it under the same covering rule:
/// Function Replacement needs an answered Function Atom, and copied
/// characters are none, so the Addition writes nothing.
///
#[test]
fn copied_characters_activate_and_replace_nothing_where_they_land() {
    // Beside the root: an answered Bang activates it and copied `**` does not.
    for (case, producer, expected) in [
        ("answered Bang", ".=0101  ", vec![timed(60)]),
        ("copied Bang", "@t0001**", vec![]),
    ] {
        let grid = Grid::with_shape(12, 2);
        let mut source = seeded(grid, &[producer, "  !~0064C404"]);
        let plan = source.execute(Tick::new(0));
        assert_eq!(rows_of(&source)[1], "**!~0064C404", "{case}");
        assert_eq!(plan.play_commands, expected, "{case}");
        assert!(
            plan.diagnostics.is_empty(),
            "{case}: {:?}",
            plan.diagnostics
        );
    }
    let grid = Grid::with_shape(12, 2);
    let mut source = seeded(grid, &["@t0001**", "**!~0064C404"]);
    let plan = source.execute(Tick::new(1));
    assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);

    // Over the root's anchor: a Jump copying the Equality's Bang activates the
    // root and writes nothing there; Track's `**` covers it.
    let grid = Grid::with_shape(10, 4);
    let mut source = seeded(grid, &[".=0101", "", "&v", "!~0064C404"]);
    let plan = source.execute(Tick::new(0));
    assert_eq!(rows_of(&source)[3], "!~0064C404");
    assert_eq!(plan.play_commands, [timed(60)]);
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);

    let grid = Grid::with_shape(10, 2);
    let mut source = seeded(grid, &["@t0001**", "!~0064C404"]);
    let plan = source.execute(Tick::new(0));
    assert_eq!(rows_of(&source)[1], "**0064C404");
    assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);

    // Over a Function's anchor: the Addition would write `03` south of itself.
    let grid = Grid::with_shape(8, 3);
    let mut source = seeded(grid, &["@t0001.-", ".+0102", ""]);
    let plan = source.execute(Tick::new(0));
    assert_eq!(rows_of(&source)[1..], [".-0102  ", "        "]);
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
}

///
/// A count that claims nothing, or a claim the row cuts short, never selects:
/// Track writes nothing and reads no Cell outside the claim. A count of `00`,
/// a malformed count and a nested count are refused by the Parser; a claim
/// past the row edge is reported against the Expression by the Tick as well.
///
#[test]
fn invalid_counts_and_claims_at_the_row_edge_never_select() {
    for (row, parsed, planned) in [
        ("@t0000C4", "a List count of 00 holds no Item", vec![]),
        ("@t00ZZC4", "expected a number, found \"ZZ\"", vec![]),
        ("@t00.+0101C4", "a List count is a literal Number", vec![]),
        (
            "@t0003C4D4",
            "expected a token",
            vec!["Expression layout crosses the row edge"],
        ),
        (
            "@t0002C4D",
            "expected a token",
            vec!["Expression layout crosses the row edge"],
        ),
    ] {
        let grid = Grid::with_shape(row.len(), 2);
        let mut source = seeded(grid, &[row, ""]);
        assert!(
            parse_messages(&source)
                .iter()
                .any(|message| message == parsed),
            "{row}: {:?}",
            parse_messages(&source)
        );
        let plan = source.execute(Tick::new(0));
        assert_eq!(messages(&plan), planned, "{row}");
        assert!(
            plan.writes.iter().all(|write| write.cell.get() < row.len()),
            "{row}: Track wrote {:?}",
            plan.writes
        );
    }
}

///
/// A Jump that copies Track's spelling onto a running Addition's anchor is a
/// Function replacement the schedule cannot admit: the Parser claimed no List
/// for the Addition, so Track would have no Items to select. The refusal names
/// the fact, and the Addition runs as parsed.
///
#[test]
fn replacing_a_function_with_track_is_refused_for_the_list_it_reads() {
    let grid = Grid::with_shape(12, 4);
    let mut source = seeded(grid, &[".+0102", "", "  &<@t0001C4", ""]);
    // The West Jump reads Track's spelling and writes it at the Addition.
    let carried: std::collections::BTreeMap<_, _> =
        [(cell(grid, 26), vec![grid.position(0, 0).unwrap()])]
            .into_iter()
            .collect();
    let (plan, _) = source.execute_carrying(Tick::new(0), &carried);
    assert_eq!(
        messages(&plan),
        ["Function replacement changes whether it reads a List"]
    );
    let rows = rows_of(&source);
    assert_eq!(&rows[0][..6], ".+0102");
    assert_eq!(&rows[1][..2], "03");
    assert_eq!(&rows[3][4..6], "C4");
}
