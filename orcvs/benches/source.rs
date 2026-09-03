//! Benchmarks for the two Source paths an editing session sits on: a Render Frame
//! re-reads an unchanged revision many times a second, and every keystroke rebuilds
//! the Language Map that the next frame reads.
//!
//! Run with `mise run bench`. The `--output-format bencher` flag it passes is not
//! cosmetic: CI parses the output with a regex that only matches that format.

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use orcvs::app::Orcvs;
use orcvs::grid::{CellIndex, Grid};
use orcvs::playback::InMemoryOutputAdapter;
use orcvs::source::SourceCommander;
use std::hint::black_box;

/// Representative Source shapes. A console opens at 32x32, and the two shapes
/// bracketing it each change the Cell count fourfold, so whole-map work shows up
/// as growth across the series instead of hiding inside one fixed size.
const SIZES: &[(usize, usize)] = &[(16, 16), (32, 32), (64, 64)];

/// The Expression shapes an editing session actually holds: complete arithmetic,
/// a nested Expression, a Bang, Activation Characters, and the malformed and
/// half-typed text that every keystroke leaves behind between two valid revisions.
const EXPRESSIONS: &[&str] = &[
    ".+0102",
    ".x0201",
    "!>010AC4",
    ".-0A05",
    ".+01XY",
    "./0402",
    ".+.+0101.-0A05",
    "**",
    ".+01",
    ">>",
];

/// The Cell every edit benchmark rewrites: an operand digit of the first
/// Expression on the second row, so an edit lands inside populated Source rather
/// than in an empty margin.
const EDIT_COLUMN: usize = 4;

/// What that Cell holds in the fixture, and what each unmeasured restore writes back.
const RESTORED: &str = "0";

/// A different digit in the same operand position: still a valid Expression, so the
/// rebuild does the full parse an accepted edit pays for.
const EDITED_VALID: &str = "3";

/// A character that no operand accepts. Source is malformed for most of the
/// keystrokes that produce it, and that revision is rebuilt and rendered too.
const EDITED_INVALID: &str = "Z";

/// The index `grid` mints for `idx`. A Cell is named by an index its Grid
/// minted, so a benchmark states the number and the Grid answers with the Cell.
fn cell(grid: Grid, idx: usize) -> CellIndex {
    grid.cell_index(idx).expect("inside the Grid")
}

/// One Cell per Grid Position. Each row tiles `EXPRESSIONS` from a different
/// starting point, separated by a space so a row holds several Expression extents
/// rather than one, and is cut to the column count wherever that lands.
fn source_text(cols: usize, rows: usize) -> String {
    let mut text = String::with_capacity(cols * rows);

    for row in 0..rows {
        let mut line = String::with_capacity(cols + EXPRESSIONS[0].len());
        let mut next = row;
        while line.len() < cols {
            line.push_str(EXPRESSIONS[next % EXPRESSIONS.len()]);
            line.push(' ');
            next += 1;
        }
        line.truncate(cols);
        text.push_str(&line);
    }

    text
}

fn populated_source(cols: usize, rows: usize) -> SourceCommander {
    let grid = Grid::new(cols, rows);
    let source = SourceCommander::new(grid);

    for (idx, content) in source_text(cols, rows).chars().enumerate() {
        if content != ' ' {
            source
                .set(cell(grid, idx), &content.to_string())
                .expect("benchmark Source content is accepted");
        }
    }

    source
}

fn populated_app(cols: usize, rows: usize) -> Orcvs<InMemoryOutputAdapter> {
    let mut orcvs = Orcvs::with_output_adapter(cols, rows, InMemoryOutputAdapter::default());
    let text = source_text(cols, rows);
    // Only the Grid that owns a Position mints one, and a Render Frame is how the
    // application hands those Positions out.
    let positions = orcvs
        .render_frame()
        .rows()
        .iter()
        .flatten()
        .map(|cell| cell.position())
        .collect::<Vec<_>>();

    for (position, content) in positions.iter().zip(text.chars()) {
        if content != ' ' {
            orcvs.select(*position);
            orcvs.write(&content.to_string());
        }
    }
    // A Cursor in the middle of the Grid pays the representative share of the
    // bloom and seam work a frame does around it.
    orcvs.select(positions[positions.len() / 2]);

    // `Orcvs::write` reports a rejected edit through `tracing` and returns nothing,
    // and a bench binary installs no subscriber. Unchecked, a fixture that never
    // landed would be measured as an empty Grid and still report a plausible number.
    assert_eq!(
        occupied_cells(&orcvs),
        text.chars().filter(|content| *content != ' ').count(),
        "every benchmark Source Cell is accepted"
    );

    orcvs
}

fn occupied_cells(orcvs: &Orcvs<InMemoryOutputAdapter>) -> usize {
    orcvs
        .render_frame()
        .rows()
        .iter()
        .flatten()
        .filter(|cell| cell.content().is_some())
        .count()
}

fn size(cols: usize, rows: usize) -> BenchmarkId {
    BenchmarkId::from_parameter(format!("{cols}x{rows}"))
}

fn read_revision(c: &mut Criterion) {
    let mut group = c.benchmark_group("source_read_revision");

    for &(cols, rows) in SIZES {
        let source = populated_source(cols, rows);

        group.bench_function(size(cols, rows), |b| {
            b.iter(|| black_box(black_box(&source).read_revision()))
        });
    }

    group.finish();
}

fn render_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("source_render_frame");

    for &(cols, rows) in SIZES {
        let orcvs = populated_app(cols, rows);

        group.bench_function(size(cols, rows), |b| {
            b.iter(|| black_box(black_box(&orcvs).render_frame()))
        });
    }

    group.finish();
}

/// Measures one accepted Cell edit and all the Language Map work the Source edit
/// path performs for it.
///
/// The restore is the setup rather than part of the routine, and
/// `BatchSize::PerIteration` is what keeps the two alternating: any larger batch
/// runs every setup before the first routine, so all but the first edit would
/// write a Cell that already holds `content` and measure a no-op.
fn edit(c: &mut Criterion, name: &str, content: &'static str) {
    let mut group = c.benchmark_group(name);

    for &(cols, rows) in SIZES {
        let source = populated_source(cols, rows);
        // Minted once, outside the measured closure: an index is what a Cell is
        // named by, and minting one is not part of what an edit costs.
        let edited = cell(source.grid(), cols + EDIT_COLUMN);

        group.bench_function(size(cols, rows), |b| {
            b.iter_batched(
                || {
                    source
                        .set(edited, RESTORED)
                        .expect("the restored Cell is accepted");
                },
                |()| {
                    black_box(&source)
                        .set(black_box(edited), content)
                        .expect("the edited Cell is accepted");
                },
                BatchSize::PerIteration,
            )
        });
    }

    group.finish();
}

fn edit_rebuild_valid(c: &mut Criterion) {
    edit(c, "source_edit_rebuild_valid", EDITED_VALID);
}

fn edit_rebuild_invalid(c: &mut Criterion) {
    edit(c, "source_edit_rebuild_invalid", EDITED_INVALID);
}

criterion_group!(
    benches,
    read_revision,
    render_frame,
    edit_rebuild_valid,
    edit_rebuild_invalid
);
criterion_main!(benches);
