//! Write claims and Cell-by-Cell conflict resolution: which reserved writes a Cell
//! range touches, and how later effects win each contested Cell independently.

use crate::grid::Grid;
use crate::source::language_map::LanguageMap;
use crate::source::{CellWrite, Cells};

use super::fixture::{anchors, carried_source, cell, diagnostic, planned, snapshot, write};
use super::observed::raw;
use super::{Effect, Tick, resolve};

#[test]
fn write_claims_answer_overlapping_reservations() {
    let claims = super::WriteClaims::new(
        Grid::with_shape(8, 1),
        vec![
            super::Claim {
                cells: 0..4,
                node: super::ComputationId(0),
            },
            super::Claim {
                cells: 2..6,
                node: super::ComputationId(1),
            },
        ],
    );
    assert_eq!(
        claims.touching(2..4).collect::<Vec<_>>(),
        [0, 1].map(super::ComputationId)
    );
}

#[test]
fn write_claims_skip_a_nested_short_span() {
    let claims = super::WriteClaims::new(
        Grid::with_shape(20, 1),
        vec![
            super::Claim {
                cells: 0..20,
                node: super::ComputationId(0),
            },
            super::Claim {
                cells: 1..3,
                node: super::ComputationId(1),
            },
        ],
    );
    assert_eq!(
        claims.touching(10..12).collect::<Vec<_>>(),
        [super::ComputationId(0)]
    );
}

#[test]
fn write_claims_on_a_dense_row_touch_only_the_covering_write() {
    let claims = super::WriteClaims::new(
        Grid::with_shape(64, 1),
        (0..8)
            .map(|index| super::Claim {
                cells: index * 8..index * 8 + 2,
                node: super::ComputationId(index),
            })
            .collect(),
    );
    assert_eq!(
        claims.touching(16..18).collect::<Vec<_>>(),
        [super::ComputationId(2)]
    );
    assert!(claims.touching(18..20).next().is_none());
}

#[test]
fn competing_writers_preserve_an_independent_rejected_destination_diagnostic() {
    let (plan, source) = carried_source(
        Grid::with_shape(16, 3),
        &[".+0102", ".+0304", ".+0506"],
        &[(0, 26), (16, 26), (32, 31)],
    );
    assert_eq!(planned(&plan), vec![(26, '0'), (27, '7')]);
    assert_eq!(&source.snapshot()[26..28], "07");
    assert_eq!(&source.snapshot()[31..32], " ");
    assert_eq!(plan.diagnostics.len(), 1);
    assert!(plan.diagnostics[0].message.contains("crosses the row edge"));
}

#[test]
fn competing_writers_publish_but_a_dependency_cycle_writes_nothing() {
    let conflict_grid = Grid::with_shape(16, 2);
    let conflict_bytes = format!("{:<16}{:<16}", ".+0102", ".+0304");
    let conflict_map = LanguageMap::build(conflict_grid, Cells::of(conflict_bytes.as_bytes()));
    let shared = conflict_grid.position(10, 1).unwrap();
    let conflict_destinations = [
        (conflict_grid.cell_index(0).unwrap(), vec![shared]),
        (conflict_grid.cell_index(16).unwrap(), vec![shared]),
    ]
    .into_iter()
    .collect();
    let (conflict, _) = super::plan_carrying(
        conflict_grid,
        Cells::of(conflict_bytes.as_bytes()),
        &conflict_map,
        Tick::ZERO,
        &conflict_destinations,
    );
    let conflict = conflict.into_plan();
    assert_eq!(planned(&conflict), vec![(26, '0'), (27, '7')]);
    assert!(conflict.play_commands.is_empty());
    assert!(conflict.diagnostics.is_empty());

    let cycle_grid = Grid::with_shape(16, 2);
    let cycle_bytes = format!("{:<16}{:<16}", ".+0001", ".+0001");
    let cycle_map = LanguageMap::build(cycle_grid, Cells::of(cycle_bytes.as_bytes()));
    let cycle_destinations = [
        (
            cycle_grid.cell_index(0).unwrap(),
            vec![cycle_grid.position(2, 1).unwrap()],
        ),
        (
            cycle_grid.cell_index(16).unwrap(),
            vec![cycle_grid.position(2, 0).unwrap()],
        ),
    ]
    .into_iter()
    .collect();
    let (cycle, _) = super::plan_carrying(
        cycle_grid,
        Cells::of(cycle_bytes.as_bytes()),
        &cycle_map,
        Tick::ZERO,
        &cycle_destinations,
    );
    let cycle = cycle.into_plan();
    // One cycle runs through both Expressions, so it is diagnosed once, at
    // the first in Parser order, and neither is reported as waiting.
    assert!(cycle.writes.is_empty());
    assert_eq!(anchors(&cycle), [(0, 0, "same-Tick dependency cycle")]);
}

