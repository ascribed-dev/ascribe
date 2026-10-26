// The landing page's demonstration (src/demo.ts): the source it shows and the
// page it draws are one file, so changing the file changes both sides.
import { readFile } from "node:fs/promises";
import path from "node:path";
import { beforeAll, expect, test } from "vitest";
import { splitAtResults } from "../src/demo.ts";
import { distDir, requireBuild, siteDir } from "./built.ts";

beforeAll(requireBuild);

const homeDir = path.join(siteDir, "home");
const demoContent = path.join(siteDir, "demo", "content");

/** Each demonstration in the landing page's source: the page it draws, and the file its snippet reads. */
async function demonstrations(): Promise<{ page: string; snippet: string }[]> {
  const index = await readFile(path.join(homeDir, "content", "index.md"), "utf8");
  return Array.from(
    index.matchAll(/^@source-and-result \{page=([^}]+)\}:\n@snippet[^\n]*: code:(\S+)$/gm),
    ([, page = "", snippet = ""]) => ({ page, snippet }),
  );
}

/** An element's text, without its tags and with HTML's entities decoded. */
function text(html: string): string {
  return html
    .replace(/<[^>]+>/g, "")
    .replace(/&#x([0-9a-f]+);/gi, (_, hex: string) => String.fromCodePoint(parseInt(hex, 16)))
    .replace(/&#(\d+);/g, (_, dec: string) => String.fromCodePoint(Number(dec)))
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&amp;", "&");
}

test("the source shown and the page drawn are the same file", async () => {
  const found = await demonstrations();
  expect(found.length).toBe(1);
  for (const { page, snippet } of found) {
    // `code:` is the repository's root (home/ascribe.toml's [sources.code]).
    expect(path.resolve(siteDir, "..", snippet)).toBe(path.join(demoContent, `${page}.md`));
  }
});

test("the built landing page shows the file's text, and the page built from it beside it", async () => {
  const html = await readFile(path.join(distDir, "index.html"), "utf8");
  for (const { page } of await demonstrations()) {
    const source = await readFile(path.join(demoContent, `${page}.md`), "utf8");
    const opener = html.indexOf(`<source-and-result page="${page}">`);
    expect(opener).toBeGreaterThan(-1);
    const element = html.slice(opener, html.indexOf("</source-and-result>", opener));
    const code = /<pre[^>]*><code[^>]*>([\s\S]*?)<\/code><\/pre>/.exec(element)?.[1] ?? "";
    expect(text(code).trim()).toBe(source.trim());
    // The page's title, from its frontmatter, and its first heading, both built.
    const title = /^title: (.+)$/m.exec(source)?.[1] ?? "";
    const heading = /^## (.+)$/m.exec(source)?.[1] ?? "";
    const result = element.slice(element.indexOf('class="result"'));
    expect(result).toContain(`<p class="result-title">${title}</p>`);
    expect(result).toMatch(new RegExp(`<h2 id="[^"]+">${RegExp.escape(heading)}</h2>`));
  }
});

test("the page's HTML is cut before each demonstration's end tag", () => {
  expect(splitAtResults("<p>a</p>")).toEqual([{ html: "<p>a</p>" }]);
  expect(
    splitAtResults('<p>a</p><source-and-result page="x"><pre>s</pre></source-and-result><p>b</p>'),
  ).toEqual([
    { html: '<p>a</p><source-and-result page="x"><pre>s</pre>' },
    { page: "x" },
    { html: "</source-and-result><p>b</p>" },
  ]);
  expect(() => splitAtResults('<source-and-result page="x">')).toThrow(/no end tag/);
});
