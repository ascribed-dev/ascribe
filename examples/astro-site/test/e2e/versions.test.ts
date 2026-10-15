// A versioned build in one site: a copy of the example with two live lines
// of `self-managed`, 3.4 and 3.5, and a page only 3.5 has. One collection
// holds both lines' pages, 3.5's at `/docs/` and 3.4's at `/docs/3.4/`, each
// page lists the lines, a 3.4 page says what it is and where the current one
// is, to a reader and in its head, and `astro dev` rebuilds both.
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
let unlistedServer: Awaited<ReturnType<typeof servePreview>> | undefined;

beforeAll(async () => {
  root = await versioned("versions");
  await buildSite(root);
  server = await servePreview(root);
  browser = await launchChromium();
}, 180_000);

afterAll(async () => {
  await browser?.close();
  await server?.stop();
  await unlistedServer?.stop();
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

  it("publishes a page only in the lines that have it, and lists the others unlinked", async () => {
    const page = await open(`${BASE}/guides/upgrade`);
    expect(await versionsOf(page)).toEqual([{ line: "3.5", href: `${BASE}/guides/upgrade`, own: true }]);
    expect(
      await page.getByRole("navigation", { name: "Versions" }).locator("[data-missing]").textContent(),
    ).toBe("3.4 (not in this version)");
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

  it("says on an older line's page which version it is, and links to the current one", async () => {
    const older = await open(`${BASE}/3.4/guides/my-setup`);
    const banner = older.locator("[data-version-banner]");
    expect((await banner.textContent())?.replace(/\s+/g, " ").trim()).toBe(
      "Documentation for Self-managed 3.4. Support ends on 31 January 2027. See this page for 3.5.",
    );
    expect(await banner.getByRole("link").getAttribute("href")).toBe(`${BASE}/guides/my-setup`);
    await older.close();
    const current = await open(`${BASE}/guides/my-setup`);
    expect(await current.locator("[data-version-banner]").count()).toBe(0);
    await current.close();
  });

  it("points an older line's canonical link at the current line's page", async () => {
    for (const route of [`${BASE}/guides/my-setup`, `${BASE}/3.4/guides/my-setup`]) {
      const page = await open(route);
      expect(await page.locator("link[rel=canonical]").getAttribute("href"), route).toBe(
        `https://docs.example.com${BASE}/guides/my-setup`,
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

  it("lists each page in the sitemap once, at the current line's address", async () => {
    const sitemap = await (await fetch(`${server.origin}${BASE}/sitemap.xml`)).text();
    expect(sitemap).toContain(`https://docs.example.com${BASE}/guides/my-setup<`);
    expect(sitemap).not.toContain(`${BASE}/3.4`);
  });

  it("publishes each line's Markdown and llms.txt for agents, an older line's pointing at the current", async () => {
    const current = await (await fetch(`${server.origin}${BASE}/guides/my-setup.md`)).text();
    expect(current).toContain("> Documentation for Self-managed 3.5.");
    const older = await fetch(`${server.origin}${BASE}/3.4/guides/my-setup.md`);
    expect(older.status).toBe(200);
    expect(await older.text()).toContain(
      `The current version is 3.5: <https://docs.example.com${BASE}/guides/my-setup>.`,
    );
    const index = await (await fetch(`${server.origin}${BASE}/3.4/llms.txt`)).text();
    expect(index).toMatch(/^# .* \(Self-managed 3\.4\)\n/);
    expect(index).toContain(`[llms.txt](https://docs.example.com${BASE}/llms.txt)`);
    expect(index).toContain(`(https://docs.example.com${BASE}/3.4/guides/my-setup.md)`);
    expect(index).not.toContain("upgrade");
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

describe("a versioned build that keeps its older lines out of an index", () => {
  it("marks every older page noindex, and leaves it out of the sitemap and llms.txt", async () => {
    const root = await versioned("versions-unlisted");
    await edit(path.join(root, "ascribe.toml"), (text) =>
      text.includes("[listing]")
        ? text.replace("[listing]", '[listing]\nolder-lines = "unlisted"')
        : `${text}\n[listing]\nolder-lines = "unlisted"\n`,
    );
    await buildSite(root);
    const unlisted = await servePreview(root);
    unlistedServer = unlisted;
    const page = await browser.newPage();
    await page.goto(`${unlisted.origin}${BASE}/3.4/guides/my-setup`);
    expect(await page.locator('meta[name="robots"]').getAttribute("content")).toContain("noindex");
    await page.close();
    const sitemap = await (await fetch(`${unlisted.origin}${BASE}/sitemap.xml`)).text();
    expect(sitemap).not.toContain(`${BASE}/3.4`);
    const index = await (await fetch(`${unlisted.origin}${BASE}/3.4/llms.txt`)).text();
    expect(index).not.toContain("my-setup.md");
  });
});
