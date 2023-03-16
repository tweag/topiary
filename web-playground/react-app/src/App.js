import "./App.css";

function App() {
    function format() {
        output = "Formatting ...";
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
                    <textarea value={output}></textarea>
                </div>
            </div>
        </div>
    );
}

export default App;
