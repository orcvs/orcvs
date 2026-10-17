//! Live editing: claims, glyphs and writes surviving Source edits, and producers
//! that settle before their consumers read.

use crate::grid::Grid;
use crate::source::Cells;
use crate::source::language_map::LanguageMap;
use lang::Token;

use super::Tick;
use super::fixture::{
    carried_destinations, carried_source, carried_tick, cell, interpreted, messages, snapshot,
    turns,
};
use super::observed::raw;

#[test]
fn live_deep_sibling_computations_preserve_operand_order() {
    // Each sibling requires more pending operands than the Parser keeps
    // inline. The second refills that stack after the first has drained it.
    // Deep enough to overflow and narrow enough to fit a Grid.
    let numerator = ".+".repeat(24) + &"02".repeat(25);
    let denominator = ".+".repeat(24) + &"01".repeat(25);
    let text = format!("./{numerator}{denominator}");
    let width = text.len();
    let (plan, source) = carried_source(Grid::with_shape(width, 2), &[&text, ""], &[]);
    // 50 / 25 = 2. Reversing the siblings instead produces zero. Each
    // sibling's outermost Addition writes its own sum under its anchor.
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert!(plan.play_commands.is_empty());
    let snapshot = source.snapshot();
    let (expression, below) = snapshot.split_at(width);
    assert_eq!(expression, text);
    let denominator_anchor = 2 + numerator.len();
    assert_eq!(&below[..2], "02");
    assert_eq!(&below[2..4], "32");
    assert_eq!(&below[denominator_anchor..denominator_anchor + 2], "19");
}

#[test]
fn live_unchanged_nested_syntax_errors_do_not_repeat_as_tick_failures() {
    let grid = Grid::with_shape(20, 2);
    let rows = [".+01.x02.+03??", ""];
    let (plan, mut source) = carried_source(grid, &rows, &[]);
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert!(
        !source
            .language_map()
            .expression_diagnostics()
            .collect::<Vec<_>>()
            .is_empty()
    );
    assert_eq!(source.snapshot(), snapshot(grid, &rows));
    let repeated = source.execute(Tick::new(1));
    assert!(
        repeated.diagnostics.is_empty(),
        "{:?}",
        repeated.diagnostics
    );
    assert!(repeated.play_commands.is_empty());
    source.set(cell(grid, 12), "0").unwrap();
    source.set(cell(grid, 13), "4").unwrap();
    let repaired = source.execute(Tick::new(2));
    assert!(
        repaired.diagnostics.is_empty(),
        "{:?}",
        repaired.diagnostics
    );
    assert_eq!(&source.snapshot()[20..22], "0F");

    // A writer can also repair the bad leaf before its reserved turn.
    let (repaired, source) = carried_source(
        Grid::with_shape(20, 3),
        &[".+01.x02.+03??", "          .+0004", ""],
        &[(0, 40), (30, 12)],
    );
    assert!(
        repaired.diagnostics.is_empty(),
        "{:?}",
        repaired.diagnostics
    );
    assert_eq!(&source.snapshot()[40..42], "0F");
}

#[test]
fn live_inactive_nested_portal_cannot_create_a_cycle() {
    let (plan, source) = carried_source(
        Grid::with_shape(16, 3),
        &["!>007F.^3C", ".+0203", ""],
        &[(6, 8), (16, 32)],
    );
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(&source.snapshot()[32..34], "05");
    assert!(plan.play_commands.is_empty());
}

