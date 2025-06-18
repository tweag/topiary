use criterion::async_executor::FuturesExecutor;
use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;
use std::io;
use topiary_core::{formatter, Language, Operation, TopiaryQuery};

async fn format(input: &String, language: &Language) {
    let mut input = input.as_bytes();
    let mut output = io::BufWriter::new(Vec::new());

    formatter(
        &mut input,
        &mut output,
        language,
        Operation::Format {
            skip_idempotence: true,
            tolerate_parsing_errors: false,
        },
    )
    .unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let input = fs::read_to_string("../topiary-cli/tests/samples/input/nickel.ncl").unwrap();

    let (grammar, query) = {
        let config = topiary_config::Configuration::default();
        let nickel = config.get_language("nickel").unwrap();

        (nickel.grammar().unwrap(), topiary_queries::nickel())
    };

    let language = Language {
        name: "nickel".to_owned(),
        query: TopiaryQuery::new(&grammar, query).unwrap(),
        grammar,
        indent: None,
    };

    c.bench_function("format_nickel", |b| {
        b.to_async(FuturesExecutor)
            .iter(|| format(&input, &language));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
