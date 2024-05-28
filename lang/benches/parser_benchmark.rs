use criterion::{
    black_box, criterion_group, criterion_main, Criterion,
};
use lang::{eval, parse, Atom, Function};

pub fn parser_benchmark(c: &mut Criterion) {
    // let mut s = "plidXY0AC4"; //basic
    let mut s = "idididAA"; //recursive
    c.bench_function("parse", |b| {
        b.iter(|| parse(black_box(&mut s)))
    });
}

pub fn eval_benchmark(c: &mut Criterion) {
    // let mut s = "plidXY0AC4"; //basic
    let a = Atom::from(Function::Add(Atom::Num(1), Atom::Num(2)));
    c.bench_function("eval", |b| {
        b.iter(|| eval(black_box(a.clone())))
    });
}

// criterion_group!(benches, parser_benchmark);
criterion_group!(benches, eval_benchmark);
criterion_main!(benches);