///
/// An Absolute Difference beside a `|` is read in two-Cell units.
///
/// `.|` is the one spelling whose second Cell is a `|`, so it is the one
/// that could present a `||` to a scan stepping over overlapping byte
/// pairs rather than in two-Cell units. The Parser reads in units, so a
/// `|` beside an Absolute Difference is that Function's first operand and
/// fails to bind, and the row holds no Comment at all.
///
/// A row of `.|` and a real introducer is the other half: the Function
/// keeps the claim its arity declares, and the `||` after it opens a
/// Comment that claims what is left. Both rows run a Tick, because the
/// answer depends on which Cells the walk hands the Parser and has to hold
/// through execution rather than only through derivation.
///
#[test]
fn an_absolute_difference_beside_a_vertical_rule_is_read_in_two_cell_units() {
    // `.|` at Cells 0 and 1 with a `|` at Cell 2: the pair at Cells 1 and
    // 2 spells `||` and means nothing, because nothing reads it.
    let (plan, source) = carried_source(Grid::with_shape(8, 2), &[".||102", ""], &[]);

    assert!(plan.writes.is_empty());
    assert!(
        !source
            .language_map()
            .units()
            .any(|unit| unit.kind() == crate::source::LanguageUnitKind::Comment)
    );
    let refused = source.language_map().expressions().next().unwrap();
    assert_eq!(refused.span().positions().count(), 6);
    assert!(refused.root().is_none());

    // The same Function beside a real introducer. The Absolute Difference
    // of 01 and 02 answers 01 and writes it below; the Comment claims the
    // rest of the row and answers nothing.
    let (plan, source) = carried_source(Grid::with_shape(8, 2), &[".|0102||", ""], &[]);

    assert_eq!(&source.snapshot()[8..10], "01");
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(
        source
            .language_map()
            .units()
            .filter(|unit| unit.kind() == crate::source::LanguageUnitKind::Comment)
            .map(|unit| unit.anchor().x())
            .collect::<Vec<_>>(),
        vec![6]
    );
    for column in 6..8 {
        assert_eq!(
            source
                .language_map()
                .token_at(source.grid().position(column, 0).unwrap()),
            Some(Token::Comment)
        );
    }
}

#[test]
fn live_claims_and_glyphs_survive_source_edits_and_publication() {
    let (plan, source) = carried_source(Grid::with_shape(16, 2), &[".+01 02", ""], &[]);
    assert!(plan.writes.is_empty());
    assert_eq!(
        source
            .language_map()
            .expressions()
            .next()
            .unwrap()
            .span()
            .end()
            .get(),
        5
    );
    assert!(source.language_map().diagnostics().any(|d| d.start() == 6));
    assert_eq!(
        source
            .language_map()
            .token_at(source.grid().position(4, 0).unwrap()),
        Some(Token::Number)
    );
    let (plan, source) = carried_source(Grid::with_shape(16, 2), &[".+0102.+0304", ""], &[]);
    assert_eq!(&source.snapshot()[16..24], "03    07");
    assert!(plan.diagnostics.is_empty());
    assert_eq!(
        source
            .language_map()
            .expressions()
            .filter_map(|entry| entry.root())
            .filter(|root| root.y() == 0)
            .count(),
        2
    );
    let (plan, source) = carried_source(Grid::with_shape(16, 2), &[".+0102Z", ""], &[]);
    assert_eq!(&source.snapshot()[16..18], "03");
    assert!(plan.diagnostics.is_empty());
    assert!(source.language_map().diagnostics().any(|d| d.start() == 6));
    let (plan, source) = carried_source(Grid::with_shape(16, 2), &["***", ""], &[]);
    assert_eq!(&source.snapshot()[..3], "  *");
    assert_eq!(plan.writes.len(), 2);
    assert!(source.language_map().diagnostics().any(|d| d.start() == 2));
    let (plan, source) = carried_source(Grid::with_shape(16, 2), &[".=0101 !>007FC4", ""], &[]);
    assert_eq!(&source.snapshot()[16..18], "**");
    assert!(plan.play_commands.is_empty());
    assert!(plan.diagnostics.is_empty());
    assert_eq!(
        source
            .language_map()
            .expressions()
            .filter_map(|entry| entry.root())
            .filter(|root| root.y() == 0)
            .count(),
        2
    );
    // Pin the current display of plausible standalone data. These rejected
    // Function candidates have Function glyphs, no units and no execution.
    let (plan, source) = carried_source(Grid::with_shape(16, 2), &["C4 EA 01", ""], &[]);
    assert!(plan.writes.is_empty());
    assert_eq!(source.language_map().units().count(), 0);
    for column in [0, 1, 3, 4, 6, 7] {
        assert_eq!(
            source
                .language_map()
                .token_at(source.grid().position(column, 0).unwrap()),
            Some(Token::Function)
        );
    }
}

