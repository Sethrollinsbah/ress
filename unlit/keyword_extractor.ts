import puppeteer from "puppeteer";
import { appendFileSync } from "fs";
import { URL } from "url";

let DOMAIN = "";
let result = [];
class DeepTextCrawler {
  constructor(baseUrl, maxPages = 100, maxDomains = 100) {
    this.baseUrl = baseUrl;
    this.visited = new Set();
    this.pagesToExplore = new Set([baseUrl]);
    this.maxPages = maxPages;
    this.maxDomains = maxDomains;
    this.visitedDomains = new Set();
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

    while (
      this.pagesToExplore.size > 0 &&
      this.visited.size < this.maxPages &&
      this.visitedDomains.size < this.maxDomains
    ) {
      const currentUrl = this.pagesToExplore.values().next().value;
      this.pagesToExplore.delete(currentUrl);
      this.visited.add(currentUrl);

      const domain = new URL(currentUrl).hostname;
      if (!this.visitedDomains.has(domain)) {
        this.visitedDomains.add(domain);
      }

      const result = await this.visitPage(page, currentUrl);
      if (result) {
        results.push({ url: currentUrl, ...result });
        result.links.forEach((link) => {
          const linkDomain = new URL(link).hostname;
          if (
            !this.visited.has(link) &&
            this.pagesToExplore.size + this.visited.size < this.maxPages &&
            (this.visitedDomains.has(linkDomain) ||
              this.visitedDomains.size < this.maxDomains)
          ) {
            this.pagesToExplore.add(link);
          }
        });
      }
    }

    await browser.close();
    return results;
  }
}

async function discoverTextContent(siteUrl, maxPages = 100, maxDomains = 100) {
  const crawler = new DeepTextCrawler(siteUrl, maxPages, maxDomains);
  const results = await crawler.crawl();

  results.forEach(({ url, text, keywords }) => {
    result = [...result, { url, keywords: [...keywords] }];
  });

  console.log(result);
}

discoverTextContent("https://planetbun.com", 100, 100);
