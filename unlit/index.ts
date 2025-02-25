import puppeteer from "puppeteer";
import { appendFileSync } from "fs";

let DOMAIN = "";
class DeepRouteCrawler {
  constructor(baseUrl, maxLinks = 100) {
    this.baseUrl = baseUrl;
    this.visited = new Set();
    this.routes = new Set();
    this.maxLinks = maxLinks;
    this.retryCount = 3;
    this.retryDelay = 2000;
    this.pagesToExplore = new Set(); // Queue of pages to explore deeply
  }

  async setupPage(browser) {
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

    await page.setRequestInterception(true);
    page.on("request", (request) => {
      const resourceType = request.resourceType();
      if (["image", "stylesheet", "font", "media"].includes(resourceType)) {
        request.abort();
      } else {
        request.continue();
      }
    });

    return page;
  }

  delay(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  normalizeUrl(url) {
    try {
      const urlObj = new URL(url, this.baseUrl);
      return urlObj.pathname;
    } catch {
      return null;
    }
  }

  async extractLinks(page) {
    try {
      await page
        .waitForSelector("a, button, [onclick]", { timeout: 5000 })
        .catch(() =>
          bunLog(DOMAIN, "WARN:No typical elements found, continuing..."),
        );

      const links = await page.evaluate(() => {
        const results = new Set();

        // Get all links
        document.querySelectorAll("a").forEach((anchor) => {
          if (anchor.href && !anchor.href.startsWith("javascript:")) {
            results.add(anchor.href);
          }
        });

        // Get onclick handlers
        document.querySelectorAll("[onclick]").forEach((element) => {
          const onclick = element.getAttribute("onclick");
          const matches = onclick?.match(
            /(?:window\.location|location\.href)\s*=\s*['"]([^'"]+)['"]/,
          );
          if (matches) {
            results.add(new URL(matches[1], window.location.origin).href);
          }
        });

        // Get form actions
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
        .map((link) => this.normalizeUrl(link))
        .filter((link) => link && link.startsWith("/"));
    } catch (err) {
      await bunLog(DOMAIN, `ERROR: Error extracting links: ${err.message}`);
      return [];
    }
  }

  async visitPage(page, url) {
    try {
      await bunLog(DOMAIN, `Visiting: ${url}`);
      await page.goto(url, {
        waitUntil: "domcontentloaded",
        timeout: 15000,
      });

      // Random delay between 1-3 seconds
      await this.delay(1000 + Math.random() * 2000);

      return await this.extractLinks(page);
    } catch (err) {
      await bunLog(DOMAIN, `ERROR: Error visiting ${url}: ${err.message}`);

      return [];
    }
  }

  async crawl() {
    const browser = await puppeteer.launch({
      headless: "new",
      args: [
        "--no-sandbox",
        "--disable-setuid-sandbox",
        "--disable-dev-shm-usage",
      ],
    });

    try {
      const page = await this.setupPage(browser);

      // Initial crawl of the homepage
      const initialLinks = await this.visitPage(page, this.baseUrl);
      initialLinks.forEach((link) => {
        if (!this.visited.has(link)) {
          this.routes.add(link);
          this.pagesToExplore.add(link);
        }
      });

      // Deep crawl of discovered routes
      while (this.pagesToExplore.size > 0 && this.routes.size < this.maxLinks) {
        const currentUrl = Array.from(this.pagesToExplore)[0];
        this.pagesToExplore.delete(currentUrl);

        if (!this.visited.has(currentUrl)) {
          this.visited.add(currentUrl);

          const fullUrl = new URL(currentUrl, this.baseUrl).href;
          const newLinks = await this.visitPage(page, fullUrl);

          await bunLog(
            DOMAIN,
            `Found ${newLinks.length} links on ${currentUrl}`,
          );


          await bunLog(
            DOMAIN,
            `$FOUND_URL::${newLinks}`,
          );

          for (const link of newLinks) {
            if (!this.visited.has(link) && this.routes.size < this.maxLinks) {
              this.routes.add(link);
              this.pagesToExplore.add(link);

              await bunLog(DOMAIN, `Added to explore queue: ${link}`);
            }
          }

          await bunLog(
            DOMAIN,
            `Total routes found: ${this.routes.size}/${this.maxLinks}`,
          );
        }
      }

      const sortedRoutes = Array.from(this.routes).sort();
      return sortedRoutes;
    } finally {
      await browser.close();
    }
  }
}

// Usage
async function discoverRoutes(siteUrl, maxLinks = 100) {
  await bunLog(DOMAIN, `Starting deep route discovery for ${siteUrl}`);
  const crawler = new DeepRouteCrawler(siteUrl, maxLinks);

  try {
    const routes = await crawler.crawl();

    await bunLog(DOMAIN, `Discovered Routes: `);
    routes.forEach(async (route) => await bunLog(DOMAIN, `${route}`));
    Bun.write(
      "http/" + siteUrl.split("https://")[1] + ".txt",
      routes.slice(0, 100).join("\n"),
    );
    await bunLog(DOMAIN, `Total unique routes found: ${routes.length}`);
    return routes;
  } catch (err) {
    await bunLog(DOMAIN, `ERROR: Crawl failed: ${err}`);
    throw err;
  }
}

// Start the crawler
const args = process.argv.slice(2); // Exclude the first two arguments (node and script path)

// Default values
let SITE = "https://miamiwalkincoolers.com";
let maxLinks = 100;

// Look for specific arguments
args.forEach((arg) => {
  const [key, value] = arg.split("=");
  if (key === "siteUrl") {
    SITE = value || SITE;
    DOMAIN = SITE.split("//")[1];
  } else if (key === "maxLinks") {
    maxLinks = parseInt(value) || maxLinks;
  }
});

discoverRoutes(SITE, maxLinks)
  .then(async () => {
    await bunLog(DOMAIN, "Deep route discovery completed!");
  })
  .catch(async (err) => {
    await bunLog(DOMAIN, "ERROR: Error when crawling webpage for links.");
  });

/**
 * Logs a message to a domain-specific file in /tmp directory
 * @param domain The domain name used in the filename
 * @param text The text message to log
 * @returns A promise that resolves when the log is written
 */
export async function bunLog(domain: string, text: string): Promise<void> {
  // Ensure /tmp directory exists (just in case)
  const timestamp = new Date().toISOString();
  const filename = `/tmp/${domain}.txt`;

  // Format the log entry with the timestamp
  const logEntry = `${timestamp}::${text}\n-----`;

  // Append the log entry to the file (creates it if it doesn't exist)
  try {
    appendFileSync(filename, logEntry, "utf8");
  } catch (error) {
    console.error(`Error writing to log file: ${error}`);
  }
}
