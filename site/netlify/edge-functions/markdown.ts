// Content negotiation, the host's part of the Web Documentation Delivery Spec:
// a page asked for with `Accept: text/markdown` is answered with its Markdown,
// which the build publishes at the page's address with `.md` (`[consumer]
// agents = true` in ../docs/ascribe.toml), with the `.md` file's headers that
// say how it's listed. Any other request, and a page without a Markdown file,
// goes on to the page as it is.
//
// Netlify runs this before its cache, on every request it matches, so the
// cache holds only the pages and their `.md` files, never an answer of this.

/** A page's address: the routing is `trailing-slash = "always"`. */
function isPage(pathname: string): boolean {
  return pathname.endsWith("/");
}

/**
 * Where a page's Markdown can be, in the order to try: its address with
 * `.md`, or, for the root page of a project or a folder (entry id `index`),
 * `index.md` under its address. The docs' root page, `/docs/`, is
 * `/docs/index.md`.
 */
export function markdownPaths(pathname: string): string[] {
  if (!isPage(pathname)) return [];
  const route = pathname.slice(0, -1);
  return route === "" ? ["/index.md"] : [`${route}.md`, `${route}/index.md`];
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
  const targets = markdownPaths(pathname);
  const read = request.method === "GET" || request.method === "HEAD";
  if (!read || targets.length === 0) return context.next();
  if (!prefersMarkdown(request.headers.get("accept"))) {
    // The page, marked as one of two answers, so a cache past Netlify's
    // doesn't hand its HTML to an agent that asked for Markdown.
    const page = await context.next();
    const headers = new Headers(page.headers);
    headers.append("vary", "Accept");
    return new Response(page.body, { status: page.status, statusText: page.statusText, headers });
  }
  let found: { target: string; source: Response } | undefined;
  for (const target of targets) {
    const source = await fetch(new URL(target, request.url));
    if (source.ok) {
      found = { target, source };
      break;
    }
    await source.body?.cancel();
  }
  if (found === undefined) return context.next();
  const { target, source } = found;
  const headers = new Headers({
    "content-type": "text/markdown; charset=utf-8",
    vary: "Accept",
    // Where the Markdown itself is, for an agent to cite or fetch again.
    "content-location": target,
  });
  // The Markdown version's own headers, which Netlify gives the `.md` path
  // from `_headers` and never gives a function's answer: the rule its page's
  // head has (`x-robots-tag`) and its page as canonical (`link`).
  for (const name of ["cache-control", "x-robots-tag", "link"]) {
    const value = source.headers.get(name);
    if (value !== null) headers.set(name, value);
  }
  return new Response(request.method === "HEAD" ? null : source.body, { status: 200, headers });
}

/** Netlify's inline configuration: every address, minus the files that are never pages. */
export const config = {
  path: "/*",
  excludedPath: ["/_astro/*", "/_ascribe/*", "/pagefind/*", "/*.md", "/*.txt"],
  onError: "bypass",
};
