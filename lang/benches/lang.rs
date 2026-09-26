//! Benchmarks for the two paths `lang` sits on: a Turn evaluates one Function over
//! resolved operands, and a Render Frame re-reads the Source many times a second.
//!
//! Run with `mise run bench`. The `--output-format bencher` flag it passes is not
//! cosmetic: CI parses the output with a regex that only matches that format.

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use lang::{Anchor, Atom, Function, Interpreter, Parser, Sequence, Tick, TickInputs, Value};
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
    c.bench_function("parse", |b| {
        b.iter(|| Parser::from(black_box(NESTED)).try_parse())
    });
}

fn parse_invalid(c: &mut Criterion) {
    // `analyze` is the permissive path used while Source is incomplete.
    c.bench_function("parse_invalid", |b| {
        b.iter(|| Parser::from(black_box(INVALID)).analyze())
    });
}

fn execute_function(c: &mut Criterion) {
    // The first Tick of a Playback run, at the Grid origin: no Function reads
    // either input yet, and the measurement is of evaluation rather than of
    // any one Tick. One scalar Add over resolved operands is the call a Turn
    // makes for the commonest Expression.
    let inputs = TickInputs::new(Tick::ZERO, Anchor::new(0, 0));
    let operands = [
        Value::Atom(Atom::Number(0x01)),
        Value::Atom(Atom::Number(0x02)),
    ];

    // The operands are handed over by value, so each iteration's copy is
    // built outside the timed routine.
    c.bench_function("execute_function", |b| {
        b.iter_batched(
            || operands.clone(),
            |operands| {
                Interpreter::execute_function(
                    black_box(Function::Add),
                    black_box(operands),
                    black_box(inputs).into(),
                )
            },
            BatchSize::SmallInput,
        )
    });
}

/// The members of every Sequence operand below: long enough that copying them
/// is visible beside the Turn's fixed cost.
const MEMBERS: usize = 64;

fn numbers(length: usize) -> Value {
    Sequence::new((0..length).map(|index| Atom::Number(index as u8)))
        .expect("Numbers are members")
        .into()
}

/// One Turn through `Interpreter::execute_function`, the call `orcvs` makes
/// once per Function per Tick. Each iteration is handed its operands by value,
/// built outside the timed routine, so what is timed is the Turn: the Operand
/// Stack, the bind, the Function body and the answer — including any copy of
/// a Sequence operand's members on the way.
fn execute_function_operands(c: &mut Criterion) {
    let inputs = TickInputs::new(Tick::ZERO, Anchor::new(0, 0));
    let turns: [(&str, Function, Vec<Value>); 6] = [
        (
            "Add",
            Function::Add,
            vec![Atom::Number(1).into(), Atom::Number(2).into()],
        ),
        (
            "ConvertToNote",
            Function::ConvertToNote,
            vec![Atom::Number(0x3C).into()],
        ),
        ("Reverse", Function::Reverse, vec![numbers(MEMBERS)]),
        (
            "Replace",
            Function::Replace,
            vec![
                Atom::Number(3).into(),
                Atom::Number(7).into(),
                numbers(MEMBERS),
            ],
        ),
        (
            "Concatenate",
            Function::Concatenate,
            vec![numbers(MEMBERS), numbers(MEMBERS)],
        ),
        (
            "Subtract",
            Function::Subtract,
            vec![numbers(MEMBERS), Atom::Number(1).into()],
        ),
    ];

    let mut group = c.benchmark_group("execute_function_operands");
    for (name, function, operands) in turns {
        group.bench_function(name, |b| {
            b.iter_batched(
                || operands.clone(),
                |operands| {
                    Interpreter::execute_function(
                        black_box(function),
                        operands,
                        black_box(inputs).into(),
                    )
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn parse_source(c: &mut Criterion) {
    c.bench_function("parse_source", |b| {
        b.iter(|| {
            let mut units = 0;
            for row in SOURCE.iter().copied() {
                units += Parser::from(black_box(row)).analyze().expression().len();
            }
            units
        })
    });
}

// Keep both sides of the inline-record boundary visible as usage evolves.
fn parse_record_counts(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_records");
    let mut source = String::from("01");
    for records in [3, 7, 15, 31, 63] {
        source = format!(".+{source}{source}");
        group.bench_with_input(
            BenchmarkId::from_parameter(records),
            &source,
            |b, source| {
                b.iter(|| Parser::from(black_box(source.as_str())).analyze());
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    parse,
    parse_invalid,
    execute_function,
    execute_function_operands,
    parse_source,
    parse_record_counts
);
criterion_main!(benches);
