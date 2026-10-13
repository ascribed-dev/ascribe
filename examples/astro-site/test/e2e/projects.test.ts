// A home page in front of the docs, and several projects in one site: a
// `[consumer] base-path` under Astro's `base`, and `ascribe({ projects })`,
// each built and served, and under `astro dev`, with the refusals the
// integration makes for what the projects share. Each test works on a copy of
// the site under `.e2e-tmp/`: Astro's `base` moves to `/`, the docs' route to
// `src/pages/docs/`, and the copy gains the pages and projects it needs.
import { readFile, rm } from "node:fs/promises";
import path from "node:path";
import type { Browser } from "playwright-core";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import {
  buildSite,
  edit,
  launchChromium,
  serveDev,
  servePreview,
  siteDir,
  untilRebuilt,
} from "./harness.js";
import { contentConfig, homeInFront, ONE_ENTRY, twoProjects, write } from "./sites.js";

let browser: Browser;

beforeAll(async () => {
  browser = await launchChromium();
});

afterAll(async () => {
  await browser?.close();
  await rm(path.join(siteDir, ".e2e-tmp"), { recursive: true, force: true });
});

/** Why `buildSite(root)` failed, or `undefined` when it didn't. */
function buildFailure(root: string): Promise<string | undefined> {
  return buildSite(root).then(
    () => undefined,
    (error: unknown) => String(error),
  );
}

/** Checks, in a browser, what a home page in front of the docs promises. */
async function checkHomeAndDocs(origin: string, home: string): Promise<void> {
  const page = await browser.newPage();
  try {
    expect((await page.goto(`${origin}/`))?.status()).toBe(200);
    expect(await page.locator("h1").first().textContent()).toBe(home);
    expect((await page.goto(`${origin}/docs/guides/my-setup`))?.status()).toBe(200);
    expect(await page.locator("main > h1").textContent()).toBe("Set up Loom");
    // The docs' links keep their base path.
    await page.getByRole("link", { name: "documentation home" }).click();
    await page.waitForURL(`${origin}/docs`);
    expect(await page.locator("main > h1").textContent()).toBe("Loom documentation");
    // A published file, at the docs' base path.
    const file = await page.request.get(`${origin}/docs/_ascribe/files/downloads/loom.yaml`);
    expect(file.status()).toBe(200);
    expect(await file.text()).toBe("weave:\n  strands: 4\n");
  } finally {
    await page.close();
  }
}

/** Follows a permalink's address in a browser, expecting to land at `to`. */
async function land(address: string, to: string): Promise<void> {
  const page = await browser.newPage();
  try {
    await page.goto(address);
    await page.waitForURL(to);
  } finally {
    await page.close();
  }
}

describe("a base path under Astro's base", () => {
  it("serves a home page at / and the docs under /docs/ in a build", async () => {
    const root = await homeInFront("home-build");
    await buildSite(root);
    // What agents read is at the docs' base path too.
    expect(await readFile(path.join(root, "dist", "docs", "llms.txt"), "utf8")).toContain(
      "# Loom documentation",
    );
    const server = await servePreview(root);
    try {
      await checkHomeAndDocs(server.origin, "Loom weaves");
      await land(
        `${server.origin}/docs/go/weave-config`,
        `${server.origin}/docs/guides/my-setup#weave-config`,
      );
      const llms = await fetch(`${server.origin}/docs/llms.txt`);
      expect(llms.status).toBe(200);
    } finally {
      await server.stop();
    }
  });

  it("serves them in astro dev, and writes the docs' address to .ascribe/dev.json", async () => {
    const root = await homeInFront("home-dev");
    const server = await serveDev(root);
    try {
      await checkHomeAndDocs(server.origin, "Loom weaves");
      await land(
        `${server.origin}/docs/go/weave-config`,
        `${server.origin}/docs/guides/my-setup#weave-config`,
      );
      const llms = await fetch(`${server.origin}/docs/llms.txt`);
      expect(llms.status).toBe(200);
      const devFile = JSON.parse(
        await readFile(path.join(root, ".ascribe", "dev.json"), "utf8"),
      ) as { url: string };
      expect(devFile.url).toMatch(/^http:\/\/[^/]+:\d+\/docs\/$/);
    } finally {
      await server.stop();
    }
  }, 90_000);
});

