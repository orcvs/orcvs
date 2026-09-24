//! Benchmarks for the stored value: the whole Source eframe storage holds
//! under `orcvs_source`, written by every autosave and read back at start.
//!
//! Every Grid is 256 by 256 (ADR 0054), 65,536 Cells, and the stored value
//! carries each of them, so writing and reading it walks every Cell. These
//! measure the codec the console's own save and start go through,
//! `eframe::set_value` and `eframe::get_value`, over a storage that only holds
//! the string, so the number is the encoding and the decode — including the
//! Language Map a restore derives — rather than a disk.
//!
//! Only compiled with `persistence` (`required-features` in `Cargo.toml`),
//! which ships on: without it there is no stored value to measure.
//!
//! Run with `mise run bench`. The `--output-format bencher` flag it passes is not
//! cosmetic: CI parses the output with a regex that only matches that format.

use criterion::{Criterion, criterion_group, criterion_main};
use orcvs::grid::Grid;
use orcvs::source::{CellContent, CellWrite, Source};
use std::collections::BTreeMap;
use std::hint::black_box;

/// The key the console stores its Source under (`console::persistence::SOURCE_KEY`).
const SOURCE_KEY: &str = "orcvs_source";

/// Copied from `orcvs/benches/source.rs`, as `paint.rs` copies it: the benches
/// share a fixture by keeping the same tiling, not a crate.
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

/// The one Grid, every Cell of the tiling written in one revision.
fn populated_source() -> Source {
    let grid = Grid::new();
    let mut source = Source::new(grid);
    let writes = source_text(grid.columns(), grid.rows())
        .bytes()
        .enumerate()
        .filter(|(_, byte)| *byte != b' ')
        .map(|(idx, byte)| CellWrite {
            cell: grid.cell_index(idx).expect("inside the Grid"),
            content: CellContent::new(byte).expect("benchmark Source is printable ASCII"),
        })
        .collect::<Vec<_>>();
    source.write_cells(&writes);
    source
}

/// Storage that holds strings and nothing else.
#[derive(Default)]
struct MemoryStorage(BTreeMap<String, String>);

impl eframe::Storage for MemoryStorage {
    fn get_string(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }

    fn set_string(&mut self, key: &str, value: String) {
        self.0.insert(key.to_owned(), value);
    }

    fn remove_string(&mut self, key: &str) {
        self.0.remove(key);
    }

    fn flush(&mut self) {}
}

fn stored_source(c: &mut Criterion) {
    let mut group = c.benchmark_group("stored_source");
    let source = populated_source();
    let mut storage = MemoryStorage::default();

    group.bench_function("save/256x256", |b| {
        b.iter(|| eframe::set_value(&mut storage, SOURCE_KEY, black_box(&source)))
    });

    eframe::set_value(&mut storage, SOURCE_KEY, &source);
    assert!(
        eframe::get_value::<Source>(&storage, SOURCE_KEY).is_some(),
        "the stored value reads back as a Source"
    );
    group.bench_function("restore/256x256", |b| {
        b.iter(|| black_box(eframe::get_value::<Source>(black_box(&storage), SOURCE_KEY)))
    });

    group.finish();
}

criterion_group!(benches, stored_source);
criterion_main!(benches);
