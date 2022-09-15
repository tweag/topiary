use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;
use std::io;
use tree_sitter_formatter::{formatter, Language};

fn criterion_benchmark(c: &mut Criterion) {
    let input_path = "tests/samples/input/ocaml.ml";
    let input_content = fs::read_to_string(input_path).unwrap();

    c.bench_function("format ocaml", |b| {
        b.iter(|| {
            let mut input = input_content.as_bytes();
            let mut output = io::BufWriter::new(Vec::new());

            formatter(&mut input, &mut output, Language::Ocaml)
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
