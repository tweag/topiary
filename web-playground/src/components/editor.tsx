import AceEditor from "react-ace";

import "ace-builds/src-noconflict/ext-language_tools";
import "ace-builds/src-noconflict/mode-plain_text";
import "ace-builds/src-noconflict/theme-clouds.js";

export default function Editor(props: {
    id: string,
    value: string,
    placeholder: string,
    readOnly?: boolean,
    onChange?: (value: string) => void
}) {
    return (
        <AceEditor
            mode="plain_text"
            theme="clouds"
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

