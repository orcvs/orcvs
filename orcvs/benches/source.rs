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
use orcvs::source::{Source, SourceCommander, Tick};
use std::hint::black_box;

/// Representative Source shapes. A console opens on 1000 Cells, which 32x32
/// stands for, and the two shapes bracketing it each change the Cell count
/// fourfold, so whole-map work shows up as growth across the series instead of
/// hiding inside one fixed size. The series is held at these shapes so the
/// measurements already recorded against them stay comparable.
const SIZES: &[(usize, usize)] = &[(16, 16), (32, 32), (64, 64)];

/// Source shapes for the Render Frame series, which reaches past the editing
/// sizes as the Tick series below does.
///
/// A Render Frame is derived whole, once per frame, over every Position of the
/// Grid (`orcvs/src/render_frame.rs:74-112`), so what one costs is a question
/// about the largest Grid a console is allowed to grow to — and no such bound
/// has been chosen. Resize is gated on the `GridId` rearchitecture that
/// `.scratch/grid-boundedness/` decides, and the spec that would set a cap does
/// not exist yet. These two added shapes therefore stand in for that undecided
/// upper end rather than claiming it: 128x128 is 16,384 Cells and 256x256 is
/// 65,536, sixty-five times the 1000 Cells a console opens on and past the
/// 200x200 at which `.scratch/render-frame-derivation/spec.md` says the question
/// starts to matter. Each step still quadruples the Cell count, so five points
/// across a 256-fold range is what tells the flat O(Cells) the code reads as
/// apart from anything steeper. The three editing shapes stay at the head of the
/// list rather than being replaced: the action keys each stored point by its
/// benchmark name, so keeping them keeps the history already recorded for
/// `source_read_revision` and `source_render_frame` at those shapes, and the two
/// added names simply start series of their own.
///
/// Nothing larger is measured, and 256x256 is already the expensive one. A
/// fixture is populated one accepted Cell edit at a time, and each edit rebuilds
/// the Language Map across every row (`orcvs/src/source/language_map.rs:201-211`
/// re-stamps the clean rows rather than skipping them), so building a fixture
/// costs about the square of its Cell count. Timed locally over the whole
/// benchmark binary, constructing every fixture takes 1.5s at the editing shapes
/// alone, 4.7s once 128x128 is added, and 62.9s once 256x256 is: that one shape
/// is some 58 seconds, paid again in the warm-up run each benchmark job performs
/// before the measured one. It is kept because it is the only point that reaches
/// the size the effort's spec says the question is about, and because its
/// measured spread is the tightest in the group — a per-Cell constant is
/// resolvable there and lost in the noise at 64x64. If the benchmark tier ever
/// needs that time back, dropping this one shape recovers nearly all of it and
/// leaves a curve that still has four points.
///
/// The Tick series keeps `TICK_SIZES`: a Tick iterates Expressions rather than
/// Positions, which `.scratch/grid-boundedness/spec.md` records, so a Grid the
/// console cannot yet open is not what would stress it.
const FRAME_SIZES: &[(usize, usize)] = &[(16, 16), (32, 32), (64, 64), (128, 128), (256, 256)];

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

///
/// Source shapes for the Tick series, which reaches past the editing sizes.
///
/// Larger shapes expose repeated scans of unrelated computations. Doubling
/// both dimensions quadruples the populated area; a much larger increase in
/// time can reveal quadratic work even when no spatial dependencies exist.
///
const TICK_SIZES: &[(usize, usize)] = &[(16, 16), (32, 32), (64, 64), (128, 128)];

///
/// A Source whose roots all deliver, laid out so scheduling is what is
/// measured.
///
/// Expressions sit on even rows only, so every ordinary result lands in the
/// blank row beneath its own and no two producers contest a Cell. The densely
/// tiled editing fixture cannot stand in for this: its rows write over each
/// other's Function Cells, so a Tick over it is rejected as a graph error and
/// never evaluates a root at all.
///
fn playing_source_text(cols: usize, rows: usize) -> String {
    let mut text = String::with_capacity(cols * rows);

    for row in 0..rows {
        let mut line = String::with_capacity(cols + EXPRESSIONS[0].len() + 2);
        if row % 2 == 0 {
            let mut next = row;
            while line.len() < cols {
                line.push_str(EXPRESSIONS[next % EXPRESSIONS.len()]);
                line.push_str("  ");
                next += 1;
            }
        }
        line.truncate(cols);
        while line.len() < cols {
            line.push(' ');
        }
        text.push_str(&line);
    }

    text
}

