// A versioned build in one site: a copy of the example with two live lines
// of `self-managed`, 3.4 and 3.5, and a page only 3.5 has. One collection
// holds both lines' pages, 3.5's at `/docs/` and 3.4's at `/docs/3.4/`, each
// page lists the lines that have it, and `astro dev` rebuilds both.
import { rm } from "node:fs/promises";
import path from "node:path";
import type { Browser, Page } from "playwright-core";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import {
  BASE,
  buildSite,
  edit,
  launchChromium,
  serveDev,
  servePreview,
  siteDir,
  untilRebuilt,
} from "./harness.js";
import { versioned } from "./sites.js";

let browser: Browser;
let root: string;
let server: Awaited<ReturnType<typeof servePreview>>;

beforeAll(async () => {
  root = await versioned("versions");
  await buildSite(root);
  server = await servePreview(root);
  browser = await launchChromium();
}, 180_000);

afterAll(async () => {
  await browser?.close();
  await server?.stop();
  await rm(path.join(siteDir, ".e2e-tmp"), { recursive: true, force: true });
});

async function open(route: string): Promise<Page> {
  const page = await browser.newPage();
  const response = await page.goto(`${server.origin}${route}`);
  expect(response?.status(), route).toBe(200);
  return page;
}

/** The lines a page's `Versions.astro` lists, each with its link, and which is marked as the page's own. */
function versionsOf(page: Page): Promise<{ line: string; href: string; own: boolean }[]> {
  return page
    .getByRole("navigation", { name: "Versions" })
    .getByRole("link")
    .evaluateAll((links) =>
      links.map((a) => ({
        line: a.textContent ?? "",
        href: a.getAttribute("href") ?? "",
        own: a.getAttribute("aria-current") === "page",
      })),
    );
}

describe("a versioned build", () => {
  it("serves each line's page at its own path, and each lists the other", async () => {
    const current = await open(`${BASE}/guides/my-setup`);
    expect(await current.locator("main > h1").textContent()).toBe("Set up Loom");
    expect(await versionsOf(current)).toEqual([
      { line: "3.5", href: `${BASE}/guides/my-setup`, own: true },
      { line: "3.4", href: `${BASE}/3.4/guides/my-setup`, own: false },
    ]);
    await current.close();
    const older = await open(`${BASE}/3.4/guides/my-setup`);
    expect(await older.locator("main > h1").textContent()).toBe("Set up Loom");
    expect(await versionsOf(older)).toEqual([
      { line: "3.5", href: `${BASE}/guides/my-setup`, own: false },
      { line: "3.4", href: `${BASE}/3.4/guides/my-setup`, own: true },
    ]);
    await older.close();
    // The older line's root page is at its base path.
    const home = await open(`${BASE}/3.4`);
    expect(await home.locator("main > h1").textContent()).toBe("Loom documentation");
    await home.close();
  });

  it("publishes a page only in the lines that have it, and lists no others", async () => {
    const page = await open(`${BASE}/guides/upgrade`);
    expect(await page.getByRole("navigation", { name: "Versions" }).count()).toBe(0);
    await page.close();
    const missing = await browser.newPage();
    expect((await missing.goto(`${server.origin}${BASE}/3.4/guides/upgrade`))?.status()).toBe(404);
    await missing.close();
  });

  it("keeps an older line's sidebar and links in that line, and every link lands", async () => {
    for (const route of ["", "/guides/my-setup", "/reference/options"]) {
      const page = await open(`${BASE}/3.4${route}`);
      const sidebar = await page
        .getByRole("navigation", { name: "Docs" })
        .getByRole("link")
        .evaluateAll((links) => links.map((a) => a.getAttribute("href")));
      // The page 3.4 doesn't have isn't listed.
      expect(sidebar).toEqual([`${BASE}/3.4`, `${BASE}/3.4/guides/my-setup`, `${BASE}/3.4/reference/options`]);
      const links = await page
        .locator("article a[href^='/']")
        .evaluateAll((anchors) => anchors.map((a) => a.getAttribute("href") ?? ""));
      for (const href of links) {
        expect(href, route).toMatch(new RegExp(`^${BASE}/3\\.4(/|$|#)`));
        const response = await page.request.get(`${server.origin}${href.split("#")[0]}`);
        expect(response.status(), href).toBe(200);
      }
      await page.close();
    }
  });

  it("processes an image on an older line's page as on the current line's", async () => {
    for (const route of [`${BASE}/reference/options`, `${BASE}/3.4/reference/options`]) {
      const page = await open(route);
      const image = page.getByRole("img", { name: "The weave diagram" });
      await expect
        .poll(() =>
          image.evaluate((element: HTMLImageElement) => element.complete && element.naturalWidth > 0),
        )
        .toBe(true);
      expect(await image.getAttribute("src"), route).toMatch(
        new RegExp(`^${BASE}/_astro/weave\\.[\\w-]+\\.webp$`),
      );
      await page.close();
    }
  });

  it("serves each line's linked files at its base path", async () => {
    for (const base of [BASE, `${BASE}/3.4`]) {
      const response = await fetch(`${server.origin}${base}/_ascribe/files/downloads/loom.yaml`);
      expect(response.status, base).toBe(200);
    }
  });

  it("forwards a permalink to the current line", async () => {
    const response = await fetch(`${server.origin}${BASE}/go/weave-config`);
    expect(await response.text()).toContain(
      `<meta http-equiv="refresh" content="0;url=${BASE}/guides/my-setup#weave-config">`,
    );
  });

  it("leaves an older line's pages out of the sitemap, as not the site's own", async () => {
    const sitemap = await (await fetch(`${server.origin}${BASE}/sitemap.xml`)).text();
    expect(sitemap).toContain(`https://docs.example.com${BASE}/guides/my-setup<`);
    expect(sitemap).not.toContain(`${BASE}/3.4`);
  });

  it("publishes the current line's Markdown for agents, and not an older line's", async () => {
    expect((await fetch(`${server.origin}${BASE}/guides/my-setup.md`)).status).toBe(200);
    expect((await fetch(`${server.origin}${BASE}/3.4/guides/my-setup.md`)).status).toBe(404);
  });

  it("rebuilds every line in astro dev", async () => {
    const dev = await serveDev(root);
    try {
      const file = path.join(root, "content", "Guides", "My Setup.md");
      await edit(file, (text) => text.replace("This guide sets up", "This updated guide sets up"));
      for (const route of [`${BASE}/guides/my-setup`, `${BASE}/3.4/guides/my-setup`]) {
        await untilRebuilt(file, async () =>
          (await (await fetch(`${dev.origin}${route}`)).text()).includes("This updated guide"),
        );
      }
      const files = await fetch(`${dev.origin}${BASE}/3.4/_ascribe/files/downloads/loom.yaml`);
      expect(files.status).toBe(200);
    } finally {
      await dev.stop();
    }
  }, 90_000);
});
