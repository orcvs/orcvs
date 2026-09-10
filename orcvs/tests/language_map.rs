use lang::Function;
use orcvs::{
    glyph::Glyph,
    grid::Grid,
    source::{LanguageMap, LanguageUnitKind, Source},
};

#[test]
fn truncated_operand_owns_the_available_row_tail() {
    let grid = Grid::new(5, 1);
    let map = LanguageMap::derive(grid, ".+01Z").unwrap();
    let expressions = map.expressions().collect::<Vec<_>>();
    assert_eq!(expressions.len(), 1);
    assert_eq!(expressions[0].span().positions().count(), 5);
    assert!(expressions[0].root().is_none());
}

/// A Comment introducer inside a Function's arity-determined claim is an
/// operand Cell of that Function, not a Comment. ADR 0035 makes `||` a
/// spelling like any other, and ADR 0033 reads a spelling only in the position
/// where a spelling is read, so the Add's second operand is `||`, it fails to
/// bind, and the row holds no Comment at all.
#[test]
fn an_operand_claim_reaches_over_a_comment_introducer() {
    let grid = Grid::new(8, 1);
    let map = LanguageMap::derive(grid, ".+01||xx").unwrap();

    assert!(
        !map.units()
            .any(|unit| unit.kind() == LanguageUnitKind::Comment)
    );
    let first = map.expressions().next().unwrap();
    assert_eq!(first.span().positions().count(), 6);
    assert!(first.root().is_none());
    // The introducer's Cells are the operand's, so they carry its Glyph.
    for column in 4..6 {
        assert_eq!(
            map.glyph_at(grid.position(column, 0).unwrap()),
            Some(Glyph::Number)
        );
    }
}

/// A Comment where an Expression could start claims every Cell after it, and
/// the Function before it keeps the claim its arity declares.
#[test]
fn a_comment_claims_the_row_after_the_expression_that_precedes_it() {
    let grid = Grid::new(10, 1);
    let map = LanguageMap::derive(grid, ".+0102||xx").unwrap();

    let expressions = map.expressions().collect::<Vec<_>>();
    assert_eq!(expressions.len(), 2);
    assert_eq!(expressions[0].span().positions().count(), 6);
    assert!(expressions[0].root().is_some());
    assert_eq!(expressions[1].span().positions().count(), 4);
    assert!(expressions[1].root().is_none());
    assert_eq!(
        map.units()
            .filter(|unit| unit.kind() == LanguageUnitKind::Comment)
            .map(|unit| unit.anchor().x())
            .collect::<Vec<_>>(),
        vec![6]
    );
    assert_eq!(map.diagnostics().count(), 0);
    for column in 6..10 {
        assert_eq!(
            map.glyph_at(grid.position(column, 0).unwrap()),
            Some(Glyph::Comment)
        );
    }
}

/// A root whose operand slot holds a Comment introducer stays inert, and an
/// unrelated root on another row still executes and still writes.
#[test]
fn a_root_refused_by_a_comment_introducer_stays_inert_while_an_unrelated_root_executes() {
    let grid = Grid::new(12, 3);
    let mut source = Source::new(grid);
    for (row, text) in [(0, "      .+0102"), (1, ".+01||xx")] {
        for (column, byte) in text.bytes().enumerate() {
            source
                .set(
                    grid.index(grid.position(column, row).unwrap()),
                    &char::from(byte).to_string(),
                )
                .unwrap();
        }
    }
    let plan = source.execute(lang::Tick::ZERO);
    assert!(source.language_map().expressions().any(|expression| {
        expression
            .span()
            .positions()
            .next()
            .is_some_and(|position| position.x() == 0 && position.y() == 1)
            && expression.root().is_none()
    }));
    // The refusal is the row's own and it names the operand that caused it:
    // an Expression that reports never becomes a computation, so it is the
    // Map that holds the diagnostic, and the `||` is what failed to bind.
    assert!(
        source
            .language_map()
            .diagnostics()
            .any(|diagnostic| diagnostic.anchor().y() == 1
                && diagnostic.message == "expected a number, found \"||\"")
    );
    assert_eq!(
        source.get(grid.cell_index(18).unwrap()).as_deref(),
        Some("0")
    );
    assert_eq!(
        source.get(grid.cell_index(19).unwrap()).as_deref(),
        Some("3")
    );
    assert!(plan.writes.iter().all(|write| write.cell.get() < 24));
}

