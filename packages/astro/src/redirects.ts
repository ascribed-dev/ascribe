// The redirects a build writes for the host: with `[consumer] host =
// "netlify"`, the compiler writes Netlify's `_redirects` in each build's site
// output, at `_ascribe/host/_redirects` (output-layout contract §1.9), from
// the moves `ascribe.addresses` records. `astro build` merges every
// project's into the site's `_redirects`, at the root of what's deployed.
//
// A site's own `public/_redirects` is kept, after Ascribe's rules: Netlify
// uses the first rule that matches, so a page's recorded move wins over a
// pattern of the site's.
import { readFileSync } from "node:fs";
import path from "node:path";
import { normalizeBase } from "./project.js";
import { routeKey } from "./records.js";

/** One redirect: an old address, from the host's root, to where it leads now. */
export interface Redirect {
  from: string;
  to: string;
  status: number;
}

/**
 * The redirects in the host's `_redirects` the compiler wrote in a build's
 * site output, in its order: none when it has none, as without `[consumer]
 * host`.
 */
export function readHostRedirects(siteRoot: string): Redirect[] {
  let text: string;
  try {
    text = readFileSync(path.join(siteRoot, "_ascribe", "host", "_redirects"), "utf8");
  } catch {
    return [];
  }
  const out: Redirect[] = [];
  for (const line of text.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (trimmed === "" || trimmed.startsWith("#")) continue;
    const [from, to, status] = trimmed.split(/\s+/);
    if (from === undefined || to === undefined) continue;
    out.push({ from, to, status: status === undefined ? 301 : Number.parseInt(status, 10) });
  }
  return out;
}

/**
 * Every project's redirects as one list, in the order given: an address two
 * projects both redirect keeps the first's, as the first entry's files win.
 */
export function mergeRedirects(projects: readonly (readonly Redirect[])[]): Redirect[] {
  const seen = new Set<string>();
  const out: Redirect[] = [];
  for (const list of projects) {
    for (const redirect of list) {
      if (seen.has(redirect.from)) continue;
      seen.add(redirect.from);
      out.push(redirect);
    }
  }
  return out;
}

/** The redirects in Netlify's `_redirects` format: each on a line, `from`, `to`, and the status. */
export function netlifyRedirects(redirects: readonly Redirect[]): string {
  return redirects.map((r) => `${r.from} ${r.to} ${String(r.status)}\n`).join("");
}

/** What the build does about the host's redirects. */
export type RedirectsOutcome =
  | { kind: "none" }
  | { kind: "write"; text: string }
  | { kind: "note"; text: string }
  | { kind: "fail"; text: string };

/** What the build does about `_redirects`, given every project's redirects and the site. */
export function redirectsOutcome(sources: {
  /** The redirects of every project that sets `[consumer] host = "netlify"`, merged. */
  redirects: readonly Redirect[];
  /** Astro's `base`. */
  base: string;
  /** Whether a route is a page the site publishes, which a redirect mustn't hide. */
  published: (route: string) => boolean;
  /** The site's own `public/_redirects`, when it has one. */
  own: string | undefined;
}): RedirectsOutcome {
  if (sources.redirects.length === 0) return { kind: "none" };
  const text = netlifyRedirects(sources.redirects);
  const base = normalizeBase(sources.base);
  if (base !== "/") {
    return {
      kind: "note",
      text: `no _redirects: Astro's \`base\` is "${base}", and Netlify reads _redirects only at the root of what's deployed. Put these lines in the host's own:\n${text}`,
    };
  }
  const taken = sources.redirects.filter((r) => sources.published(routeKey(r.from)));
  if (taken.length > 0) {
    return {
      kind: "fail",
      text: `a page the site publishes is at an address ascribe.addresses forwards somewhere else, so its redirect would hide the page or never be used. Remove the entry, or move the page:\n${netlifyRedirects(taken)}`,
    };
  }
  if (sources.own === undefined) return { kind: "write", text };
  const own = sources.own.endsWith("\n") || sources.own === "" ? sources.own : `${sources.own}\n`;
  return {
    kind: "write",
    text: `${text}\n# The site's own, from public/_redirects: after Ascribe's, since the first rule that matches is used.\n${own}`,
  };
}
