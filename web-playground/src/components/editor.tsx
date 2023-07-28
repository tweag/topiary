import AceEditor from "react-ace";

import "ace-builds/src-noconflict/ext-language_tools";
import "ace-builds/src-noconflict/theme-sqlserver"

import "ace-builds/src-noconflict/mode-json";
import "ace-builds/src-noconflict/mode-ocaml";
import "ace-builds/src-noconflict/mode-plain_text";
import "ace-builds/src-noconflict/mode-rust";
import "ace-builds/src-noconflict/mode-scheme";
import "ace-builds/src-noconflict/mode-sh";
import "ace-builds/src-noconflict/mode-toml";

import "ace-builds/src-noconflict/snippets/sh";

export default function Editor(props: {
    id: string,
    value: string,
    placeholder: string,
    language: string;
    readOnly?: boolean,
    onChange?: (value: string) => void
}) {
    return (
        <AceEditor
            mode={toMode(props.language)}
            theme="sqlserver"
            name={props.id}
            value={props.value}
            placeholder={props.placeholder}
            readOnly={props.readOnly}
            onChange={props.onChange}
            width="100%"
            height="100%"
            tabSize={2}
            enableBasicAutocompletion={true}
            enableLiveAutocompletion={true}
            enableSnippets={true}
        />
    )
}

function toMode(language: string) {
    switch (language) {
        case "bash":
            return "sh";
        case "json":
            return "json";
        case "ocaml":
        case "ocaml-interface":
        case "ocamllex":
            return "ocaml";
        case "nickel":
            // Missing highlighting for Nickel, but we know.
            return "plain_text";
        case "rust":
            return "rust";
        case "toml":
            return "toml";
        case "tree-sitter-query":
            return "scheme";
        default:
            console.warn(`Missing syntax highlighting for ${language}.`);
            return "plain_text";
    }
}
