// Several Ascribe projects in one Astro site: the entries of `projects`, and
// the rules for what they share. Each entry has a name, which is what a
// collection, `review`, a schema's folder, and an error message say. Two
// entries may share a project and differ by build; entries of one project are
// one project to these rules, since they share its `ascribe.toml`.
import { readFileSync } from "node:fs";
import path from "node:path";
import { slug } from "github-slugger";
import { hasPermalinks, type PermalinkList } from "./permalinks.js";
import { normalizeBase, type ProjectInfo } from "./project.js";

/** One entry of the integration's `projects` option. */
export interface ProjectEntry {
  /** The entry's name: its collection's, `review`'s, and its schema folder's. */
  name: string;
  /** The build whose site output is the collection (a name in `ascribe.toml`). */
  build: string;
  /** The directory holding `ascribe.toml`, relative to the Astro root. Default: the root. */
  project?: string;
}

/** What a name may be: letters, digits, `-`, and `_`, starting with a letter. */
const NAME = /^[A-Za-z][A-Za-z0-9_-]*$/;

/** The ways the entries' names are wrong, as sentences. */
export function nameProblems(entries: readonly ProjectEntry[]): string[] {
  const problems: string[] = [];
  if (entries.length === 0)
    problems.push("`projects` is empty. Give it an entry for each project.");
  const seen = new Set<string>();
  for (const entry of entries) {
    if (typeof entry.name !== "string" || !NAME.test(entry.name)) {
      problems.push(
        `an entry's name is ${JSON.stringify(entry.name)}. A name has letters, digits, "-", and "_", and starts with a letter.`,
      );
    } else if (seen.has(entry.name)) {
      problems.push(`two entries are named "${entry.name}". Each entry's name must be its own.`);
    }
    seen.add(entry.name);
  }
  return problems;
}

/** An entry with its project read. */
export interface ResolvedEntry {
  name: string;
  build: string;
  project: ProjectInfo;
}

/** The entries of distinct projects, each project once with its first entry's name. */
function distinctProjects(entries: readonly ResolvedEntry[]): ResolvedEntry[] {
  const byDir = new Map<string, ResolvedEntry>();
  for (const entry of entries)
    if (!byDir.has(entry.project.dir)) byDir.set(entry.project.dir, entry);
  return [...byDir.values()];
}

/** Two projects publishing at one base path, as sentences. */
export function basePathClashes(entries: readonly ResolvedEntry[]): string[] {
  const problems: string[] = [];
  const projects = distinctProjects(entries);
  for (const [i, a] of projects.entries()) {
    for (const b of projects.slice(i + 1)) {
      if (a.project.consumer.basePath === b.project.consumer.basePath) {
        problems.push(
          `"${a.name}" and "${b.name}" both have [consumer] base-path "${a.project.consumer.basePath}". Give each project its own.`,
        );
      }
    }
  }
  return problems;
}

/**
 * A page's entry id, as Astro's `glob` loader computes it from the page's
 * path in the site output: each segment slugged, without the extension, and
 * without a final `/index`.
 */
export function entryId(pagePath: string): string {
  return pagePath
    .replace(/\.md$/, "")
    .split("/")
    .map((segment) => slug(segment))
    .join("/")
    .replace(/\/index$/, "");
}

/** The pages of a build's site output, by their paths in it, from its manifest. */
export function sitePages(project: ProjectInfo, build: string): string[] {
  const manifest = path.join(path.dirname(project.siteRoot(build)), "site.manifest.json");
  let files: unknown;
  try {
    files = (JSON.parse(readFileSync(manifest, "utf8")) as { files?: unknown }).files;
  } catch {
    return [];
  }
  if (!Array.isArray(files)) return [];
  return (files as { kind?: unknown; path?: unknown }[])
    .filter((file) => file.kind === "page" && typeof file.path === "string")
    .map((file) => file.path as string);
}

/**
 * A page of one project whose route falls under another project's base path,
 * as sentences: one project at `/` with another at `/docs/` is allowed, and a
 * page of the first at `/docs/…` is not. `pages` gives an entry's pages.
 */
export function pagesUnderOtherPaths(
  entries: readonly ResolvedEntry[],
  pages: (entry: ResolvedEntry) => readonly string[] = (entry) =>
    sitePages(entry.project, entry.build),
): string[] {
  const problems: string[] = [];
  const projects = distinctProjects(entries);
  for (const a of entries) {
    const base = a.project.consumer.basePath;
    const inner = projects.filter(
      (b) =>
        b.project.dir !== a.project.dir &&
        b.project.consumer.basePath !== base &&
        b.project.consumer.basePath.startsWith(base),
    );
    if (inner.length === 0) continue;
    for (const page of pages(a)) {
      const id = entryId(page);
      const route = id === "index" ? base : `${base}${id}/`;
      const under = inner.find((b) => route.startsWith(b.project.consumer.basePath));
      if (under !== undefined) {
        problems.push(
          `the page ${page} of "${a.name}" is at ${route}, under the base path of "${under.name}", ${under.project.consumer.basePath}. Move the page, or give "${under.name}" another base path.`,
        );
      }
    }
  }
  return problems;
}

/**
 * The problems with serving every entry's permalinks from one route, as
 * sentences: the lists that name anything must be published at one path (an
 * empty one, which every build writes, can be anywhere), and two projects may
 * not declare one name, since neither build can see the other. Entries of one
 * project may: the first entry's wins. `lists` holds each entry's list, or
 * `undefined` where its build wrote none.
 */
export function permalinkProblems(
  entries: readonly ResolvedEntry[],
  lists: readonly (PermalinkList | undefined)[],
): string[] {
  const problems: string[] = [];
  let first: { entry: ResolvedEntry; path: string } | undefined;
  const owners = new Map<string, ResolvedEntry>();
  for (const [i, entry] of entries.entries()) {
    const list = lists[i];
    if (list === undefined || !hasPermalinks(list)) continue;
    const at = normalizeBase(list.path);
    if (first === undefined) first = { entry, path: at };
    else if (at !== first.path) {
      problems.push(
        `"${first.entry.name}" publishes its permalinks at ${first.path}, and "${entry.name}" at ${at}. One route serves them all: set [consumer] permalink-path to one path in both.`,
      );
      continue;
    }
    for (const name of [...Object.keys(list.permalinks), ...Object.keys(list.removed ?? {})]) {
      const owner = owners.get(name);
      if (owner === undefined) owners.set(name, entry);
      else if (owner.project.dir !== entry.project.dir) {
        problems.push(
          `"${owner.name}" and "${entry.name}" both declare the permalink "${name}". A name leads to one place: rename or retire it in one of them.`,
        );
      }
    }
  }
  return problems;
}
