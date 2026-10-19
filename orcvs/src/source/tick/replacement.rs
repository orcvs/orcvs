//! Function replacement: which replacements a running computation admits, and
//! what a refused or admitted one keeps.

use crate::grid::Grid;
use crate::render_frame::PortalRole;
use crate::source::TickPlan;
use lang::{Atom, ReplacementChange};

use super::PortalError;
use super::fixture::{carried_tick, interpreted, messages, stated_tick};
use super::observed::{committed_portals, painted, writes};
use super::portals::{PortalOrigin, WriteRefusal};

///
/// [`replaced_source`], with the inputs the Interpreter received for each
/// computation it ran for. Split from it for the reason [`carried_tick`]
/// gives.
///
fn replaced_tick(
    grid: Grid,
    rows: &[&str],
    outputs: &[(usize, usize)],
    replacements: &[(usize, lang::Function)],
) -> (TickPlan, Vec<lang::TickInputs>, crate::source::Source) {
    let (plan, states, source) = stated_tick(
        grid,
        rows,
        outputs,
        &replacements
            .iter()
            .map(|(anchor, function)| (*anchor, Atom::Function(*function)))
            .collect::<Vec<_>>(),
    );
    (plan, interpreted(&states), source)
}

///
/// One Tick in which the computations at `replacements` answer a Function
/// value, which nothing spells in Source.
///
fn replaced_source(
    grid: Grid,
    rows: &[&str],
    outputs: &[(usize, usize)],
    replacements: &[(usize, lang::Function)],
) -> (TickPlan, crate::source::Source) {
    let (plan, _, source) = replaced_tick(grid, rows, outputs, replacements);
    (plan, source)
}

#[test]
fn live_function_replacement_cannot_change_activation_or_output_kind() {
    let (plan, source) = replaced_source(
        Grid::with_shape(16, 3),
        &[".+0204", ".+0000", ""],
        &[(0, 32), (16, 0)],
        &[(16, lang::Function::RawPlay)],
    );
    assert_eq!(&source.snapshot()[..6], ".+0204");
    assert_eq!(&source.snapshot()[32..34], "06");
    assert!(
        plan.diagnostics
            .iter()
            .any(|d| { d.message.contains("whether it answers a value") })
    );
}

#[test]
fn a_replacement_that_changes_only_the_activation_source_is_refused() {
    // The activation-source term of that guard. Raw Play and `^^` differ on
    // where their activation comes from, and that is the first difference
    // the guard finds: neither answers a value, so the term ahead of it
    // agrees. They differ on the Source write as well — `^^` declares one
    // and Raw Play declares none — so this fixture changes two facts and is
    // named for the one that is reported.
    //
    // No pair changes activation alone. Every Function whose activation is
    // intrinsic either answers a value or declares a Source write, and no
    // Bang-activated Function does either, so a fixture that changed this
    // fact and nothing else cannot be written;
    // `every_declared_change_is_the_first_difference_for_some_pair` in
    // `lang` holds that. A guard asking `answers_value` for this
    // question would admit the replacement and leave the schedule holding
    // edges derived from a root that now needs no Bang.
    let (plan, source) = replaced_source(
        Grid::with_shape(16, 3),
        &[".+0000", "!>007FC4", ""],
        &[(0, 16)],
        &[(0, lang::Function::SelfBangingNorth)],
    );

    assert_eq!(&source.snapshot()[16..24], "!>007FC4");
    assert!(
        plan.diagnostics
            .iter()
            .any(|d| { d.message.contains("where its activation comes from") })
    );
}

#[test]
fn a_replacement_that_changes_only_bang_emission_is_refused() {
    // One fact of the five that a pair can differ on alone. Equality and
    // Addition agree on every other column the guard reads — both answer a
    // value, both are intrinsically active, neither declares a Source
    // write, both reserve a Cell pair — and Equality can answer Bang where
    // Addition never can.
    //
    // Scheduling reads that declaration to decide which roots can supply
    // activation, so admitting this replacement would leave the Turn run by
    // a Function that can answer Bang with no activation edge built from
    // it, and a neighbouring root waiting on a Bang the schedule never
    // ordered.
    let (plan, source) = replaced_source(
        Grid::with_shape(16, 3),
        &[".+0000", ".+0102", ""],
        &[(0, 16)],
        &[(0, lang::Function::Equality)],
    );

    assert_eq!(
        &source.snapshot()[16..22],
        ".+0102",
        "the refused replacement wrote no Cell, so the parsed Function stands"
    );
    assert!(
        plan.diagnostics
            .iter()
            .any(|d| { d.message.contains("whether it can emit Bang") }),
        "{:?}",
        plan.diagnostics
    );
}

