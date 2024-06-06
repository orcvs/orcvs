use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use lang::{midi_note_to_number, midi_note_to_number_hash, Parser};

fn midi_lookup_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("Midi Lookup");

    let notes = ["f2", "G2", "g2", "A2", "a2", "B2", "C3", "c3", "D3"];

    for n in notes.iter() {
        group.bench_with_input(BenchmarkId::new("Match", n), n, |b, n| {
            b.iter(|| midi_note_to_number(*n))
        });
        group.bench_with_input(BenchmarkId::new("Hash", n), n, |b, n| {
            b.iter(|| midi_note_to_number_hash(*n))
        });
    }
    group.finish();
}

pub fn parser_benchmark(c: &mut Criterion) {
    // let mut s = "plidXY0AC4"; //basic

    c.bench_function("parse", |b| {
        b.iter(|| {
            let mut s = String::from("idididAA");
            let mut parser = Parser::new(black_box(&mut s));

            let result = parser.parse();
            let _result = black_box(result);
        })
    });
}

pub fn eval_benchmark(c: &mut Criterion) {
    // let mut s = "plidXY0AC4"; //basic
    // let a = Atom::from(Function::Add(Atom::Number(1), Atom::Number(2)));
    // c.bench_function("eval", |b| b.iter(|| eval(black_box(a.clone()))));
}

// criterion_group!(benches, midi_lookup_benchmarks);
criterion_group!(benches, parser_benchmark);
// criterion_group!(benches, eval_benchmark);
criterion_main!(benches);
