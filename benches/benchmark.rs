use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;
use std::io;
use topiary::Configuration;
use topiary::{formatter, Operation};

fn criterion_benchmark(c: &mut Criterion) {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let input = fs::read_to_string("tests/samples/input/ocaml.ml").unwrap();
            let query = fs::read_to_string("languages/ocaml.scm").unwrap();
            c.bench_function("format ocaml", |b| {
                b.iter(|| async {
                    let configuration = Configuration::parse(&query).unwrap();
                    let grammars = configuration.language.grammars().await.unwrap();

                    let mut input = input.as_bytes();
                    let mut query = query.as_bytes();
                    let mut output = io::BufWriter::new(Vec::new());

                    formatter(
                        &mut input,
                        &mut output,
                        &mut query,
                        &grammars,
                        &configuration,
                        Operation::Format {
                            skip_idempotence: false,
                        },
                    )
                })
            });
        });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
