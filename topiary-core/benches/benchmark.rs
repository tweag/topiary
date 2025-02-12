use criterion::async_executor::FuturesExecutor;
use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;
use std::io;
use topiary_core::{formatter, Language, Operation, TopiaryQuery};

async fn format() {
    let input = fs::read_to_string("../topiary-cli/tests/samples/input/json.json").unwrap();
    let query_content = fs::read_to_string("../topiary-queries/queries/json.scm").unwrap();
    let json = tree_sitter_json::LANGUAGE;

    let mut input = input.as_bytes();
    let mut output = io::BufWriter::new(Vec::new());

    let language: Language = Language {
        name: "json".to_owned(),
        query: TopiaryQuery::new(&json.into(), &query_content).unwrap(),
        grammar: json.into(),
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
    c.bench_function("format_json", |b| {
        b.to_async(FuturesExecutor).iter(format);
    });
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
