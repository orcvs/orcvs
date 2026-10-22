//! Benchmarks for the Paint path a Render Frame takes on the way to shapes.
//!
//! `Paint::derive_with_theme` decides each drawn Cell from a Render Frame, a
//! Position range, and the resolved Theme to paint it from. `Paint::background_runs`
//! folds those backgrounds. `SourceShapes::geometry` places a Paint at a
//! presented Grid as its backgrounds, borders, seams and Cursor. None of them
//! takes an `egui::Context`. Glyph placement is absent: it needs a
//! `GlyphTable`, which needs a Context, and that harness would be most of the
//! number. Tessellation is absent too; it is egui's, after the console hands
//! the Shapes over. The gate that reads these lines alerts at 150%
//! and fails at 300% across hosted runners, so a benchmark added here is a
//! regression alarm, not a measurement anyone reads off.
//!
//! Two ranges per Grid: fitted (the whole Grid) and culled (a fixed 16×16
//! window). Fitted should grow with the Source. Culled should stay flat from
//! 32×32 up, which the counting tests already assert by shape. At 16×16 the
//! two ranges are the same point. The window cases draw what the default
//! window shows of the largest Grid, at a Cell size each: the per-Cell cost a
//! Cell size moves.
//!
//! Run with `mise run bench`. The `--output-format bencher` flag it passes is not
//! cosmetic: CI parses the output with a regex that only matches that format.

use console::{
    FramePaint, Paint, VisiblePositions,
    source_shapes::{DEFAULT_SOURCE_AREA, SourceShapes, presented_grid, visible_positions},
    theme::{Theme, okabe_ito},
};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use egui::{Pos2, Rect, Vec2};
use orcvs::app::Orcvs;
use orcvs::grid::Grid;
use orcvs::playback::InMemoryOutputAdapter;
use orcvs::render_frame::RenderFrame;
use orcvs::source::Source;
use std::hint::black_box;
use std::sync::OnceLock;

/// The same shapes `orcvs/benches/source.rs` uses for Render Frames. Held here
/// so the two series talk about the same Grids rather than inventing a second
/// set that cannot be read against them.
const FRAME_SIZES: &[(usize, usize)] = &[(16, 16), (32, 32), (64, 64), (128, 128), (256, 256)];

/// Culled window. The smallest `FRAME_SIZE`, so at 16×16 fitted and culled
/// coincide, and from 32×32 up the walk is a constant Cell count.
const CULLED: usize = 16;

/// The Cell sizes the window cases draw at: a 16 point square, a reference
/// the others are read against that no floor gates, the default, and the
/// smallest a settings file allows, which shows about four times the square's
/// Cells.
const WINDOW_CELLS: [Vec2; 3] = [Vec2::splat(16.0), Vec2::new(12.0, 18.0), Vec2::splat(8.0)];

/// Copied from `orcvs/benches/source.rs`. The two benches do not share a crate;
/// they share a fixture by keeping the same tiling.
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

fn benchmark_runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("a benchmark runtime")
    })
}

fn populated_app(cols: usize, rows: usize) -> Orcvs<()> {
    let _runtime = benchmark_runtime().enter();
    // `Grid::with_shape` is test-only; 256x256 is the shipped Grid.
    let mut orcvs = Orcvs::with_source_and_output_adapter(
        Source::new(Grid::with_shape(cols, rows)),
        InMemoryOutputAdapter::default(),
    )
    .expect("a benchmark runtime");
    let text = source_text(cols, rows);
    let positions = orcvs
        .grid()
        .positions_by_row()
        .flatten()
        .collect::<Vec<_>>();

    for (position, content) in positions.iter().zip(text.chars()) {
        if content != ' ' {
            orcvs.select(*position);
            orcvs.write(&content.to_string());
        }
    }
    orcvs.select(positions[positions.len() / 2]);

    assert_eq!(
        occupied_cells(&orcvs),
        text.chars().filter(|content| *content != ' ').count(),
        "every benchmark Source Cell is accepted"
    );

    orcvs
}

fn occupied_cells(orcvs: &Orcvs<()>) -> usize {
    orcvs
        .render_frame()
        .cells()
        .iter()
        .filter(|cell| cell.content().is_some())
        .count()
}

fn size(kind: &str, cols: usize, rows: usize) -> BenchmarkId {
    BenchmarkId::from_parameter(format!("{kind}/{cols}x{rows}"))
}

fn fitted(frame: &RenderFrame) -> VisiblePositions {
    let grid = frame.grid();
    VisiblePositions::for_grid(grid, 0..grid.columns(), 0..grid.rows())
}

fn culled(frame: &RenderFrame) -> VisiblePositions {
    let grid = frame.grid();
    VisiblePositions::for_grid(
        grid,
        0..CULLED.min(grid.columns()),
        0..CULLED.min(grid.rows()),
    )
}

