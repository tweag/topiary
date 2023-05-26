import { ReactElement, useCallback, useEffect, useRef, useState } from "react";
import useDebounce from "./hooks/useDebounce";
import languages from './samples/languages_export';
import init, {
    topiaryInit,
    format,
} from "./wasm-app/topiary_playground.js";
import "./App.css";

const debounceDelay = 500;

function App() {
    const [isInitialised, setIsInitialised] = useState(false);
    const initCalled = useRef(false);
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

    const runFormat = useCallback((i: string, q: string) => {
        const outputFormat = async () => {
            setOutput(await format(i, q, currentLanguage));
        }

        try {
            if (!isInitialised) {
                setOutput("Cannot format yet, as the formatter engine is being initialised. Try again soon.");
                return;
            }

            setOutput("Formatting ...");
            outputFormat();
        } catch (e) {
            setOutput(String(e));
        }
    }, [currentLanguage, isInitialised]);

    // Init page (runs only once, but twice in strict mode in dev)
    useEffect(() => {
        const initWasm = async () => {
            // Make sure we only run this once
            if (initCalled.current) return;
            initCalled.current = true;

            await init();
            await topiaryInit();
            setIsInitialised(true);
        }

        // Populate the language list
        let languageItems: ReactElement[] = [];

        for (let l in languages) {
            let displayName = languages[l].supported === "true" ? l : l + " (experimental)";
            languageItems.push(<option key={l} value={l}>{displayName}</option>)
        }

        setLanguageOptions(languageItems);

        // Async in useEffect needs to be handled like this:
        initWasm()
            .catch(console.error);
    }, []);

    // Run on every (debounced) input change, as well as when isInitialised is set.
    useEffect(() => {
        if (!onTheFlyFormatting) return;
        runFormat(debouncedInput, debouncedQuery);
    }, [isInitialised, debouncedInput, debouncedQuery, onTheFlyFormatting, runFormat])

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

    function handleFormat() {
        runFormat(input, query);
    };

    function handleOnTheFlyFormatting() {
        setOnTheFlyFormatting(!onTheFlyFormatting);
    };

    return (
        <div className="App">
            <div className="header">
                <button id="formatButton" className="btn btn-primary" onClick={handleFormat}>
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