#[test]
fn live_bang_in_half_typed_terminal_claim_diagnoses_without_activation() {
    let (plan, source) = carried_source(
        Grid::with_shape(16, 3),
        &["    .=0101", "!>00", "    !>007FC4"],
        &[],
    );
    assert!(plan.play_commands.is_empty());
    assert_eq!(&source.snapshot()[16..24], "!>00**  ");
    assert!(
        source
            .language_map()
            .diagnostics()
            .any(|d| d.start() == 16 && d.message.contains("expected a number"))
    );
    assert_eq!(
        source
            .language_map()
            .token_at(source.grid().position(4, 1).unwrap()),
        Some(Token::Number)
    );
    assert_eq!(source.language_map().bang_functions().count(), 0);
}

#[test]
fn live_two_overlapping_results_resolve_cell_by_cell() {
    // Cell-wise conflict resolution: one admitted write is one validated
    // effect until the Tick Plan resolves, and then as many independently
    // contested Cells as it has characters. Two roots aim at Cell pairs one
    // Cell apart; the later producer wins the Cell they share and the
    // earlier producer's first Cell stands, which a rule resolving whole
    // writes would have replaced together.
    let grid = Grid::with_shape(16, 2);
    let (plan, source) = carried_source(grid, &[".+000A.+000D", ""], &[(0, 16), (6, 17)]);

    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(plan.writes.len(), 3, "two Cell pairs sharing one Cell");
    assert_eq!(source.snapshot(), snapshot(grid, &[".+000A.+000D", "00D"]));
}

#[cfg(not(target_arch = "wasm32"))]
proptest::proptest! {
    #[test]
    fn live_writes_agree_with_an_independent_cell_overlay(
        first in proptest::prelude::any::<u8>(), second in proptest::prelude::any::<u8>(),
        first_column in 2usize..6, second_column in 2usize..6,
    ) {
        let grid = Grid::with_shape(16, 4);
        let first_source = format!(".+00{first:02X}");
        let second_source = format!(".+00{second:02X}");
        let rows = [".+0101", first_source.as_str(), second_source.as_str(), ""];
        let outputs = [(0, 48), (16, first_column), (32, second_column)];
        let mut expected = snapshot(grid, &rows).into_bytes();
        // Independent oracle: two literal Cell overlays in known Position
        // order, followed by ordinary hexadecimal arithmetic.
        expected[first_column..first_column + 2].copy_from_slice(format!("{first:02X}").as_bytes());
        expected[second_column..second_column + 2].copy_from_slice(format!("{second:02X}").as_bytes());
        let left = u16::from_str_radix(std::str::from_utf8(&expected[2..4]).unwrap(), 16).unwrap();
        let right = u16::from_str_radix(std::str::from_utf8(&expected[4..6]).unwrap(), 16).unwrap();
        expected[48..50].copy_from_slice(format!("{:02X}", (left + right) % 256).as_bytes());
        let (plan, source) = carried_source(grid, &rows, &outputs);
        proptest::prop_assert_eq!(source.snapshot().into_bytes(), expected);
        proptest::prop_assert!(plan.diagnostics.is_empty());
        let (repeated, _) = carried_source(grid, &rows, &outputs);
        proptest::prop_assert_eq!(plan, repeated);
        let full = LanguageMap::derive(grid, &source.snapshot()).unwrap();
        proptest::prop_assert_eq!(source.language_map().units().collect::<Vec<_>>(), full.units().collect::<Vec<_>>());
        for position in grid.positions_by_row().flatten() {
            proptest::prop_assert_eq!(source.language_map().token_at(position), full.token_at(position));
        }
    }

    #[test]
    fn live_rejected_complete_write_changes_no_cell(value in proptest::prelude::any::<u8>()) {
        let grid = Grid::with_shape(16, 2);
        let producer = format!(".+00{value:02X}");
        let rows = [producer.as_str(), ""];
        let (plan, source) = carried_source(grid, &rows, &[(0, 31)]);
        proptest::prop_assert!(plan.writes.is_empty());
        proptest::prop_assert_eq!(source.snapshot(), snapshot(grid, &rows));
        proptest::prop_assert!(plan.diagnostics.iter().any(|d| d.message.contains("crosses the row edge")));
    }
}

