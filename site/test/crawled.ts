// What a crawler finds in a built site, and the rules it holds the site to:
// the sitemap lists every page that's meant to be found and nothing else, and
// a page that says `noindex` is in neither the sitemap nor `llms.txt`. Read
// from a folder, so test/crawlers.test.ts runs it on dist/ and on a fixture.
import { readFile } from "node:fs/promises";
import path from "node:path";
import { filesUnder } from "./built.ts";

/** A built page: its route from the host's root, and whether it says `noindex`. */
export interface CrawledPage {
  route: string;
  noindex: boolean;
}

/** What a crawler reads of a built site. */
export interface Crawled {
  /** Every page, `<route>/index.html`, but the not-found page, `404.html`. */
  pages: CrawledPage[];
  /** `sitemap.xml`'s addresses, absolute; `undefined` with no sitemap. */
  sitemap: string[] | undefined;
  /** `robots.txt`, or `undefined` with none. */
  robots: string | undefined;
  /** Every link in every `llms.txt`, as written. */
  llms: string[];
}

/** A `<meta name="robots">` that says `noindex`, its attributes in either order. */
const NOINDEX =
  /<meta\s+(?:name="robots"\s+content="[^"]*\bnoindex\b[^"]*"|content="[^"]*\bnoindex\b[^"]*"\s+name="robots")/i;

function unescapeXml(text: string): string {
  return text
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&apos;", "'")
    .replaceAll("&amp;", "&");
}

async function maybe(file: string): Promise<string | undefined> {
  try {
    return await readFile(file, "utf8");
  } catch {
    return undefined;
  }
}

/** Reads a built site in `dir`, as a crawler would find it served. */
export async function crawl(dir: string): Promise<Crawled> {
  const files = await filesUnder(dir);
  const pages: CrawledPage[] = [];
  for (const file of files.filter(
    (name) => name === "index.html" || name.endsWith("/index.html"),
  )) {
    const html = await readFile(path.join(dir, file), "utf8");
    pages.push({ route: `/${file.slice(0, -"index.html".length)}`, noindex: NOINDEX.test(html) });
  }
  const xml = await maybe(path.join(dir, "sitemap.xml"));
  const sitemap =
    xml === undefined
      ? undefined
      : [...xml.matchAll(/<loc>([^<]*)<\/loc>/g)].map((match) => unescapeXml(match[1] ?? ""));
  const llms: string[] = [];
  for (const file of files.filter((name) => path.posix.basename(name) === "llms.txt")) {
    const text = await readFile(path.join(dir, file), "utf8");
    llms.push(...[...text.matchAll(/\]\(([^)\s]+)\)/g)].map((match) => match[1] ?? ""));
  }
  return { pages, sitemap, robots: await maybe(path.join(dir, "robots.txt")), llms };
}

/** What a page that isn't found must not be found by: its route, and its Markdown's. */
function addressesOf(route: string): string[] {
  return route === "/"
    ? ["/", "/index.md"]
    : [route, `${route.slice(0, -1)}.md`, `${route}index.md`];
}

/**
 * What's wrong with a crawled site, for a site at `site`, an origin: each
 * sitemap address that isn't a built page or says `noindex`; each built page
 * that doesn't say `noindex`, isn't a forwarding page or in `skipped`, and
 * isn't in the sitemap; and each page that says `noindex` and is in
 * `llms.txt`. Empty when the site is right.
 */
export function crawlerProblems(
  crawled: Crawled,
  { site, skipped }: { site: string; skipped: readonly string[] },
): string[] {
  const problems: string[] = [];
  const pages = new Map(crawled.pages.map((page) => [page.route, page]));
  const sitemap = crawled.sitemap ?? [];
  if (crawled.sitemap === undefined) problems.push("no sitemap.xml");
  const listed = new Set<string>();
  for (const address of sitemap) {
    const url = new URL(address);
    if (url.origin !== site) {
      problems.push(`sitemap.xml: ${address} isn't on ${site}`);
      continue;
    }
    listed.add(url.pathname);
    const page = pages.get(url.pathname);
    if (page === undefined) problems.push(`sitemap.xml: ${address} isn't a built page`);
    else if (page.noindex) problems.push(`sitemap.xml: ${address} says noindex`);
  }
  const skip = new Set(skipped);
  for (const page of crawled.pages) {
    if (page.noindex || skip.has(page.route) || listed.has(page.route)) continue;
    problems.push(`${page.route}: a built page that isn't in sitemap.xml`);
  }
  const llms = new Set(crawled.llms.map((link) => new URL(link, site).pathname));
  for (const page of crawled.pages.filter((each) => each.noindex)) {
    const found = addressesOf(page.route).filter((address) => llms.has(address));
    for (const address of found)
      problems.push(`llms.txt: ${address} is of a page that says noindex`);
  }
  return problems;
}
