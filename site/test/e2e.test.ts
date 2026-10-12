// The built site in Chromium: the landing page, the changelog and its entries, the head and the header,
// each page's social card, variants, availability, code titles, links between
// pages, search, the edit link, the 404 page, the narrow layout, the landing
// page's demonstration, marks, phone width, and reading without a script, and
// Why Ascribe, with its screenshots.
import { afterAll, beforeAll, expect, test } from "vitest";
import type { Browser, Page } from "playwright-core";
import { readdirSync } from "node:fs";
import path from "node:path";
import { launchChromium, requireBuild, serve, siteAddress, siteDir } from "./built.ts";

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

/** `#rrggbb` as getComputedStyle writes it. */
function rgb(hex: string): string {
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
  return `rgb(${r}, ${g}, ${b})`;
}

async function open(path: string, width = 1280): Promise<Page> {
  const page = await browser.newPage({ viewport: { width, height: 900 } });
  await page.goto(server.url + path);
  return page;
}

test("the head names the icons, the social card, and a theme color per scheme", async () => {
  const page = await open("/docs/getting-started/");
  const icons = await page
    .locator('link[rel="icon"], link[rel="apple-touch-icon"]')
    .evaluateAll((links) => links.map((link) => (link as HTMLLinkElement).href));
  expect(icons.map((href) => new URL(href).pathname)).toEqual([
    "/favicon.ico",
    "/favicon.svg",
    "/apple-touch-icon.png",
  ]);
  for (const href of icons) {
    const response = await page.request.get(href);
    expect(response.status(), href).toBe(200);
  }

  // A shared link's card is an absolute address on the site; it's served here.
  const card = await page.locator('meta[property="og:image"]').getAttribute("content");
  expect(card).toBe(`${await siteAddress()}/social-card.png`);
  const served = await page.request.get(`${server.url}${new URL(card ?? "").pathname}`);
  expect(served.status()).toBe(200);
  expect(served.headers()["content-type"]).toBe("image/png");
  expect(await page.locator('meta[name="twitter:card"]').getAttribute("content")).toBe(
    "summary_large_image",
  );

  const themes = await page
    .locator('meta[name="theme-color"]')
    .evaluateAll((metas) =>
      metas.map((meta) => [meta.getAttribute("media"), meta.getAttribute("content")]),
    );
  expect(themes.map(([media]) => media)).toEqual([
    "(prefers-color-scheme: light)",
    "(prefers-color-scheme: dark)",
  ]);
  // Each is the page's background in its scheme.
  for (const [media, color] of themes) {
    const scheme = media?.includes("dark") ? "dark" : "light";
    await page.emulateMedia({ colorScheme: scheme });
    const background = await page.evaluate(() => getComputedStyle(document.body).backgroundColor);
    expect(background, scheme).toBe(rgb(color ?? ""));
  }
  await page.close();
});

test("a screenshot shows the reader's scheme, optimized by Astro, with text that says what it shows", async () => {
  for (const scheme of ["light", "dark"] as const) {
    const page = await open("/why/");
    await page.emulateMedia({ colorScheme: scheme });
    const shown = page.locator("claim-example img[scheme]:visible");
    await expect(shown.count()).resolves.toBe(2);
    for (const image of await shown.all()) {
      await image.scrollIntoViewIfNeeded();
      expect(await image.getAttribute("scheme")).toBe(scheme);
      expect(await image.getAttribute("src")).toMatch(/^\/_astro\/.+\.webp$/);
      expect((await image.getAttribute("alt"))?.length).toBeGreaterThan(40);
      await expect
        .poll(() => image.evaluate((img) => (img as HTMLImageElement).naturalWidth))
        .toBeGreaterThan(0);
    }
    await page.close();
  }
});

