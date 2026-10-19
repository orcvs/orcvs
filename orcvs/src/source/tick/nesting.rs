//! Nested Functions: Returns and Portals decode alike, a nested Function keeps
//! its state in its own Output Portal, and a nested Bang activates its aligned
//! roots.

use crate::grid::Grid;

use super::fixture::{messages, tick_by_tick};
use super::observed::raw;

#[test]
fn a_return_and_a_portal_decode_the_same_characters_the_same_way() {
    // `.^3C` answers Note `C4`. Returned into Addition's Number operand,
    // and written there by a Portal, the same two characters are Number
    // `C4` either way: both sums are `C5`. The nested `.^` also writes its
    // own `C4` one row south of its anchor.
    let (plans, returned, _) = tick_by_tick(Grid::with_shape(8, 2), &[".+.^3C01", ""], 1);
    assert_eq!(returned[0], [".+.^3C01", "C5C4    "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
    let (plans, written, _) = tick_by_tick(Grid::with_shape(8, 3), &["  .^3C", ".+0001", ""], 1);
    assert_eq!(written[0], ["  .^3C  ", ".+C401  ", "C5      "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );

    // The receiving operand decides the reading, so a conversion nested in
    // one of the same direction reads its child's result as the other
    // type and diagnoses, as the same characters written by a Portal do.
    for (returned, child, written, message) in [
        (
            [".v.vC4", ""],
            "  3C  ",
            ["  .vC4", ".vC4", ""],
            "expected a note, found \"3C\"",
        ),
        (
            [".^.^3C", ""],
            "  C4  ",
            ["  .^3C", ".^3C", ""],
            "Number C4 cannot be converted to a Note",
        ),
    ] {
        let (plans, grids, _) = tick_by_tick(Grid::with_shape(6, 2), &returned, 1);
        assert_eq!(messages(&plans[0]), [message], "{returned:?}");
        assert_eq!(grids[0][1], child, "{returned:?}");
        let (plans, _, _) = tick_by_tick(Grid::with_shape(6, 3), &written, 1);
        assert_eq!(messages(&plans[0]), [message], "{written:?}");
    }

    // Conversions in opposite directions compose through the Return.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(6, 2), &[".v.^3C", ""], 1);
    assert_eq!(grids[0], [".v.^3C", "3CC4  "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(6, 2), &[".^.vC4", ""], 1);
    assert_eq!(grids[0], [".^.vC4", "C43C  "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

#[test]
fn nested_feedback_functions_keep_their_state_in_their_own_output_portals() {
    // A nested Increment reads its previous value from the Cells its own
    // Output Portal writes, so it advances every Tick and wraps at its
    // modulus; its parent adds `00` and writes the same count beside it.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(10, 2), &[".+~+010400", ""], 5);
    let counted: Vec<&str> = grids.iter().map(|grid| grid[1].as_str()).collect();
    assert_eq!(
        counted,
        [
            "0101      ",
            "0202      ",
            "0303      ",
            "0000      ",
            "0101      "
        ]
    );
    assert!(plans.iter().all(|plan| plan.diagnostics.is_empty()));

    // A nested Interpolation moves toward its target by its rate each Tick.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(10, 2), &[".+~>020500", ""], 4);
    let moved: Vec<&str> = grids.iter().map(|grid| grid[1].as_str()).collect();
    assert_eq!(
        moved,
        ["0202      ", "0404      ", "0505      ", "0505      "]
    );
    assert!(plans.iter().all(|plan| plan.diagnostics.is_empty()));
}

#[test]
fn a_nested_bang_writes_its_bang_and_activates_its_aligned_roots() {
    // Nested Equality answers Bang: it writes `**` one row south of its
    // own anchor, which activates the Terminal Output root aligned east of
    // that `**`. Its parent reads the returned `**` as a Number operand
    // and diagnoses, which does not undo the child's write or activation.
    let (plans, grids, _) = tick_by_tick(
        Grid::with_shape(12, 3),
        &[".+.=010101", "    !>007FC4", ""],
        1,
    );
    assert_eq!(grids[0], [".+.=010101  ", "  **!>007FC4", "            "]);
    assert_eq!(plans[0].play_commands, [raw(0, 0x7F, 60)]);
    assert_eq!(messages(&plans[0]), ["expected a number, found \"**\""]);
}

#[test]
fn a_nested_bang_activates_the_root_aligned_south_of_its_bang() {
    // The `**` stands at the Equality's Output Portal, one row south of
    // its anchor, so the root one row further south is aligned with it.
    // The north anchor is the Equality's own, inside its parent
    // Expression, so no root can stand there.
    let (plans, grids, _) = tick_by_tick(
        Grid::with_shape(10, 3),
        &[".+.=010101", "", "  !>007FC4"],
        1,
    );
    assert_eq!(grids[0], [".+.=010101", "  **      ", "  !>007FC4"]);
    assert_eq!(plans[0].play_commands, [raw(0, 0x7F, 60)]);
    assert_eq!(messages(&plans[0]), ["expected a number, found \"**\""]);
}

#[test]
fn a_nested_bang_activates_the_root_aligned_west_of_its_bang() {
    // A west root anchors two columns before the `**`, so only a
    // two-Cell Expression leaves the `**`'s Cells to it. The South
    // Directional Bang there is inert until Bang activation, and the
    // `vv` it emits is the trace that the `**` activated it.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(10, 3), &[".+.=010101", "*v", ""], 1);
    assert_eq!(grids[0], [".+.=010101", "*v**      ", "vv        "]);
    assert_eq!(messages(&plans[0]), ["expected a number, found \"**\""]);
}

#[test]
fn an_unactivated_west_root_beside_a_nested_number_emits_nothing() {
    // The control for the West test: the same root beside a nested Number
    // rather than a Bang stays inert. The parent fails as it does there,
    // so its own answer does not land on the root either.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(10, 3), &["./.+010100", "*v", ""], 1);
    assert_eq!(grids[0], ["./.+010100", "*v02      ", "          "]);
    assert_eq!(messages(&plans[0]), ["cannot divide by zero"]);
}

#[test]
fn a_child_write_survives_its_parents_failure() {
    // Divide refuses a zero divisor after the nested Multiply has written
    // `0C` under its own anchor and returned it.
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(10, 2), &["./.x030400", ""], 1);
    assert_eq!(grids[0], ["./.x030400", "  0C      "]);
    assert_eq!(messages(&plans[0]), ["cannot divide by zero"]);
}

#[test]
fn a_nested_output_portal_orders_an_earlier_consumer_after_its_writer() {
    // The nested North Copy copies `05` from below it into the second
    // operand of the Addition on the row above. That Addition comes first
    // in Grid order, so only the Copy's reservation orders it after the
    // write; read in Grid order it would execute over `00` and the Copy's
    // write would then reach a computation that had already run.
    let (plans, grids, _) =
        tick_by_tick(Grid::with_shape(8, 3), &[".+0000", "  .+=^01", "    05"], 1);
    assert_eq!(grids[0], [".+0005  ", "05.+=^01", "  0605  "]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
}

///
/// Generated nested Expressions through the shipped Tick. `lang` evaluates one
/// Function over resolved operands; resolving nested operands is this
/// module's, so the breadth of nested shapes is proved here, against the
/// planning path a Playback Tick takes.
///
/// The `cfg` matches the `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]`
/// table that declares proptest, so a WASM build never sees the dependency.
///
#[cfg(not(target_arch = "wasm32"))]
mod nested_property {
    use crate::grid::{COL_COUNT, Grid};
    use crate::source::language_map::LanguageMap;
    use crate::source::tick::observed::{diagnostic, first};
    use crate::source::tick::{Cells, Tick, plan};
    use lang::{Atom, Function, Note, Parser, Token, Tokens};
    use proptest::collection::vec;
    use proptest::prelude::*;
    use proptest::sample::select;
    use proptest::test_runner::{Config, TestRunner};
    /// Generation budgets only; neither constrains accepted Source.
    const NESTING: u32 = 3;
    const CHAIN_LENGTH: usize = 24;

    fn apply(function: Function, operands: &[String]) -> String {
        let mut source = function.to_string();
        source.extend(operands.iter().map(String::as_str));
        source
    }

    /// One Operand Literal of the type its position declares. Reading the
    /// Token keeps every generated Expression parseable: a literal of the
    /// wrong type is refused as Source text rather than diagnosed.
    fn literal_source(token: Token) -> BoxedStrategy<String> {
        match token {
            Token::Number => any::<u8>()
                .prop_map(|number| format!("{number:02X}"))
                .boxed(),
            Token::Note => (0x00u8..=0x7F)
                .prop_map(|note| Atom::Note(Note::try_from(note).unwrap()).to_string())
                .boxed(),
            // An untyped slot carries any Language Unit a literal spells.
            Token::Untyped => prop_oneof![
                literal_source(Token::Number),
                literal_source(Token::Note),
                Just("**".to_owned()),
            ]
            .boxed(),
            other => panic!("no operand is declared as {other:?}"),
        }
    }

    /// Every Value Function, read from the definitions so a Function added
    /// later is generated the day it exists.
    fn value_functions() -> Vec<Function> {
        Function::ALL
            .iter()
            .copied()
            .filter(|function| function.answers_value())
            .collect()
    }

    fn binary_value_functions() -> Vec<Function> {
        value_functions()
            .into_iter()
            .filter(|function| {
                let signature = Tokens::from(function);
                signature.len() == 2 && signature.iter().all(|token| *token == Token::Number)
            })
            .collect()
    }

    /// A left-leaning chain of binary Value Functions: prefix order puts every
    /// Function ahead of every operand, the deepest nesting per Cell.
    fn chain_source() -> BoxedStrategy<String> {
        prop_oneof![4 => 1usize..4, 1 => 1usize..=CHAIN_LENGTH]
            .prop_flat_map(|length| {
                (
                    vec(select(binary_value_functions()), length),
                    vec(literal_source(Token::Number), length + 1),
                )
            })
            .prop_map(|(functions, literals)| {
                let mut source: String = functions.iter().map(Function::to_string).collect();
                source.extend(literals.iter().map(String::as_str));
                source
            })
            .boxed()
    }

    fn operand_source(token: Token, depth: u32) -> BoxedStrategy<String> {
        if depth == 0 {
            return literal_source(token);
        }
        prop_oneof![
            5 => literal_source(token),
            2 => chain_source(),
            3 => nested_source(depth),
        ]
        .boxed()
    }

    fn nested_source(depth: u32) -> BoxedStrategy<String> {
        select(value_functions())
            .prop_flat_map(move |function| {
                let operands: Vec<BoxedStrategy<String>> = Tokens::from(&function)
                    .into_iter()
                    .map(|token| operand_source(token, depth - 1))
                    .collect();
                (Just(function), operands)
            })
            .prop_map(|(function, operands)| apply(function, &operands))
            .boxed()
    }

    /// One whole Expression, including terminal and effect roots, that fits a
    /// row of the widest Grid.
    fn expression_source() -> BoxedStrategy<String> {
        select(Function::ALL)
            .prop_flat_map(|function| {
                let operands: Vec<BoxedStrategy<String>> = Tokens::from(&function)
                    .into_iter()
                    .map(|token| operand_source(token, NESTING))
                    .collect();
                (Just(function), operands)
            })
            .prop_map(|(function, operands)| apply(function, &operands))
            .prop_filter("fits one Grid row", |source| source.len() <= COL_COUNT)
            .boxed()
    }

    /// One fixed Operand Literal of the type `token` names.
    fn literal(token: Token) -> &'static str {
        match token {
            Token::Number => "01",
            Token::Note => "C4",
            Token::Untyped => "G4",
            other => panic!("no literal spells {other:?}"),
        }
    }

    /// The most operands any Function declares: one Function over literals is
    /// one Interpreter call, so a Tick making more calls than this has
    /// evaluated a nested Expression.
    fn widest_signature() -> usize {
        Function::ALL
            .iter()
            .map(|function| Tokens::from(function).len())
            .max()
            .expect("the definitions declare at least one Function")
    }

    /// Plans one Tick over `source` in the middle row of a Grid, so Portals
    /// north and south of it fall inside the Grid, and checks what every
    /// accepted Expression owes: no Operand Stack is exhausted, and an active
    /// root reaches the Interpreter or the Tick says why not. Answers how many
    /// Interpreter calls the Tick made.
    fn settle(source: &str) -> Result<usize, TestCaseError> {
        prop_assert!(
            Parser::from(source).try_parse().is_ok(),
            "{source:?} failed to parse"
        );
        let width = source.len().max(2);
        let grid = Grid::with_shape(width, 3);
        let bytes = format!("{:width$}{source:width$}{:width$}", "", "");
        let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));

        let (tick, states) = plan(grid, Cells::of(bytes.as_bytes()), &map, Tick::ZERO);
        let tick = tick.into_plan();

        prop_assert!(
            !tick
                .diagnostics
                .iter()
                .any(|d| d.message.starts_with("the Operand Stack cannot hold")),
            "{source:?} exhausted an Operand Stack: {:?}",
            tick.diagnostics
        );
        // An Expression that parsed has no syntax error for a block to defer
        // to. A Copy inside it that copies empty Cells into the operand chain
        // leaves a slot unwritten, which makes the root invalid and diagnoses.
        let (_, root) = map
            .expressions()
            .find_map(|expression| expression.function_candidate())
            .expect("a parsed Expression has a leading Function");
        if root.is_intrinsically_active() {
            prop_assert!(
                states
                    .first()
                    .is_some_and(|root| root.interpreted().is_some())
                    || !tick.diagnostics.is_empty(),
                "{source:?} left its active root unanswered and undiagnosed"
            );
        }
        Ok(states
            .iter()
            .filter(|state| state.interpreted().is_some())
            .count())
    }

    ///
    /// A Tick over any nested Expression the Parser accepts returns rather
    /// than panicking, never exhausts an Operand Stack, and settles an active
    /// root: the root reaches the Interpreter, or the Tick says why not.
    ///
    #[test]
    fn a_tick_over_every_nested_expression_the_parser_accepts_settles_its_root() {
        // Naming the source file is what `proptest!` would have done, so a
        // regression file is written for a failing case.
        let config = Config {
            source_file: Some(file!()),
            ..Config::default()
        };

        TestRunner::new(config)
            .run(&expression_source(), |source| settle(&source).map(drop))
            .unwrap_or_else(|error| panic!("{error}"));
    }

    ///
    /// A nested Copy whose input is empty, deep inside a nested Expression,
    /// parses, exhausts no Operand Stack, and leaves its active root either
    /// interpreted or diagnosed. The generators draw from
    /// `Function::ALL`, so a stored seed replays a different case whenever the
    /// table changes length, and this shape is stated here instead.
    ///
    #[test]
    fn a_nested_copy_of_empty_cells_leaves_its_root_answered_or_diagnosed() {
        settle(".-.|00./.|0000=^~*~>.-~>6C56D82B.+AC0A").unwrap_or_else(|error| panic!("{error}"));
    }

    ///
    /// A nested dynamic Write whose destination is its own root's spelling
    /// suppresses that root, and the Tick diagnoses the Write. `@<` answers
    /// 229 and writes `E5` over `.<`.
    ///
    #[test]
    fn a_nested_write_over_its_own_root_is_diagnosed_at_the_write() {
        let source = ".<.|00.v~?000000=$35BA@<.x64FBE506";
        settle(source).unwrap_or_else(|error| panic!("{error}"));

        let width = source.len();
        let grid = Grid::with_shape(width, 3);
        let tick = first(grid, &["", source, ""]);
        assert_eq!(&tick.rows[1][..2], "E5");
        assert_eq!(
            tick.diagnostics,
            [diagnostic(
                22,
                1,
                "dynamic output reached its own Expression"
            )]
        );
        let bytes = format!("{:width$}{source}{:width$}", "", "");
        let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));
        let (_, states) = plan(grid, Cells::of(bytes.as_bytes()), &map, Tick::ZERO);
        assert_eq!(states[0].interpreted(), None);
    }

    ///
    /// The property's checks hold where nesting is certain rather than drawn:
    /// every intrinsically active root over a chain one link longer than the
    /// widest signature makes more Interpreter calls than any flat Expression
    /// could. A random batch at the pull-request tier's case count need not
    /// contain such a shape, so it is stated here.
    ///
    #[test]
    fn a_tick_over_a_chain_deeper_than_any_signature_interprets_every_link() {
        let widest = widest_signature();
        let chain = ".+".repeat(widest) + &"01".repeat(widest + 1);
        let roots: Vec<(Function, String)> = Function::ALL
            .iter()
            .copied()
            .filter(|root| root.is_intrinsically_active())
            .filter_map(|root| {
                // Only roots whose operands are all Numbers and Notes are
                // stated here.
                let signature = Tokens::from(&root);
                if !signature
                    .iter()
                    .all(|token| matches!(token, Token::Number | Token::Note))
                {
                    return None;
                }
                let first = signature.iter().position(|token| *token == Token::Number)?;
                let operands: Vec<String> = signature
                    .iter()
                    .enumerate()
                    .map(|(slot, token)| {
                        if slot == first {
                            chain.clone()
                        } else {
                            literal(*token).to_owned()
                        }
                    })
                    .collect();
                Some((root, apply(root, &operands)))
            })
            .collect();
        assert!(!roots.is_empty(), "no active root takes a Number operand");

        for (root, source) in roots {
            let interpreted = settle(&source).unwrap_or_else(|error| panic!("{error}"));
            assert!(
                interpreted > widest,
                "{root} over {source:?} made {interpreted} Interpreter calls, which a flat \
                 Expression of {widest} operands could make",
            );
        }
    }
}