///
/// What the default window's Source area draws of `frame` at `cell`, with the
/// Grid's corner on the area's.
///
fn window(frame: &RenderFrame, cell: Vec2) -> VisiblePositions {
    let viewport = presented_grid(Pos2::ZERO, frame.grid(), cell, 1.0);
    let area = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_SOURCE_AREA));
    visible_positions(&viewport, area, frame.grid())
}

fn window_id(cell: Vec2) -> BenchmarkId {
    BenchmarkId::from_parameter(format!("window/{}x{}", cell.x, cell.y))
}

///
/// A range of one Cell, and a range of none. The smallest `FRAME_SIZE` is
/// 16×16, so the ranges above never price a Paint whose drawn count is too
/// small to amortise whatever the walk resolves before it — the console
/// derives a Paint per frame at whatever size the viewport culls to, down to
/// a fully scrolled-away Grid. These two are where a fixed per-Paint cost
/// shows as the whole number rather than a rounding error.
///
fn single(frame: &RenderFrame) -> VisiblePositions {
    VisiblePositions::for_grid(frame.grid(), 0..1, 0..1)
}

fn empty(frame: &RenderFrame) -> VisiblePositions {
    VisiblePositions::for_grid(frame.grid(), 0..0, 0..0)
}

///
/// The Okabe–Ito built-in, the same Theme every fixture below is walked
/// against. `Theme`'s fields are `pub(crate)`, and a benchmark is a separate
/// crate, so this cannot override the Cursor or Region fill the way
/// `console::tests` do; it does not need to — the per-Cell walk
/// [`paint`] measures reads every Theme channel but branches on none of the
/// *values*, only on which fact a Cell carries (its own doc explains why), so
/// the built-in's own defaults (an unset Cursor fill, the default Region
/// fill) exercise the same code paths a custom Theme's colours would.
///
fn bench_theme() -> Theme {
    okabe_ito()
}

///
/// One Paint of `drawn`, resolved from the Theme the console would hand the
/// walk.
///
/// The walk reads every Theme channel but branches on none of their
/// *values*: the Cursor's fill lands on the one selected Cell, the Region
/// fill on the Cells of a Region spanning more than one (only
/// [`region_frame`] selects one), and the resolved Theme answers one channel
/// per Token fact whatever its colours are.
///
fn paint(frame: &RenderFrame, drawn: VisiblePositions, theme: &Theme) -> Paint {
    Paint::derive_with_theme(FramePaint::new(frame, drawn), theme)
}

///
/// The 32×32 fixture with a Region spanning its middle 24×24 Cells, so a
/// fitted walk prices the Region, Region-Cursor and plain placements in one
/// Paint.
///
fn region_frame() -> &'static RenderFrame {
    static FRAME: OnceLock<RenderFrame> = OnceLock::new();
    FRAME.get_or_init(|| {
        let mut orcvs = populated_app(32, 32);
        let at = |x, y| orcvs.grid().position(x, y).expect("inside the grid");
        let (from, to) = (at(4, 4), at(27, 27));
        orcvs.select(from);
        orcvs.extend(to);
        let frame = orcvs.render_frame();
        assert!(
            !frame.region().is_one_cell(),
            "the Region spans more than one Cell"
        );
        frame
    })
}

fn frames() -> &'static [(usize, usize, RenderFrame)] {
    static FRAMES: OnceLock<Vec<(usize, usize, RenderFrame)>> = OnceLock::new();
    FRAMES.get_or_init(|| {
        FRAME_SIZES
            .iter()
            .map(|&(cols, rows)| {
                let orcvs = populated_app(cols, rows);
                (cols, rows, orcvs.render_frame())
            })
            .collect()
    })
}

///
/// Measures `Paint::derive_with_theme` over a fitted range and a culled one.
///
/// The number is the per-Cell walk: `cell_visuals`, bloom, and seam colours,
/// collected into a `Vec` sized to the drawn Positions. Fixture construction
/// and the Render Frame sit outside the timed iteration.
///
fn derive_paint(c: &mut Criterion) {
    let mut group = c.benchmark_group("paint_derive");
    let theme = bench_theme();

    let (_, _, smallest) = &frames()[0];
    let single_range = single(smallest);
    let empty_range = empty(smallest);

    group.bench_function(BenchmarkId::from_parameter("single"), |b| {
        b.iter(|| {
            black_box(paint(
                black_box(smallest),
                black_box(single_range.clone()),
                black_box(&theme),
            ))
        })
    });
    group.bench_function(BenchmarkId::from_parameter("empty"), |b| {
        b.iter(|| {
            black_box(paint(
                black_box(smallest),
                black_box(empty_range.clone()),
                black_box(&theme),
            ))
        })
    });

    for &(cols, rows, ref frame) in frames() {
        let fitted_range = fitted(frame);
        let culled_range = culled(frame);

        group.bench_function(size("fitted", cols, rows), |b| {
            b.iter(|| {
                black_box(paint(
                    black_box(frame),
                    black_box(fitted_range.clone()),
                    black_box(&theme),
                ))
            })
        });
        group.bench_function(size("culled", cols, rows), |b| {
            b.iter(|| {
                black_box(paint(
                    black_box(frame),
                    black_box(culled_range.clone()),
                    black_box(&theme),
                ))
            })
        });
    }

    let region = region_frame();
    let region_range = fitted(region);
    group.bench_function(BenchmarkId::from_parameter("region/32x32"), |b| {
        b.iter(|| {
            black_box(paint(
                black_box(region),
                black_box(region_range.clone()),
                black_box(&theme),
            ))
        })
    });

    let (_, _, largest) = frames().last().expect("a benchmark frame");
    for cell in WINDOW_CELLS {
        let range = window(largest, cell);
        group.bench_function(window_id(cell), |b| {
            b.iter(|| {
                black_box(paint(
                    black_box(largest),
                    black_box(range.clone()),
                    black_box(&theme),
                ))
            })
        });
    }

    group.finish();
}

