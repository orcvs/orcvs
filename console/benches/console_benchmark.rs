use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use lang::{eval, midi_note_to_number, midi_note_to_number_hash, parse, Atom, Function};
use rand::Rng;

fn lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("Lookup");

    let size = 1000;
    let mut array: Vec<Option<String>> = vec![None; size];
    let mut lookup: Vec<Option<String>> = vec![None; 100];

    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let index = rng.gen_range(0..size);
        array[index] = Some(index.to_string());
    }

    for index in 0..100 {
        lookup[index] = Some(index.to_string());
    }

    for i in 0..10 {
        group.bench_function(BenchmarkId::new("Array", i), |b| {
            b.iter(|| {
                let r = array.iter().filter(|x| x.is_some());
                let r = r.map(|x| x.as_ref().unwrap());

                let v: Vec<&String> = r.collect();
                black_box(v);
            })
        });
        group.bench_function(BenchmarkId::new("Lookup", i), |b| {
            b.iter(|| {
                let r = lookup.iter().map(|x| black_box(x.as_ref().unwrap()));

                let v: Vec<&String> = r.collect();
                black_box(v);
            })
        });
    }
    group.finish();
}

criterion_group!(benches, lookup);
// criterion_group!(benches, parser_benchmark);
// criterion_group!(benches, eval_benchmark);
criterion_main!(benches);