#[test]
fn a_replacement_that_changes_only_the_declared_portal_offset_is_refused() {
    // Two Self-Banging Functions agree on every other column the guard
    // reads — neither answers a value, both are intrinsically active,
    // neither can return Bang, and both reserve a Cell pair — and they
    // differ only in the Portal offset they declare. The schedule reserved
    // the Cells `^^` declares, so admitting `>>` here would leave the Turn
    // writing at Cells no dependency edge names, which is the defect this
    // guard exists to refuse.
    let (plan, source) = replaced_source(
        Grid::with_shape(16, 3),
        &[".+0000", "^^", ""],
        &[(0, 16)],
        &[(0, lang::Function::SelfBangingEast)],
    );

    // `^^` stays the running Function and takes its own Turn, which the
    // Addition to its north blocks, so its Span is the Bang a blocked move
    // leaves. That is the assertion: an admitted replacement moves east
    // instead, leaving a space and a `>` here and writing the second `>`
    // one Cell outside the reservation.
    assert_eq!(&source.snapshot()[16..18], "**");
    assert!(
        plan.diagnostics
            .iter()
            .any(|d| { d.message.contains("the Source write it declares") }),
        "{:?}",
        plan.diagnostics
    );
}

#[test]
fn live_the_second_replacement_at_one_anchor_replaces_the_first() {
    // The guard reads the Function the computation is running, and the
    // only Tick that can tell that from the Function the Parser found is
    // one in which two replacements reach a single anchor. Both are
    // admitted here, so the Turn taken at column 0 is the second one's:
    // Multiplication, not the Subtraction that replaced the parsed
    // Addition before it.
    //
    // Reading the parsed Function would answer this the same way, and no
    // fixture can make the two disagree: this guard admits no replacement
    // that changes any of the five facts it compares, so the running
    // Function agrees with the parsed one on all five for as long as the
    // guard is the only thing that changes it. That agreement is an
    // invariant about the guard itself, held nowhere and by nothing else,
    // and reading the running Function is what stops it being load-bearing.
    let grid = Grid::with_shape(16, 3);
    let (plan, source) = replaced_source(
        grid,
        &[".+0503", ".x0405", ".-0607"],
        &[(0, 40), (16, 0), (32, 0)],
        &[
            (16, lang::Function::Subtract),
            (32, lang::Function::Multiply),
        ],
    );

    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(
        &source.snapshot()[..2],
        ".x",
        "the later producer wins the Cells both replacements wrote"
    );
    assert_eq!(
        &source.snapshot()[40..42],
        "0F",
        "the Turn ran the second replacement, not the first and not the parsed Function"
    );
}

#[test]
fn live_a_second_replacement_at_one_anchor_faces_the_same_refusal() {
    // The same anchor, reached twice, where the second replacement changes
    // the output kind: it is refused, its write is refused whole, and the
    // first replacement stands as the Function the Turn runs. The refusal
    // is stated against the Subtraction already in place rather than
    // against the parsed Addition; the two answer alike, which is the
    // invariant the test above records.
    let grid = Grid::with_shape(16, 3);
    let (plan, source) = replaced_source(
        grid,
        &[".+0503", ".x0405", ".-0607"],
        &[(0, 40), (16, 0), (32, 0)],
        &[
            (16, lang::Function::Subtract),
            (32, lang::Function::RawPlay),
        ],
    );

    assert!(
        plan.diagnostics
            .iter()
            .any(|d| { d.message.contains("whether it answers a value") }),
        "{:?}",
        plan.diagnostics
    );
    assert_eq!(
        &source.snapshot()[..2],
        ".-",
        "the refused replacement wrote no Cell, so the admitted one stands"
    );
    assert_eq!(
        &source.snapshot()[40..42],
        "02",
        "the Turn ran the replacement that was admitted"
    );
}

#[test]
fn live_deep_and_top_level_replacement_suppress_descendant_portals() {
    let (plan, interpreted, source) = carried_tick(
        Grid::with_shape(20, 3),
        &[".+02.x03.+0101", ".+0203", ""],
        &[(0, 40), (4, 44), (8, 48), (20, 4)],
    );
    assert_eq!(&source.snapshot()[..14], ".+020503.+0101");
    assert_eq!(&source.snapshot()[40..42], "07");
    assert_eq!(&source.snapshot()[44..50], "      ");
    assert_eq!(interpreted.len(), 2);
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert!(source.language_map().expressions().any(|entry| {
        entry
            .root()
            .is_some_and(|root| root.x() == 8 && root.y() == 0)
    }));
    let (plan, interpreted, source) = carried_tick(
        Grid::with_shape(16, 3),
        &[".+02.x0304", ".+0203", ""],
        &[(0, 32), (4, 36), (16, 0)],
    );
    assert_eq!(&source.snapshot()[..10], "0502.x0304");
    assert_eq!(&source.snapshot()[32..38], "      ");
    assert_eq!(interpreted.len(), 1);
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(
        source
            .language_map()
            .expressions()
            .find_map(|entry| entry.root())
            .unwrap()
            .x(),
        4
    );
}

