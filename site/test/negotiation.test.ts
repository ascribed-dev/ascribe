// The content negotiation edge function, netlify/edge-functions/markdown.ts:
// which requests it answers with Markdown, with the headers of the Markdown it
// answers with, and that every page of the built site has that Markdown.
import { readFile } from "node:fs/promises";
import { afterAll, afterEach, beforeAll, describe, expect, test, vi } from "vitest";
import markdown, { markdownPaths, prefersMarkdown } from "../netlify/edge-functions/markdown.ts";
import { distDir, fileFor, filesUnder, forwardingPages, requireBuild, serve } from "./built.ts";

describe("which requests ask for Markdown", () => {
  test.each([
    ["text/markdown", true],
    ["text/markdown;q=0.9, text/html;q=0.8", true],
    ["text/markdown, text/html", true],
    ["text/x-markdown", true],
    ["Text/Markdown", true],
    // A browser's.
    ["text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8", false],
    ["*/*", false],
    ["text/html, text/markdown;q=0.5", false],
    ["text/markdown;q=0", false],
    ["", false],
  ])("%j: %s", (accept, expected) => {
    expect(prefersMarkdown(accept)).toBe(expected);
  });

  test("no Accept header asks for HTML", () => {
    expect(prefersMarkdown(null)).toBe(false);
  });
});

test("a page's Markdown is at its address with .md, or its index.md", () => {
  expect(markdownPaths("/")).toEqual(["/index.md"]);
  expect(markdownPaths("/docs/")).toEqual(["/docs.md", "/docs/index.md"]);
  expect(markdownPaths("/docs/astro/")).toEqual(["/docs/astro.md", "/docs/astro/index.md"]);
  expect(markdownPaths("/guides/astro")).toEqual([]);
  expect(markdownPaths("/llms.txt")).toEqual([]);
});

describe("the Markdown version's headers", () => {
  afterEach(() => vi.restoreAllMocks());

  /** The function's answer to `pathname` asked for as Markdown, its `.md` answering with `headers`. */
  async function answerWith(pathname: string, headers: Record<string, string>): Promise<Response> {
    vi.spyOn(globalThis, "fetch").mockImplementation((input) => {
      const url = new URL(input instanceof Request ? input.url : String(input));
      return Promise.resolve(
        url.pathname === `${pathname.slice(0, -1)}.md`
          ? new Response("# A page\n", { headers: { "content-type": "text/markdown", ...headers } })
          : new Response("not found", { status: 404 }),
      );
    });
    const request = new Request(new URL(pathname, "https://ascribed-dev.com"), {
      headers: { accept: "text/markdown" },
    });
    return markdown(request, { next: () => Promise.resolve(new Response("the page")) });
  }

  test("an unlisted page asked for as Markdown says noindex, and names its page as canonical", async () => {
    const link = '<https://ascribed-dev.com/docs/internal/>; rel="canonical"';
    const response = await answerWith("/docs/internal/", { "x-robots-tag": "noindex", link });
    expect(await response.text()).toBe("# A page\n");
    expect(response.headers.get("x-robots-tag")).toBe("noindex");
    expect(response.headers.get("link")).toBe(link);
  });

  test("a listed page asked for as Markdown says nothing of how it's listed", async () => {
    const response = await answerWith("/docs/astro/", {});
    expect(await response.text()).toBe("# A page\n");
    expect(response.headers.get("x-robots-tag")).toBeNull();
  });
});