test("each page outside the docs names its own social card, which is served", async () => {
  // Generated into public/cards/ by ../design: one per page of home/, by its
  // entry id, and the changelog's.
  const cards = readdirSync(path.join(siteDir, "public", "cards")).map((f) => f.slice(0, -4));
  expect(cards.sort()).toEqual(["changelog", "index", "why"]);
  for (const name of cards) {
    const page = await open(name === "index" ? "/" : `/${name}/`);
    const card = await page.locator('meta[property="og:image"]').getAttribute("content");
    expect(card, name).toBe(`${await siteAddress()}/cards/${name}.png`);
    const served = await page.request.get(`${server.url}${new URL(card ?? "").pathname}`);
    expect(served.status(), name).toBe(200);
    expect(served.headers()["content-type"]).toBe("image/png");
    const alt = await page.locator('meta[property="og:image:alt"]').getAttribute("content");
    const heading = await page.locator("h1").textContent();
    expect(alt, name).toBe(`Ascribe: ${heading?.trim()}`);
    await page.close();
  }
});

test("the header's mark links home and is named, and Docs links to the docs", async () => {
  const page = await open("/docs/guides/astro/");
  const home = page.getByRole("link", { name: "Ascribe", exact: true });
  expect(await home.getAttribute("href")).toBe("/");
  expect(await home.getByRole("img", { name: "Ascribe" }).isVisible()).toBe(true);
  const docs = page.getByRole("link", { name: "Docs", exact: true });
  expect(await docs.getAttribute("href")).toBe("/docs/");
  expect(await docs.getAttribute("aria-current")).toBe("true");
  await page.close();
});

test("the landing page is at the root, with one heading in its hero and no docs furniture", async () => {
  const page = await open("/");
  expect(await page.title()).toBe("Ascribe");
  expect(await page.locator("hero-banner").count()).toBe(1);
  const heading = page.locator("h1");
  expect(await heading.count()).toBe(1);
  expect(await page.locator("hero-banner h1").isVisible()).toBe(true);
  // The hero is drawn by the stylesheet: its heading takes the hero's size.
  const size = await heading.evaluate((el) => parseFloat(getComputedStyle(el).fontSize));
  expect(size).toBeGreaterThan(36);
  for (const docsOnly of ["#sidebar", ".search-button", ".menu-button", ".toc"]) {
    expect(await page.locator(docsOnly).count(), docsOnly).toBe(0);
  }
  expect(
    await page.getByRole("link", { name: "Docs", exact: true }).getAttribute("aria-current"),
  ).toBeNull();
  await page.close();
});

test("the landing page's link to getting started opens it under /docs/", async () => {
  const page = await open("/");
  const link = page.getByRole("link", { name: "read getting started" });
  // The permalink's address on the site the build was for.
  const href = new URL((await link.getAttribute("href")) ?? "");
  expect(href.origin).toBe(await siteAddress());
  expect(href.pathname).toBe("/go/getting-started/");
  await page.goto(server.url + href.pathname);
  await page.waitForURL(`${server.url}/docs/getting-started/`);
  expect(await page.locator("h1").textContent()).toBe("Getting started");
  await page.close();
});

test("the header's Changelog opens the changelog, in the frame of the site's own pages", async () => {
  const page = await open("/");
  const link = page.getByRole("link", { name: "Changelog", exact: true });
  expect(await link.getAttribute("href")).toBe("/changelog/");
  await link.click();
  await page.waitForURL(`${server.url}/changelog/`);
  expect(await page.title()).toBe("Changelog · Ascribe");
  expect(await link.getAttribute("aria-current")).toBe("page");
  expect(await page.locator("h1").textContent()).toBe("Changelog");
  for (const docsOnly of ["#sidebar", ".search-button", ".menu-button", ".toc"]) {
    expect(await page.locator(docsOnly).count(), docsOnly).toBe(0);
  }
  // The unreleased section says so, and each release has its version's anchor.
  expect(await page.locator("#unreleased + .unreleased-note").isVisible()).toBe(true);
  expect(await page.locator("h2[id^='v0.']").count()).toBeGreaterThan(1);
  // A link to a docs page's source opens the page, once its entry is open.
  await page.getByRole("button", { name: "Expand all" }).click();
  await page.locator("article a[href^='/docs/']").first().click();
  await page.waitForURL(new RegExp(`^${server.url}/docs/.+`));
  expect(await page.locator("h1").count()).toBe(1);
  await page.close();
});