///
/// A Source built out of the two edges a Tick can carry, tiled.
///
/// The ordinary fixture has neither. Its results land in blank rows, so it
/// plans a graph with no edges at all: nothing activates, nothing waits on a
/// supplier, and the whole dependency half of a Tick is measured empty. That
/// is not a shape a pattern has — an Equality Bang firing a neighbouring MIDI
/// root is what ADR 0032 exists for.
///
/// One block, repeating every six rows, holds both. `.=` writes a Bang into
/// the blank row under it, which activates the Terminal Output root two rows
/// below; `.+` writes its result directly onto the left operand of the `.x`
/// beside it, which is a data dependency. Blocks repeat across a row every
/// `BLOCK_STRIDE` columns, and the `.x` sits two columns left of its block so
/// the operand it waits on is the Cell the `.+` above writes.
///
const BLOCK_STRIDE: usize = 10;

/// The first block's column. A block reaches two Cells to its left.
const BLOCK_COLUMN: usize = 2;

fn edged_source_text(cols: usize, rows: usize) -> String {
    let mut grid = vec![vec![b' '; cols]; rows];

    let mut place = |column: usize, row: usize, text: &str| {
        if row < rows && column + text.len() <= cols {
            grid[row][column..column + text.len()].copy_from_slice(text.as_bytes());
        }
    };

    for row in (0..rows).step_by(6) {
        let mut column = BLOCK_COLUMN;
        while column + 8 <= cols {
            // A Bang producer, the blank row its Bang lands in, and the
            // Terminal Output root that Bang activates.
            place(column, row, ".=0101");
            place(column, row + 2, "!>010AC4");
            // A value producer writing onto the left operand of its neighbour.
            place(column, row + 3, ".+0102");
            place(column - 2, row + 4, ".x0000");
            column += BLOCK_STRIDE;
        }
    }

    grid.into_iter()
        .map(|row| String::from_utf8(row).expect("benchmark Source is ASCII"))
        .collect()
}

///
/// That Source after one settling Tick, so the benchmark measures a steady
/// state rather than the one Tick that first writes every result.
///
fn settled_source(cols: usize, rows: usize, text: fn(usize, usize) -> String) -> Source {
    let grid = Grid::new(cols, rows);
    let mut source = Source::new(grid);

    for (idx, content) in text(cols, rows).chars().enumerate() {
        if content != ' ' {
            source
                .set(cell(grid, idx), &content.to_string())
                .expect("benchmark Source content is accepted");
        }
    }

    // A Tick rejected as a graph error plans nothing, costs a fraction of an
    // executed one, and would be reported as a plausible number. So the
    // fixture is required to have delivered, at a floor low enough that only a
    // rejection or a near-empty Grid falls under it. Expressions cut off at
    // the row edge do diagnose, and are left in: half-typed Source is what a
    // Grid this dense actually holds.
    let plan = source.execute(Tick::new(0));
    assert!(
        plan.writes.len() * 32 >= cols * rows,
        "the {cols}x{rows} Tick fixture executes its roots: {} writes, {} diagnostics",
        plan.writes.len(),
        plan.diagnostics.len(),
    );

    source
}

fn size(cols: usize, rows: usize) -> BenchmarkId {
    BenchmarkId::from_parameter(format!("{cols}x{rows}"))
}

