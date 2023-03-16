import { useState } from "react";
import "./App.css";

function App() {
    const [output, setOutput] = useState("");

    function format() {
        setOutput("Formatting ...");
    }

    return (
        <div className="App">
            <div className="header">
                <button className="btn btn-primary" onClick={format}>
                    Format
                </button>
            </div>
            <div className="columns">
                <div className="column">
                    <h1>Query</h1>
                    <textarea></textarea>
                </div>
                <div className="column">
                    <h1>Input</h1>
                    <textarea></textarea>
                </div>
                <div className="column">
                    <h1>Output</h1>
                    <textarea value={output} readOnly></textarea>
                </div>
            </div>
        </div>
    );
}

export default App;
