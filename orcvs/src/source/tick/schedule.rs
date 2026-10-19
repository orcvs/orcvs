//! What a schedule computes: the Function walk, the per-computation tables, and
//! the stated-destination fixture the other topics drive Ticks through.

use crate::grid::Grid;
use lang::Atom;

use super::fixture::{carried_source, seeded_source, snapshot, stated_source};

/// Naming a Cell for a silent Terminal Output Function cannot mint a
/// Portal. The Tick's only diagnostic is the row-edge layout `.+01` owes.
#[test]
fn a_carried_destination_on_terminal_output_does_not_displace_row_edge_layout() {
    let grid = Grid::with_shape(16, 4);
    let rows = ["!>007FC4", "", "", "            .+01"];

    let (plan, _) = carried_source(grid, &rows, &[(0, 16)]);

    assert_eq!(
        plan.diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>(),
        vec!["Expression layout crosses the row edge"]
    );
}

#[test]
fn a_stated_answer_faces_every_refusal_the_turn_it_replaces_faces() {
    // What a computation answers is the only thing a stated answer states.
    // Whether it answers at all is the Turn's to decide, and a Turn its own
    // prologue refuses has no answer to deliver, stated or interpreted. A
    // seam that delivered past those refusals would let these tests assert
    // outcomes production cannot produce.

    // An Addition whose second operand is Source it cannot read: the Turn
    // is syntax-blocked, which settles it without a Tick diagnostic.
    let grid = Grid::with_shape(16, 2);
    let rows = [".+02", ""];
    let (plan, source) = stated_source(grid, &rows, &[(0, 16)], &[(0, Atom::Number(1))]);
    assert!(plan.writes.is_empty(), "{:?}", plan.writes);
    assert!(plan.play_commands.is_empty());
    assert_eq!(source.snapshot(), snapshot(grid, &rows));
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);

    // A Terminal Output Function standing inside an Expression: the
    // Parser refused it from the Source, so its Turn is blocked, and the
    // Addition above it with it, without a Tick diagnostic.
    let rows = [".+01!>007FC4", ""];
    let (plan, source) = stated_source(grid, &rows, &[(0, 16)], &[(4, Atom::Number(1))]);
    assert!(plan.writes.is_empty(), "{:?}", plan.writes);
    assert!(plan.play_commands.is_empty());
    assert_eq!(source.snapshot(), snapshot(grid, &rows));
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert!(source.language_map().diagnostics().any(|d| {
        d.message
            .contains("valid only at the root of an Expression")
    }),);
}

#[test]
fn the_function_walk_yields_what_the_schedule_computes_in_preorder() {
    // Root and nested Functions, in row, Source and then Parser order. The
    // bare Number has no leading Function, so no schedule computes it.
    let grid = Grid::with_shape(16, 3);
    let source = seeded_source(grid, &[".+~+0104", "01", "  .+.+010203"]);
    let map = source.shared_language_map();
    let inputs = map.schedule_inputs();
    let at = |x, y| grid.position(x, y).expect("inside the Grid");

    let walked: Vec<_> = inputs
        .functions(grid)
        .map(|site| (site.anchor, site.function))
        .collect();

    assert_eq!(
        walked,
        vec![
            (at(0, 0), lang::Function::Add),
            (at(2, 0), lang::Function::Increment),
            (at(2, 2), lang::Function::Add),
            (at(4, 2), lang::Function::Add),
        ]
    );
    let (nodes, _) = super::computations(grid, inputs);
    assert!(
        walked
            .iter()
            .copied()
            .eq(nodes.iter().map(|node| (node.anchor, node.function))),
        "the walk and the schedule hold the same Functions in the same order",
    );
}

#[test]
fn a_table_per_computation_is_read_and_written_by_computation_id() {
    let [first, second, third] = [0, 1, 2].map(super::ComputationId);
    let mut table: super::PerComputation<char> = ['a', 'b', 'c'].into_iter().collect();

    assert_eq!(table.len(), 3);
    assert_eq!(table.ids().collect::<Vec<_>>(), [first, second, third]);
    assert_eq!(table[second], 'b');
    table[third] = 'z';
    assert_eq!(
        table.iter_enumerated().collect::<Vec<_>>(),
        [(first, &'a'), (second, &'b'), (third, &'z')]
    );
    let subtree = super::ComputationRange::new(second, 3);
    assert_eq!(subtree.clone().collect::<Vec<_>>(), [second, third]);
    assert!(subtree.contains(third) && !subtree.contains(first));
    assert_eq!(&table[subtree], ['b', 'z']);
    assert_eq!(table.into_vec(), ['a', 'b', 'z']);
}

#[test]
fn a_nested_computation_is_owned_by_its_root_and_answers_its_parent() {
    // `.+.+.+01020304` nests three Additions: the innermost answers the
    // middle one, and both nested ones belong to the root's Expression.
    let grid = Grid::with_shape(16, 1);
    let source = seeded_source(grid, &[".+.+.+01020304"]);
    let (nodes, _) = super::computations(grid, source.shared_language_map().schedule_inputs());
    let [root, middle, inner] = [0, 1, 2].map(super::ComputationId);

    assert_eq!(nodes.len(), 3);
    assert_eq!(nodes[root].place, super::Place::Root);
    assert_eq!(nodes[root].owner(), root);
    assert_eq!(
        nodes[middle].place,
        super::Place::Nested {
            parent: root,
            owner: root
        }
    );
    assert_eq!(
        nodes[inner].place,
        super::Place::Nested {
            parent: middle,
            owner: root
        }
    );
    assert_eq!(nodes[inner].parent(), Some(middle));
    assert_eq!(nodes[middle].operands[0].child, Some(inner));
    assert_eq!(nodes[inner].operands[0].child, None);
}

