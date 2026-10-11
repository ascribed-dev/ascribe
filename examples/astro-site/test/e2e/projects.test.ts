// A home page in front of the docs, and several projects in one site: a
// `[consumer] base-path` under Astro's `base`, and `ascribe({ projects })`,
// each built and served, and under `astro dev`, with the refusals the
// integration makes for what the projects share. Each test works on a copy of
// the site under `.e2e-tmp/`: Astro's `base` moves to `/`, the docs' route to
// `src/pages/docs/`, and the copy gains the pages and projects it needs.
import { mkdir, readFile, rename, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Browser } from "playwright-core";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import {
  buildSite,
  copySite,
  edit,
  launchChromium,
  serveDev,
  servePreview,
  siteDir,
  untilRebuilt,
} from "./harness.js";

let browser: Browser;

beforeAll(async () => {
  browser = await launchChromium();
});

afterAll(async () => {
  await browser?.close();
  await rm(path.join(siteDir, ".e2e-tmp"), { recursive: true, force: true });
});

/** A plain Astro home page at the site's root, linking to the docs. */
const HOME_PAGE = `---
---

<!doctype html>
<html lang="en">
  <head><meta charset="utf-8" /><title>Loom</title></head>
  <body><main><h1>Loom weaves</h1><a href="/docs">Read the docs</a></main></body>
</html>
`;

/**
 * A copy of the site with Astro's `base` at `/` and the docs still at
 * `/docs/`, their route under `src/pages/docs/`, and a home page at `/`.
 */
async function homeInFront(name: string): Promise<string> {
  const root = await copySite(name);
  await edit(path.join(root, "astro.config.mjs"), (text) =>
    text.replace('base: "/docs"', 'base: "/"'),
  );
  const pages = path.join(root, "src", "pages");
  await mkdir(path.join(pages, "docs"));
  await rename(path.join(pages, "[...slug].astro"), path.join(pages, "docs", "[...slug].astro"));
  await edit(path.join(pages, "docs", "[...slug].astro"), (text) =>
    text.replace('"../layouts/Docs.astro"', '"../../layouts/Docs.astro"'),
  );
  await writeFile(path.join(pages, "index.astro"), HOME_PAGE);
  return root;
}

/** Writes `files`, by path under `root`. */
async function write(root: string, files: Record<string, string>): Promise<void> {
  for (const [relative, text] of Object.entries(files)) {
    await mkdir(path.dirname(path.join(root, relative)), { recursive: true });
    await writeFile(path.join(root, relative), text);
  }
}

/** The second project, `home/`: the site's root, with a permalink of its own. */
const HOME_PROJECT = {
  "home/ascribe.toml": `spec = "0.1"

[project]
content-root = "content"

[consumer]
profile = "astro"
site = "https://docs.example.com"
base-path = "/"
trailing-slash = "never"
permalink-path = "/go/"

[types.page]
default = true

[types.page.frontmatter]
title = "string"
`,
  "home/content/index.md": `---
title: Loom
---

## Loom weaves
@id {permalink=true}: welcome

Loom weaves your strands together.
`,
  "home/content/why.md": `---
title: Why Loom
---

Because strands belong together.
`,
};

/** The route of the home collection's pages, at the site's root. */
const HOME_ROUTE = `---
import { getCollection, render } from "astro:content";

export async function getStaticPaths() {
  const entries = await getCollection("home");
  return entries.map((entry) => ({
    params: { slug: entry.id === "index" ? undefined : entry.id },
    props: { entry },
  }));
}

const { entry } = Astro.props;
const { Content } = await render(entry);
---

<!doctype html>
<html lang="en">
  <head><meta charset="utf-8" /><title>{entry.data.title}</title></head>
  <body><main><h1>{entry.data.title}</h1><Content /></main></body>
</html>
`;

/** The collections of two named entries, `docs` and `second`. */
function contentConfig(second: string): string {
  return `import { defineCollection } from "astro:content";
import { ascribeCollection } from "@ascribed/astro/content";
import { schema as docs } from "../.astro/integrations/_ascribed_astro/docs/schema.ts";
import { schema as second } from "../.astro/integrations/_ascribed_astro/${second}/schema.ts";

export const collections = {
  docs: defineCollection(ascribeCollection({ schema: docs, name: "docs" })),
  ${second}: defineCollection(ascribeCollection({ schema: second, name: "${second}" })),
};
`;
}

/**
 * A copy with two projects: the docs at `/docs/` and `home/` at `/`, each
 * with its permalinks at `/go/`.
 */
async function twoProjects(name: string): Promise<string> {
  const root = await homeInFront(name);
  await rm(path.join(root, "src", "pages", "index.astro"));
  await write(root, {
    ...HOME_PROJECT,
    "src/pages/[...slug].astro": HOME_ROUTE,
    "src/content.config.ts": contentConfig("home"),
  });
  await edit(path.join(root, "ascribe.toml"), (text) =>
    text.replace('trailing-slash = "never"', 'trailing-slash = "never"\npermalink-path = "/go/"'),
  );
  await edit(path.join(root, "astro.config.mjs"), (text) =>
    text.replace(
      'ascribe({ build: "site" })',
      `ascribe({
      projects: [
        { name: "docs", build: "site" },
        { name: "home", project: "home", build: "site" },
      ],
      review: "docs",
    })`,
    ),
  );
  return root;
}

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
      await land(`${server.origin}/go/weave-config`, `${server.origin}/docs/guides/my-setup#weave-config`);
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
      await land(`${server.origin}/go/weave-config`, `${server.origin}/docs/guides/my-setup#weave-config`);
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
        'ascribe({ build: "site" })',
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
    await write(root, { "home/content/docs/extra.md": "---\ntitle: Extra\n---\n\nIn the way.\n" });
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
