import AceEditor from "react-ace";

import "ace-builds/src-noconflict/mode-html";
import "ace-builds/src-noconflict/theme-solarized_light";
import "ace-builds/src-noconflict/ext-language_tools";

export default function Editor(props: {
    id: string,
    value: string,
    readOnly?: boolean,
    onChange?: (value: string) => void
}) {
    return (
        <AceEditor mode="html"
            theme="solarized_light" name={props.id} value={props.value} readOnly={props.readOnly} onChange={props.onChange} editorProps={{ $blockScrolling: true }}
            width="200px"
        />
    )
}