#[test]
fn live_function_replacement_keeps_nesting_and_reinterprets_only_literals() {
    let (plan, source) = replaced_source(
        Grid::with_shape(16, 4),
        &[".+02.x0304", ".+0000", ".+0001", ""],
        &[(0, 48), (4, 34), (16, 0), (32, 52)],
        &[(16, lang::Function::Multiply)],
    );
    // The nested `.x` keeps its place under the replacement and writes
    // its `0C` into the third row's first operand.
    assert_eq!(&source.snapshot()[..10], ".x02.x0304");
    assert_eq!(&source.snapshot()[48..50], "18");
    assert_eq!(&source.snapshot()[52..54], "0D");
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    let (plan, source) = replaced_source(
        Grid::with_shape(16, 3),
        &[".vC4", ".+0000", ""],
        &[(0, 32), (16, 0)],
        &[(16, lang::Function::ConvertToNote)],
    );
    assert_eq!(&source.snapshot()[..4], ".^C4");
    assert_eq!(&source.snapshot()[32..34], "  ");
    assert!(
        plan.diagnostics
            .iter()
            .any(|d| d.message == "Number C4 cannot be converted to a Note")
    );
    // The retained nested `.^` returns `C4`, which the replacement's
    // Number operand reads as Number `C4`, outside the Note range. The
    // child's own write under its anchor stands.
    let (plan, source) = replaced_source(
        Grid::with_shape(16, 3),
        &[".v.^3C", "    .+0000", ""],
        &[(0, 32), (20, 0)],
        &[(20, lang::Function::ConvertToNote)],
    );
    assert_eq!(&source.snapshot()[..6], ".^.^3C");
    assert_eq!(&source.snapshot()[18..20], "C4");
    assert_eq!(&source.snapshot()[32..34], "  ");
    assert_eq!(messages(&plan), ["Number C4 cannot be converted to a Note"]);
}

#[test]
fn live_replacement_checks_retained_arity_and_never_runs_new_anchors() {
    let (plan, source) = replaced_source(
        Grid::with_shape(16, 3),
        &[".+0204", ".+0000", ""],
        &[(0, 32), (16, 0)],
        &[(16, lang::Function::ConvertToNote)],
    );
    assert_eq!(&source.snapshot()[..6], ".^0204");
    assert_eq!(&source.snapshot()[32..34], "  ");
    assert!(
        plan.diagnostics
            .iter()
            .any(|d| d.message.contains("expected 1, found 2"))
    );
    assert_eq!(
        source
            .language_map()
            .expressions()
            .next()
            .unwrap()
            .span()
            .end()
            .get(),
        3
    );
    assert!(source.language_map().diagnostics().any(|d| d.start() == 4));
    let (plan, interpreted, source) = replaced_tick(
        Grid::with_shape(16, 3),
        &[".+0101", ".+0000", ""],
        &[(0, 32), (16, 1)],
        &[(16, lang::Function::Multiply)],
    );
    assert_eq!(&source.snapshot()[..6], "..x101");
    assert_eq!(&source.snapshot()[32..34], "  ");
    // No Turn is taken at the `.x` the replacement spelled at column 1, and
    // none at the `.+` it covered: the original anchor's computation is
    // suppressed and the new one was never scheduled. The producer that
    // states the replacement answers without interpreting anything, so the
    // Interpreter is not called at all this Tick.
    assert_eq!(interpreted.len(), 0);
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert!(
        source
            .language_map()
            .diagnostics()
            .any(|d| d.start() == 1 && d.message.contains("1 "))
    );
}

#[test]
fn original_anchor_function_replacement_retains_inputs() {
    let (plan, source) = replaced_source(
        Grid::with_shape(16, 3),
        &[".+0204", ".+0000", ""],
        &[(0, 32), (16, 0)],
        &[(16, lang::Function::Multiply)],
    );
    assert_eq!(&source.snapshot()[..6], ".x0204");
    assert_eq!(&source.snapshot()[32..34], "08");
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
}

#[test]
fn a_refused_replacement_keeps_its_reason_in_the_record_and_paints_nothing() {
    // `.+0000` answers Equality at (0, 2), the anchor of `.+0102`, which
    // Equality would change by being able to answer Bang. The pair it was
    // refused at is not `.+0000`'s own static Output Portal, so nothing
    // else paints it. `.+0102` then has no row below it to write to.
    let grid = Grid::with_shape(16, 3);
    let (_, _, source) = stated_tick(
        grid,
        &[".+0000", "", ".+0102"],
        &[(0, 32)],
        &[(0, Atom::Function(lang::Function::Equality))],
    );
    assert_eq!(&source.snapshot()[32..38], ".+0102");
    assert_eq!(
        writes(&committed_portals(&source)),
        [
            (
                (0, 0),
                lang::Function::Add,
                PortalOrigin::Declared,
                Err(WriteRefusal::Replacement(ReplacementChange::BangEmission))
            ),
            (
                (0, 2),
                lang::Function::Add,
                PortalOrigin::Declared,
                Err(WriteRefusal::Portal {
                    error: PortalError::BelowSource,
                    at: None
                })
            ),
        ]
    );
    let output = painted(source, PortalRole::Output);
    assert!(!output.contains(&(0, 2)), "{output:?}");
    assert!(!output.contains(&(1, 2)), "{output:?}");
}
