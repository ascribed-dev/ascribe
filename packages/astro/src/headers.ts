// The response headers a build's files need from the host:
// `_ascribe/headers.json` in its site output (output-layout contract §1.4).
// A Markdown version has no head, so its page's rules are headers, which only
// the host sends. With `[consumer] headers = "netlify"`, `astro build` writes
// them into the site's `_headers`, Netlify's format; `astro dev` sends them
// with each Markdown version whatever the setting, so a site can be tried
// with no host.
//
// A site's own `public/_headers` is never rewritten: the build checks it has
// the lines, and fails with the lines to add when it doesn't.
import { readFileSync, statSync } from "node:fs";
import path from "node:path";
import { normalizeBase } from "./project.js";

/** The format version of `_ascribe/headers.json` this reads. */
const VERSION = 1;

/** The headers a host sends with one path, matched exactly. */
export interface HeaderRule {
  /** The path, from the host's root. */
  path: string;
  /** Each header's value, by its name. */
  headers: Record<string, string>;
}

/** `_ascribe/headers.json`. */
interface HeaderList {
  version: number;
  headers: HeaderRule[];
}

function headersFile(siteRoot: string): string {
  return path.join(siteRoot, "_ascribe", "headers.json");
}

/** The rules in a build's site output, or `undefined` when it has no list. */
export function readHeaders(siteRoot: string): HeaderRule[] | undefined {
  const file = headersFile(siteRoot);
  let text: string;
  try {
    text = readFileSync(file, "utf8");
  } catch {
    return undefined;
  }
  const list = JSON.parse(text) as HeaderList;
  if (list.version !== VERSION) {
    throw new Error(
      `@ascribed/astro: ${file} is version ${String(list.version)} of the headers list, and this version of the integration reads version ${VERSION}. Update @ascribed/astro and @ascribed/cli together.`,
    );
  }
  return list.headers;
}

/** A build's rules by path, as last read, and when its file was written. */
const byPath = new Map<string, { written: number; rules: Map<string, Record<string, string>> }>();

/**
 * The headers the rules in the build whose site output is at `siteRoot` give
 * `pathname`, a path from the host's root: none when no rule names it. Read
 * again when the file changes, as it does with each build in `astro dev`.
 */
export function headersFor(siteRoot: string, pathname: string): Record<string, string> {
  let written: number;
  try {
    written = statSync(headersFile(siteRoot)).mtimeMs;
  } catch {
    return {};
  }
  let known = byPath.get(siteRoot);
  if (known === undefined || known.written !== written) {
    const rules = new Map<string, Record<string, string>>();
    for (const rule of readHeaders(siteRoot) ?? []) {
      if (!rules.has(rule.path)) rules.set(rule.path, rule.headers);
    }
    known = { written, rules };
    byPath.set(siteRoot, known);
  }
  return known.rules.get(pathname) ?? {};
}

/**
 * Every project's rules as one list, sorted by path: a path two projects
 * both have keeps the first's rule, as the first entry's files win.
 */
export function mergeRules(projects: readonly (readonly HeaderRule[])[]): HeaderRule[] {
  const rules = new Map<string, HeaderRule>();
  for (const list of projects) {
    for (const rule of list) if (!rules.has(rule.path)) rules.set(rule.path, rule);
  }
  return [...rules.values()].sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
}

/** The rules in Netlify's `_headers` format: each path, then its headers indented below it. */
export function netlifyHeaders(rules: readonly HeaderRule[]): string {
  return rules
    .map(
      (rule) =>
        `${rule.path}\n${Object.entries(rule.headers)
          .map(([name, value]) => `  ${name}: ${value}\n`)
          .join("")}`,
    )
    .join("");
}

/**
 * The headers `_headers` gives each path exactly, as Netlify reads the file:
 * a line that isn't indented is a path, an indented `Name: value` line is a
 * header of the path above it, and a line starting with `#` is a comment.
 * Names are compared in lower case, as Netlify compares them. A path with a
 * wildcard or a placeholder is kept as written, and matches only itself here.
 */
function parseHeaders(text: string): Map<string, Set<string>> {
  const paths = new Map<string, Set<string>>();
  let current: Set<string> | undefined;
  for (const line of text.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (trimmed === "" || trimmed.startsWith("#")) continue;
    if (!/^\s/.test(line)) {
      current = paths.get(trimmed) ?? new Set();
      paths.set(trimmed, current);
      continue;
    }
    const colon = trimmed.indexOf(":");
    if (current === undefined || colon === -1) continue;
    current.add(headerKey(trimmed.slice(0, colon), trimmed.slice(colon + 1)));
  }
  return paths;
}

function headerKey(name: string, value: string): string {
  return `${name.trim().toLowerCase()}: ${value.trim()}`;
}

/**
 * The rules, each with only the headers the site's own `_headers` doesn't
 * give its path, leaving out a rule it has whole. Empty when it has every
 * line.
 */
export function missingRules(own: string, rules: readonly HeaderRule[]): HeaderRule[] {
  const has = parseHeaders(own);
  const missing: HeaderRule[] = [];
  for (const rule of rules) {
    const given = has.get(rule.path);
    const headers = Object.fromEntries(
      Object.entries(rule.headers).filter(([name, value]) => !given?.has(headerKey(name, value))),
    );
    if (Object.keys(headers).length > 0) missing.push({ path: rule.path, headers });
  }
  return missing;
}

/** What the build does about the host's headers. */
export type HeadersOutcome =
  | { kind: "none" }
  | { kind: "write"; text: string }
  | { kind: "note"; text: string }
  | { kind: "fail"; text: string };

/** What the build does about `_headers`, given every project's rules and the site. */
export function headersOutcome(sources: {
  /** The rules of every project that sets `[consumer] headers = "netlify"`. */
  rules: readonly HeaderRule[];
  /** Astro's `base`. */
  base: string;
  /** The site's own `public/_headers`, when it has one. */
  own: string | undefined;
}): HeadersOutcome {
  if (sources.rules.length === 0) return { kind: "none" };
  const text = netlifyHeaders(sources.rules);
  const base = normalizeBase(sources.base);
  if (base !== "/") {
    return {
      kind: "note",
      text: `no _headers: Astro's \`base\` is "${base}", and Netlify reads _headers only at the root of what's deployed. Put these lines in the host's own:\n${text}`,
    };
  }
  if (sources.own !== undefined) {
    const missing = missingRules(sources.own, sources.rules);
    if (missing.length === 0) {
      return {
        kind: "note",
        text: "no _headers: public/_headers is the site's own, and has Ascribe's lines.",
      };
    }
    return {
      kind: "fail",
      text: `public/_headers is the site's own, and lacks the headers Ascribe's pages need. Add these lines to it:\n${netlifyHeaders(missing)}`,
    };
  }
  return { kind: "write", text };
}
