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
    const defaultLanguage = "json";
    const defaultQuery = languages[defaultLanguage].query;
    const defaultInput = languages[defaultLanguage].input;

    // These don't have to be useState, as they don't need to trigger UI changes.
    const initCalled = useRef(false);
    const isQueryCompiling = useRef(false);
    const queryChanged = useRef(true);
    const previousDebouncedInput = useRef("");
    const previousDebouncedQuery = useRef("");
    const previousIsInitialised = useRef(false);

    const [isInitialised, setIsInitialised] = useState(false);
    const [languageOptions, setLanguageOptions] = useState([] as ReactElement[]);
    const [currentLanguage, setCurrentLanguage] = useState(defaultLanguage);
    const [onTheFlyFormatting, setOnTheFlyFormatting] = useState(true);
    const [idempotence, setIdempotence] = useState(false);
    const [tolerateParsingErrors, setTolerateParsingErrors] = useState(false);
    const [input, setInput] = useState(defaultInput);
    const [output, setOutput] = useState("");
    const [query, setQuery] = useState(defaultQuery);
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
            console.log("Initialised");
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

    const runFormat = useCallback(() => {
        console.log(`runFormat`);

        if (!isInitialised) {
            console.log("Cannot format yet, as the formatter engine is being initialised.");
            setOutput("Cannot format yet, as the formatter engine is being initialised. Try again soon.");
            return;
        }

        if (isQueryCompiling.current) {
            console.log("Query is being compiled.");
            setOutput("Query is being compiled. Try again soon.");
            return;
        }

        const outputFormat = async () => {
            try {
                if (queryChanged.current) {
                    isQueryCompiling.current = true;
                    console.log(`Compiling query`);
                    setOutput("Compiling query ...");
                    console.log(`Initialising ${currentLanguage} with ${query} because ${queryChanged.current}`);
                    let fut = queryInit(query, currentLanguage);
                    console.log(`future: ${fut}`);
                    let res = await fut;
                    console.log(`queryInit result: ${res}`);
                    queryChanged.current = false;
                    isQueryCompiling.current = false;
                }

                try {
                    console.log(`Formatting`);
                    setOutput("Formatting ...");
                    setOutput(await format(input, idempotence, tolerateParsingErrors));
                    setProcessingTime(performance.now() - start);
                } catch (e) {
                    setOutput(String(e));
                }
            } catch (e) {
                console.error(`error when compiling query: ${e}`);
                queryChanged.current = false;
                isQueryCompiling.current = false;
                setOutput(String(e));
            }
        }

        const start = performance.now();
        outputFormat();
    }, [currentLanguage, idempotence, isInitialised, tolerateParsingErrors, input, query]);

    // Run on every (debounced) input change, as well as when isInitialised is set, and when the dirty flag changes.
    useEffect(() => {
        if (!onTheFlyFormatting) return;

        // This is how to run async within useEffect.
        // https://devtrium.com/posts/async-functions-useeffect
        const run = async () => {
            await runFormat();
        }

        // We don't want to run whenever a dependency changes, but only when either of these do:
        if (previousDebouncedInput.current !== debouncedInput || previousDebouncedQuery.current !== debouncedQuery || previousIsInitialised.current !== isInitialised) {
            console.log(`On the fly formatting kicking in.`);

            if (!isInitialised) {
                console.log("Cannot format yet, as the formatter engine is being initialised.");
                setOutput("Cannot format yet, as the formatter engine is being initialised. Try again soon.");
                return;
            }

            if (isQueryCompiling.current) {
                console.log("Query is being compiled.");
                setOutput("Query is being compiled. Try again soon.");
                return;
            }

            run()
                .catch(console.error);
        }

        previousDebouncedInput.current = debouncedInput;
        previousDebouncedQuery.current = debouncedQuery;
        previousIsInitialised.current = isInitialised;
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
                queryChanged.current = true;
                setOutput("");
                setCurrentLanguage(l);
            }
        }
    }

    function handleFormat() {
        runFormat();
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
                    <Editor id="query" value={query} onChange={s => { setQuery(s); queryChanged.current = true; }} placeholder="Enter your query here ..." />
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
