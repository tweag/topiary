import { ReactElement, useCallback, useEffect, useRef, useState } from "react";
import Editor from "./components/editor";
import useDebounce from "./hooks/useDebounce";
import languages from './samples/languages_export';
import init, {
    topiaryInit,
    queryInit,
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
    const [idempotence, setIdempotence] = useState(false);
    const [tolerateParsingErrors, setTolerateParsingErrors] = useState(false);
    const [input, setInput] = useState(defaultInput);
    const [output, setOutput] = useState("");
    const [query, setQuery] = useState(defaultQuery);
    const [queryChanged, setQueryChanged] = useState(true);
    const [processingTime, setProcessingTime] = useState(0);

    // We want to debounce the input and query changes so that we can run
    // on-the-fly formatting after the user stops typing.
    const debouncedInput = useDebounce(input, debounceDelay);
    const debouncedQuery = useDebounce(query, debounceDelay);

    // Init page (runs only once, but twice in strict mode in dev)
    useEffect(() => {
        const initWasm = async () => {
            // Make sure we only run this once
            if (initCalled.current) return;
            initCalled.current = true;

            await init(); // Does the WebAssembly.instantiate()
            await topiaryInit(); // Does the TreeSitter::init()
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

        console.log(`On the fly formatting kicking in.`);
        runFormat(debouncedInput, debouncedQuery, queryChanged);
    }, [isInitialised, debouncedInput, debouncedQuery, queryChanged, onTheFlyFormatting])

    function runFormat(i: string, q: string, qChanged: boolean) {
        console.log(`runFormat`);

        const outputFormat = async () => {
            try {
                const start = performance.now();

                if (qChanged) {
                    console.log(`Initialising ${currentLanguage} with ${q} because ${qChanged}`);
                    let fut = queryInit(q, currentLanguage);
                    console.log(`future: ${fut}`);
                    let res = await fut;
                    console.log(`queryInit result: ${res}`);
                    setQueryChanged(false);
                }

                console.log(`Formatting`);
                setOutput(await format(i, idempotence, tolerateParsingErrors));
                setProcessingTime(performance.now() - start);
            } catch (e) {
                setOutput(String(e));
            }
        }

        if (!isInitialised) {
            setOutput("Cannot format yet, as the formatter engine is being initialised. Try again soon.");
            return;
        }

        setOutput("Formatting ...");
        outputFormat();
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
                setQueryChanged(true);
                setOutput("");
                setCurrentLanguage(l);
            }
        }
    }

    function handleFormat() {
        runFormat(input, query, queryChanged);
    };

    function handleOnTheFlyFormatting() {
        setOnTheFlyFormatting(!onTheFlyFormatting);
    };

    function formatNumber(n: number, decimals: number) {
        return n.toFixed(decimals);
    }

    function handleIdempotence() {
        setIdempotence(!idempotence);
    };

    function handleTolerateParsingErrors() {
        setTolerateParsingErrors(!tolerateParsingErrors);
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
                    <label>
                        <input type="checkbox" id="idempotence" checked={idempotence} onChange={handleIdempotence} />
                        Check idempotence (formatting twice is the same as formatting once)
                    </label>
                    <label>
                        <input type="checkbox" id="tolerateParsingErros" checked={tolerateParsingErrors} onChange={handleTolerateParsingErrors} />
                        Tolerate parsing errors
                    </label>
                </div>
                <div className="headerColumn growRightColumn">
                    Processing time: {formatNumber(processingTime, 1)} ms
                </div>
            </div>
            <div className="columns">
                <div className="column">
                    <h1>Query</h1>
                    <Editor id="query" value={query} onChange={s => { setQuery(s); setQueryChanged(true); }} placeholder="Enter your query here ..." />
                </div>
                <div className="column">
                    <h1>Input</h1>
                    <Editor id="input" value={input} onChange={s => setInput(s)} placeholder="Enter your input here ..." />
                </div>
                <div className="column">
                    <h1>Output</h1>
                    <Editor id="output" value={output} readOnly placeholder="The formatted code will appear here ..." />
                    <textarea id="rawOutput" value={output} readOnly className="hidden"></textarea>
                </div>
            </div>
        </div>
    );
}

export default App;
