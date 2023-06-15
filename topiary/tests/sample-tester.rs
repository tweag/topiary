use std::fs;
use std::io::BufReader;
use std::path::Path;

use log::info;
use prettydiff::text::{diff_lines, ContextConfig};
use test_log::test;

use topiary::{apply_query, formatter, Configuration, FormatterError, Language, Operation};

fn pretty_assert_eq(v1: &str, v2: &str) {
    if v1 != v2 {
        let diff = diff_lines(v1, v2);
        panic!(
            "\n{}",
            diff.format_with_context(
                Some(ContextConfig {
                    context_size: 2,
                    skipping_marker: "...",
                }),
                true,
            )
        )
    }
}

#[test(tokio::test)]
async fn input_output_tester() {
    let input_dir = fs::read_dir("tests/samples/input").unwrap();
    let expected_dir = Path::new("tests/samples/expected");
    let config = Configuration::parse_default_configuration().unwrap();
    let extensions = config.known_extensions();

    for file in input_dir {
        let file = file.unwrap();
        if let Some(ext) = file.path().extension().map(|ext| ext.to_string_lossy()) {
            if !extensions.contains(ext.as_ref()) {
                continue;
            }

            let language = Language::detect(file.path(), &config).unwrap();

            let expected_path = expected_dir.join(file.file_name());
            let expected = fs::read_to_string(expected_path).unwrap();

            let mut input = BufReader::new(fs::File::open(file.path()).unwrap());
            let mut output = Vec::new();
            let query = fs::read_to_string(language.query_file().unwrap()).unwrap();

            let grammar = language.grammar().await.unwrap();

            info!(
                "Formatting file {} as {}.",
                file.path().display(),
                language.name,
            );

            formatter(
                &mut input,
                &mut output,
                &query,
                language,
                &grammar,
                Operation::Format {
                    skip_idempotence: false,
                },
            )
            .unwrap();

            let formatted = String::from_utf8(output).unwrap();
            log::debug!("{}", formatted);

            pretty_assert_eq(&expected, &formatted);
        }
    }
}

// Test that our query files are properly formatted
#[test(tokio::test)]
async fn formatted_query_tester() {
    let config = Configuration::parse_default_configuration().unwrap();
    let language_dir = fs::read_dir("../languages").unwrap();

    for file in language_dir {
        let file = file.unwrap();
        let language = Language::detect(file.path(), &config).unwrap();

        let expected = fs::read_to_string(file.path()).unwrap();

        let mut input = BufReader::new(fs::File::open(file.path()).unwrap());
        let mut output = Vec::new();
        let query = fs::read_to_string(language.query_file().unwrap()).unwrap();

        let grammar = language.grammar().await.unwrap();

        formatter(
            &mut input,
            &mut output,
            &query,
            language,
            &grammar,
            Operation::Format {
                skip_idempotence: false,
            },
        )
        .unwrap();

        let formatted = String::from_utf8(output).unwrap();
        log::debug!("{}", formatted);

        pretty_assert_eq(&expected, &formatted);
    }
}

// Test that all queries are used on sample files
#[test(tokio::test)]
async fn exhaustive_query_tester() {
    let config = Configuration::parse_default_configuration().unwrap();
    let input_dir = fs::read_dir("tests/samples/input").unwrap();

    for file in input_dir {
        let file = file.unwrap();
        // We skip "ocaml.mli", as its query file is already tested by "ocaml.ml"
        if file.file_name().to_string_lossy() == "ocaml-interface.mli" {
            continue;
        }
        let language = Language::detect(file.path(), &config).unwrap();
        let query_file = language.query_file().unwrap();

        let input_content = fs::read_to_string(&file.path()).unwrap();
        let query_content = fs::read_to_string(&query_file).unwrap();

        let grammar = language.grammar().await.unwrap();

        apply_query(&input_content, &query_content, &grammar, true).unwrap_or_else(|e| {
            if let FormatterError::PatternDoesNotMatch(_) = e {
                panic!("Found untested query in file {query_file:?}:\n{e}");
            } else {
                panic!("{e}");
            }
        });
    }
}
