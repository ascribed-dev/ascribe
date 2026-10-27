// The archive of `examples/quill`'s `self-managed 3.4`, opened from a disk
// in Chromium with the network off: every page loads with nothing from
// outside the folder and no script but the search's, every link in it leads
// to a file in it, printing a page hides its sidebar, and the search box
// finds pages and headings, or is absent with scripts off. axe finds nothing
// on any page in either color scheme.
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";
import { resolveBinary } from "@ascribed/cli/binary";
import type { Browser, Page } from "playwright-core";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { launchChromium } from "./harness.js";

const axeSource = readFileSync(createRequire(import.meta.url).resolve("axe-core/axe.min.js"), "utf8");

const quill = fileURLToPath(new URL("../../../quill", import.meta.url));

let browser: Browser;
let folder: string;
/** Every request a page made outside the folder, which none should. */
const outside: string[] = [];

beforeAll(async () => {
  folder = await mkdtemp(path.join(tmpdir(), "ascribe-archive-"));
  const binary = process.env["ASCRIBE_BIN"] ?? resolveBinary();
  execFileSync(
    binary,
    ["versions", "export", "self-managed 3.4", "--to", path.join(folder, "3.4"), "--current", "https://docs.quill.dev/"],
    { cwd: quill, stdio: "pipe" },
  );
  folder = path.join(folder, "3.4");
  browser = await launchChromium();
}, 120_000);

afterAll(async () => {
  await browser?.close();
  await rm(path.dirname(folder), { recursive: true, force: true });
});

/** Each HTML file of the archive, from its folder. */
function pages(): string[] {
  const found: string[] = [];
  const walk = (dir: string): void => {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const at = path.join(dir, entry.name);
      if (entry.isDirectory()) walk(at);
      else if (entry.name.endsWith(".html")) found.push(path.relative(folder, at));
    }
  };
  walk(folder);
  return found.sort();
}

/** A page of the archive, opened from the disk, with every request outside the folder refused. */
async function open(
  file: string,
  media: "screen" | "print" = "screen",
  javaScriptEnabled = true,
  colorScheme: "light" | "dark" = "light",
): Promise<Page> {
  // The page's policy would refuse axe's inline script, which only the
  // audit adds.
  const context = await browser.newContext({ offline: true, javaScriptEnabled, colorScheme, bypassCSP: true });
  const page = await context.newPage();
  const inside = pathToFileURL(folder).href;
  await page.route("**/*", (route) => {
    const url = route.request().url();
    if (url.startsWith(`${inside}/`)) return route.continue();
    outside.push(`${file}: ${url}`);
    return route.abort();
  });
  await page.emulateMedia({ media });
  await page.goto(pathToFileURL(path.join(folder, file)).href);
  return page;
}

describe("an archive opened from a disk", () => {
  it("loads every page with its stylesheet, nothing from outside, and no script but the search's", async () => {
    expect(pages()).toContain("index.html");
    for (const file of pages()) {
      const scripts = readFileSync(path.join(folder, file), "utf8").match(/<script[^>]*>/g) ?? [];
      for (const script of scripts) expect(script, file).toMatch(/^<script src="(\.\/|(\.\.\/)+)search\.js" /);
      expect(scripts.length, file).toBe(file === "all.html" ? 0 : 1);
      const page = await open(file);
      expect(await page.locator('meta[name="robots"]').getAttribute("content"), file).toBe("noindex");
      await expect(page.locator(".archive-statement").innerText()).resolves.toContain(
        "Documentation for self-managed 3.4.",
      );
      // The stylesheet applied: the element library's and the archive's.
      const display = await page.locator(".archive-layout, .archive-all").first().evaluate((e) => getComputedStyle(e).display);
      expect(display, file).not.toBe("inline");
      await page.context().close();
    }
    expect(outside).toEqual([]);
  });

  it("leads every link and image to a file in the folder", async () => {
    for (const file of pages()) {
      const page = await open(file);
      const targets = await page.evaluate(() =>
        [...document.querySelectorAll<HTMLAnchorElement | HTMLImageElement>("a[href], img[src]")].map((e) =>
          e instanceof HTMLAnchorElement ? e.href : e.src,
        ),
      );
      for (const target of targets.filter((t) => t.startsWith("file:"))) {
        const at = fileURLToPath(target.split("#")[0] ?? target);
        expect(at.startsWith(folder), `${file} → ${target}`).toBe(true);
        expect(existsSync(at), `${file} → ${target}`).toBe(true);
      }
      await page.context().close();
    }
  });

  it("reaches every page from the contents page and from the sidebar of any other", async () => {
    const page = await open("upgrade.html");
    const listed = await page
      .getByRole("navigation", { name: "Pages" })
      .getByRole("link")
      .evaluateAll((links) => links.map((a) => (a as HTMLAnchorElement).getAttribute("href")));
    // The contents page first, then every page.
    expect(listed[0]).toBe("./index.html");
    expect([...listed].sort((a, b) => String(a).localeCompare(String(b), "en"))).toEqual(
      pages()
        .filter((p) => p !== "all.html")
        .map((p) => `./${p}`)
        .sort((a, b) => a.localeCompare(b, "en")),
    );
    await page.context().close();
  });

  it("hides the sidebar in print and keeps the statement", async () => {
    const page = await open("install-agent.html", "print");
    await expect(page.locator(".archive-sidebar").isVisible()).resolves.toBe(false);
    await expect(page.locator(".archive-current").isVisible()).resolves.toBe(false);
    await expect(page.locator(".archive-statement").isVisible()).resolves.toBe(true);
    await page.context().close();
  });

  it("leaves the search box out of print", async () => {
    const page = await open("install-agent.html", "print");
    await expect(page.locator(".archive-search").count()).resolves.toBe(1);
    await expect(page.locator(".archive-search").isVisible()).resolves.toBe(false);
    await page.context().close();
  });

  it("shows each tab under its label without a script", async () => {
    const page = await open("install-agent.html");
    const tabs = page.locator("ascribe-tab");
    expect(await tabs.count()).toBeGreaterThan(1);
    for (const tab of await tabs.all()) await expect(tab.isVisible()).resolves.toBe(true);
    await page.context().close();
  });
});