#[test]
fn live_cross_boundary_chain_uses_lower_producers() {
    let (plan, source) = carried_source(
        Grid::with_shape(16, 5),
        &[".+0101", ".+0001", ".+0001", "", ""],
        &[(0, 64), (16, 34), (32, 3)],
    );
    assert_eq!(&source.snapshot()[..6], ".+0021");
    assert_eq!(&source.snapshot()[64..66], "21");
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
}

#[test]
fn live_competing_writers_follow_position_and_emissions_follow_their_destinations() {
    let (plan, source) = carried_source(
        Grid::with_shape(16, 4),
        &[".+0101", ".+0101", ".+0102", ""],
        &[(0, 48), (16, 3), (32, 3)],
    );
    assert_eq!(&source.snapshot()[..6], ".+0031");
    assert_eq!(&source.snapshot()[48..50], "31");
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    let (plan, interpreted, source) = carried_tick(
        Grid::with_shape(16, 3),
        &[".+0101", ".+0203", ""],
        &[(0, 32), (16, 2), (16, 3)],
    );
    assert_eq!(&source.snapshot()[..6], ".+0051");
    assert_eq!(&source.snapshot()[32..34], "51");
    assert_eq!(interpreted.len(), 2);
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
}

#[test]
fn live_pending_note_decodes_only_after_all_writers_settle() {
    let (plan, source) = carried_source(
        Grid::with_shape(16, 4),
        &[".vE4", ".+E901", ".+E401", ""],
        &[(0, 48), (16, 2), (32, 2)],
    );
    assert_eq!(&source.snapshot()[..4], ".vE5");
    assert_eq!(&source.snapshot()[48..50], "4C");
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    let (plan, source) = carried_source(
        Grid::with_shape(16, 3),
        &[".vE4", ".+E901", ""],
        &[(0, 32), (16, 2)],
    );
    assert_eq!(&source.snapshot()[..4], ".vEA");
    assert_eq!(&source.snapshot()[32..34], "  ");
    assert!(
        plan.diagnostics
            .iter()
            .any(|d| d.message.contains("expected a note")),
        "{:?}",
        plan.diagnostics
    );
    let (plan, source) = carried_source(
        Grid::with_shape(16, 3),
        &[".+0000", ".+E901", ""],
        &[(0, 32), (16, 2)],
    );
    assert_eq!(&source.snapshot()[32..34], "EA");
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
}

#[test]
fn live_spatial_and_returned_notes_are_both_encodings() {
    let (plan, source) = carried_source(
        Grid::with_shape(16, 3),
        &[".+0001", ".^48", ""],
        &[(0, 32), (16, 2)],
    );
    assert_eq!(&source.snapshot()[..6], ".+C501");
    assert_eq!(&source.snapshot()[32..34], "C6");
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    // Returned rather than written, Note `C5` is the same characters and
    // the Number operand reads them the same way.
    let (plan, source) = carried_source(Grid::with_shape(16, 2), &[".+.^4801", ""], &[(0, 16)]);
    assert_eq!(&source.snapshot()[16..20], "C6C5");
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(&source.snapshot()[..8], ".+.^4801");
}

#[test]
fn live_child_write_survives_parent_failure_and_rejected_portal_keeps_its_return() {
    // Nested `.x` returns 0C to `./` and writes it into the sibling `.+`'s
    // first operand. The parent divides by zero; the child's write stands
    // and the sibling consumes it.
    let (plan, source) = carried_source(
        Grid::with_shape(16, 4),
        &["./.x030400", ".+0001", "", ""],
        &[(0, 48), (2, 18), (16, 52)],
    );
    assert_eq!(&source.snapshot()[18..20], "0C");
    assert_eq!(&source.snapshot()[48..50], "  ");
    assert_eq!(&source.snapshot()[52..54], "0D");
    assert!(
        plan.diagnostics
            .iter()
            .any(|d| d.message == "cannot divide by zero")
    );
    // The nested Multiply's Portal crosses the row edge, so its write is
    // refused whole and diagnosed. Its Return still supplies 0C, so `.+`
    // writes 0E.
    let (plan, source) = carried_source(
        Grid::with_shape(16, 2),
        &[".+02.x0304", ""],
        &[(0, 16), (4, 31)],
    );
    assert_eq!(&source.snapshot()[16..18], "0E");
    assert_eq!(&source.snapshot()[31..], " ");
    assert_eq!(messages(&plan), ["result \"0C\" crosses the row edge"]);
}

