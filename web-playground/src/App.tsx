import { ReactElement, useEffect, useState } from "react";
import "./App.css";
import useDebounce from "./hooks/useDebounce";
import init, {
    topiaryInit,
    format,
} from "./wasm-app/topiary_playground.js";
import languages from './samples/languages_export';

const debounceDelay = 500;

function App() {
    const [isInitialised, setIsInitialised] = useState(false);
    const defaultLanguage = "json";
    const defaultQuery = languages[defaultLanguage].query;
    const defaultInput = languages[defaultLanguage].input;
    const [languageOptions, setLanguageOptions] = useState([] as ReactElement[]);
    const [currentLanguage, setCurrentLanguage] = useState(defaultLanguage);
    const [onTheFlyFormatting, setOnTheFlyFormatting] = useState(true);
    const [query, setQuery] = useState(defaultQuery);
    const [input, setInput] = useState(defaultInput);
    const [output, setOutput] = useState("");

    // We want to debounce the input and query changes so that we can run
    // on-the-fly formatting after the user stops typing.
    const debouncedInput = useDebounce(input, debounceDelay);
    const debouncedQuery = useDebounce(query, debounceDelay);

    // Init page (run only once)
    useEffect(() => {
        let languageItems: ReactElement[] = [];

        for (let l in languages) {
            let displayName = languages[l].supported === "true" ? l : l + " (experimental)";
            languageItems.push(<option value={l}>{displayName}</option>)
        }

        setLanguageOptions(languageItems);
    }, [])

    // Run on every (debounced) input change
    useEffect(() => {
        if (!onTheFlyFormatting) return;
        runFormat();
    }, [onTheFlyFormatting, debouncedInput, debouncedQuery])

    async function runFormat() {
        try {
            if (!isInitialised) {
                await init();
                await topiaryInit();
                setIsInitialised(true);
            }

            setOutput("Formatting ...");
            setOutput(await format(input, query, currentLanguage));
        } catch (e) {
            setOutput(String(e));
        }
    }

    function changeLanguage(l: string) {
        if (languages[l]) {
            let hasModification =
                input !== languages[currentLanguage].input
                || query !== languages[currentLanguage].query;
            let confirmationMessage = "Modifications to the input and query " +
                "are going to be overwritten if you change the language. " +
                "Do you wish to proceed?";
            if (!hasModification || window.confirm(confirmationMessage)) {
                setInput(languages[l].input);
                setQuery(languages[l].query);
                setOutput("");
                setCurrentLanguage(l);
            }
        }
    }

    function handleOnTheFlyFormatting() {
        setOnTheFlyFormatting(!onTheFlyFormatting);
    };

    return (
        <div className="App">
            <div className="header">
                <button id="formatButton" className="btn btn-primary" onClick={runFormat}>
                    Format
                </button>
                <select id="languageMenu" onChange={e => changeLanguage(e.target.value)}>
                    <option value="">Choose a reference language</option>
                    {languageOptions}
                </select>
                <div className="headerColumn">
                    <label>
                        <input type="checkbox" id="onTheFlyFormatting" checked={onTheFlyFormatting} onChange={handleOnTheFlyFormatting} />
                        On-the-fly formatting
                    </label>
                </div>
            </div>
            <div className="columns">
                <div className="column">
                    <h1>Query</h1>
                    <textarea id="query" value={query} onChange={e => setQuery(e.target.value)} />
                </div>
                <div className="column">
                    <h1>Input</h1>
                    <textarea id="input" value={input} onChange={e => setInput(e.target.value)} />
                </div>
                <div className="column">
                    <h1>Output</h1>
                    <textarea id="output" value={output} readOnly></textarea>
                </div>
            </div>
        </div>
    );
}

export default App;
