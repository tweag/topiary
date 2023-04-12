import { useState } from "react";
import "./App.css";
import init, {
    topiaryInit,
    format,
} from "./wasm-app/topiary_playground.js";
import inputSample from './samples/input';
import querySample from './samples/query';

function App() {
    const [isInitialised, setIsInitialised] = useState(false);
    const [query, setQuery] = useState(querySample);
    const [input, setInput] = useState(inputSample);
    const [output, setOutput] = useState("");

    async function runFormat() {
        try {
            if (!isInitialised) {
                await init();
                await topiaryInit();
                setIsInitialised(true);
            }

            setOutput("Formatting ...");
            setOutput(await format(input, query));
        } catch (e) {
            setOutput(String(e));
        }
    }

    return (
        <div className="App">
            <div className="header">
                <button id="formatButton" className="btn btn-primary" onClick={runFormat}>
                    Format
                </button>
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
