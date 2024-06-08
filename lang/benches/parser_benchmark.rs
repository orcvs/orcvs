use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use lang::{Interpreter, Parser};

pub fn parser_benchmark(c: &mut Criterion) {
    c.bench_function("parse", |b| {
        b.iter(|| {
            let mut s = String::from("idididAA");
            let mut parser = Parser::new(black_box(&mut s));

            let result = parser.try_parse();
            let _result = black_box(result);
        })
    });

    c.bench_function("try_parse", |b| {
        b.iter(|| {
            let mut s = String::from("idididAA");
            let mut parser = Parser::new(black_box(&mut s));

            let result = parser.parse();
            let _result = black_box(result);
        })
    });
}
pub fn invalid_parser_benchmark(c: &mut Criterion) {
    c.bench_function("try_parse_with_invalid", |b| {
        b.iter(|| {
            let mut s = String::from("id");
            let mut parser = Parser::new(black_box(&mut s));

            let result = parser.try_parse();
            let _result = black_box(result);
        })
    });

    c.bench_function("parse_with_invalid", |b| {
        b.iter(|| {
            let mut s = String::from("id");
            let mut parser = Parser::new(black_box(&mut s));

            let result = parser.parse();
            let _result = black_box(result);
        })
    });
}

fn interpret(exp: &String) {
    let mut exp = exp.clone();
    let mut parser = Parser::new(&mut exp);

    let result = parser.parse();
    assert!(result.is_ok());

    let mut interpreter = Interpreter::new(black_box(parser.pool));
    let result = interpreter.interpret();
    assert!(result.is_ok());
}

pub fn interpret_benchmark(c: &mut Criterion) {
    let expressions = [
        "idAA".to_string(),
        "idididAA".to_string(),
        "++0A01".to_string(),
        "--0A01".to_string(),
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