///
/// Measures `Paint::background_runs` on an already-derived Paint.
///
/// The fold is a second pass over the drawn Cells. Timing it apart from
/// `derive` is what lets a later move of the fold into `derive` show up as
/// one group shrinking and the other growing, rather than as one number that
/// stays put.
///
fn background_runs(c: &mut Criterion) {
    let mut group = c.benchmark_group("paint_background_runs");
    let theme = bench_theme();

    for &(cols, rows, ref frame) in frames() {
        let fitted_paint = paint(frame, fitted(frame), &theme);
        let culled_paint = paint(frame, culled(frame), &theme);

        group.bench_function(size("fitted", cols, rows), |b| {
            b.iter(|| black_box(black_box(&fitted_paint).background_runs()))
        });
        group.bench_function(size("culled", cols, rows), |b| {
            b.iter(|| black_box(black_box(&culled_paint).background_runs()))
        });
    }

    group.finish();
}

///
/// Measures `SourceShapes::geometry` on an already-derived Paint, at a Grid
/// presented from the origin.
///
/// The number is the walk that places each drawn Cell's background runs and
/// builds every Cell edge, borders, seams and the selection outline among
/// them, into one Mesh. Every case runs at 1×; the
/// fitted ranges run at 1.5× too, because a fractional device scale snaps
/// Cell sides and edges through different rounding than a whole one, and at
/// 2×, where lines of one width at 1× round to different widths and their
/// junctions draw in several pieces.
///
fn shapes(c: &mut Criterion) {
    // The fitted, culled, single and empty cases draw a fixed 16 point square
    // Cell. Do not change it: each series is read against its own earlier
    // points, so another Cell would read as a regression.
    const SQUARE: Vec2 = Vec2::splat(16.0);
    let mut group = c.benchmark_group("paint_shapes");
    let theme = bench_theme();
    let mut measure =
        |id: BenchmarkId, paint: &Paint, grid: Grid, cell: Vec2, pixels_per_point: f32| {
            let viewport = presented_grid(Pos2::ZERO, grid, cell, pixels_per_point);
            group.bench_function(id, |b| {
                b.iter(|| {
                    black_box(SourceShapes::geometry(
                        black_box(paint),
                        black_box(&viewport),
                        black_box(pixels_per_point),
                    ))
                })
            });
        };

    let (_, _, smallest) = &frames()[0];
    measure(
        BenchmarkId::from_parameter("single"),
        &paint(smallest, single(smallest), &theme),
        smallest.grid(),
        SQUARE,
        1.0,
    );
    measure(
        BenchmarkId::from_parameter("empty"),
        &paint(smallest, empty(smallest), &theme),
        smallest.grid(),
        SQUARE,
        1.0,
    );

    for &(cols, rows, ref frame) in frames() {
        let fitted_paint = paint(frame, fitted(frame), &theme);
        measure(
            size("fitted", cols, rows),
            &fitted_paint,
            frame.grid(),
            SQUARE,
            1.0,
        );
        measure(
            size("culled", cols, rows),
            &paint(frame, culled(frame), &theme),
            frame.grid(),
            SQUARE,
            1.0,
        );
        measure(
            size("fitted@1.5x", cols, rows),
            &fitted_paint,
            frame.grid(),
            SQUARE,
            1.5,
        );
        measure(
            size("fitted@2x", cols, rows),
            &fitted_paint,
            frame.grid(),
            SQUARE,
            2.0,
        );
    }

    let (_, _, largest) = frames().last().expect("a benchmark frame");
    for cell in WINDOW_CELLS {
        measure(
            window_id(cell),
            &paint(largest, window(largest, cell), &theme),
            largest.grid(),
            cell,
            1.0,
        );
    }

    group.finish();
}

criterion_group!(benches, derive_paint, background_runs, shapes);
criterion_main!(benches);