describe("searching an archive opened from a disk", () => {
  /** The results listed for `query`, typed into the box of `file`. */
  async function searchFor(page: Page, query: string): Promise<string[]> {
    await page.getByRole("searchbox", { name: "Search this version" }).fill(query);
    await expect
      .poll(() => page.getByRole("status").innerText())
      .toMatch(/result|Nothing found/);
    return page
      .locator(".archive-search-results a")
      .evaluateAll((links) => links.map((a) => (a as HTMLAnchorElement).getAttribute("href") ?? ""));
  }

  it("finds a heading by a word in it, and the result opens the page there", async () => {
    const page = await open("keys.html");
    const found = await searchFor(page, "troubleshooting");
    expect(found[0]).toBe("./install-agent.html#troubleshooting");
    await expect(page.getByRole("status").innerText()).resolves.toMatch(/^\d+ results? for “troubleshooting”\.$/);
    await page.locator(".archive-search-results a").first().click();
    await page.waitForURL(/install-agent\.html#troubleshooting$/);
    await expect(page.locator("#troubleshooting").isVisible()).resolves.toBe(true);
    await page.context().close();
  });

  it("finds a word in a code block", async () => {
    const page = await open("upgrade.html");
    expect(await searchFor(page, "--version")).toEqual(["./install-agent.html#install-agent"]);
    await expect(page.locator(".archive-search-results mark").first().innerText()).resolves.toBe("--version");
    await page.context().close();
  });

  it("says when nothing is found", async () => {
    const page = await open("index.html");
    expect(await searchFor(page, "zeppelin")).toEqual([]);
    await expect(page.getByRole("status").innerText()).resolves.toBe("Nothing found for “zeppelin”.");
    expect(outside).toEqual([]);
    await page.context().close();
  });

  it("leaves the box out with scripts off, and every page whole", async () => {
    for (const file of pages().filter((p) => p !== "all.html")) {
      const page = await open(file, "screen", false);
      await expect(page.locator(".archive-search").count(), file).resolves.toBe(0);
      await expect(page.getByRole("searchbox").count(), file).resolves.toBe(0);
      await expect(page.locator("main h1").isVisible(), file).resolves.toBe(true);
      expect(await page.getByRole("navigation", { name: "Pages" }).getByRole("link").count(), file).toBeGreaterThan(1);
      await page.context().close();
    }
  });
});

describe("an archive's accessibility", () => {
  /** What axe finds on `page`: WCAG 2.1 A and AA, and its best practice. */
  async function audit(page: Page): Promise<unknown[]> {
    await page.addScriptTag({ content: axeSource });
    return page.evaluate(async () => {
      const axe = (
        window as unknown as {
          axe: {
            run: (
              context: Document,
              options: { runOnly: string[] },
            ) => Promise<{ violations: { id: string; nodes: { target: unknown }[] }[] }>;
          };
        }
      ).axe;
      const { violations } = await axe.run(document, {
        runOnly: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "best-practice"],
      });
      return violations.map((v) => ({ id: v.id, targets: v.nodes.map((n) => n.target) }));
    });
  }

  for (const scheme of ["light", "dark"] as const) {
    it(`finds nothing on any page, the contents page and all.html among them, in ${scheme}`, async () => {
      for (const file of pages()) {
        const page = await open(file, "screen", true, scheme);
        expect(await page.locator("html").getAttribute("lang"), file).toBe("en");
        expect(await audit(page), `${file}, ${scheme}`).toEqual([]);
        await page.context().close();
      }
    });
  }

  it("starts each page with a skip link to its main, shown when it has focus", async () => {
    for (const file of pages()) {
      const page = await open(file);
      const skip = page.getByRole("link", { name: "Skip to content" });
      await page.keyboard.press("Tab");
      await expect(skip.evaluate((a) => a === document.activeElement), file).resolves.toBe(true);
      await expect(skip.isVisible(), file).resolves.toBe(true);
      const box = await skip.boundingBox();
      expect(box !== null && box.y >= 0, file).toBe(true);
      expect(await skip.getAttribute("href"), file).toBe(`#${await page.locator("main").getAttribute("id")}`);
      await page.keyboard.press("Enter");
      await page.waitForURL(/#archive-main$/);
      // The next Tab goes on from the page, past the sidebar.
      await page.keyboard.press("Tab");
      const inMain = await page.evaluate(() => document.activeElement?.closest("main") !== null);
      expect(inMain, file).toBe(true);
      await page.context().close();
    }
  });
});