#[test]
fn live_failed_suppliers_preserve_spatial_data_but_not_nested_answers() {
    let (plan, source) = carried_source(
        Grid::with_shape(16, 4),
        &[".+0001", ".+0203", "./0100", ""],
        &[(0, 48), (16, 2), (32, 2)],
    );
    assert_eq!(&source.snapshot()[..6], ".+0501");
    assert_eq!(&source.snapshot()[48..50], "06");
    assert!(
        plan.diagnostics
            .iter()
            .any(|d| d.message == "cannot divide by zero")
    );
    let (plan, source) = carried_source(Grid::with_shape(16, 2), &[".+02./0100", ""], &[(0, 16)]);
    assert_eq!(&source.snapshot()[16..22], "      ");
    assert!(
        plan.diagnostics
            .iter()
            .any(|d| d.message.contains("returned nothing"))
    );
    // A failed structural writer leaves the original computation connected.
    let (plan, source) = carried_source(
        Grid::with_shape(16, 3),
        &[".+02.x0304", "      ./0100", ""],
        &[(0, 32), (22, 4)],
    );
    assert_eq!(&source.snapshot()[..10], ".+02.x0304");
    assert_eq!(&source.snapshot()[32..34], "0E");
    assert!(
        plan.diagnostics
            .iter()
            .any(|d| d.message == "cannot divide by zero")
    );
}

#[test]
fn live_inactive_ownership_and_silent_terminal_output_are_independent() {
    // Carrying a Cell onto `!>` cannot mint a Portal. The root is never
    // activated, so its nested `.^80` takes no Turn and writes nothing
    // through the Portal carried onto it, and this Expression neither
    // plays nor writes.
    let (plan, interpreted, source) = carried_tick(
        Grid::with_shape(16, 3),
        &["!>007F.^80", "", ""],
        &[(0, 16), (6, 20)],
    );
    assert!(plan.writes.is_empty());
    assert!(plan.play_commands.is_empty());
    assert_eq!(interpreted.len(), 0);
    assert!(
        plan.diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.message.contains("Portal")),
        "{:?}",
        plan.diagnostics
    );
    assert_eq!(&source.snapshot()[16..32], "                ");
    let (plan, _) = carried_source(
        Grid::with_shape(16, 4),
        &["!>007FC4", "", ".=0101", ""],
        &[(0, 34), (32, 16)],
    );
    assert_eq!(plan.play_commands, vec![raw(0, 0x7F, 60)]);
    assert!(
        plan.diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.message.contains("Portal")),
        "{:?}",
        plan.diagnostics
    );
}

