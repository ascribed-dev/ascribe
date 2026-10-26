// The Astro end-to-end test: a real `astro build` of this site,
// then the built HTML checked in Chromium. Run by `pnpm test:e2e`; it needs a
// built `ascribe` binary (`cargo build -p ascribe-cli`, or ASCRIBE_BIN) and
// `pnpm --filter @ascribed/astro build` and `pnpm --filter @ascribed/elements build`.
import { execFileSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { resolveBinary } from "@ascribed/cli/binary";
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
      [`${BASE}/reference/options`, "loom.yaml options"],
    ] as const) {
      const page = await open(route);
      await expect(page.locator("main > h1").textContent()).resolves.toBe(title);
      await page.close();
    }
  });

  it("writes each page as the file a static host serves at its address, without a trailing slash", async () => {
    // A host serves `x/index.html` at `x/`, so `x` would answer with a redirect.
    const dist = path.join(siteDir, "dist");
    for (const file of ["index.html", "guides/my-setup.html", "reference/options.html"]) {
      expect(existsSync(path.join(dist, file)), file).toBe(true);
    }
    expect(existsSync(path.join(dist, "guides", "my-setup", "index.html"))).toBe(false);
    // What agents read and the linked files are beside the pages, at their own addresses.
    for (const file of ["llms.txt", "guides/my-setup.md", "_ascribe/files/downloads/loom.yaml"]) {
      expect(existsSync(path.join(dist, file)), file).toBe(true);
    }
  });

  it("shows a title's code spans in the heading, and the plain title in <title>", async () => {
    const page = await open(`${BASE}/reference/options`);
    await expect(page.locator("main > h1").innerHTML()).resolves.toBe("<code>loom.yaml</code> options");
    await expect(page.title()).resolves.toBe("loom.yaml options");
    await page.close();
  });

  it("follows links that only work if Ascribe's routes are Astro's", async () => {
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
    // The image loads lazily: wait for it rather than reading it once.
    await expect
      .poll(() =>
        image.evaluate((element: HTMLImageElement) => element.complete && element.naturalWidth > 0),
      )
      .toBe(true);
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
    // The image loads lazily: wait for it rather than reading it once.
    await expect
      .poll(() =>
        image.evaluate((element: HTMLImageElement) => element.complete && element.naturalWidth > 0),
      )
      .toBe(true);
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

  it("shows a code block's title above it", async () => {
    const page = await open(`${BASE}/reference/options`);
    const figure = page.locator("article figure.code-title");
    await expect(figure.locator("> figcaption").textContent()).resolves.toBe("loom.yaml");
    await expect(figure.locator("> pre").textContent()).resolves.toContain("threads: 4");
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

  it("forwards a permalink's address to its heading, from a page that isn't indexed", async () => {
    const page = await context.newPage();
    // `@id {permalink=true}: weave-config`, published under `go/` in the base path.
    await page.goto(`${origin}${BASE}/go/weave-config`);
    await page.waitForURL(`${origin}${BASE}/guides/my-setup#weave-config`);
    const heading = page.locator("h2#weave-config");
    await expect(heading.evaluate((element) => element.matches(":target"))).resolves.toBe(true);
    await page.close();

    // A refresh that needs no script, a link to the same place, and no indexing.
    const response = await fetch(`${origin}${BASE}/go/weave-config`);
    expect(response.status).toBe(200);
    const html = await response.text();
    const to = `${BASE}/guides/my-setup#weave-config`;
    expect(html).toContain(`<meta http-equiv="refresh" content="0;url=${to}">`);
    expect(html).toContain(`<a href="${to}"`);
    expect(html).toContain('<meta name="robots" content="noindex">');
    expect(html).toContain(`<link rel="canonical" href="https://docs.example.com${to}">`);
  });

  it("forwards a page's permalink to the top of the page", async () => {
    const page = await context.newPage();
    // `permalink: options` in the page's frontmatter.
    await page.goto(`${origin}${BASE}/go/options`);
    await page.waitForURL(`${origin}${BASE}/reference/options`);
    await expect(page.locator("main > h1").textContent()).resolves.toBe("loom.yaml options");
    await page.close();
  });

  it("forwards a permalink moved to another, and one moved to a page", async () => {
    const page = await context.newPage();
    // `"go:config" = "go:weave-config"` in `ascribe.addresses`.
    await page.goto(`${origin}${BASE}/go/config`);
    await page.waitForURL(`${origin}${BASE}/guides/my-setup#weave-config`);
    // `"go:old-options" = "Reference/Options.md"`.
    await page.goto(`${origin}${BASE}/go/old-options`);
    await page.waitForURL(`${origin}${BASE}/reference/options`);
    await page.close();
  });

  it("says why a retired permalink was removed", async () => {
    const page = await context.newPage();
    const response = await page.goto(`${origin}${BASE}/go/legacy-sync`);
    expect(response?.status()).toBe(200);
    await expect(page.locator("main > h1").textContent()).resolves.toBe("Removed from these docs");
    // The reason as the authors wrote it, as text.
    await expect(page.locator("main [data-reason]").textContent()).resolves.toBe(
      "Sync was removed in Loom 3.0, & nothing replaced it.",
    );
    await expect(
      page.getByRole("link", { name: "Go to the docs' home page" }).getAttribute("href"),
    ).resolves.toBe(BASE);
    expect(page.url()).toBe(`${origin}${BASE}/go/legacy-sync`);
    const html = await (await fetch(`${origin}${BASE}/go/legacy-sync`)).text();
    expect(html).toContain('<meta name="robots" content="noindex">');
    expect(html).not.toContain("http-equiv");
    await page.close();
  });

  it("forwards with scripts off", async () => {
    const offline = await browser.newContext({ javaScriptEnabled: false });
    const page = await offline.newPage();
    await page.goto(`${origin}${BASE}/go/weave-config?pm=pnpm`);
    await page.waitForURL(`${origin}${BASE}/guides/my-setup#weave-config`);
    await offline.close();
  });

  it("opens the tab a permalink's parameter names, and remembers it as a click does", async () => {
    const reader = await browser.newContext();
    const page = await reader.newPage();
    // `pm` is a dimension of the content model, and `pnpm` one of its values.
    await page.goto(`${origin}${BASE}/go/weave-config?pm=pnpm`);
    await page.waitForURL(`${origin}${BASE}/guides/my-setup#weave-config`);
    const group = page.locator('ascribe-tabs[sync="pm"]');
    await expect(group.getByRole("tab", { selected: true }).textContent()).resolves.toBe("pnpm");
    await expect(group.getByText("pnpm add -g @loom/cli").isVisible()).resolves.toBe(true);
    // The choice holds on the next page that has the group, with no parameter.
    await page.goto(`${origin}${BASE}/reference/options`);
    await page.goto(`${origin}${BASE}/guides/my-setup`);
    await expect(group.getByRole("tab", { selected: true }).textContent()).resolves.toBe("pnpm");
    await reader.close();
  });

  it("ignores parameters the docs don't know, as if the link had none", async () => {
    const reader = await browser.newContext();
    const page = await reader.newPage();
    // A value `pm` doesn't have, a dimension the model doesn't have, and a version.
    await page.goto(`${origin}${BASE}/go/weave-config?pm=bun&platform=macos&self-managed=3.4`);
    await page.waitForURL(`${origin}${BASE}/guides/my-setup#weave-config`);
    const group = page.locator('ascribe-tabs[sync="pm"]');
    await expect(group.getByRole("tab", { selected: true }).textContent()).resolves.toBe("npm");
    await expect(page.evaluate(() => localStorage.length)).resolves.toBe(0);
    await reader.close();
  });

  it("explains, on the not-found page, a permalink the docs don't have", async () => {
    const page = await context.newPage();
    const response = await page.goto(`${origin}${BASE}/go/rotate-keys`);
    expect(response?.status()).toBe(404);
    await expect(page.locator("main > h1").textContent()).resolves.toBe("Page not found");
    const note = page.locator("main section");
    await expect(note.isVisible()).resolves.toBe(true);
    await expect(note.locator("code").textContent()).resolves.toBe("rotate-keys");
    await expect(note.getByRole("link", { name: "home page" }).getAttribute("href")).resolves.toBe(
      BASE,
    );

    // Anywhere else, the not-found page says nothing about permalinks.
    for (const address of [`${BASE}/no/such/page`, `${BASE}/go/rotate-keys/more`]) {
      const other = await page.goto(`${origin}${address}`);
      expect(other?.status(), address).toBe(404);
      await expect(note.isVisible(), address).resolves.toBe(false);
    }
    await page.close();
  });

  it("leaves permalinks' addresses out of llms.txt, and serves each section as Markdown", async () => {
    const llms = await (await fetch(`${origin}${BASE}/llms.txt`)).text();
    expect(llms).toContain("/guides/my-setup.md");
    expect(llms).not.toContain("/go/");
    // The section a permalink names, under its title, for an agent to read in one fetch.
    const markdown = await fetch(`${origin}${BASE}/go/weave-config.md`);
    expect(markdown.status).toBe(200);
    expect(await markdown.text()).toMatch(/^# .+\n/);
  });

  it("gives every heading the id Ascribe validated, at the route Ascribe computed", async () => {
    // `ascribe build --emit json` records each page's route and heading ids.
    const binary = process.env["ASCRIBE_BIN"] ?? resolveBinary();
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
      "Guides/Teams/Share a loom.md",
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

/** The page documents under `dir`, leaving out `_ascribe/`, the output's own files. */
function jsonFiles(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) =>
    entry.isDirectory()
      ? entry.name === "_ascribe"
        ? []
        : jsonFiles(path.join(dir, entry.name))
      : entry.name.endsWith(".json")
        ? [path.join(dir, entry.name)]
        : [],
  );
}