///
/// Measures one revision read: the clone of the whole Source string
/// (`orcvs/src/source/model.rs:266-268`) and the `Arc` clone of the Language
/// Map beside it.
///
/// On `FRAME_SIZES` rather than `SIZES`, and the only group here that earned
/// them. `Orcvs::render_frame` reads a revision and then derives over the
/// result (`orcvs/src/app.rs:217-227`), so this clone is already inside every
/// `source_render_frame` number. Measured at the same shapes it is the one part
/// of a frame that can be subtracted back out of that number without a
/// benchmark group that pulls `derive` apart; measured at different shapes it
/// could not be subtracted at all, and the share a frame spends cloning at the
/// upper end would be a guess.
///
fn read_revision(c: &mut Criterion) {
    let mut group = c.benchmark_group("source_read_revision");

    for &(cols, rows) in FRAME_SIZES {
        let source = populated_source(cols, rows);

        group.bench_function(size(cols, rows), |b| {
            b.iter(|| black_box(black_box(&source).read_revision()))
        });
    }

    group.finish();
}

///
/// Measures one whole Render Frame: the revision read, the walk over every
/// Position, and the `Vec` per row the walk collects into.
///
/// Deliberately not split into those parts. They are separate costs with
/// separate answers, but the bloom arithmetic `derive` pays at every Cell is a
/// constant per Cell that would hide inside the walk however it were timed, so
/// the curve across `FRAME_SIZES` is what says whether any of it is worth
/// separating.
///
fn render_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("source_render_frame");

    for &(cols, rows) in FRAME_SIZES {
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
///
/// Stays on `SIZES` while the frame groups above reach further, and the reason is
/// not that an edit is cheap. One edit dirties the single row it lands in
/// (`orcvs/src/source/model.rs:255-261`), but the rebuild that row forces walks
/// every row of the Language Map and re-stamps the clean ones
/// (`orcvs/src/source/language_map.rs:201-211`), so what an edit costs follows the
/// Cell count as much as a frame does. That is a question about the edit path,
/// which is not what the Render Frame effort is measuring, and it is the
/// expensive one to ask: each added shape here needs a fixture of its own, and a
/// 256x256 one takes tens of seconds to build — once per edit group, in each of
/// the two runs a benchmark job performs, on every run. Whoever asks
/// what a keystroke costs on a resized Grid should extend this series and pay for
/// it deliberately.
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

///
/// Measures one Tick: scheduling every root in dependency order, evaluating
/// them, committing the writes, and rebuilding the Language Map.
///
/// This is the operation with a deadline. The Playback Engine drives it once
/// per Tick period, which `Bpm::delay_ms` puts at 125ms at 120 BPM and under
/// 20ms at the tempos a fast pattern reaches, and everything a Tick does
/// happens inside that window. Growth across the series is the measurement
/// that matters: ordering work between roots grows with the square of their
/// number if each root asks about every other, and a series is what tells that
/// apart from a Language Map rebuild growing with the Cell count.
///
/// The fixture has settled, so each measured Tick re-plans the same graph and
/// writes the same Cells. That is also the ordinary case: a pattern holds its
/// shape while it plays.
fn execute_tick(c: &mut Criterion) {
    tick_series(c, "source_execute_tick", playing_source_text);
}

///
/// The same Tick over a Source whose roots depend on one another.
///
/// Separate from the series above rather than folded into it, because the two
/// answer different questions and one cannot stand in for the other: the
/// ordinary series measures what a Tick costs per Cell, and this one measures
/// what it costs to discover and order dependencies between roots, including
/// Bang activation. Keep both series: an all-pairs lookup regression also
/// makes independent roots expensive even though their outputs touch no input.
///
fn execute_tick_with_edges(c: &mut Criterion) {
    tick_series(c, "source_execute_tick_edges", edged_source_text);
}

fn tick_series(c: &mut Criterion, name: &str, text: fn(usize, usize) -> String) {
    let mut group = c.benchmark_group(name);

    for &(cols, rows) in TICK_SIZES {
        let mut source = settled_source(cols, rows, text);
        let mut tick = 1u64;

        group.bench_function(size(cols, rows), |b| {
            b.iter(|| {
                tick += 1;
                black_box(source.execute(black_box(Tick::new(tick))))
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    read_revision,
    render_frame,
    edit_rebuild_valid,
    edit_rebuild_invalid,
    execute_tick,
    execute_tick_with_edges
);
criterion_main!(benches);
