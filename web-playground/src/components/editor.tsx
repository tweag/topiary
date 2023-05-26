export default function Editor(props: {
    id: string,
    value: string,
    readOnly?: boolean,
    onChange?: (e: React.ChangeEvent<HTMLTextAreaElement | HTMLInputElement>) => void
}) {
    return (
        <textarea id={props.id} value={props.value} readOnly={props.readOnly} onChange={props.onChange} />
    )
}
