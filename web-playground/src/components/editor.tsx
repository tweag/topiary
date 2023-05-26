import AceEditor from "react-ace";

export default function Editor(props: {
    id: string,
    value: string,
    readOnly?: boolean,
    onChange?: (value: string) => void
}) {
    return (
        <AceEditor value={props.value} readOnly={props.readOnly} onChange={props.onChange} />
    )
}
