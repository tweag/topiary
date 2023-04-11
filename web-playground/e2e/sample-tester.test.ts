import { FrameWaitForFunctionOptions, Page } from "puppeteer";
import * as fs from 'fs';
import * as path from 'path';

const TimeoutMs = 20000;

describe('test all grammars with puppeteer', () => {
    beforeEach(async () => {
        await page.goto('http://localhost:3000/playground');
    });

    it('can format', async () => {
        const rootDir = path.join(__dirname, "../../");
        const inputDir = path.join(rootDir, "topiary/tests/samples/input/");
        const expectedDir = path.join(rootDir, "topiary/tests/samples/expected/");
        const queryDir = path.join(rootDir, "languages/");

        for (let inputFileName of await fs.promises.readdir(inputDir)) {
            const inputPath = path.join(inputDir, inputFileName);
            const expectedPath = path.join(expectedDir, inputFileName);
            const queryFileName = inputFileName.replace(/\..*$/, ".scm");
            const queryPath = path.join(queryDir, queryFileName);

            console.log(`Testing ${inputPath} - ${expectedPath} - ${queryPath}`);

            const encoding = "utf8";
            const input = await fs.promises.readFile(inputPath, encoding);
            const expected = await fs.promises.readFile(expectedPath, encoding);
            const query = await fs.promises.readFile(queryPath, encoding);

            await testInputFile(input, expected, query);
        }
    }, TimeoutMs);
})

async function testInputFile(input: string, expected: string, query: string) {
    await setTextarea("#input", input);
    await setTextarea("#query", query);

    const button = await page.$('#formatButton') ?? fail('Did not find button');
    await button.click();

    await waitForOutput(page, "#output");
    const output = await readOutput();

    // Useful for debugging:
    //await page.screenshot({ path: 'screenshot.png' });

    expect(output).toBe(expected);
}

async function setTextarea(selector: string, text: string) {
    await page.evaluate((selector, text) => {
        (<HTMLInputElement>document.querySelector(selector)).value = text;
    }, selector, text);

    // Without this hack, the textarea simply won't get updated.
    await page.focus(selector);
    await page.keyboard.type(" ");
}

async function readOutput() {
    const outputElement = await page.$("#output");
    return await page.evaluate(element => element?.textContent, outputElement);
}

const waitForOutput = async (
    page: Page,
    selector: string,
    options: FrameWaitForFunctionOptions = { polling: "mutation", timeout: 30000 }
) => {
    const el = typeof selector === "string" ?
        (await page.waitForSelector(selector)) : selector;

    return page.waitForFunction(
        el => el?.textContent !== "" && el?.textContent !== "Formatting ...",
        options,
        el
    );
};
