module.exports = {
    preset: "jest-puppeteer",
    transform: {
        "^.+\\.ts?$": "ts-jest"
    },
    globals: {
        URL: "http://localhost:3000/playground"
    },
};
