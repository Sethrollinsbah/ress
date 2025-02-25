import puppeteer from "puppeteer";
import sqlite3 from "sqlite3";
import { readFileSync, writeFileSync } from "fs";

class DeepTextCrawler {
  db: any;
  keywordData: {};
  maxPages: number;
  pagesToExplore: Set<unknown>;
  visited: Set<unknown>;
  domains: any;
  constructor(domains, maxPages = 100) {
    this.domains = domains;
    this.visited = new Set();
    this.pagesToExplore = new Set(domains);
    this.maxPages = maxPages;
    this.keywordData = {};
    this.db = new sqlite3.Database("competitor_keywords.db", (err) => {
      if (err) {
        console.error("Error opening database", err);
      } else {
        this.setupDatabase();
      }
    });
  }

  setupDatabase() {
    const db = new sqlite3.Database("bunscore-cache.db"); // Open your SQLite database
    const sql = readFileSync("schema.sql", "utf8"); // Read the SQL file

    db.serialize(() => {
      // Run the SQL commands from the file
      db.exec(sql, (err) => {
        if (err) {
          console.error("Error executing SQL:", err.message);
        } else {
          console.log("Database schema setup successfully!");
        }
      });
    });

    db.close(); // Close the database connection
  }

  async setupPage(browser) {
    const page = await browser.newPage();
    await page.setViewport({ width: 1366, height: 768 });
    await page.setUserAgent(
      "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    );
    return page;
  }

  async extractTextAndKeywords(page) {
    return await page.evaluate(() => {
      const textContent = document.body.innerText;
      const words = textContent.match(/\b\w{4,}\b/g) || [];
      const wordFreq = {};

      words.forEach((word) => {
        word = word.toLowerCase();
        wordFreq[word] = (wordFreq[word] || 0) + 1;
      });

      const sortedKeywords = Object.entries(wordFreq)
        .sort((a, b) => b[1] - a[1])
        .slice(0, 20)
        .map(([word]) => word);

      const links = Array.from(document.querySelectorAll("a"))
        .map((a) => a.href)
        .filter((href) => href.startsWith(window.location.origin));

      return {
        text: textContent.slice(0, 5000),
        keywords: sortedKeywords,
        links,
      };
    });
  }

  async visitPage(page, url) {
    try {
      await page.goto(url, { waitUntil: "domcontentloaded", timeout: 15000 });
      await new Promise((res) => setTimeout(res, 1000));
      return await this.extractTextAndKeywords(page);
    } catch (err) {
      console.error(`Error visiting ${url}: ${err.message}`);
      return null;
    }
  }

  async crawl() {
    const browser = await puppeteer.launch({ headless: "new" });
    const page = await this.setupPage(browser);
    let results = [];

    while (this.pagesToExplore.size > 0 && this.visited.size < this.maxPages) {
      const currentUrl = this.pagesToExplore.values().next().value;
      this.pagesToExplore.delete(currentUrl);
      this.visited.add(currentUrl);

      const result = await this.visitPage(page, currentUrl);
      if (result) {
        const domain = new URL(currentUrl).hostname;
        if (!this.keywordData[domain]) {
          this.keywordData[domain] = new Set();
        }
        result.keywords.forEach((keyword) =>
          this.keywordData[domain].add(keyword),
        );

        results.push({ url: currentUrl, ...result });
        result.links.forEach((link) => {
          if (
            !this.visited.has(link) &&
            this.pagesToExplore.size + this.visited.size < this.maxPages
          ) {
            this.pagesToExplore.add(link);
          }
        });
      }
    }

    await browser.close();
    this.saveResults(results);
    return results;
  }

  saveResults(results) {
    const data = Object.fromEntries(
      Object.entries(this.keywordData).map(([domain, keywords]) => [
        domain,
        Array.from(keywords),
      ]),
    );
    writeFileSync("competitor_keywords.json", JSON.stringify(data, null, 2));
    console.log("Keyword data saved to competitor_keywords.json");
  }
}

async function discoverCompetitorKeywords(domains, maxPages = 100) {
  const crawler = new DeepTextCrawler(domains, maxPages);
  await crawler.crawl();
}

discoverCompetitorKeywords(
  ["https://fb.com", "https://github.com", "https://bun.sh"],
  50,
);
