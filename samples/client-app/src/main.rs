use topiary_core::{formatter, Configuration, Operation, TopiaryQuery};

#[tokio::main]
async fn main() {
    let mut input =
        "{\"name\":\"John Doe\",\"age\":43,\n\"phones\":[\"+44 1234567\",\"+44 2345678\"]}"
            .as_bytes();

    let mut output = Vec::new();
    let query = TopiaryQuery::json();
    let config = Configuration::parse_default_configuration().expect("config");
    let language = config.get_language("json").expect("language");
    let grammar = language.grammar().await.expect("grammar");

    formatter(
        &mut input,
        &mut output,
        &query,
        &language,
        &grammar,
        Operation::Format {
            skip_idempotence: false,
            tolerate_parsing_errors: false,
        },
    )
    .expect("formatter");

    let formatted = String::from_utf8(output).expect("valid utf-8");
    println!("Here is some formatted JSON:\n{formatted}");
}
