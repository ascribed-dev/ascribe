// The content negotiation edge function, netlify/edge-functions/markdown.ts:
// which requests it answers with Markdown, and that every page of the built
// site has the Markdown it would answer with.
import { readFile } from "node:fs/promises";
import { afterAll, beforeAll, describe, expect, test } from "vitest";
import markdown, { markdownPath, prefersMarkdown } from "../netlify/edge-functions/markdown.ts";
import { distDir, fileFor, filesUnder, requireBuild, serve } from "./built.ts";

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

test("a page's Markdown is at its address with .md", () => {
  expect(markdownPath("/")).toBe("/index.md");
  expect(markdownPath("/guides/astro/")).toBe("/guides/astro.md");
  expect(markdownPath("/guides/astro")).toBeUndefined();
  expect(markdownPath("/llms.txt")).toBeUndefined();
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

  test("every page has the Markdown the function answers with", async () => {
    const pages = (await filesUnder(distDir))
      .filter((file) => file.endsWith("index.html"))
      .map((file) => `/${file.slice(0, -"index.html".length)}`);
    expect(pages).toContain("/");
    const missing: string[] = [];
    for (const page of pages) {
      const target = markdownPath(page);
      if (target === undefined || (await fileFor(target)) === undefined) missing.push(page);
    }
    expect(missing).toEqual([]);
  });

  test("a page asked for as Markdown is its .md file, typed as Markdown", async () => {
    const response = await answer("/guides/astro/", "text/markdown");
    expect(response.status).toBe(200);
    expect(response.headers.get("content-type")).toBe("text/markdown; charset=utf-8");
    expect(response.headers.get("vary")).toBe("Accept");
    expect(await response.text()).toBe(await readFile(`${distDir}/guides/astro.md`, "utf8"));
  });

  test("a browser gets the page, marked as varying by Accept", async () => {
    const response = await answer("/guides/astro/", "text/html,*/*;q=0.8");
    expect(response.headers.get("content-type")).toMatch(/^text\/html/);
    expect(response.headers.get("vary")).toBe("Accept");
    expect(await response.text()).toMatch(/^<!DOCTYPE html>/i);
  });

  test("a missing page asked for as Markdown is still the 404 page", async () => {
    const response = await answer("/no-such-page/", "text/markdown");
    expect(response.status).toBe(404);
    expect(response.headers.get("content-type")).toMatch(/^text\/html/);
  });
});
