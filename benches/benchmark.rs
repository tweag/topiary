use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use std::io::{BufReader, BufWriter};
use tree_sitter_formatter::{formatter, Language};

fn criterion_benchmark(c: &mut Criterion) {
    let input_path = "tests/samples/input/ocaml.ml";

    c.bench_function("format ocaml", |b| {
        b.iter(|| {
            let mut input = BufReader::new(fs::File::open(input_path).unwrap());
            let mut output = BufWriter::new(Vec::new());

            formatter(&mut input, &mut output, Language::Ocaml)
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
