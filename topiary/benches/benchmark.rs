use criterion::async_executor::FuturesExecutor;
use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;
use std::io;
use topiary::Configuration;
use topiary::{formatter, Operation};

async fn format() {
    let input = fs::read_to_string("../topiary/tests/samples/input/ocaml.ml").unwrap();
    let query = fs::read_to_string("../languages/ocaml.scm").unwrap();
    let configuration = Configuration::parse(&query).unwrap();
    let grammars = configuration.language.grammars().await.unwrap();

    let mut input = input.as_bytes();
    let mut output = io::BufWriter::new(Vec::new());

    formatter(
        &mut input,
        &mut output,
        &query,
        &configuration,
        &grammars,
        Operation::Format {
            skip_idempotence: true,
        },
    )
    .unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("format_ocaml", |b| {
        b.to_async(FuturesExecutor).iter(format);
    });
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
