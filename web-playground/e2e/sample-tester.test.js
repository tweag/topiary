describe('test all grammars with puppeteer', () => {
    beforeEach(async () => {
        await page.goto('http://localhost:3000/playground');
    });

    it('can format', async () => {
        const queryElement = await page.waitForSelector('#query');
        queryElement.type("foo");

        await page.type('#query', 'automate beyond recorder');
        await page.type('#input', 'automate beyond recorder');

        const button = await page.$('#formatButton');
        await button.click();
        await waitForOutput(page, "#output");

        // Useful for debugging:
        // await page.screenshot({ path: 'screenshot.png' });

        const output = await readOutput();

        expect(output).toBe("foo");
    });
})

async function readOutput() {
    const outputElement = await page.$("#output");
    return await page.evaluate(element => element.textContent, outputElement);
}

const waitForOutput = async (
    page,
    sel,
    opts = { polling: "mutation", timeout: 30000 }
) => {
    const el = typeof sel === "string" ?
        (await page.waitForSelector(sel)) : sel;

    return page.waitForFunction(
        (el, originalText) => el.textContent !== "" && el.textContent !== "Formatting ...",
        opts,
        el
    );
};