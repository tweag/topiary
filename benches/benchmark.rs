use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;
use std::io;
use topiary::formatter;

fn criterion_benchmark(c: &mut Criterion) {
    let input = fs::read_to_string("tests/samples/input/ocaml.ml").unwrap();
    let query = fs::read_to_string("languages/ocaml.scm").unwrap();

    // TODO!
    // c.bench_function("format ocaml", |b| {
    //     b.iter(|| {
    //         let mut input = input.as_bytes();
    //         let mut query = query.as_bytes();
    //         let mut output = io::BufWriter::new(Vec::new());

    //         formatter(&mut input, &mut output, &mut query, false)
    //     })
    // });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
