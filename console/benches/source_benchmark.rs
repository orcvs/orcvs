use console::source::Source;
use criterion::{criterion_group, criterion_main, Criterion};

use rand::{distributions::Alphanumeric, Rng};

fn edit() {
    let cols = 200;
    let rows = 200;

    let mut source = Source::new(cols, rows);

    for _i in 0..100 {
        let len = rand::thread_rng().gen_range(0..11);

        let x = rand::thread_rng().gen_range(0..(200 - len));
        let y = rand::thread_rng().gen_range(0..200);

        let random: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(len)
            .map(char::from)
            .collect();

        for (i, c) in random.chars().enumerate() {
            source.set_at(x + i, y, &c.to_string());
        }
    }

    let expressions = [
        String::from("idAA"),
        String::from("idididAA"),
        String::from("++0A01"),
        String::from("--0A01"),
        String::from("++idididAA01"),
        String::from("++++0101--0A05"),
    ];

    for exp in expressions {
        let x = rand::thread_rng().gen_range(0..(200 - exp.len()));
        let y = rand::thread_rng().gen_range(0..200);

        for (i, c) in exp.chars().enumerate() {
            source.set_at(x + i, y, &c.to_string());
        }
    }
    // println!("{:?}", source.as_string());
}

pub fn simplified_benchmark(c: &mut Criterion) {
    c.bench_function("simplified", |b| b.iter(|| edit()));
}

criterion_group!(benches, simplified_benchmark);

criterion_main!(benches);
