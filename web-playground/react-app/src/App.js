import "./App.css";

function App() {
    return (
        <div className="App">
            <div className="header">
                <button>Format</button>
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
                    <textarea></textarea>
                </div>
            </div>
        </div>
    );
}

export default App;
