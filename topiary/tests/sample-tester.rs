use std::fs;
use std::io::BufReader;
use std::path::Path;

use log::info;
use test_log::test;

use topiary::{
    apply_query, formatter, test_utils::pretty_assert_eq, Configuration, FormatterError, Language,
    Operation, TopiaryQuery,
};

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
            let query_content = fs::read_to_string(language.query_file().unwrap()).unwrap();

            let grammar = language.grammar().await.unwrap();

            let query = TopiaryQuery::new(grammar, &query_content).unwrap();

            info!(
                "Formatting file {} as {}.",
                file.path().display(),
                language.name,
            );

            info!("Formatting {}", file.path().display());

            formatter(
                &mut input,
                &mut output,
                &query,
                language,
                grammar,
                Operation::Format {
                    skip_idempotence: false,
                    tolerate_parsing_errors: true,
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
    let language_dir = fs::read_dir("../queries").unwrap();

    for file in language_dir {
        let file = file.unwrap();
        let language = Language::detect(file.path(), &config).unwrap();

        let expected = fs::read_to_string(file.path()).unwrap();

        let mut input = BufReader::new(fs::File::open(file.path()).unwrap());
        let mut output = Vec::new();
        let query_content = fs::read_to_string(language.query_file().unwrap()).unwrap();

        let grammar = language.grammar().await.unwrap();

        let query = TopiaryQuery::new(grammar, &query_content).unwrap();

        formatter(
            &mut input,
            &mut output,
            &query,
            language,
            grammar,
            Operation::Format {
                skip_idempotence: false,
                tolerate_parsing_errors: false,
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
        // We skip "ocaml-interface.mli", as its query file is already tested by "ocaml.ml"
        if file.file_name().to_string_lossy() == "ocaml-interface.mli" {
            continue;
        }
        let language = Language::detect(file.path(), &config).unwrap();
        let query_file = language.query_file().unwrap();

        let input_content = fs::read_to_string(&file.path()).unwrap();
        let query_content = fs::read_to_string(&query_file).unwrap();

        let grammar = language.grammar().await.unwrap();

        let query = TopiaryQuery::new(grammar, &query_content).unwrap();

        apply_query(&input_content, &query, grammar, false, true).unwrap_or_else(|e| {
            if let FormatterError::PatternDoesNotMatch(_) = e {
                panic!("Found untested query in file {query_file:?}:\n{e}");
            } else {
                panic!("{e}");
            }
        });
    }
}