test("the changelog's entries open one at a time or all at once, and can be only behavior changes", async () => {
  const page = await open("/changelog/");
  const entries = page.locator(".entry > details");
  const first = entries.first();
  expect(await entries.count()).toBeGreaterThan(50);
  // Each entry shows what it says first, and the rest when it's opened.
  expect(await first.evaluate((details) => (details as HTMLDetailsElement).open)).toBe(false);
  expect(await first.locator(".entry-body").isVisible()).toBe(false);
  await first.locator("summary").click();
  expect(await first.locator(".entry-body").isVisible()).toBe(true);
  await page.getByRole("button", { name: "Collapse all" }).click();
  expect(await page.locator(".entry > details[open]").count()).toBe(0);
  await page.getByRole("button", { name: "Expand all" }).click();
  expect(await page.locator(".entry > details:not([open])").count()).toBe(0);

  // Each area's heading counts its entries.
  const area = page.locator(".release").first().locator(".area").first();
  const count = Number(await area.locator("h3 .count").textContent());
  expect(await area.locator(".entry").count()).toBe(count);

  // Behavior changes only hides the rest, and the areas with none.
  const filter = page.getByRole("button", { name: "Behavior changes only" });
  await filter.click();
  expect(await filter.getAttribute("aria-pressed")).toBe("true");
  const shown = await page
    .locator(".entry")
    .evaluateAll((all) =>
      all
        .filter((entry) => (entry as HTMLElement).offsetParent !== null)
        .map((entry) => entry.className),
    );
  expect(shown.length).toBeGreaterThan(0);
  expect(shown.every((name) => name.includes("behavior"))).toBe(true);
  expect(await page.locator(".badge").first().isVisible()).toBe(true);
  await filter.click();
  expect(await page.locator(".entry:not(.behavior)").first().isVisible()).toBe(true);

  // The contents beside the page lead to each release and the newest ones' areas.
  const contents = page.locator(".contents");
  expect(await contents.locator("summary").isVisible()).toBe(false);
  await contents
    .getByRole("link")
    .filter({ hasText: /^Fixes/ })
    .first()
    .click();
  await page.waitForURL(/#unreleased-fixes$/);
  expect(await contents.getByRole("link", { name: /^0\.1\.0/ }).isVisible()).toBe(true);
  await page.close();
});

test("at phone width, the changelog's contents are above it, behind a button", async () => {
  const page = await open("/changelog/", 390);
  const contents = page.locator(".contents");
  const box = await contents.boundingBox();
  const release = await page.locator("#unreleased").boundingBox();
  expect(box && release && box.y < release.y).toBe(true);
  expect(await contents.getByRole("link").first().isVisible()).toBe(false);
  await contents.locator("summary").click();
  expect(await contents.getByRole("link").first().isVisible()).toBe(true);
  await page.close();
});

test("a variant group is a switcher, and the reader's choice holds on the next page", async () => {
  const page = await open("/docs/getting-started/");
  const tabs = page.locator("ascribe-tabs").first();
  await expect.poll(() => tabs.getByRole("tab").allTextContents()).toEqual(["npm", "pnpm"]);
  expect(await tabs.getByText("npm install --save-dev @ascribed/cli").isVisible()).toBe(true);
  expect(await tabs.getByText("pnpm add --save-dev @ascribed/cli").isVisible()).toBe(false);

  await tabs.getByRole("tab", { name: "pnpm" }).click();
  expect(await tabs.getByText("pnpm add --save-dev @ascribed/cli").isVisible()).toBe(true);

  await page.goto(`${server.url}/docs/guides/astro/`);
  const next = page.locator("ascribe-tabs").first();
  await expect
    .poll(() => next.getByRole("tab", { name: "pnpm" }).getAttribute("aria-selected"))
    .toBe("true");
  expect(await next.getByText("pnpm add @ascribed/astro").isVisible()).toBe(true);
  await page.close();
});

test("a page marked with an availability shows its badge", async () => {
  const page = await open("/docs/guides/review/");
  const badge = page.locator('ascribe-availability[scope="page"] ascribe-availability-target');
  // "unreleased" until a release ships the page's feature, then the release.
  expect(await badge.getAttribute("target")).toBe("ascribe");
  expect(await badge.isVisible()).toBe(true);
  // The element library's stylesheet draws it as a badge.
  const radius = await badge.evaluate((el) => getComputedStyle(el).borderTopLeftRadius);
  expect(radius).not.toBe("0px");
  await page.close();
});

test("notes and steps render as elements", async () => {
  const page = await open("/docs/getting-started/");
  expect(await page.locator('ascribe-note[type="tip"]').isVisible()).toBe(true);
  expect(await page.locator("ascribe-steps ol > li").count()).toBe(3);
  await page.close();
});

test("a code example taken from a file shows its title", async () => {
  const page = await open("/docs/guides/astro/");
  const figure = page.locator("figure.code-title", { hasText: "export default defineConfig" });
  await expect.poll(() => figure.locator("figcaption").textContent()).toBe("astro.config.mjs");
  await page.close();
});

test("a link between pages lands on the page, and a glossary link on its heading", async () => {
  const page = await open("/docs/getting-started/");
  await page.locator('article a[href="/docs/guides/astro/"]').first().click();
  await page.waitForURL(`${server.url}/docs/guides/astro/`);
  expect(await page.locator("h1").textContent()).toBe("Astro");

  await page.goto(`${server.url}/docs/reference/diagnostics/`);
  // A glossary link to a heading (the first one on this page links to a page).
  const term = page.locator('article a[data-ascribe-term][href*="#"]').first();
  const href = await term.getAttribute("href");
  expect(href).toMatch(/#/);
  await term.click();
  const id = decodeURIComponent(new URL(page.url()).hash.slice(1));
  expect(await page.locator(`[id="${id}"]`).count()).toBe(1);
  await page.close();
});

test("the sidebar lists the pages and marks the current one", async () => {
  const page = await open("/docs/reference/cli/");
  const current = page.locator('#sidebar a[aria-current="page"]');
  expect(await current.textContent()).toContain("Command reference");
  expect(await page.locator("#sidebar a").count()).toBeGreaterThanOrEqual(13);
  await page.close();
});

test("search finds a known word, with no network", async () => {
  const page = await open("/docs/");
  await page.route(/^(?!http:\/\/127\.0\.0\.1)/, (route) => route.abort());
  await page.keyboard.press("/");
  const input = page.locator("#search input");
  await input.fill("slugger");
  // The page among the results, not the first: Pagefind ranks by how often
  // a page says the word for its length, which moves as pages grow.
  const result = page.locator('.pagefind-ui__result-link[href="/docs/reference/content-model/"]');
  await result.waitFor();
  expect(await result.textContent()).toBeTruthy();
  await result.click();
  await page.waitForURL(/\/docs\/reference\/content-model\//);
  await page.close();
});

test("each page links to its source on GitHub", async () => {
  const page = await open("/docs/guides/astro/");
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
  expect(await page.getByText("This address is a permalink").isVisible()).toBe(false);
  expect(await page.getByRole("link", { name: "docs' start page" }).getAttribute("href")).toBe(
    "/docs/",
  );
  await page.close();
});

test("a permalink the docs don't have gets the 404 page, which says so", async () => {
  const page = await browser.newPage();
  const response = await page.goto(`${server.url}/go/no-such-name/`);
  expect(response?.status()).toBe(404);
  const note = page.getByText("This address is a permalink");
  expect(await note.isVisible()).toBe(true);
  expect(await note.locator("code").textContent()).toBe("no-such-name");
  await page.close();
});

test("at phone width, the sidebar opens from the menu and nothing scrolls sideways", async () => {
  const page = await open("/docs/reference/content-model/", 390);
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

test("at phone width, the changelog and the landing page don't scroll sideways", async () => {
  for (const path of ["/changelog/", "/"]) {
    const page = await open(path, 390);
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
    );
    expect(overflow, path).toBe(0);
    expect(await page.getByRole("link", { name: "Changelog", exact: true }).isVisible()).toBe(true);
    await page.close();
  }
});

test("the demonstration draws its page beside its source, and under it at phone width", async () => {
  for (const [width, beside] of [
    [1280, true],
    [390, false],
  ] as const) {
    const page = await open("/", width);
    const source = page.locator("source-and-result > figure");
    const result = page.locator("source-and-result > .result");
    expect(await result.locator(".result-title").textContent()).toBe("Install Quill");
    // Drawn by the element library, as on any page.
    expect(await result.locator("ascribe-note").count()).toBe(1);
    expect(await result.locator("ascribe-tabs").count()).toBe(1);
    expect(await result.locator("ascribe-steps").count()).toBe(1);
    const [left, right] = [await source.boundingBox(), await result.boundingBox()];
    if (left === null || right === null) throw new Error("not drawn");
    expect(right.x > left.x + left.width, `${width}: beside`).toBe(beside);
    expect(right.y >= left.y + left.height, `${width}: under`).toBe(!beside);
    await page.close();
  }
});

test("the landing page marks a claim no release has, as the docs do", async () => {
  const page = await open("/");
  const marks = page.locator("claim-example ascribe-availability");
  expect(await marks.count()).toBeGreaterThan(0);
  expect(await marks.first().isVisible()).toBe(true);
  expect(await marks.first().textContent()).toContain("not yet released");
  await page.close();
});

test("at phone width, nothing on the landing page scrolls sideways", async () => {
  const page = await open("/", 390);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
  );
  expect(overflow).toBe(0);
  await page.close();
});

test("the landing page reads without a script", async () => {
  const context = await browser.newContext({
    javaScriptEnabled: false,
    viewport: { width: 1280, height: 900 },
  });
  const page = await context.newPage();
  await page.goto(server.url + "/");
  for (const text of [
    "Documentation written as code",
    "Install Quill",
    "Checked",
    "Reviewed as pages",
    "npm install --save-dev @ascribed/cli",
    "Where it’s going",
  ]) {
    expect(await page.getByText(text, { exact: true }).first().isVisible(), text).toBe(true);
  }
  expect(await page.locator("source-and-result .result ascribe-note").isVisible()).toBe(true);
  await context.close();
});

test("the header's Why Ascribe opens the page, in the frame of the site's own pages", async () => {
  const page = await open("/");
  const link = page.locator("header").getByRole("link", { name: "Why Ascribe", exact: true });
  expect(await link.getAttribute("href")).toBe("/why/");
  await link.click();
  await page.waitForURL(`${server.url}/why/`);
  expect(await page.title()).toBe("Why Ascribe · Ascribe");
  expect(await link.getAttribute("aria-current")).toBe("page");
  const heading = page.locator("h1");
  expect(await heading.count()).toBe(1);
  expect(await page.locator("hero-banner h1").textContent()).toBe("Why Ascribe");
  // The badge is the landing page's: this hero has no status.
  expect(await page.locator("hero-banner[status]").count()).toBe(0);
  for (const docsOnly of ["#sidebar", ".search-button", ".menu-button", ".toc"]) {
    expect(await page.locator(docsOnly).count(), docsOnly).toBe(0);
  }
  await page.close();
});

test("the landing page links to Why Ascribe", async () => {
  const page = await open("/");
  const link = page.locator("article").getByRole("link", { name: "Why Ascribe", exact: true });
  expect(await link.getAttribute("href")).toBe("/why/");
  await page.close();
});

test("Why Ascribe shows its examples, marks what no release has, and links to the comparison", async () => {
  const page = await open("/why/");
  const problems = page.locator("claim-example");
  expect(await problems.count()).toBeGreaterThan(3);
  // Each example is a code block read from a file in the repository.
  expect(await page.locator("claim-example pre").count()).toBeGreaterThan(3);
  const marks = page.locator("claim-example ascribe-availability");
  expect(await marks.count()).toBeGreaterThan(0);
  expect(await marks.first().textContent()).toContain("not yet released");
  const comparison = page.getByRole("link", { name: "examples/comparison" });
  expect(await comparison.getAttribute("href")).toBe(
    "https://github.com/ascribed-dev/ascribe/tree/main/examples/comparison",
  );
  expect(await page.getByRole("heading", { name: "What it doesn’t do" }).isVisible()).toBe(true);
  await page.close();
});

test("at phone width, nothing on Why Ascribe scrolls sideways, and it reads without a script", async () => {
  const page = await open("/why/", 390);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
  );
  expect(overflow).toBe(0);
  await page.close();

  const context = await browser.newContext({ javaScriptEnabled: false });
  const plain = await context.newPage();
  await plain.goto(server.url + "/why/");
  for (const text of ["Examples stop matching the code", "No hosting."]) {
    expect(await plain.getByText(text, { exact: true }).first().isVisible(), text).toBe(true);
  }
  await context.close();
});
