//! Benchmarks for the two paths `lang` sits on: a Tick interprets the Expressions in
//! a Source Snapshot, and a Render Frame re-reads the Source many times a second.
//!
//! Run with `mise run bench`. The `--output-format bencher` flag it passes is not
//! cosmetic: CI parses the output with a regex that only matches that format.

use criterion::{Criterion, criterion_group, criterion_main};
use lang::{Interpreter, Parser};
use std::hint::black_box;

/// Nested arithmetic: the shape an Expression takes once a Function consumes
/// another Function's result.
const NESTED: &str = ".+.+0101.-0A05";

/// Malformed operands. A Source being edited is malformed for most of the
/// keystrokes that produce it, and every Render Frame re-reads it.
const INVALID: &str = ".+01XY";

/// One Source revision, row by row: complete Expressions, a Bang, an Activation
/// Character, malformed text, and empty rows.
const SOURCE: &[&str] = &[
    ".+0102",
    ".x0201",
    "",
    "!>010AC4",
    ".-0A05",
    "**",
    "./0402",
    ">>",
    "",
    ".+.+0101.-0A05",
    ".+01XY",
    "",
    ".x0F10",
    ".-0201",
    "./0100",
    "",
];

fn parse(c: &mut Criterion) {
    let mut source = String::from(NESTED);

    c.bench_function("parse", |b| {
        b.iter(|| Parser::from(black_box(source.as_mut_str())).try_parse())
    });
}

fn parse_invalid(c: &mut Criterion) {
    let mut source = String::from(INVALID);

    // `parse` is the checking path: it diagnoses without erroring, which is what an
    // incomplete Source needs.
    c.bench_function("parse_invalid", |b| {
        b.iter(|| Parser::from(black_box(source.as_mut_str())).parse())
    });
}

fn execute(c: &mut Criterion) {
    let mut source = String::from(NESTED);
    let atoms = Parser::from(source.as_mut_str())
        .try_parse()
        .expect("NESTED is a valid Expression");

    c.bench_function("execute", |b| {
        b.iter(|| Interpreter::execute(black_box(&atoms)))
    });
}

fn parse_source(c: &mut Criterion) {
    let mut rows: Vec<String> = SOURCE.iter().map(|row| row.to_string()).collect();

    c.bench_function("parse_source", |b| {
        b.iter(|| {
            let mut units = 0;
            for row in rows.iter_mut() {
                if let Ok(expression) = Parser::from(black_box(row.as_mut_str())).parse() {
                    units += expression.len();
                }
            }
            units
        })
    });
}

criterion_group!(benches, parse, parse_invalid, execute, parse_source);
criterion_main!(benches);
