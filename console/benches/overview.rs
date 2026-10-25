//! Benchmark for the Overview's picture.
//!
//! `draw_picture` draws it, one texel per Cell, from a whole Render Frame: the
//! cost the Overview pays once per Source revision, which during Playback is
//! every Tick. It redraws one picture in place, as the Overview does, so the
//! number is the per-Cell walk and not an allocation.
//!
//! Run with `mise run bench`. The `--output-format bencher` flag it passes is not
//! cosmetic: CI parses the output with a regex that only matches that format.

use console::source_shapes::{dark_default_theme, draw_picture};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orcvs::app::Orcvs;
use orcvs::grid::Grid;
use orcvs::playback::InMemoryOutputAdapter;
use orcvs::source::Source;
use std::hint::black_box;

/// The same tiling as `paint.rs` and `orcvs/benches/source.rs`.
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

///
/// The shipped 256 by 256 Grid tiled with [`EXPRESSIONS`], each row starting
/// one expression on, as `paint.rs` tiles its largest frame.
///
fn populated_app() -> Orcvs<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("a benchmark runtime");
    let _runtime = runtime.enter();
    let mut orcvs = Orcvs::with_source_and_output_adapter(
        Source::new(Grid::new()),
        InMemoryOutputAdapter::default(),
    )
    .expect("a benchmark runtime");
    let cols = orcvs.grid().columns();
    let rows: Vec<_> = orcvs
        .grid()
        .positions_by_row()
        .map(Vec::from_iter)
        .collect();
    for (row, positions) in rows.iter().enumerate() {
        let mut line = String::with_capacity(cols + EXPRESSIONS[0].len());
        let mut next = row;
        while line.len() < cols {
            line.push_str(EXPRESSIONS[next % EXPRESSIONS.len()]);
            line.push(' ');
            next += 1;
        }
        for (position, content) in positions.iter().zip(line.chars()) {
            if content != ' ' {
                orcvs.select(*position);
                orcvs.write(&content.to_string());
            }
        }
    }
    orcvs
}

fn overview_picture(c: &mut Criterion) {
    let mut group = c.benchmark_group("paint_overview");
    let theme = dark_default_theme();
    let frame = populated_app().render_frame();
    let grid = frame.grid();
    let mut picture = egui::ColorImage::default();

    group.bench_function(
        BenchmarkId::from_parameter(format!("picture/{}x{}", grid.columns(), grid.rows())),
        |b| {
            b.iter(|| {
                draw_picture(
                    black_box(&frame),
                    black_box(&theme),
                    black_box(&mut picture),
                )
            })
        },
    );

    group.finish();
}

criterion_group!(benches, overview_picture);
criterion_main!(benches);
