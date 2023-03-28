const { join } = require('path');

/**
 * @type {import("puppeteer").Configuration}
 */
module.exports = {
    // Changes the cache location for Puppeteer so it works in CI.
    cacheDirectory: join(__dirname, '.cache', 'puppeteer'),
};
