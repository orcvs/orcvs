//! Benchmarks for the Paint path a Render Frame takes on the way to shapes.
//!
//! `Paint::derive_with_theme` decides each drawn Cell from a Render Frame, a
//! Position range, and the resolved Theme to paint it from. `Paint::background_runs`
//! folds those backgrounds. Neither takes an `egui::Context`.
//! `SourceShapes::new` is the other half of the path and is absent here: it
//! needs a `GlyphTable`, which needs a Context, and that harness would be most
//! of the number. The gate that reads these lines alerts at 150%
//! and fails at 300% across hosted runners, so a benchmark added here is a
//! regression alarm, not a measurement anyone reads off.
//!
//! Two ranges per Grid: fitted (the whole Grid) and culled (a fixed 16×16
//! window). Fitted should grow with the Source. Culled should stay flat from
//! 32×32 up, which is the claim `source-paint/07` made and the counting tests
//! already assert by shape. At 16×16 the two ranges are the same point.
//!
//! Run with `mise run bench`. The `--output-format bencher` flag it passes is not
//! cosmetic: CI parses the output with a regex that only matches that format.

use console::{
    FramePaint, Paint, VisiblePositions,
    cursor_effects::{CursorEffectAnimation, CursorEffectSettings, cursor_effect_shapes},
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
    // `Grid::with_shape` is test-only; 256x256 is the shipped Grid (ADR 0054).
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
/// `console.rs`'s own tests do; it does not need to — the per-Cell walk
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
/// fill on no Cell at all (nothing here selects a Region spanning more than
/// one), and the resolved Theme answers one channel per Token fact whatever
/// its colours are. So what this benchmark measures — the per-Cell walk — is
/// the same number for any Theme, which is why [`bench_theme`] is the
/// built-in rather than a custom one built to match the console's own tests.
///
fn paint(frame: &RenderFrame, drawn: VisiblePositions, theme: &Theme) -> Paint {
    Paint::derive_with_theme(FramePaint::new(frame, drawn), theme)
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

/// The Cursor frame/living-area colours the console would hand this walk —
/// `theme.cursor_area`/`theme.cursor_border` in production. Restated as
/// literals for the same reason `REGION_FILL` used to be: `Theme`'s fields
/// are `pub(crate)`, a benchmark is a separate crate, and this geometry-only
/// walk reads a colour's bytes without branching on them, so any opaque
/// colour measures the same cost as the Theme's own.
const AREA_COLOUR: egui::Color32 = egui::Color32::from_rgb(76, 190, 156);
/// The Cursor/Region outline: `theme.cursor_border`/`theme.region_border` at
/// its nominal display-point width (`theme.cursor_border_width`/
/// `theme.region_border_width` in production, both 1 point in Okabe–Ito) —
/// restated as a literal for the same reason `AREA_COLOUR` is: `Theme`'s
/// fields are `pub(crate)` and this bench is a separate crate.
const FRAME: egui::Stroke = egui::Stroke {
    width: 1.0,
    color: egui::Color32::from_rgb(234, 235, 229),
};

fn cursor_effects(c: &mut Criterion) {
    let settings = CursorEffectSettings::default();
    let mut animation = CursorEffectAnimation::default();
    let sample = animation.advance(std::time::Duration::ZERO, settings);
    let cursor = Rect::from_min_size(Pos2::new(200.0, 200.0), Vec2::splat(16.0));
    let clip = cursor.expand(175.0);

    c.bench_function("cursor effects/frame and living area", |b| {
        b.iter(|| {
            black_box(cursor_effect_shapes(
                black_box(cursor),
                black_box(cursor),
                black_box(clip),
                black_box(16.0),
                black_box(sample),
                black_box(settings),
                black_box(AREA_COLOUR),
                black_box(FRAME),
            ))
        });
    });

    // A Region lasso twenty Cells by twelve: the frame is built per
    // Cell-length of edge, so its cost follows the Region's perimeter.
    let outline = Rect::from_min_max(cursor.min - Vec2::new(19.0, 11.0) * 16.0, cursor.max);
    c.bench_function("cursor effects/region lasso 20x12", |b| {
        b.iter(|| {
            black_box(cursor_effect_shapes(
                black_box(cursor),
                black_box(outline),
                black_box(clip),
                black_box(16.0),
                black_box(sample),
                black_box(settings),
                black_box(AREA_COLOUR),
                black_box(FRAME),
            ))
        });
    });
}

criterion_group!(benches, derive_paint, background_runs, cursor_effects);
criterion_main!(benches);