describe("the built site", () => {
  let site: { url: string; close: () => Promise<void> };
  beforeAll(async () => {
    requireBuild();
    site = await serve();
  });
  afterAll(() => site.close());

  /** The edge function's answer, with Netlify's own response served from dist/. */
  async function answer(pathname: string, accept: string | null): Promise<Response> {
    const url = new URL(pathname, site.url);
    const headers = accept === null ? undefined : { accept };
    const request = new Request(url, { headers });
    return markdown(request, { next: () => fetch(url) });
  }

  test("every docs page has the Markdown the function answers with", async () => {
    // A permalink's page only forwards, and gets no Markdown; nor do the
    // site's own pages, outside the docs (home/ascribe.toml's `agents = false`).
    const forwarding = await forwardingPages();
    const pages = (await filesUnder(distDir))
      .filter((file) => file.startsWith("docs/") && file.endsWith("index.html"))
      .map((file) => `/${file.slice(0, -"index.html".length)}`)
      .filter((page) => !forwarding.includes(page));
    expect(pages).toContain("/docs/");
    const missing: string[] = [];
    for (const page of pages) {
      const found = [];
      for (const target of markdownPaths(page)) {
        if ((await fileFor(target)) !== undefined) found.push(target);
      }
      if (found.length === 0) missing.push(page);
    }
    expect(missing).toEqual([]);
  });

  test("a page asked for as Markdown is its .md file, typed as Markdown", async () => {
    const response = await answer("/docs/astro/", "text/markdown");
    expect(response.status).toBe(200);
    expect(response.headers.get("content-type")).toBe("text/markdown; charset=utf-8");
    expect(response.headers.get("vary")).toBe("Accept");
    expect(await response.text()).toBe(await readFile(`${distDir}/docs/astro.md`, "utf8"));
  });

  test("the docs' root page asked for as Markdown is its index.md", async () => {
    const response = await answer("/docs/", "text/markdown");
    expect(response.status).toBe(200);
    expect(response.headers.get("content-type")).toBe("text/markdown; charset=utf-8");
    expect(response.headers.get("content-location")).toBe("/docs/index.md");
    expect(await response.text()).toBe(await readFile(`${distDir}/docs/index.md`, "utf8"));
  });

  test("the changelog asked for as Markdown is /changelog.md, which its head names", async () => {
    const response = await answer("/changelog/", "text/markdown");
    expect(response.status).toBe(200);
    expect(response.headers.get("content-type")).toBe("text/markdown; charset=utf-8");
    expect(response.headers.get("content-location")).toBe("/changelog.md");
    expect(await response.text()).toBe(await readFile(`${distDir}/changelog.md`, "utf8"));
    const page = await (await answer("/changelog/", "text/html")).text();
    expect(page).toContain('<link rel="alternate" type="text/markdown" href="/changelog.md">');
  });

  test("the changelog's Markdown has each release the page has, anchored as on the page", async () => {
    const page = await readFile(`${distDir}/changelog/index.html`, "utf8");
    const text = await readFile(`${distDir}/changelog.md`, "utf8");
    const releases = Array.from(page.matchAll(/<h2 id="([^"]*)"/g), (match) => match[1] ?? "");
    expect(releases).toContain("v0.4.4");
    expect(
      Array.from(text.matchAll(/^## <a id="([^"]*)"><\/a>/gm), (match) => match[1] ?? ""),
    ).toEqual(releases);
    // Every link, outside code, is one a reader of the file alone can follow.
    const prose = text.replace(/(`+)[\s\S]*?\1/g, "");
    expect(prose).toMatch(/\]\(https:\/\/[^)]+\/docs\//);
    expect(prose).not.toMatch(/\]\((?!https?:|#|mailto:)/);
  });

  test("the docs' llms.txt lists the changelog's Markdown, under Optional", async () => {
    const llms = await readFile(`${distDir}/docs/llms.txt`, "utf8");
    const optional = llms.slice(llms.indexOf("\n## Optional\n"));
    expect(optional).toMatch(
      /^\n## Optional\n\n- \[Changelog\]\(https?:\/\/[^)]+\/changelog\.md\): /,
    );
  });

  test("a page outside the docs asked for as Markdown is its page", async () => {
    const response = await answer("/", "text/markdown");
    expect(response.status).toBe(200);
    expect(response.headers.get("content-type")).toMatch(/^text\/html/);
    expect(await response.text()).toContain("<hero-banner");
  });

  test("a browser gets the page, marked as varying by Accept", async () => {
    const response = await answer("/docs/astro/", "text/html,*/*;q=0.8");
    expect(response.headers.get("content-type")).toMatch(/^text\/html/);
    expect(response.headers.get("vary")).toBe("Accept");
    expect(await response.text()).toMatch(/^<!DOCTYPE html>/i);
  });

  test("a permalink asked for as Markdown is its section, from its .md", async () => {
    const response = await answer("/go/ASC036/", "text/markdown");
    expect(response.status).toBe(200);
    expect(response.headers.get("content-type")).toBe("text/markdown; charset=utf-8");
    expect(response.headers.get("content-location")).toBe("/go/ASC036.md");
    const text = await response.text();
    expect(text).toBe(await readFile(`${distDir}/go/ASC036.md`, "utf8"));
    // The section alone, under its heading, not the whole page.
    expect(text).toMatch(/^# /);
    expect(text.length).toBeLessThan(10_000);
  });

  test("a permalink's forwarding page names its section's Markdown and JSON", async () => {
    const response = await answer("/go/ASC036/", "text/html");
    const page = await response.text();
    expect(page).toContain('<link rel="alternate" type="text/markdown" href="/go/ASC036.md">');
    expect(page).toContain('<link rel="alternate" type="application/json" href="/go/ASC036.json">');
  });

  test("a missing page asked for as Markdown is still the 404 page", async () => {
    const response = await answer("/no-such-page/", "text/markdown");
    expect(response.status).toBe(404);
    expect(response.headers.get("content-type")).toMatch(/^text\/html/);
  });
});
