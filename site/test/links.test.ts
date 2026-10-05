// Every link and anchor inside the built site lands: on a file dist/ serves,
// and on an element with that id.
import { readFile } from "node:fs/promises";
import path from "node:path";
import { beforeAll, expect, test } from "vitest";
import { distDir, fileFor, filesUnder, requireBuild, siteAddress } from "./built.ts";

/** The site's address, which absolute links to the site start with. */
let SITE: string;

beforeAll(async () => {
  requireBuild();
  SITE = await siteAddress();
});

/** An attribute's values on the page's tags (not in its text, where `<` is escaped). */
function attributeValues(html: string, name: string): string[] {
  const attribute = new RegExp(`\\s${name}="([^"]*)"`, "g");
  return Array.from(html.matchAll(/<[a-zA-Z][^>]*>/g), ([tag]) =>
    Array.from(tag.matchAll(attribute), (match) => decodeEntities(match[1] ?? "")),
  ).flat();
}

function decodeEntities(value: string): string {
  return value
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&amp;", "&");
}

test("internal links and anchors resolve", async () => {
  const pages = (await filesUnder(distDir)).filter(
    (file) => file.endsWith(".html") && !file.startsWith("pagefind/"),
  );
  const ids = new Map<string, Set<string>>();
  async function idsOf(file: string): Promise<Set<string>> {
    let found = ids.get(file);
    if (found === undefined) {
      found = new Set(attributeValues(await readFile(file, "utf8"), "id"));
      ids.set(file, found);
    }
    return found;
  }

  const broken: string[] = [];
  let checked = 0;
  for (const page of pages) {
    const file = path.join(distDir, ...page.split("/"));
    const html = await readFile(file, "utf8");
    const pageUrl = new URL(`/${page.replace(/(^|\/)index\.html$/, "$1")}`, SITE);
    for (const link of [...attributeValues(html, "href"), ...attributeValues(html, "src")]) {
      const url = new URL(link, pageUrl);
      if (url.origin !== SITE || url.pathname.startsWith("/pagefind/")) continue;
      checked++;
      const target = await fileFor(url.pathname);
      if (target === undefined) {
        broken.push(`${page}: ${link} (no file)`);
        continue;
      }
      // Chromium reads a fragment as a percent-decoded id.
      const fragment = decodeURIComponent(url.hash.slice(1));
      if (fragment !== "" && target.endsWith(".html") && !(await idsOf(target)).has(fragment)) {
        broken.push(`${page}: ${link} (no element with id "${fragment}")`);
      }
    }
  }
  expect(broken).toEqual([]);
  expect(checked).toBeGreaterThan(100);
});
