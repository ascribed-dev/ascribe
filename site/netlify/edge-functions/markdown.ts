// Content negotiation, the host's part of the Web Documentation Delivery Spec:
// a page asked for with `Accept: text/markdown` is answered with its Markdown,
// which the build publishes at the page's address with `.md` (`[consumer]
// agents = true` in ../docs/ascribe.toml). Any other request, and a page
// without a Markdown file, goes on to the page as it is.
//
// Netlify runs this before its cache, on every request it matches, so the
// cache holds only the pages and their `.md` files, never an answer of this.

/** A page's address: the routing is `trailing-slash = "always"`. */
function isPage(pathname: string): boolean {
  return pathname.endsWith("/");
}

/** The address of a page's Markdown: its entry id with `.md`, `index.md` for the home page. */
export function markdownPath(pathname: string): string | undefined {
  if (!isPage(pathname)) return undefined;
  const route = pathname.slice(1, -1);
  return `/${route === "" ? "index" : route}.md`;
}

/** The weight an `Accept` header gives the first of `types` it names, 0 if none. */
function weight(accept: string, types: readonly string[]): number {
  let best = 0;
  for (const range of accept.split(",")) {
    const [type = "", ...params] = range.split(";").map((part) => part.trim().toLowerCase());
    if (!types.includes(type)) continue;
    const q = params.find((param) => param.startsWith("q="));
    const value = q === undefined ? 1 : Number(q.slice(2));
    if (Number.isFinite(value)) best = Math.max(best, value);
  }
  return best;
}

/**
 * Whether a request's `Accept` header asks for Markdown before HTML. A
 * browser's names HTML and not Markdown; `*\/*` counts for neither.
 */
export function prefersMarkdown(accept: string | null): boolean {
  if (accept === null) return false;
  const markdown = weight(accept, ["text/markdown", "text/x-markdown"]);
  return markdown > 0 && markdown >= weight(accept, ["text/html", "application/xhtml+xml"]);
}

/** What this needs of Netlify's context: the response it would give without this. */
interface Context {
  next(): Promise<Response>;
}

export default async function markdown(request: Request, context: Context): Promise<Response> {
  const pathname = new URL(request.url).pathname;
  const target = markdownPath(pathname);
  const read = request.method === "GET" || request.method === "HEAD";
  if (!read || target === undefined) return context.next();
  if (!prefersMarkdown(request.headers.get("accept"))) {
    // The page, marked as one of two answers, so a cache past Netlify's
    // doesn't hand its HTML to an agent that asked for Markdown.
    const page = await context.next();
    const headers = new Headers(page.headers);
    headers.append("vary", "Accept");
    return new Response(page.body, { status: page.status, statusText: page.statusText, headers });
  }
  const source = await fetch(new URL(target, request.url));
  if (!source.ok) {
    await source.body?.cancel();
    return context.next();
  }
  const headers = new Headers({
    "content-type": "text/markdown; charset=utf-8",
    vary: "Accept",
    // Where the Markdown itself is, for an agent to cite or fetch again.
    "content-location": target,
  });
  const cacheControl = source.headers.get("cache-control");
  if (cacheControl !== null) headers.set("cache-control", cacheControl);
  return new Response(request.method === "HEAD" ? null : source.body, { status: 200, headers });
}

/** Netlify's inline configuration: every address, minus the files that are never pages. */
export const config = {
  path: "/*",
  excludedPath: ["/_astro/*", "/_ascribe/*", "/pagefind/*", "/*.md", "/*.txt"],
  onError: "bypass",
};
