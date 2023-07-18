// Import necessary modules
use topiary_config::Configuration;
use topiary_core::{formatter, FormatConfiguration, Language, Operation, TopiaryQueries};

#[tokio::main]
async fn main() {
    // Define input JSON string
    let mut input =
        "{\"name\":\"John Doe\",\"age\":43,\n\"phones\":[\"+44 1234567\",\"+44 2345678\"]}"
            .as_bytes();
    let mut output = Vec::new();

    // Load configuration
    let config = Configuration::default();

    // Get JSON language configuration
    let json = config.get_language("json").unwrap();

    // Get default query for JSON
    let query = topiary_queries::json();

    // Get grammar for JSON language
    let grammar = json.grammar().unwrap();

    // Create Language struct
    let language: Language = Language {
        name: "json".to_owned(),
        query: TopiaryQueries::new(&grammar, query, None).unwrap(),
        grammar,
        indent: None,
    };

    // Format the input JSON using the language configuration
    formatter(
        &mut input,
        &mut output,
        &language,
        Operation::Format(FormatConfiguration {
            skip_idempotence: false,
            tolerate_parsing_errors: false,
        }),
        Vec::new(),
    )
    .unwrap();

    // Convert the formatted output to a string
    let formatted = String::from_utf8(output).expect("valid utf-8");

    // Print the formatted JSON
    println!("Here is some formatted JSON:\n{formatted}");
}
