// The Astro end-to-end slice (phase 21): a real `astro build` of this site,
// then the built HTML checked in Chromium. Run by `pnpm test:e2e`; it needs a
// built `ascribe` binary (`cargo build -p tessera-cli`, or TESSERA_BIN) and
// `pnpm --filter @ascribed/astro build` and `pnpm --filter @ascribed/elements build`.
import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import type { Browser, BrowserContext, Page } from "playwright-core";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { BASE, buildSite, launchChromium, servePreview, siteDir } from "./harness.js";

let browser: Browser;
let context: BrowserContext;
let server: Awaited<ReturnType<typeof servePreview>>;
let origin: string;

beforeAll(async () => {
  await buildSite();
  server = await servePreview();
  origin = server.origin;
  browser = await launchChromium();
  context = await browser.newContext();
}, 180_000);

afterAll(async () => {
  await context?.close();
  await browser?.close();
  await server?.stop();
});

async function open(route: string): Promise<Page> {
  const page = await context.newPage();
  const response = await page.goto(`${origin}${route}`);
  expect(response?.status(), route).toBe(200);
  return page;
}

describe("the built site", () => {
  it("serves the root page at the base path, and the others at their entry ids", async () => {
    for (const [route, title] of [
      [BASE, "Loom documentation"],
      [`${BASE}/guides/my-setup`, "Set up Loom"],
      [`${BASE}/reference/options`, "Options"],
    ] as const) {
      const page = await open(route);
      await expect(page.locator("main > h1").textContent()).resolves.toBe(title);
      await page.close();
    }
  });

  it("follows links that only work if Tessera's routes are Astro's", async () => {
    const page = await open(BASE);
    // `[setting up Loom](<Guides/My Setup.md>)`: a slugged, lower-cased entry id.
    await page.getByRole("link", { name: "setting up Loom" }).click();
    await page.waitForURL(`${origin}${BASE}/guides/my-setup`);
    await expect(page.locator("main > h1").textContent()).resolves.toBe("Set up Loom");
    // `[documentation home](../index.md)`: the root page, at the base path with no final slash.
    await page.getByRole("link", { name: "documentation home" }).click();
    await page.waitForURL(`${origin}${BASE}`);
    await expect(page.locator("main > h1").textContent()).resolves.toBe("Loom documentation");
    await page.close();
  });

  it("lands an explicit-id link on the heading", async () => {
    const page = await open(`${BASE}/reference/options`);
    await page.getByRole("link", { name: "the weave configuration section" }).click();
    await page.waitForURL(`${origin}${BASE}/guides/my-setup#weave-config`);
    const heading = page.locator("h2#weave-config");
    await expect(heading.textContent()).resolves.toBe("Weave configuration");
    await expect(heading.evaluate((element) => element.matches(":target"))).resolves.toBe(true);
    // The heading is on screen (the page is too short to scroll it to the top).
    const box = await heading.evaluate((element) => {
      const { top, bottom } = element.getBoundingClientRect();
      return { top, bottom, height: window.innerHeight };
    });
    expect(box.top).toBeGreaterThanOrEqual(0);
    expect(box.bottom).toBeLessThanOrEqual(box.height);
    await page.close();
  });

  it("loads the fragment's image, processed by Astro, with the marker's attributes", async () => {
    const page = await open(`${BASE}/guides/my-setup`);
    const image = page.getByRole("img", { name: "Checklist of requirements" });
    await expect(
      image.evaluate((element: HTMLImageElement) => element.complete && element.naturalWidth > 0),
    ).resolves.toBe(true);
    const attributes = await image.evaluate((element: HTMLImageElement) => ({
      src: element.getAttribute("src"),
      width: element.getAttribute("width"),
      height: element.getAttribute("height"),
      loading: element.getAttribute("loading"),
    }));
    // Astro replaced the mirrored `../_fragments/requirements.png` with a hashed, optimized file
    // under `<base>/_astro/`, and kept `width=300` from the marker (the image is 64 by 40).
    expect(attributes.src).toMatch(new RegExp(`^${BASE}/_astro/requirements\\.[\\w-]+\\.webp$`));
    expect(attributes.width).toBe("300");
    expect(attributes.height).toBe("188");
    expect(attributes.loading).toBe("lazy");
    await page.close();
  });

  it("processes an image beside its page the same way", async () => {
    const page = await open(`${BASE}/reference/options`);
    const image = page.getByRole("img", { name: "The weave diagram" });
    await expect(
      image.evaluate((element: HTMLImageElement) => element.complete && element.naturalWidth > 0),
    ).resolves.toBe(true);
    expect(await image.getAttribute("src")).toMatch(
      new RegExp(`^${BASE}/_astro/weave\\.[\\w-]+\\.webp$`),
    );
    expect(await image.getAttribute("width")).toBe("120");
    await page.close();
  });

  it("shows the one arm a partial selection leaves, and switches the group it doesn't reduce", async () => {
    const page = await open(`${BASE}/guides/my-setup`);
    // The `deployment` group is reduced to its cloud arm: plain content, no tabs.
    const connect = page.locator("article");
    await expect(
      connect.getByText("Sign in to Loom Cloud and copy your API key.").isVisible(),
    ).resolves.toBe(true);
    await expect(page.getByText("Point the CLI at your own server").count()).resolves.toBe(0);
    await expect(page.locator('ascribe-tabs[sync="deployment"]').count()).resolves.toBe(0);
    // The `pm` group is still a switcher.
    const group = page.locator('ascribe-tabs[sync="pm"]');
    await expect(group.getByRole("tab").allTextContents()).resolves.toEqual([
      "npm",
      "pnpm",
      "yarn",
    ]);
    await expect(group.getByText("npm install -g @loom/cli").isVisible()).resolves.toBe(true);
    await expect(group.getByText("pnpm add -g @loom/cli").isVisible()).resolves.toBe(false);
    await group.getByRole("tab", { name: "pnpm" }).click();
    await expect(group.getByText("pnpm add -g @loom/cli").isVisible()).resolves.toBe(true);
    await expect(group.getByText("npm install -g @loom/cli").isVisible()).resolves.toBe(false);
    await page.close();
  });

  it("renders the availability badges, page-level from frontmatter and section-level", async () => {
    const page = await open(`${BASE}/guides/my-setup`);
    const pageBadge = page.locator('ascribe-availability[scope="page"]');
    await expect(pageBadge.innerText()).resolves.toContain(
      "Loom Cloud (GA); Self-managed (preview, 3.4+)",
    );
    // The element library's stylesheet applies: a lead-in and a coloured, rounded badge.
    await expect(
      pageBadge.evaluate((element) => getComputedStyle(element, "::before").content),
    ).resolves.toBe('"Available: "');
    const background = await pageBadge
      .locator("ascribe-availability-target")
      .first()
      .evaluate((element) => getComputedStyle(element).backgroundColor);
    expect(background).not.toBe("rgba(0, 0, 0, 0)");
    const section = page.locator('ascribe-availability[scope="section"]');
    await expect(section.innerText()).resolves.toContain("Self-managed (preview, 3.4+)");
    await page.close();
  });

  it("serves a linked file at <base>/_ascribe/files/", async () => {
    const page = await open(`${BASE}/guides/my-setup`);
    const href = await page.getByRole("link", { name: "sample config" }).getAttribute("href");
    expect(href).toBe(`${BASE}/_ascribe/files/downloads/loom.yaml`);
    const response = await page.request.get(`${origin}${href}`);
    expect(response.status()).toBe(200);
    await expect(response.text()).resolves.toBe("weave:\n  strands: 4\n");
    await page.close();
  });

  it("gives every heading the id Tessera validated, at the route Tessera computed", async () => {
    // `ascribe build --emit json` records each page's route and heading ids.
    const binary = process.env["TESSERA_BIN"] ?? findWorkspaceBinary();
    execFileSync(binary, ["build", "--build", "site", "--emit", "json", "--color", "never"], {
      cwd: siteDir,
      stdio: "pipe",
    });
    const jsonDir = path.join(siteDir, ".ascribe", "build", "site", "json");
    const documents = jsonFiles(jsonDir).map(
      (file) =>
        JSON.parse(readFileSync(file, "utf8")) as {
          path: string;
          route: string;
          headings: { id: string }[];
        },
    );
    expect(documents.map((d) => d.path).sort()).toEqual([
      "Guides/My Setup.md",
      "Reference/Options.md",
      "index.md",
    ]);
    for (const document of documents) {
      // The route is a URL the site serves, and the built HTML has exactly the ids, in order.
      const page = await open(document.route);
      const ids = await page
        .locator("article :is(h1, h2, h3, h4, h5, h6)")
        .evaluateAll((elements) => elements.map((e) => e.id));
      expect(ids, document.path).toEqual(document.headings.map((h) => h.id));
      // Astro's table of contents (the layout's nav) recorded the same ids.
      const toc = await page
        .locator("nav[aria-label='On this page'] a")
        .evaluateAll((links) => links.map((a) => a.getAttribute("href")));
      expect(toc, document.path).toEqual(document.headings.map((h) => `#${h.id}`));
      await page.close();
    }
  });
});

function findWorkspaceBinary(): string {
  for (let dir = siteDir; dir !== path.dirname(dir); dir = path.dirname(dir)) {
    for (const profile of ["debug", "release"]) {
      const candidate = path.join(dir, "target", profile, "ascribe");
      try {
        readFileSync(candidate);
        return candidate;
      } catch {
        // Try the next.
      }
    }
  }
  throw new Error("no ascribe binary: run `cargo build -p tessera-cli` or set TESSERA_BIN");
}

function jsonFiles(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) =>
    entry.isDirectory()
      ? jsonFiles(path.join(dir, entry.name))
      : entry.name.endsWith(".json")
        ? [path.join(dir, entry.name)]
        : [],
  );
}