#[test]
fn a_late_spatial_write_is_refused_and_the_tick_continues_even_when_the_earlier_turn_failed() {
    for target in [".+0101", "./0100", ".+01??"] {
        let grid = Grid::with_shape(16, 11);
        let rows = [
            "", "!>007FC4", ".=0101", "./0100", target, ".+0203", "", ".+0304", "", "", "",
        ];
        let bytes = snapshot(grid, &rows);
        let map = LanguageMap::derive(grid, &bytes).unwrap();
        let carried =
            carried_destinations(grid, &[(32, 0), (48, 160), (64, 96), (80, 64), (112, 160)]);
        let mut schedule = super::schedule_carrying(grid, &map, &carried);
        // Supply a broken order at the execution seam: the scheduler must
        // never produce this, but execution promises to refuse the one
        // write it gets wrong rather than panic or drop the Tick.
        schedule.ordered.order = [32, 16, 48, 64, 80, 112]
            .into_iter()
            .map(|anchor| {
                schedule
                    .lookup
                    .nodes()
                    .iter()
                    .find(|node| grid.index(node.anchor) == cell(grid, anchor))
                    .unwrap()
                    .id
            })
            .collect();
        let (plan, states) = super::execution::execute(
            grid,
            Cells::of(bytes.as_bytes()),
            &map,
            Tick::ZERO,
            &schedule,
        );
        // The broken order was walked as given, and to its end: the
        // computation anchored at Cell 32 took the first Turn ahead of the
        // one anchored at Cell 16, and the one anchored at Cell 112 took
        // the last. Read in parse order, which is anchor order here, so
        // the entries name Cells 16, 32, 48, 64, 80 and 112.
        assert_eq!(
            turns(&states),
            vec![Some(1), Some(0), Some(2), Some(3), Some(4), Some(5)],
        );
        // Only the write onto Cell 64 is refused. The target keeps its
        // spelling and its own answer, and every other effect publishes.
        let mut published = bytes.clone().into_bytes();
        for write in &plan.writes {
            published[write.cell.get()] = write.content.as_char() as u8;
        }
        let published: Vec<_> = published
            .chunks(grid.columns())
            .map(|row| std::str::from_utf8(row).unwrap())
            .collect();
        let answer = if target == ".+0101" { "02" } else { "" };
        assert_eq!(
            published,
            [
                "**", "!>007FC4", ".=0101", "./0100", target, ".+0203", answer, ".+0304", "", "",
                "07",
            ]
            .map(|row| format!("{row:16}")),
            "{target}"
        );
        assert_eq!(plan.play_commands, vec![raw(0, 0x7F, 60)]);
        let mut expected = vec![(48, "cannot divide by zero")];
        if target == "./0100" {
            expected.push((64, "cannot divide by zero"));
        }
        expected.push((80, "spatial output reached an executed computation"));
        assert_eq!(
            plan.diagnostics
                .iter()
                .map(|diagnostic| {
                    (
                        grid.index(diagnostic.anchor()).get(),
                        diagnostic.message.as_str(),
                    )
                })
                .collect::<Vec<_>>(),
            expected,
        );
        // Attempting a syntax-blocked Turn still prevents later writes
        // reaching it, although that Turn never calls the Evaluator.
        assert_eq!(
            interpreted(&states).len(),
            if target == ".+01??" { 5 } else { 6 }
        );
    }
}

#[test]
fn a_value_replaces_nested_computation_and_preserves_next_tick_source() {
    let (plan, interpreted, source) = carried_tick(
        Grid::with_shape(16, 4),
        &[".+02.x0304", ".+0203", "", ""],
        &[(0, 48), (4, 52), (16, 4)],
    );
    assert_eq!(&source.snapshot()[..10], ".+02050304");
    assert_eq!(&source.snapshot()[48..50], "07");
    assert_eq!(&source.snapshot()[52..54], "  ");
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(interpreted.len(), 2);
    let next = source.language_map();
    assert_eq!(next.expressions().next().unwrap().span().end().get(), 5);
    assert!(next.diagnostics().any(|diagnostic| diagnostic.start() == 6));
    assert!(next.diagnostics().any(|diagnostic| diagnostic.start() == 8));
}

#[test]
fn nested_computation_returns_and_projects_once() {
    // Nested `.x` returns 0C to `.+` and writes the same 0C through its
    // own Portal, into the sibling `.+`'s first operand, from one
    // interpretation.
    let grid = Grid::with_shape(16, 4);
    let (plan, interpreted, source) = carried_tick(
        grid,
        &[".+02.x0304", ".+0101", "", ""],
        &[(0, 48), (4, 18), (16, 52)],
    );
    assert_eq!(&source.snapshot()[18..20], "0C");
    assert_eq!(&source.snapshot()[48..50], "0E");
    assert_eq!(&source.snapshot()[52..54], "0D");
    assert_eq!(&source.snapshot()[..10], ".+02.x0304");
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(
        interpreted
            .iter()
            .filter(
                |inputs| **inputs == super::tick_inputs(Tick::ZERO, grid.position(4, 0).unwrap())
            )
            .count(),
        1
    );
}

#[test]
fn partial_writers_settle_before_consumption() {
    let (plan, source) = carried_source(
        Grid::with_shape(16, 4),
        &[".+0101", ".+0203", ".+0101", ""],
        &[(0, 56), (16, 2), (32, 3)],
    );
    assert_eq!(&source.snapshot()[..6], ".+0021");
    assert_eq!(&source.snapshot()[56..58], "21");
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(
        source
            .language_map()
            .expressions()
            .next()
            .unwrap()
            .span()
            .end()
            .get(),
        5
    );
}
