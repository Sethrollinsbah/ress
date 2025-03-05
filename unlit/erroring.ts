import StealthPlugin from "puppeteer-extra-plugin-stealth";
import puppeteer from "puppeteer-extra";
import { appendFileSync } from "fs";

class WebsiteMapper {
  constructor(targetUrl, maxRoutesToCollect = 100) {
    this.targetUrl = targetUrl;
    this.visitedPaths = new Set();
    this.discoveredRoutes = new Set();
    this.maxRoutesToCollect = maxRoutesToCollect;
    this.retryAttempts = 3;
    this.requestPauseMs = 2000;
    this.pendingUrlsQueue = new Set(); // Pages waiting to be crawled
    this.domainName = this.extractDomainName(targetUrl);
    this.clientErrors = new Map(); // Store errors by URL
  }

  extractDomainName(url) {
    try {
      return new URL(url).hostname;
    } catch {
      return "";
    }
  }

  async initializeBrowser() {
    const browser = await puppeteer.launch({
      headless: "new",
      args: [
        "--no-sandbox",
        "--disable-setuid-sandbox",
        "--disable-dev-shm-usage",
      ],
    });
    puppeteer.use(StealthPlugin());
    return browser;
  }

  async configurePage(browser) {
    const page = await browser.newPage();
    await page.setViewport({ width: 1366, height: 768 });
    await page.setUserAgent(
      "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    );

    await page.setExtraHTTPHeaders({
      Accept:
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
      "Accept-Language": "en-US,en;q=0.5",
    });

    // Optimize performance by blocking non-essential resources
    await page.setRequestInterception(true);
    page.on("request", (request) => {
      const resourceType = request.resourceType();
      if (["image", "stylesheet", "font", "media"].includes(resourceType)) {
        request.abort();
      } else {
        request.continue();
      }
    });

    // Set up error listener to capture client-side JavaScript errors
    await this.setupErrorListener(page);

    return page;
  }

  async setupErrorListener(page) {
    // Listen for console errors
    page.on("console", async (msg) => {
      if (msg.type() === "error") {
        const currentUrl = page.url();
        const errorMessage = msg.text();

        if (!this.clientErrors.has(currentUrl)) {
          this.clientErrors.set(currentUrl, []);
        }

        this.clientErrors.get(currentUrl).push(errorMessage);
        await logMessage(
          this.domainName,
          `CLIENT ERROR on ${currentUrl}: ${errorMessage}`,
        );
      }
    });

    // Listen for page errors
    page.on("pageerror", async (error) => {
      const currentUrl = page.url();
      const errorMessage = error.message;

      if (!this.clientErrors.has(currentUrl)) {
        this.clientErrors.set(currentUrl, []);
      }

      this.clientErrors.get(currentUrl).push(errorMessage);
      await logMessage(
        this.domainName,
        `PAGE ERROR on ${currentUrl}: ${errorMessage}`,
      );
    });

    // Listen for request failures
    page.on("requestfailed", async (request) => {
      const currentUrl = page.url();
      const failureText = request.failure()
        ? request.failure().errorText
        : "unknown error";
      const resourceType = request.resourceType();
      const resourceUrl = request.url();

      // Only log failures for important resources
      if (["script", "xhr", "fetch", "document"].includes(resourceType)) {
        if (!this.clientErrors.has(currentUrl)) {
          this.clientErrors.set(currentUrl, []);
        }

        const errorMessage = `Failed to load ${resourceType}: ${resourceUrl} - ${failureText}`;
        this.clientErrors.get(currentUrl).push(errorMessage);
        await logMessage(
          this.domainName,
          `RESOURCE ERROR on ${currentUrl}: ${errorMessage}`,
        );
      }
    });
  }

  waitFor(milliseconds) {
    return new Promise((resolve) => setTimeout(resolve, milliseconds));
  }

  extractPathFromUrl(url) {
    try {
      const urlObj = new URL(url, this.targetUrl);
      return urlObj.pathname;
    } catch {
      return null;
    }
  }

  async findLinksOnPage(page) {
    try {
      await page
        .waitForSelector("a, button, [onclick]", { timeout: 5000 })
        .catch(() =>
          logMessage(
            this.domainName,
            "WARN: No typical elements found, continuing...",
          ),
        );

      const links = await page.evaluate(() => {
        const results = new Set();

        // Extract href attributes from anchor tags
        document.querySelectorAll("a").forEach((anchor) => {
          if (anchor.href && !anchor.href.startsWith("javascript:")) {
            results.add(anchor.href);
          }
        });

        // Extract links from onclick handlers
        document.querySelectorAll("[onclick]").forEach((element) => {
          const onclick = element.getAttribute("onclick");
          const matches = onclick?.match(
            /(?:window\.location|location\.href)\s*=\s*['"]([^'"]+)['"]/,
          );
          if (matches) {
            results.add(new URL(matches[1], window.location.origin).href);
          }
        });

        // Extract form submission targets
        document.querySelectorAll("form").forEach((form) => {
          if (
            form.action &&
            typeof form.action === "string" &&
            !form.action.startsWith("javascript:")
          ) {
            results.add(form.action);
          }
        });

        return Array.from(results);
      });

      return links
        .map((link) => this.extractPathFromUrl(link))
        .filter((link) => link && link.startsWith("/"));
    } catch (err) {
      await logMessage(
        this.domainName,
        `ERROR: Failed to extract links: ${err.message}`,
      );
      return [];
    }
  }

  async detectClientSideErrors(page, url) {
    try {
      // Trigger JavaScript execution by scrolling the page
      await page.evaluate(() => {
        window.scrollTo(0, document.body.scrollHeight / 2);
        window.scrollTo(0, document.body.scrollHeight);
        window.scrollTo(0, 0);

        // Force any lazy-loaded resources to load
        const event = new Event("scroll");
        window.dispatchEvent(event);

        // Check for any React or Vue error boundaries
        return {
          hasReactError: !!document.querySelector(
            "[data-reactroot] .error-boundary, #root .error-boundary",
          ),
          hasVueError: !!document.querySelector(
            ".vue-error-boundary, [data-v-error]",
          ),
        };
      });

      // Check for any framework-specific error indicators
      const errorIndicators = await page.evaluate(() => {
        const errorTexts = [];

        // Common error message elements
        document
          .querySelectorAll(
            '.error-message, .error-text, .alert-danger, [role="alert"]',
          )
          .forEach((el) => {
            if (el.innerText && el.innerText.trim().length > 0) {
              errorTexts.push(el.innerText.trim());
            }
          });

        return errorTexts;
      });

      if (errorIndicators.length > 0) {
        if (!this.clientErrors.has(url)) {
          this.clientErrors.set(url, []);
        }

        errorIndicators.forEach((errorText) => {
          this.clientErrors.get(url).push(`UI Error Indicator: ${errorText}`);
          logMessage(this.domainName, `UI ERROR on ${url}: ${errorText}`);
        });
      }
    } catch (err) {
      await logMessage(
        this.domainName,
        `ERROR: Failed to check for client errors on ${url}: ${err.message}`,
      );
    }
  }

  async navigateAndScanPage(page, url) {
    try {
      await logMessage(this.domainName, `Navigating to: ${url}`);

      // Track navigation errors
      let navigationError = null;
      await page.goto(url, { waitUntil: "networkidle2", timeout: 30000 });

      // Log navigation errors
      if (navigationError) {
        if (!this.clientErrors.has(url)) {
          this.clientErrors.set(url, []);
        }
        this.clientErrors
          .get(url)
          .push(`Navigation Error: ${navigationError.message}`);
        await logMessage(
          this.domainName,
          `NAVIGATION ERROR for ${url}: ${navigationError.message}`,
        );
      }

      // Add random delay to mimic human browsing and avoid rate limiting
      await this.waitFor(1000 + Math.random() * 2000);

      // Actively look for client-side errors
      await this.detectClientSideErrors(page, url);

      return await this.findLinksOnPage(page);
    } catch (err) {
      await logMessage(
        this.domainName,
        `ERROR: Failed to navigate to ${url}: ${err.message}`,
      );
      return [];
    }
  }

  async mapWebsite() {
    const browser = await this.initializeBrowser();

    try {
      const page = await this.configurePage(browser);

      // Start with the homepage
      const initialLinks = await this.navigateAndScanPage(page, this.targetUrl);
      initialLinks.forEach((link) => {
        if (!this.visitedPaths.has(link)) {
          this.discoveredRoutes.add(link);
          this.pendingUrlsQueue.add(link);
        }
      });

      // Continue exploring discovered paths
      while (
        this.pendingUrlsQueue.size > 0 &&
        this.discoveredRoutes.size < this.maxRoutesToCollect
      ) {
        const currentPath = Array.from(this.pendingUrlsQueue)[0];
        this.pendingUrlsQueue.delete(currentPath);

        if (!this.visitedPaths.has(currentPath)) {
          this.visitedPaths.add(currentPath);

          const fullUrl = new URL(currentPath, this.targetUrl).href;
          const newLinks = await this.navigateAndScanPage(page, fullUrl);

          await logMessage(
            this.domainName,
            `Found ${newLinks.length} links on ${currentPath}`,
          );

          await logMessage(
            this.domainName,
            `$FOUND_URL::${JSON.stringify([...newLinks])}`,
          );

          for (const link of newLinks) {
            if (
              !this.visitedPaths.has(link) &&
              this.discoveredRoutes.size < this.maxRoutesToCollect
            ) {
              this.discoveredRoutes.add(link);
              this.pendingUrlsQueue.add(link);

              await logMessage(
                this.domainName,
                `Added to exploration queue: ${link}`,
              );
            }
          }

          await logMessage(
            this.domainName,
            `Total routes discovered: ${this.discoveredRoutes.size}/${this.maxRoutesToCollect}`,
          );
        }
      }

      // Generate error report
      await this.generateErrorReport();

      const sortedRoutes = Array.from(this.discoveredRoutes).sort();
      return sortedRoutes;
    } finally {
      await browser.close();
    }
  }

  async generateErrorReport() {
    if (this.clientErrors.size === 0) {
      await logMessage(
        this.domainName,
        "No client-side errors detected during crawl.",
      );
      return;
    }

    await logMessage(
      this.domainName,
      "==========================================",
    );
    await logMessage(this.domainName, "CLIENT-SIDE ERROR SUMMARY");
    await logMessage(
      this.domainName,
      "==========================================",
    );

    let totalErrors = 0;

    for (const [url, errors] of this.clientErrors.entries()) {
      totalErrors += errors.length;
      await logMessage(this.domainName, `\nURL: ${url}`);
      await logMessage(this.domainName, `Errors detected: ${errors.length}`);

      errors.forEach((error, index) => {
        logMessage(this.domainName, `  ${index + 1}. ${error}`);
      });
    }

    await logMessage(
      this.domainName,
      "\n==========================================",
    );
    await logMessage(
      this.domainName,
      `TOTAL ERRORS: ${totalErrors} across ${this.clientErrors.size} pages`,
    );
    await logMessage(
      this.domainName,
      "==========================================",
    );

    // Save a dedicated error report file
    try {
      const errorReport = Array.from(this.clientErrors.entries())
        .map(([url, errors]) => {
          return `URL: ${url}\nErrors: ${errors.length}\n${errors.map((e, i) => `  ${i + 1}. ${e}`).join("\n")}\n`;
        })
        .join("\n---\n\n");

      Bun.write(
        `./tmp/reports/${this.domainName}-errors.txt`,
        `CLIENT-SIDE ERROR REPORT\nTotal Errors: ${totalErrors}\nPages with Errors: ${this.clientErrors.size}\n\n${errorReport}`,
      );

      await logMessage(
        this.domainName,
        `Error report saved to ./tmp/reports/${this.domainName}-errors.txt`,
      );
    } catch (error) {
      await logMessage(
        this.domainName,
        `ERROR: Failed to save error report: ${error.message}`,
      );
    }
  }
}

/**
 * Discovers and maps all accessible routes on a website
 * @param {string} websiteUrl - The full URL of the website to map
 * @param {number} maxRoutes - Maximum number of routes to collect
 * @returns {Promise<string[]>} Array of discovered routes
 */
async function mapWebsiteRoutes(websiteUrl, maxRoutes = 100) {
  const domainName = new URL(websiteUrl).hostname;
  await logMessage(domainName, `Starting website mapping for ${websiteUrl}`);
  const mapper = new WebsiteMapper(websiteUrl, maxRoutes);

  try {
    const routes = await mapper.mapWebsite();

    await logMessage(domainName, `Discovered Routes: `);
    routes.forEach(async (route) => await logMessage(domainName, `${route}`));

    // Save routes to a file
    Bun.write(
      "http/" + websiteUrl.split("https://")[1] + ".txt",
      routes.slice(0, 100).join("\n"),
    );

    await logMessage(
      domainName,
      `Total unique routes discovered: ${routes.length}`,
    );
    return routes;
  } catch (err) {
    await logMessage(domainName, `ERROR: Website mapping failed: ${err}`);
    throw err;
  }
}

/**
 * Logs a message to a domain-specific file in the reports directory
 * @param {string} domain - The domain name used in the filename
 * @param {string} message - The message to log
 * @returns {Promise<void>} A promise that resolves when the log is written
 */
async function logMessage(domain, message) {
  const timestamp = new Date().toISOString();
  const filename = `./tmp/reports/${domain}.txt`;

  // Format the log entry with the timestamp
  const logEntry = `${timestamp}::${message}\n-----`;

  // Append the log entry to the file
  try {
    appendFileSync(filename, logEntry, "utf8");
  } catch (error) {
    console.error(`Error writing to log file: ${error}`);
  }
}

// Parse command line arguments
const args = process.argv.slice(2);

// Default values
let targetWebsite = "https://miamiwalkincoolers.com";
let maxRoutes = 100;

// Parse command line arguments
args.forEach((arg) => {
  const [key, value] = arg.split("=");
  if (key === "siteUrl") {
    targetWebsite = value || targetWebsite;
  } else if (key === "maxLinks") {
    maxRoutes = parseInt(value) || maxRoutes;
  }
});

// Execute the website mapping
mapWebsiteRoutes(targetWebsite, maxRoutes)
  .then(async () => {
    const domain = new URL(targetWebsite).hostname;
    await logMessage(domain, "Website mapping completed successfully!");
  })
  .catch(async (err) => {
    const domain = new URL(targetWebsite).hostname;
    await logMessage(domain, "ERROR: Failed to map website routes.");
  });