#[test]
fn test_a_write_whose_destination_leaves_the_grid_emits_no_partial_write() {
    // A complete write validates its whole destination before
    // any Cell of it enters the Tick Plan. This root sits in the last row,
    // so its result has nowhere to go and the Tick contributes a
    // diagnostic and nothing else — not the first Cell of a result that
    // could not be placed.
    let grid = Grid::with_shape(10, 2);
    let bytes = snapshot(grid, &["", ".+0102"]);
    let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));

    let (plan, _) = super::plan(grid, Cells::of(bytes.as_bytes()), &map, Tick::ZERO);
    let plan = plan.into_plan();

    assert!(plan.writes.is_empty());
    assert_eq!(
        plan.diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>(),
        vec!["result \"03\" falls below the Source"]
    );
}

#[test]
fn test_later_effects_win_cell_conflicts_independently() {
    // An earlier producer writes three Cells and a later one writes two,
    // overlapping the third. Resolution is Cell-wise: the later producer
    // takes only the Cell they share, and every other Cell of the earlier
    // producer's complete write still stands.
    //
    // The resolution is pinned here, at the seam that owns it, rather than
    // through a Source whose overlapping writes depend on which Functions
    // emit them.
    let grid = Grid::with_shape(20, 3);
    let plan = resolve(vec![write(grid, 10, "ABC"), write(grid, 12, "XY")]);

    assert_eq!(
        plan.writes,
        vec![
            CellWrite {
                cell: cell(grid, 10),
                content: crate::source::CellContent::new(b'A').unwrap()
            },
            CellWrite {
                cell: cell(grid, 11),
                content: crate::source::CellContent::new(b'B').unwrap()
            },
            CellWrite {
                cell: cell(grid, 12),
                content: crate::source::CellContent::new(b'X').unwrap()
            },
            CellWrite {
                cell: cell(grid, 13),
                content: crate::source::CellContent::new(b'Y').unwrap()
            },
        ]
    );
}

#[test]
fn test_play_commands_and_diagnostics_keep_producer_and_emission_order() {
    // Play Commands and diagnostics are ordered, never merged: unlike a
    // Cell, which one producer can take from another, each command and
    // each diagnostic keeps the place its producer's turn gave it. The
    // three commands differ in every field that can be read back, so a
    // producer order that let a later Expression in first is a different
    // Tick Plan rather than the same one.
    let grid = Grid::with_shape(10, 3);
    let first = raw(0, 1, 60);
    let second = raw(0, 1, 64);
    let third = raw(1, 2, 61);
    let earlier = diagnostic(grid, 0, 5, "earlier producer");
    let later = diagnostic(grid, 20, 25, "later producer");

    let plan = resolve(vec![
        Effect::Play(first),
        Effect::Play(second),
        earlier.clone(),
        write(grid, 10, "0"),
        Effect::Play(third),
        later.clone(),
    ]);

    assert_eq!(plan.play_commands, vec![first, second, third]);
    assert_eq!(
        plan.diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>(),
        vec!["earlier producer", "later producer"]
    );
    assert_eq!(
        plan.writes,
        vec![CellWrite {
            cell: cell(grid, 10),
            content: crate::source::CellContent::new(b'0').unwrap()
        }]
    );
}