#[test]
fn long_expressions_keep_their_complete_ownership() {
    for source in [
        ".+".repeat(33) + ".=0101",
        ".+".repeat(33) + &"01".repeat(34) + ".=0101",
    ] {
        let grid = Grid::new(source.len(), 1);
        let map = LanguageMap::derive(grid, &source).unwrap();
        let expressions = map.expressions().collect::<Vec<_>>();
        let first = expressions[0];
        let expected_end = if source.len() == 72 { 71 } else { 133 };
        assert_eq!(first.span().positions().last().unwrap().x(), expected_end);
        assert_eq!(
            first.root(),
            if source.len() == 72 {
                None
            } else {
                grid.position(0, 0)
            }
        );
        assert_eq!(
            map.expressions()
                .filter_map(|entry| entry.root())
                .collect::<Vec<_>>(),
            if source.len() == 72 {
                vec![]
            } else {
                vec![grid.position(0, 0).unwrap(), grid.position(134, 0).unwrap()]
            }
        );
        if source.len() != 72 {
            assert_eq!(map.diagnostics().count(), 0);
        }
    }
}

#[test]
fn long_expressions_execute_and_keep_independent_roots() {
    let row = ".+".repeat(33) + &"01".repeat(34) + ".=0101";
    let grid = Grid::new(row.len(), 2);
    let mut source = Source::new(grid);
    for (index, byte) in row.bytes().enumerate() {
        source
            .set(
                grid.cell_index(index).unwrap(),
                &char::from(byte).to_string(),
            )
            .unwrap();
    }
    let plan = source.execute(lang::Tick::ZERO);
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(
        source.get(grid.cell_index(140).unwrap()).as_deref(),
        Some("2")
    );
    assert_eq!(
        source.get(grid.cell_index(141).unwrap()).as_deref(),
        Some("2")
    );
    assert_eq!(
        source.get(grid.cell_index(274).unwrap()).as_deref(),
        Some("*")
    );
    assert_eq!(
        source.get(grid.cell_index(275).unwrap()).as_deref(),
        Some("*")
    );
}

#[test]
fn language_map_derives_row_confined_expressions_with_roots_and_nested_functions() {
    let grid = Grid::new(10, 2);
    let map = LanguageMap::derive(grid, ".+.x010203**        ").unwrap();
    let expressions = map.expressions().collect::<Vec<_>>();

    assert_eq!(expressions.len(), 2);
    assert_eq!(expressions[0].root(), grid.position(0, 0));
    assert_eq!(expressions[1].root(), None);
    assert_eq!(
        map.expression_units(expressions[0])
            .iter()
            .map(|unit| unit.kind())
            .collect::<Vec<_>>(),
        vec![
            LanguageUnitKind::Function(Function::Add),
            LanguageUnitKind::Function(Function::Multiply),
            // The operands stay literals. `01` spells a Number here and a Note
            // in a Note slot; the Source does not carry that decision.
            LanguageUnitKind::OperandLiteral,
            LanguageUnitKind::OperandLiteral,
            LanguageUnitKind::OperandLiteral,
        ]
    );
    assert_eq!(
        map.units().nth(2).unwrap().kind(),
        LanguageUnitKind::OperandLiteral
    );
    assert!(
        expressions[0]
            .span()
            .positions()
            .all(|position| position.y() == 0)
    );
    assert!(
        expressions[1]
            .span()
            .positions()
            .all(|position| position.y() == 1)
    );
}

