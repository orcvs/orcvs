use lang::{Atom, Function};
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
        expressions[0]
            .units()
            .map(|unit| unit.kind())
            .collect::<Vec<_>>(),
        vec![
            LanguageUnitKind::Function(Function::Add),
            LanguageUnitKind::Function(Function::Multiply),
            LanguageUnitKind::Atom(Atom::Number(1)),
            LanguageUnitKind::Atom(Atom::Number(2)),
            LanguageUnitKind::Atom(Atom::Number(3)),
        ]
    );
    assert_eq!(
        map.units().nth(2).unwrap().kind(),
        LanguageUnitKind::Atom(Atom::Number(1))
    );
    assert!(
        expressions[0]
            .footprint()
            .positions()
            .all(|position| position.y() == 0)
    );
    assert!(
        expressions[1]
            .footprint()
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
fn unmatched_characters_have_revision_consistent_diagnostic_footprints() {
    let grid = Grid::new(3, 1);
    let map = LanguageMap::derive(grid, "***").unwrap();
    let unmatched = map
        .diagnostics()
        .find(|diagnostic| diagnostic.start == 2)
        .unwrap();

    assert_eq!(unmatched.anchor(), grid.position(2, 0));
    assert_eq!(
        unmatched.footprint().positions().collect::<Vec<_>>(),
        vec![grid.position(2, 0).unwrap()]
    );
}

#[test]
fn source_exposes_the_current_map_and_rebuilds_hints_and_diagnostics_on_edit() {
    let grid = Grid::new(6, 2);
    let mut source = Source::new(grid);
    source.set(4, ".").unwrap();
    source.set(5, "+").unwrap();

    assert_eq!(source.get_glyph_at(4), Some(Glyph::Function));
    assert_eq!(source.get_glyph_at(5), Some(Glyph::Function));
    assert_eq!(source.get_glyph_at(6), None);
    assert_eq!(source.language_map().diagnostics().count(), 1);

    source.unset(5).unwrap();
    assert_eq!(source.get_glyph_at(4), Some(Glyph::Char));
    assert_eq!(source.language_map().expressions().count(), 1);
    let diagnostic = source.language_map().diagnostics().next().unwrap();
    assert_eq!(diagnostic.anchor(), grid.position(4, 0));
    assert_eq!(
        diagnostic.footprint().positions().collect::<Vec<_>>(),
        vec![grid.position(4, 0).unwrap()]
    );
}
