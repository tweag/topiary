import * as fs from 'fs';
import * as path from 'path';

// Extending this from 20 to 60 seconds. If we are still seeing timeouts in CI,
// it is likely due to an actual race condition or some other flakiness.
const TimeoutMs = 60000;

// automatically confirm dialogs
page.on("dialog", (dialog) => {
    dialog.accept();
});

describe('test all grammars with puppeteer', () => {
    beforeEach(async () => {
        // Forward the console log from the browser
        page.on('console', msg => console.log('PAGE LOG:', msg.text()));

        await page.goto('http://localhost:5173/playground');

        // Test without on-the-fly formatting, because the debounce makes things
        // less explicit and predictable.
        const onTheFlyCheckbox = await page.waitForSelector("#onTheFlyFormatting") ?? fail('Did not find checkbox');
        let isOnTheFlyEnabled = await (await onTheFlyCheckbox.getProperty("checked")).jsonValue();

        if (isOnTheFlyEnabled)
            await onTheFlyCheckbox.click();
    });

    it('can format', async () => {
        const rootDir = path.join(__dirname, "../../");
        const inputDir = path.join(rootDir, "topiary/tests/samples/input/");
        const expectedDir = path.join(rootDir, "topiary/tests/samples/expected/");
        const queryDir = path.join(rootDir, "queries/");

        for (let inputFileName of await fs.promises.readdir(inputDir)) {
            let parts = inputFileName.split(".");
            if (parts.length < 2) {
                continue;
            }
            const language = String(parts[0]);
            const inputPath = path.join(inputDir, inputFileName);
            const expectedPath = path.join(expectedDir, inputFileName);
            const queryFileName = inputFileName === "ocaml-interface.mli" ? "ocaml.scm" : inputFileName.replace(/\..*$/, ".scm");
            const queryPath = path.join(queryDir, queryFileName);

            console.log(`Testing ${inputPath} - ${expectedPath} - ${queryPath}`);

            const encoding = "utf8";
            const input = await fs.promises.readFile(inputPath, encoding);
            const expected = await fs.promises.readFile(expectedPath, encoding);
            const query = await fs.promises.readFile(queryPath, encoding);

            await testInputFile(input, expected, query, language);
        }
    }, TimeoutMs);

    it('outputs error messages', async () => {
        await setTextarea("#input", "foo");

        const button = await page.$('#formatButton') ?? fail('Did not find button');
        await button.click();

        const output = await readOutput();

        // Useful for debugging:
        //await page.screenshot({ path: 'screenshot-error.png' });

        expect(output).toContain("Parsing error");
    }, TimeoutMs);
})

async function testInputFile(input: string, expected: string, query: string, language: string) {
    // Set language before input/query, otherwise they will get overwritten.
    await page.select('#languageMenu', language);

    await setTextarea("#input", input);
    await setTextarea("#query", query);

    const button = await page.$('#formatButton') ?? fail('Did not find button');
    await button.click();

    const output = await readOutput();

    // Useful for debugging:
    //await page.screenshot({ path: `screenshot-${language}.png` });

    expect(output).toBe(expected);
}

async function setTextarea(selector: string, text: string) {
    let textInput = await page.$(selector) ?? fail('Did not find text input control');
    let textAreaSelector = `${selector} textarea`;

    // Clear the text area first, otherwise the following doesn't work.
    await textInput.click();
    await page.keyboard.down('ControlLeft')
    await page.keyboard.press('KeyA')
    await page.keyboard.up('ControlLeft')
    await textInput.press('Backspace');

    // Quick way to enter text into a field. See https://github.com/puppeteer/puppeteer/issues/4192
    await page.evaluate((selector, text) => {
        (<HTMLInputElement>document.querySelector(selector)).value = text;
    }, textAreaSelector, text);

    // Without this hack, the textarea simply won't get updated.
    await page.keyboard.type("X");
    await textInput.press('Backspace');
}

async function readOutput() {
    const outputSelector = "#rawOutput";
    const el = await page.waitForSelector(outputSelector);

    // Wait for useful output.
    await page.waitForFunction(
        el => el?.textContent !== "" && el?.textContent !== "Formatting ..." && el?.textContent !== "Compiling query ...",
        { polling: "mutation", timeout: 30000 },
        el
    );

    const outputElement = await page.$(outputSelector);
    return await page.evaluate(element => element?.textContent, outputElement);
}
