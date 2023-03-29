import { FrameWaitForFunctionOptions, Page } from "puppeteer";

describe('test all grammars with puppeteer', () => {
    beforeEach(async () => {
        await page.goto('http://localhost:3000/playground');
    });

    it('can format', async () => {
        const fs = require("fs");
        const path = require("path");

        const file = path.join(__dirname, "../../topiary/tests/samples/input/", "json.json");
        const fdr = fs.readFileSync(file, "utf8", function (err: any, data: any) {
            return data;
        });

        const s = "foo"

        expect(s).toBe(fdr)

        const queryElement = await page.waitForSelector('#query') ?? fail('Did not find query element');
        queryElement.type("foo");

        await page.type('#query', 'automate beyond recorder');
        await page.type('#input', 'automate beyond recorder');

        const button = await page.$('#formatButton') ?? fail('Did not find button');
        expect(button).not.toBeNull();

        await button.click();
        await waitForOutput(page, "#output");

        // Useful for debugging:
        // await page.screenshot({ path: 'screenshot.png' });

        const output = await readOutput();

        expect(output).toBe("foo");
    }, 30000);
})

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