describe("several projects in one site", () => {
  it("builds two projects, each at its base path, with a permalink from each", async () => {
    const root = await twoProjects("two-build");
    await buildSite(root);
    const server = await servePreview(root);
    try {
      await checkHomeAndDocs(server.origin, "Loom");
      const why = await fetch(`${server.origin}/why`);
      expect(why.status).toBe(200);
      expect(await why.text()).toContain("Because strands belong together.");
      await land(
        `${server.origin}/go/weave-config`,
        `${server.origin}/docs/guides/my-setup#weave-config`,
      );
      await land(`${server.origin}/go/welcome`, `${server.origin}/#welcome`);
      // A retired permalink's page links to the root of the project that retired it.
      const removed = await (await fetch(`${server.origin}/go/legacy-sync`)).text();
      expect(removed).toContain('<a href="/docs">Go to the docs\' home page</a>');
      // The home project publishes nothing for agents; the docs do, at their base path.
      expect((await fetch(`${server.origin}/docs/llms.txt`)).status).toBe(200);
      expect((await fetch(`${server.origin}/llms.txt`)).status).toBe(404);
    } finally {
      await server.stop();
    }
  });

  it("serves both in astro dev, rebuilds the project a change is in, and writes each its dev.json", async () => {
    const root = await twoProjects("two-dev");
    const server = await serveDev(root);
    try {
      await checkHomeAndDocs(server.origin, "Loom");
      await land(
        `${server.origin}/go/weave-config`,
        `${server.origin}/docs/guides/my-setup#weave-config`,
      );
      await land(`${server.origin}/go/welcome`, `${server.origin}/#welcome`);
      const file = path.join(root, "home", "content", "why.md");
      await edit(file, (text) => text.replace("Because strands", "Because woven strands"));
      await untilRebuilt(file, async () =>
        (await (await fetch(`${server.origin}/why`)).text()).includes("Because woven strands"),
      );
      const devFile = (dir: string) =>
        readFile(path.join(dir, ".ascribe", "dev.json"), "utf8").then(
          (text) => (JSON.parse(text) as { url: string }).url,
        );
      expect(await devFile(root)).toMatch(/^http:\/\/[^/]+:\d+\/docs\/$/);
      expect(await devFile(path.join(root, "home"))).toMatch(/^http:\/\/[^/]+:\d+\/$/);
    } finally {
      await server.stop();
    }
  }, 90_000);

  it("builds two entries for one project with two builds", async () => {
    const root = await homeInFront("two-builds");
    await edit(path.join(root, "ascribe.toml"), (text) =>
      text.replace("[editor]", '[builds.all]\nvariants = "switch"\n\n[editor]'),
    );
    await write(root, {
      "src/content.config.ts": contentConfig("all"),
      "src/pages/all.astro": `---
import { getCollection, render } from "astro:content";
const entry = (await getCollection("all")).find((e) => e.id === "guides/my-setup");
if (entry === undefined) throw new Error("no guides/my-setup in the all collection");
const { Content } = await render(entry);
---
<html lang="en"><body><main><Content /></main></body></html>
`,
    });
    await edit(path.join(root, "astro.config.mjs"), (text) =>
      text.replace(
        ONE_ENTRY,
        'ascribe({ projects: [{ name: "docs", build: "site" }, { name: "all", build: "all" }] })',
      ),
    );
    await buildSite(root);
    const server = await servePreview(root);
    try {
      // The site build reduces the deployment group to its cloud arm; the other keeps both.
      const docs = await (await fetch(`${server.origin}/docs/guides/my-setup`)).text();
      expect(docs).toContain("Sign in to Loom Cloud");
      expect(docs).not.toContain("Point the CLI at your own server.");
      const all = await (await fetch(`${server.origin}/all`)).text();
      expect(all).toContain("Point the CLI at your own server.");
      await land(
        `${server.origin}/docs/go/weave-config`,
        `${server.origin}/docs/guides/my-setup#weave-config`,
      );
    } finally {
      await server.stop();
    }
  });
});

describe("what several projects can't share", () => {
  it("refuses two projects at one base path", async () => {
    const root = await twoProjects("refuse-base");
    await edit(path.join(root, "home", "ascribe.toml"), (text) =>
      text.replace('base-path = "/"', 'base-path = "/docs/"'),
    );
    expect(await buildFailure(root)).toContain(
      '"docs" and "home" both have [consumer] base-path "/docs/"',
    );
  });

  it("refuses a page of one project under another's base path", async () => {
    const root = await twoProjects("refuse-page");
    await write(root, { "home/content/docs/extra.md": "---\ntype: page\ntitle: Extra\n---\n\nIn the way.\n" });
    expect(await buildFailure(root)).toContain(
      'the page docs/extra.md of "home" is at /docs/extra/, under the base path of "docs", /docs/',
    );
  });

  it("refuses a permalink two projects declare", async () => {
    const root = await twoProjects("refuse-permalink");
    await edit(path.join(root, "home", "content", "index.md"), (text) =>
      text.replace("@id {permalink=true}: welcome", "@id {permalink=true}: weave-config"),
    );
    expect(await buildFailure(root)).toContain(
      '"docs" and "home" both declare the permalink "weave-config"',
    );
  });

  it("refuses an entry's name used twice", async () => {
    const root = await twoProjects("refuse-name");
    await edit(path.join(root, "astro.config.mjs"), (text) =>
      text.replace('{ name: "home", project: "home"', '{ name: "docs", project: "home"'),
    );
    expect(await buildFailure(root)).toContain('two entries are named "docs"');
  });
});