#[test]
fn language_map_reports_literal_incomplete_and_invalid_outcomes() {
    // `00` is not a Function spelling, so it is two refused Cells rather than
    // one Expression: each is diagnosed where a Function was expected, and
    // each is diagnosed again as a Cell that spells no Language Unit.
    let literal_grid = Grid::new(2, 1);
    let literal = LanguageMap::derive(literal_grid, "00").unwrap();
    assert_eq!(literal.expressions().next().unwrap().root(), None);
    assert_eq!(literal.diagnostics().count(), 4);

    let incomplete_grid = Grid::new(4, 1);
    let incomplete = LanguageMap::derive(incomplete_grid, ".+01").unwrap();
    assert_eq!(
        incomplete.diagnostics().next().unwrap().message,
        "expected a token"
    );

    let invalid_grid = Grid::new(2, 1);
    let invalid = LanguageMap::derive(invalid_grid, "xx").unwrap();
    assert_eq!(
        invalid.diagnostics().next().unwrap().message,
        "unknown function \"xx\""
    );

    let source = ".+".repeat(16) + "00";
    let long_incomplete_grid = Grid::new(source.len(), 1);
    let long_incomplete = LanguageMap::derive(long_incomplete_grid, &source).unwrap();
    assert_eq!(
        long_incomplete.diagnostics().next().unwrap().message,
        "expected a token"
    );
}

#[test]
fn unmatched_characters_have_revision_consistent_diagnostic_spans() {
    let grid = Grid::new(3, 1);
    let map = LanguageMap::derive(grid, "***").unwrap();
    let unmatched = map
        .diagnostics()
        .find(|diagnostic| diagnostic.start() == 2)
        .unwrap();

    assert_eq!(
        unmatched.anchor(),
        grid.position(2, 0).expect("inside the Grid")
    );
    assert_eq!(
        unmatched.span().positions().collect::<Vec<_>>(),
        vec![grid.position(2, 0).unwrap()]
    );
}

#[test]
fn source_exposes_the_current_map_and_rebuilds_hints_and_diagnostics_on_edit() {
    let grid = Grid::new(6, 2);
    let mut source = Source::new(grid);
    let cell = |idx| grid.cell_index(idx).expect("inside the Grid");
    source.set(cell(4), ".").unwrap();
    source.set(cell(5), "+").unwrap();

    assert_eq!(
        source.language_map().glyph_at(grid.position(4, 0).unwrap()),
        Some(Glyph::Function)
    );
    assert_eq!(
        source.language_map().glyph_at(grid.position(5, 0).unwrap()),
        Some(Glyph::Function)
    );
    assert_eq!(
        source.language_map().glyph_at(grid.position(0, 1).unwrap()),
        None
    );
    assert_eq!(source.language_map().diagnostics().count(), 1);

    source.unset(cell(5));
    assert_eq!(
        source.language_map().glyph_at(grid.position(4, 0).unwrap()),
        Some(Glyph::Function)
    );
    assert_eq!(source.language_map().expressions().count(), 1);
    let diagnostic = source.language_map().diagnostics().next().unwrap();
    assert_eq!(
        diagnostic.anchor(),
        grid.position(4, 0).expect("inside the Grid")
    );
    assert_eq!(
        diagnostic.span().positions().collect::<Vec<_>>(),
        vec![grid.position(4, 0).unwrap()]
    );
}

#[test]
#[should_panic(expected = "ExpressionEntry belongs to another LanguageMap")]
fn expression_units_refuses_an_expression_from_another_revision() {
    // Two revisions of the same Source share a Grid, so an extent minted by
    // one is a valid extent in the other. Without a revision identity the
    // foreign Expression is silently answered with this Map's own units:
    // `[Function(Add), OperandLiteral]` becomes `[OperandLiteral]`.
    let grid = Grid::new(10, 1);
    let first = LanguageMap::derive(grid, ".+01      ").unwrap();
    let second = LanguageMap::derive(grid, ".*0203    ").unwrap();
    let expression = first.expressions().next().unwrap();

    second.expression_units(expression);
}

#[test]
#[should_panic(expected = "Position belongs to another Grid")]
fn glyph_at_refuses_a_position_minted_by_another_grid() {
    // `glyph_at` and its sibling `SourceRevision::content_at` sit either side
    // of one render loop and have to teach the same rule about the same
    // argument. Without the refusal, a Position from another Grid reads this
    // Map's Glyph for whatever Cell the coordinates happen to land on, or
    // `None` once they land past the end — a plausible wrong answer either way.
    //
    // Two Grids of the same shape, because identity is what is being tested:
    // the coordinates are perfectly valid here, and that is the point.
    let grid = Grid::new(4, 1);
    let map = LanguageMap::derive(grid, ".+01").unwrap();
    let foreign = Grid::new(4, 1).position(0, 0).expect("inside the Grid");

    map.glyph_at(foreign);
}