#[test]
fn stated_destinations_reach_computations_without_a_terminal_output_portal() {
    // A Self-Banging Function states its displaced destination from its
    // declaration; a Terminal Output Function resolves none. The two share
    // a schedule through [`computations`], not through [`carry`].
    let grid = Grid::with_shape(16, 3);
    let source = seeded_source(grid, &["^^", "", "!>007FC4"]);
    let (nodes, diagnostics) =
        super::computations(grid, source.shared_language_map().schedule_inputs());

    let self_banging = nodes
        .iter()
        .find(|node| node.function == lang::Function::SelfBangingNorth)
        .expect("Self-Banging Function in schedule");
    let terminal = nodes
        .iter()
        .find(|node| node.function == lang::Function::RawPlay)
        .expect("Terminal Output Function in schedule");

    assert!(
        self_banging
            .portal_access
            .write_sites()
            .iter()
            .any(Result::is_ok),
        "a declared Source write states its Portal in computations",
    );
    assert!(
        !terminal.portal_access.writes_cells(),
        "a Terminal Output Function acquires no Portal",
    );
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.message.contains("Portal")),
        "no refusal when only Source-writing Functions state destinations",
    );
}

#[test]
#[should_panic(expected = "a stated answer names a computation the schedule contains")]
fn a_cyclic_source_does_not_excuse_a_fixture_error() {
    // A Source that admits no order publishes diagnostics and nothing
    // else, which is indistinguishable from what a mistyped fixture makes
    // happen — so a fixture error checked after ordering would be the one
    // check a cycle switches off. The Addition writes over its own operand
    // and the answer names an operand Cell: both are wrong, and the one
    // the fixture author can fix is the one reported.
    let grid = Grid::with_shape(16, 2);
    stated_source(grid, &[".+0102", ""], &[(0, 2)], &[(4, Atom::Number(1))]);
}

#[test]
#[should_panic(expected = "two are stated here for the same anchor")]
fn two_answers_at_one_anchor_are_a_fixture_error() {
    // The second answer could only ever be dropped: one computation takes
    // one Turn. Reported as the duplicate it is rather than as the Turn
    // that appeared not to reach it.
    let grid = Grid::with_shape(16, 2);
    stated_source(
        grid,
        &[".+0203", ""],
        &[(0, 16)],
        &[(0, Atom::Number(1)), (0, Atom::Number(2))],
    );
}

#[test]
#[should_panic(expected = "a stated answer belongs to a computation that answers a value")]
fn a_stated_answer_at_a_terminal_output_root_is_a_fixture_error() {
    // A root Terminal Output Function is the one Function a Turn lets
    // answer no value at all, and what it answers is a Play Command. A
    // fixture stating a value there would have this seam plan a write no
    // Turn of that computation can produce.
    //
    // The Equality above it answers the Bang that activates it, because an
    // unactivated Terminal Output root takes no Turn at all and would be
    // refused a Cell before reaching the arm under test.
    let grid = Grid::with_shape(16, 3);
    stated_source(
        grid,
        &[".=0101", "", "!>007FC4"],
        &[(0, 16)],
        &[(32, Atom::Number(1))],
    );
}

#[test]
#[should_panic(expected = "a stated answer names a computation the schedule contains")]
fn a_stated_answer_at_no_computations_anchor_is_a_fixture_error() {
    // Cell 2 holds the Addition's first operand rather than a Function
    // anchor, so nothing in the schedule would ever state this answer.
    // Falling through to an ordinary Tick would leave a test asking
    // whether little happened and being told that it did, which is what
    // this refuses to do quietly.
    let grid = Grid::with_shape(16, 2);
    stated_source(grid, &[".+0203", ""], &[(0, 16)], &[(2, Atom::Number(1))]);
}

/// The first and last column of the Cells each computation's subtree holds
/// in `row`, in Parser preorder.
fn subtree_columns(width: usize, row: &str) -> Vec<(usize, usize)> {
    let grid = Grid::with_shape(width, 1);
    let source = seeded_source(grid, &[row]);
    let (nodes, _) = super::computations(grid, source.shared_language_map().schedule_inputs());
    let lookup = super::Lookup::new(grid, nodes);
    lookup
        .nodes()
        .ids()
        .map(|id| {
            let span = lookup.subtree_cells(id);
            let columns: Vec<_> = span.positions().map(|position| position.x()).collect();
            (columns[0], columns[columns.len() - 1])
        })
        .collect()
}

#[test]
fn a_subtree_holds_its_anchor_to_the_last_operand_any_of_its_computations_holds() {
    // Deep: each Read's subtree runs to the end of the innermost's operand.
    assert_eq!(subtree_columns(8, "&v&v&v01"), [(0, 7), (2, 7), (4, 7)]);
    // Wide: two sibling Reads hold their own Cells, and the Add holds both.
    assert_eq!(subtree_columns(10, ".+&v01&v02"), [(0, 9), (2, 5), (6, 9)]);
    // Nested, then a literal: the Add's last operand follows the nested Reads.
    assert_eq!(subtree_columns(10, ".+&v&v0102"), [(0, 9), (2, 7), (4, 7)]);
    // Missing operands at the row edge claim no Cells, so a subtree ends with
    // the last Cell the row holds of it.
    assert_eq!(subtree_columns(6, "  .+01"), [(2, 5)]);
    assert_eq!(subtree_columns(6, "    .+"), [(4, 5)]);
}