#[test]
fn the_cells_of_two_overlapping_writes_are_contested_one_by_one() {
    // Every admitted write participates Cell-wise in producer order. A
    // write is one validated effect while it is being planned and as many
    // independently contested Cells as it has characters once it is
    // resolved, so the later write takes only the four Cells the two
    // encodings share and the earlier write's first two Cells still stand.
    let grid = Grid::with_shape(20, 3);
    let effects = vec![write(grid, 20, "0A0B0C"), write(grid, 22, "0D0E0F")];

    assert_eq!(
        planned(&resolve(effects)),
        vec![
            (20, '0'),
            (21, 'A'),
            (22, '0'),
            (23, 'D'),
            (24, '0'),
            (25, 'E'),
            (26, '0'),
            (27, 'F'),
        ]
    );
}

///
/// The Cell-wise half of producer-order conflict resolution, over overlap
/// shapes no example states.
///
/// The `cfg` matches the `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]`
/// table that declares proptest, so a WASM build never sees the dependency.
///
#[cfg(not(target_arch = "wasm32"))]
mod property {
    use crate::grid::Grid;
    use crate::source::tick::{Effect, Encoding, Portal, resolve};
    use proptest::prelude::*;

    /// How wide the Grid every generated write lands in is. Stated once, so
    /// the column strategy, the Grid, and the row the oracle scans cannot
    /// drift apart into three literals that must be changed together.
    const WIDTH: usize = 16;

    proptest! {
        ///
        /// Later Cell effects win conflicts at each Cell independently, and the
        /// other half: a Tick Plan contains a Cell exactly when some admitted
        /// write covers it. Together those are the promise that an ordinary
        /// result writes only its current encoding and never clears a stale
        /// tail outside that Span — no shorter later write can reach a Cell it
        /// does not cover, in either direction.
        ///
        /// A property rather than an example because the interesting input is
        /// the shape of the overlaps: partial at either end, one write wholly
        /// inside another, two writes on the very same Cells, and runs that
        /// stop short of a row's end. Examples state one shape each, and the
        /// third one written by hand is already an enumeration.
        ///
        /// The expectation is computed by scanning the writes in reverse for
        /// the last one covering each Cell, which is a different computation
        /// from the forward fold under test rather than a copy of it. Writes
        /// the Portal refuses are dropped rather than made to fit, so what is
        /// resolved is only ever a set of complete writes.
        ///
        #[test]
        fn a_tick_plan_gives_each_cell_to_the_last_admitted_write_covering_it(
            requested in prop::collection::vec((0usize..WIDTH, "[A-Z]{1,8}"), 0..5),
        ) {
            let grid = Grid::with_shape(WIDTH, 2);
            // The first Cell of the destination row, asked of the same Grid the
            // writes are admitted through rather than recomputed from its width.
            let destination_row = grid
                .index(grid.position(0, 1).expect("inside the Grid"))
                .get();
            let mut admitted: Vec<(usize, String)> = Vec::new();
            let mut effects: Vec<Effect> = Vec::new();
            for (column, encoding) in requested {
                let destination = grid.position(column, 1).expect("inside the Grid");
                let cells = Encoding::literal(&encoding).expect("printable test content");
                if let Ok(write) = Portal::at(grid, destination).admit(&cells) {
                    effects.push(Effect::Write(write));
                    admitted.push((column, encoding));
                }
            }

            let plan = resolve(effects);

            let owner = |idx: usize| {
                admitted.iter().rev().find_map(|(column, encoding)| {
                    encoding
                        .chars()
                        .nth(idx.checked_sub(destination_row + column)?)
                })
            };
            prop_assert_eq!(
                plan.writes
                    .iter()
                    .map(|write| (write.cell.get(), write.content.as_char()))
                    .collect::<Vec<_>>(),
                (destination_row..grid.count())
                    .filter_map(|idx| owner(idx).map(|content| (idx, content)))
                    .collect::<Vec<_>>()
            );
        }
    }
}
