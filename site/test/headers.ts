// The response headers a host sends with each docs page's Markdown version,
// and the rules a built site's `_headers` is held to: each page with a
// Markdown version has its rule, a canonical `Link` to the page and
// `X-Robots-Tag` when the page isn't `listed`, and no other path has one.
// Read from a folder, so test/headers.test.ts runs it on dist/ and on a
// fixture.
import { readFile } from "node:fs/promises";
import path from "node:path";

/** A page's record in `_ascribe/pages.json` (output-layout contract §1.3), the parts read here. */
export interface PageRecord {
  canonical?: string;
  listing: "listed" | "unquoted" | "unlisted";
  markdown?: string;
}

/** The headers of each path in a `_headers` file, Netlify's format, with each name in lower case. */
export function parseHeaders(text: string): Map<string, Record<string, string>> {
  const paths = new Map<string, Record<string, string>>();
  let current: Record<string, string> | undefined;
  for (const line of text.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (trimmed === "" || trimmed.startsWith("#")) continue;
    if (!/^\s/.test(line)) {
      current = paths.get(trimmed) ?? {};
      paths.set(trimmed, current);
      continue;
    }
    const colon = trimmed.indexOf(":");
    if (current === undefined || colon === -1) continue;
    current[trimmed.slice(0, colon).trim().toLowerCase()] = trimmed.slice(colon + 1).trim();
  }
  return paths;
}

/** What a page's listing says to a crawler, as `X-Robots-Tag`; none when it's `listed`. */
const ROBOTS: Record<PageRecord["listing"], string | undefined> = {
  listed: undefined,
  unquoted: "nosnippet",
  unlisted: "noindex",
};

/** The headers each page's Markdown version needs, by its path, names in lower case. */
export function expectedHeaders(
  pages: Record<string, PageRecord>,
): Map<string, Record<string, string>> {
  const expected = new Map<string, Record<string, string>>();
  for (const record of Object.values(pages)) {
    if (record.markdown === undefined) continue;
    const headers: Record<string, string> = {};
    if (record.canonical !== undefined) headers["link"] = `<${record.canonical}>; rel="canonical"`;
    const robots = ROBOTS[record.listing];
    if (robots !== undefined) headers["x-robots-tag"] = robots;
    expected.set(record.markdown, headers);
  }
  return expected;
}

/**
 * What's wrong with a `_headers` file for the pages in a `pages.json`: each
 * page's Markdown version without its rule, or with a header that isn't
 * the page's, and each path with a rule that isn't a page's Markdown
 * version. Empty when the file is right.
 */
export function headerProblems(text: string, pages: Record<string, PageRecord>): string[] {
  const problems: string[] = [];
  const given = parseHeaders(text);
  const expected = expectedHeaders(pages);
  for (const [file, headers] of expected) {
    const has = given.get(file);
    if (has === undefined) {
      problems.push(`${file}: no rule`);
      continue;
    }
    for (const [name, value] of Object.entries(headers)) {
      if (has[name] === undefined) problems.push(`${file}: no ${name}`);
      else if (has[name] !== value) problems.push(`${file}: ${name} is ${has[name]}, not ${value}`);
    }
    for (const name of Object.keys(has)) {
      if (!(name in headers)) problems.push(`${file}: ${name}, which its page doesn't give`);
    }
  }
  for (const file of given.keys()) {
    if (!expected.has(file))
      problems.push(`${file}: a rule for a path that isn't a Markdown version`);
  }
  return problems;
}

/** `_ascribe/pages.json`'s records, by route, in the site output at `dir`. */
export async function readPages(dir: string): Promise<Record<string, PageRecord>> {
  const text = await readFile(path.join(dir, "_ascribe", "pages.json"), "utf8");
  return (JSON.parse(text) as { pages: Record<string, PageRecord> }).pages;
}
