// The integration's contract with Astro, beyond the happy path: it fails the
// build on Tessera errors and on routing that disagrees with ascribe.toml, it
// serves `_tessera/files/` in the dev server, and its markdown plugin works
// under Astro's other processor too.
import { readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { afterAll, describe, expect, it } from "vitest";
import {
  BASE,
  buildSite,
  copySite,
  launchChromium,
  serveDev,
  servePreview,
  siteDir,
} from "./harness.js";

afterAll(async () => {
  await rm(path.join(siteDir, ".e2e-tmp"), { recursive: true, force: true });
});

async function edit(file: string, change: (text: string) => string): Promise<void> {
  await writeFile(file, change(await readFile(file, "utf8")));
}

describe("the integration", () => {
  it("fails the Astro build when Tessera reports errors", async () => {
    const root = await copySite("errors");
    await edit(path.join(root, "content", "Guides", "My Setup.md"), (text) =>
      text.replace("(../index.md)", "(../missing.md)"),
    );
    const failure = await buildSite(root).then(
      () => undefined,
      (error: unknown) => error,
    );
    expect(String(failure)).toContain("link-target-missing");
    expect(String(failure)).toContain("missing.md");
  });

  it("fails the Astro build when ascribe.toml and astro.config disagree on routing", async () => {
    const root = await copySite("routing");
    await edit(path.join(root, "astro.config.mjs"), (text) =>
      text
        .replace('trailingSlash: "never"', 'trailingSlash: "always"')
        .replace('base: "/docs"', 'base: "/manual"'),
    );
    const failure = await buildSite(root).then(
      () => undefined,
      (error: unknown) => error,
    );
    expect(String(failure)).toContain('base-path is "/docs/", but Astro\'s `base` is "/manual/"');
    expect(String(failure)).toContain('trailing-slash is "never"');
  });

  it("builds a project with no [builds] table (the implicit `site` build), and reports an unknown build", async () => {
    const root = await copySite("implicit");
    await edit(path.join(root, "ascribe.toml"), (text) =>
      text.replace(/\[builds\.site\][^[]*/, "").replace(/\[editor\][^[]*/, ""),
    );
    await buildSite(root);
    await edit(path.join(root, "astro.config.mjs"), (text) =>
      text.replace('build: "site"', 'build: "nope"'),
    );
    const failure = await buildSite(root).then(
      () => undefined,
      (error: unknown) => error,
    );
    expect(String(failure)).toContain("nope");
  });

  it("routes agree under a root base path with trailing slashes", async () => {
    const root = await copySite("always");
    await edit(path.join(root, "ascribe.toml"), (text) =>
      text
        .replace('base-path = "/docs/"', 'base-path = "/"')
        .replace('trailing-slash = "never"', 'trailing-slash = "always"'),
    );
    await edit(path.join(root, "astro.config.mjs"), (text) =>
      text
        .replace('base: "/docs"', 'base: "/"')
        .replace('trailingSlash: "never"', 'trailingSlash: "always"'),
    );
    await buildSite(root);
    const server = await servePreview(root);
    const browser = await launchChromium();
    try {
      const page = await browser.newPage();
      await page.goto(`${server.origin}/`);
      await page.getByRole("link", { name: "setting up Loom" }).click();
      await page.waitForURL(`${server.origin}/guides/my-setup/`);
      await page.getByRole("link", { name: "documentation home" }).click();
      await page.waitForURL(`${server.origin}/`);
      expect(await page.locator("main > h1").textContent()).toBe("Loom documentation");
      const file = await page.request.get(`${server.origin}/_tessera/files/downloads/loom.yaml`);
      expect(file.status()).toBe(200);
    } finally {
      await browser.close();
      await server.stop();
    }
  });

  it("serves _tessera/files/ and the markdown plugin's ids in the dev server", async () => {
    const server = await serveDev();
    try {
      const file = await fetch(`${server.origin}${BASE}/_tessera/files/downloads/loom.yaml`);
      expect(file.status).toBe(200);
      expect(await file.text()).toBe("weave:\n  strands: 4\n");
      const missing = await fetch(`${server.origin}${BASE}/_tessera/files/downloads/nope.yaml`);
      expect(missing.status).toBe(404);
      // Only under the base path, as in the build.
      const unprefixed = await fetch(`${server.origin}/_tessera/files/downloads/loom.yaml`);
      expect(unprefixed.status).toBe(404);
      const html = await (await fetch(`${server.origin}${BASE}/guides/my-setup`)).text();
      expect(html).toContain('id="weave-config"');
    } finally {
      await server.stop();
    }
  });

  it("applies markers under the unified() processor too", async () => {
    const root = await copySite("unified");
    await edit(path.join(root, "astro.config.mjs"), (text) =>
      text
        .replace(
          'import tessera from "@tessera/astro";',
          'import { unified } from "@astrojs/markdown-remark";\nimport tessera from "@tessera/astro";',
        )
        .replace("integrations:", "markdown: { processor: unified() },\n  integrations:"),
    );
    await buildSite(root);
    const server = await servePreview(root);
    const browser = await launchChromium();
    try {
      const page = await browser.newPage();
      await page.goto(`${server.origin}${BASE}/guides/my-setup`);
      const ids = await page
        .locator("article :is(h1, h2, h3, h4, h5, h6)")
        .evaluateAll((elements) => elements.map((e) => e.id));
      expect(ids).toEqual(["requirements", "install", "connect", "weave-config", "streaming"]);
      const toc = await page
        .locator("nav[aria-label='On this page'] a")
        .evaluateAll((links) => links.map((a) => a.getAttribute("href")));
      expect(toc).toEqual(ids.map((id) => `#${id}`));
      const image = page.getByRole("img", { name: "Checklist of requirements" });
      expect(await image.getAttribute("src")).toMatch(/\/_astro\/requirements\.[\w-]+\.webp$/);
      expect(await image.getAttribute("width")).toBe("300");
      expect(await page.locator("tessera-attributes").count()).toBe(0);
    } finally {
      await browser.close();
      await server.stop();
    }
  });
});
