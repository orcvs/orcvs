use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use lang::{Interpreter, Parser};

fn parse(exp: &String) {
    let mut exp = exp.clone();

    let result = Parser::from(black_box(&mut exp)).try_parse();
    assert!(result.is_ok());
}

fn try_parse(exp: &String) {
    let mut exp = exp.clone();

    let result = Parser::from(black_box(&mut exp)).try_parse();
    assert!(result.is_ok());
}

fn interpret(exp: &String) {
    let mut exp = exp.clone();
    let result = Parser::from(black_box(&mut exp)).try_parse();

    assert!(result.is_ok());

    let mut interpreter = Interpreter::new(black_box(result.unwrap()));
    let result = interpreter.interpret();
    assert!(result.is_ok());
}

pub fn parser_benchmark(c: &mut Criterion) {
    let expressions = [
        String::from("idAA"),
        String::from("idididAA"),
        String::from("++0A01"),
        String::from("--0A01"),
        String::from("++idididAA01"),
        String::from("++++0101--0A05"),
    ];

    let mut group = c.benchmark_group("parse");
    for exp in expressions.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(exp), exp, |b, exp| {
            b.iter(|| parse(exp));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("try_parse");
    for exp in expressions.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(exp), exp, |b, exp| {
            b.iter(|| try_parse(exp));
        });
    }
    group.finish();
}

pub fn interpret_benchmark(c: &mut Criterion) {
    let expressions = [
        String::from("idAA"),
        String::from("idididAA"),
        String::from("++0A01"),
        String::from("--0A01"),
        String::from("++idididAA01"),
        String::from("++++0101--0A05"),
    ];

    let mut group = c.benchmark_group("interpret");
    for exp in expressions.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(exp), exp, |b, exp| {
            b.iter(|| interpret(exp));
        });
    }
    group.finish();
}

// criterion_group!(benches, parser_benchmark);
// criterion_group!(benches, invalid_parser_benchmark);
criterion_group!(benches, interpret_benchmark);

criterion_main!(benches);
