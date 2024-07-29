use criterion::async_executor::FuturesExecutor;
use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;
use std::io;
use topiary_core::{formatter, Language, Operation, TopiaryQuery};

async fn format() {
    let input = fs::read_to_string("../topiary-cli/tests/samples/input/ocaml.ml").unwrap();
    let query_content = fs::read_to_string("../topiary-queries/queries/ocaml.scm").unwrap();
    let ocaml = tree_sitter_ocaml::language_ocaml();

    let mut input = input.as_bytes();
    let mut output = io::BufWriter::new(Vec::new());

    let language: Language = Language {
        name: "ocaml".to_owned(),
        query: TopiaryQuery::new(&ocaml.clone().into(), &query_content).unwrap(),
        grammar: ocaml.into(),
        indent: None,
    };

    formatter(
        &mut input,
        &mut output,
        &language,
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
