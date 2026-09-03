use lang::Function;
use orcvs::{
    glyph::Glyph,
    grid::Grid,
    source::{LanguageMap, LanguageUnitKind, Source},
};

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
fn language_map_reports_literal_incomplete_invalid_and_over_capacity_outcomes() {
    let literal_grid = Grid::new(2, 1);
    let literal = LanguageMap::derive(literal_grid, "00").unwrap();
    assert_eq!(literal.expressions().next().unwrap().root(), None);
    assert_eq!(literal.diagnostics().count(), 1);

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
    let over_capacity_grid = Grid::new(source.len(), 1);
    let over_capacity = LanguageMap::derive(over_capacity_grid, &source).unwrap();
    assert_eq!(
        over_capacity.diagnostics().next().unwrap().message,
        "expression exceeds the parser capacity of 32 atoms"
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
        Some(Glyph::Char)
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
