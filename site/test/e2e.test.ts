// The built site in Chromium: variants, availability, code titles, links
// between pages, search, the edit link, the 404 page, and the narrow layout.
import { afterAll, beforeAll, expect, test } from "vitest";
import type { Browser, Page } from "playwright-core";
import { launchChromium, requireBuild, serve } from "./built.ts";

let server: Awaited<ReturnType<typeof serve>>;
let browser: Browser;

beforeAll(async () => {
  requireBuild();
  server = await serve();
  browser = await launchChromium();
});

afterAll(async () => {
  await browser?.close();
  await server?.close();
});

async function open(path: string, width = 1280): Promise<Page> {
  const page = await browser.newPage({ viewport: { width, height: 900 } });
  await page.goto(server.url + path);
  return page;
}

test("a variant group is a switcher, and the reader's choice holds on the next page", async () => {
  const page = await open("/getting-started/");
  const tabs = page.locator("ascribe-tabs").first();
  await expect.poll(() => tabs.getByRole("tab").allTextContents()).toEqual(["npm", "pnpm"]);
  expect(await tabs.getByText("npm install --save-dev @ascribed/cli").isVisible()).toBe(true);
  expect(await tabs.getByText("pnpm add --save-dev @ascribed/cli").isVisible()).toBe(false);

  await tabs.getByRole("tab", { name: "pnpm" }).click();
  expect(await tabs.getByText("pnpm add --save-dev @ascribed/cli").isVisible()).toBe(true);

  await page.goto(`${server.url}/guides/astro/`);
  const next = page.locator("ascribe-tabs").first();
  await expect
    .poll(() => next.getByRole("tab", { name: "pnpm" }).getAttribute("aria-selected"))
    .toBe("true");
  expect(await next.getByText("pnpm add @ascribed/astro").isVisible()).toBe(true);
  await page.close();
});

test("a page marked unreleased shows its availability badge", async () => {
  const page = await open("/guides/review/");
  const badge = page.locator('ascribe-availability[scope="page"] ascribe-availability-target');
  expect(await badge.getAttribute("states")).toContain("unreleased");
  expect(await badge.isVisible()).toBe(true);
  // The element library's stylesheet draws it as a badge.
  const radius = await badge.evaluate((el) => getComputedStyle(el).borderTopLeftRadius);
  expect(radius).not.toBe("0px");
  await page.close();
});

test("notes and steps render as elements", async () => {
  const page = await open("/getting-started/");
  expect(await page.locator('ascribe-note[type="tip"]').isVisible()).toBe(true);
  expect(await page.locator("ascribe-steps ol > li").count()).toBe(3);
  await page.close();
});

test("a code example taken from a file shows its title", async () => {
  const page = await open("/guides/astro/");
  const figure = page.locator("figure.code-title", { hasText: "export default defineConfig" });
  await expect.poll(() => figure.locator("figcaption").textContent()).toBe("astro.config.mjs");
  await page.close();
});

test("a link between pages lands on the page, and a glossary link on its heading", async () => {
  const page = await open("/getting-started/");
  await page.locator('article a[href="/guides/astro/"]').first().click();
  await page.waitForURL(`${server.url}/guides/astro/`);
  expect(await page.locator("h1").textContent()).toBe("Astro");

  await page.goto(`${server.url}/reference/diagnostics/`);
  // A glossary link carries its definition as a title; take one to a heading.
  const term = page.locator('article a[title][href*="#"]').first();
  const href = await term.getAttribute("href");
  expect(href).toMatch(/#/);
  await term.click();
  const id = decodeURIComponent(new URL(page.url()).hash.slice(1));
  expect(await page.locator(`[id="${id}"]`).count()).toBe(1);
  await page.close();
});

test("the sidebar lists the pages and marks the current one", async () => {
  const page = await open("/reference/cli/");
  const current = page.locator('#sidebar a[aria-current="page"]');
  expect(await current.textContent()).toContain("Command reference");
  expect(await page.locator("#sidebar a").count()).toBeGreaterThanOrEqual(13);
  await page.close();
});

test("search finds a known word, with no network", async () => {
  const page = await open("/");
  await page.route(/^(?!http:\/\/127\.0\.0\.1)/, (route) => route.abort());
  await page.keyboard.press("/");
  const input = page.locator("#search input");
  await input.fill("slugger");
  const result = page.locator(".pagefind-ui__result-link").first();
  await result.waitFor();
  expect(await result.textContent()).toBeTruthy();
  await result.click();
  await page.waitForURL(/\/reference\/content-model\//);
  await page.close();
});

test("each page links to its source on GitHub", async () => {
  const page = await open("/guides/astro/");
  const edit = page.getByRole("link", { name: "Edit this page on GitHub" });
  expect(await edit.getAttribute("href")).toBe(
    "https://github.com/ascribed-dev/ascribe/edit/main/docs/content/guides/astro.md",
  );
  await page.close();
});

test("an unknown address gets the 404 page", async () => {
  const page = await browser.newPage();
  const response = await page.goto(`${server.url}/no/such/page/`);
  expect(response?.status()).toBe(404);
  expect(await page.locator("h1").textContent()).toBe("Page not found");
  await page.close();
});

test("at phone width, the sidebar opens from the menu and nothing scrolls sideways", async () => {
  const page = await open("/reference/content-model/", 390);
  const sidebar = page.locator("#sidebar");
  expect(await sidebar.isVisible()).toBe(false);
  await page.getByRole("button", { name: "Menu" }).click();
  expect(await sidebar.isVisible()).toBe(true);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
  );
  expect(overflow).toBe(0);
  await page.close();
});
