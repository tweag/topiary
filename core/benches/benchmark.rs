use criterion::async_executor::FuturesExecutor;
use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;
use std::io;
use topiary_core::{formatter, Operation};
use topiary_core::{Configuration, TopiaryQuery};

async fn format() {
    let input = fs::read_to_string("tests/samples/input/ocaml.ml").unwrap();
    let query_content = fs::read_to_string("../queries/queries/ocaml.scm").unwrap();
    let configuration = Configuration::parse_default_configuration().unwrap();
    let language = configuration.get_language("ocaml").unwrap();
    let grammar = language.grammar().await.unwrap();
    let query = TopiaryQuery::new(&grammar, &query_content).unwrap();

    let mut input = input.as_bytes();
    let mut output = io::BufWriter::new(Vec::new());

    formatter(
        &mut input,
        &mut output,
        &query,
        language,
        &grammar,
        Operation::Format {
            skip_idempotence: true,
            tolerate_parsing_errors: false,
        },
    )
    .unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("format_ocaml", |b| {
        b.to_async(FuturesExecutor).iter(format);
    });
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
