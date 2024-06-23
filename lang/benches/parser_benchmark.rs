use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use lang::{Interpreter, Parser};

fn parse(exp: &String) {
    let mut exp = exp.clone();

    let mut parser = Parser::new(black_box(&mut exp));

    let result = parser.try_parse();
    assert!(result.is_ok());
}

fn try_parse(exp: &String) {
    let mut exp = exp.clone();

    let mut parser = Parser::new(black_box(&mut exp));

    let result = parser.parse();
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

fn interpret(exp: &String) {
    let mut exp = exp.clone();
    let mut parser = Parser::new(&mut exp);

    let result = parser.try_parse();
    assert!(result.is_ok());

    let mut interpreter = Interpreter::new(black_box(&mut parser));
    let result = interpreter.interpret();
    assert!(result.is_ok());
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

criterion_group!(benches, parser_benchmark);
// criterion_group!(benches, invalid_parser_benchmark);
// criterion_group!(benches, interpret_benchmark);

criterion_main!(benches);

// pub fn invalid_parser_benchmark(c: &mut Criterion) {
//     c.bench_function("try_parse_with_invalid", |b| {
//         b.iter(|| {
//             let mut s = String::from("id");
//             let mut parser = Parser::new(black_box(&mut s));

//             let result = parser.try_parse();
//             let _result = black_box(result);
//         })
//     });

//     c.bench_function("parse_with_invalid", |b| {
//         b.iter(|| {
//             let mut s = String::from("id");
//             let mut parser = Parser::new(black_box(&mut s));

//             let result = parser.parse();
//             let _result = black_box(result);
//         })
//     });
// }
