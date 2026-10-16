// A versioned build's lines, from `<output-dir>/_ascribe/versions.json`
// (the output layout contract's versions file): which lines a build has, the
// site output of each, the lines that have left it and where each can still be
// read, and the lines that publish each page, with its route in each. The compiler writes the file with every build of a project that
// has `[releases]`, whichever build was asked for, so the current line's
// build is enough to know the others.
import { readFileSync, statSync } from "node:fs";
import path from "node:path";

/** The version of the file this integration reads. */
const VERSION = 1;

/** A live line of a versioned build, as the file has it. */
interface LiveLine {
  /** The line, as `[releases]` writes it: `3.4`. */
  line: string;
  /** `current` for the newest line, `live` for the others. */
  status: "current" | "live";
  /** The build that writes the line's output: the declared build, or `<build>@<line>`. */
  build: string;
  /** The path the line's pages are under, from the host's root, ending in `/`. */
  base: string;
  /** The last day the line is supported, `YYYY-MM-DD`, when the project says. */
  ends?: string;
}

/** A line that has left the window, as the file has it. */
interface ArchivedLine {
  line: string;
  status: "archived";
  /** Where the line can still be read, when anywhere. */
  url?: string;
  /** The day its support ended, `YYYY-MM-DD`, when the project says. */
  ended?: string;
}

type Line = LiveLine | ArchivedLine;

/** A line with where its site output is. */
export interface LineOutput extends LiveLine {
  /** The line's site output root: `<output-dir>/<build>/site`. */
  siteRoot: string;
  /** The line's plain output root, its outputs for agents: `<output-dir>/<build>/plain`. */
  plainRoot: string;
}

/** A page's entry: its route in each line that has it, and the pages it replaces or is replaced by. */
interface VersionsPage {
  lines: Record<string, string>;
  replaces?: string;
  "replaced-by"?: string;
}

/** The file, as much as the integration reads. */
interface VersionsFile {
  version: number;
  builds: Record<
    string,
    {
      target: string;
      /** The target's label: `Self-managed`. Absent in a file an earlier Ascribe wrote. */
      label?: string;
      lines: Line[];
      pages: Record<string, VersionsPage>;
    }
  >;
}

/** A build's site output root (`<output-dir>/<build>/site`) split into the two it's made of. */
function outputOf(siteRoot: string): { outputRoot: string; build: string } {
  const buildDir = path.dirname(siteRoot);
  return { outputRoot: path.dirname(buildDir), build: path.basename(buildDir) };
}

function versionsFile(outputRoot: string): string {
  return path.join(outputRoot, "_ascribe", "versions.json");
}

/** The file as last read, by its path, and when it was written. */
const read = new Map<string, { written: number; file: VersionsFile }>();

/**
 * The versions file of the project whose output is at `outputRoot`, or
 * `undefined` when it has none: a project with no `[releases]`. Read again
 * when the file changes, as it does with each build in `astro dev`.
 */
function readVersions(outputRoot: string): VersionsFile | undefined {
  const file = versionsFile(outputRoot);
  let written: number;
  try {
    written = statSync(file).mtimeMs;
  } catch {
    return undefined;
  }
  const known = read.get(file);
  if (known !== undefined && known.written === written) return known.file;
  const parsed = JSON.parse(readFileSync(file, "utf8")) as VersionsFile;
  if (parsed.version !== VERSION) {
    throw new Error(
      `@ascribed/astro: ${file} is version ${String(parsed.version)} of the versions file, and this version of the integration reads version ${VERSION}. Update @ascribed/astro and @ascribed/cli together.`,
    );
  }
  read.set(file, { written, file: parsed });
  return parsed;
}

/** The entry of the versioned build whose site output is at `siteRoot`. */
function versionedBuild(siteRoot: string): VersionsFile["builds"][string] | undefined {
  const { outputRoot, build } = outputOf(siteRoot);
  return readVersions(outputRoot)?.builds[build];
}

function isLive(line: Line): line is LiveLine {
  return line.status !== "archived";
}

/**
 * The live lines of the build whose site output is at `siteRoot`, newest
 * first, each with its site output, or `undefined` when the build isn't
 * versioned. `siteRoot` is the declared build's, which is the current line's.
 */
export function linesOf(siteRoot: string): LineOutput[] | undefined {
  const { outputRoot } = outputOf(siteRoot);
  const versioned = versionedBuild(siteRoot);
  if (versioned === undefined) return undefined;
  return versioned.lines.filter(isLive).map((line) => ({
    ...line,
    siteRoot: path.join(outputRoot, line.build, "site"),
    plainRoot: path.join(outputRoot, line.build, "plain"),
  }));
}

/** The lines other than the current one, newest first: empty when the build isn't versioned. */
export function olderLines(siteRoot: string): LineOutput[] {
  return (linesOf(siteRoot) ?? []).filter((line) => line.status !== "current");
}

/** A line that publishes a page, and the page's route in it. */
export interface PageLine {
  line: string;
  current: boolean;
  route: string;
}

/**
 * The lines that publish the page whose source path is `source`, newest
 * first, each with the page's route in it, in the versioned build whose
 * current line's site output is at `siteRoot`. A line that doesn't have the
 * page has the page that takes its place there, when one does: the page
 * that replaces it, or that it replaces, nearest along the chain first.
 * Empty when the build isn't versioned or no line has the page.
 */
export function pageLines(siteRoot: string, source: string): PageLine[] {
  const versioned = versionedBuild(siteRoot);
  if (versioned?.pages[source] === undefined) return [];
  const pages = versioned.pages;
  return versioned.lines.filter(isLive).flatMap((line) => {
    const route = inLine(pages, source, line.line);
    return route === undefined
      ? []
      : [{ line: line.line, current: line.status === "current", route }];
  });
}

/**
 * The route, in `line`, of the page `source` or of the page nearest it along
 * its `replaces` and `replaced-by`, the one that replaces it first.
 */
function inLine(
  pages: Record<string, VersionsPage>,
  source: string,
  line: string,
): string | undefined {
  const seen = new Set([source]);
  const queue = [source];
  for (let at = queue.shift(); at !== undefined; at = queue.shift()) {
    const page = pages[at];
    const route = page?.lines[line];
    if (route !== undefined) return route;
    for (const next of [page?.["replaced-by"], page?.replaces]) {
      if (next !== undefined && !seen.has(next)) {
        seen.add(next);
        queue.push(next);
      }
    }
  }
  return undefined;
}

/** What a page of an older line says about the line, and where the current one is. */
export interface LineNotice {
  /** The label of the target the build is versioned by: `Self-managed`. */
  label: string;
  /** The page's line: `3.4`. */
  line: string;
  /** The last day the line is supported, `YYYY-MM-DD`, when the project says. */
  ends?: string;
  /** The current line: `3.5`. */
  current: string;
  /** The page's route in the current line, or the current line's home's. */
  route: string;
  /** Whether `route` is the page, or the page that takes its place, rather than the home. */
  samePage: boolean;
}

/**
 * What the page whose source path is `source`, in the line `line`, says about
 * that line, in the versioned build whose current line's site output is at
 * `siteRoot`: `undefined` on the current line, and for a build that isn't
 * versioned. `home` is the current line's home's route, the link when the
 * current line has neither the page nor one in its place.
 */
export function lineNotice(
  siteRoot: string,
  source: string,
  line: string,
  home: string,
): LineNotice | undefined {
  const versioned = versionedBuild(siteRoot);
  if (versioned === undefined) return undefined;
  const live = versioned.lines.filter(isLive);
  const own = live.find((each) => each.line === line);
  const current = live.find((each) => each.status === "current");
  if (own === undefined || current === undefined || own === current) return undefined;
  const route = inLine(versioned.pages, source, current.line);
  return {
    label: versioned.label ?? versioned.target,
    line,
    ...(own.ends === undefined ? {} : { ends: own.ends }),
    current: current.line,
    route: route ?? home,
    samePage: route !== undefined,
  };
}

/** A line of a versioned build, as a page's list of versions shows it. */
export interface ListedLine {
  line: string;
  /** `current`, `live`, or `archived`. */
  status: Line["status"];
  /**
   * Where the line is linked to: the page's route in a live line that
   * publishes it, or the page in its place; an archived line's address.
   * Absent for a live line with neither, and an archived line with no address.
   */
  route?: string;
}

/**
 * Every line of the versioned build whose current line's site output is at
 * `siteRoot`, newest first, live then archived, each with where a list of
 * versions on the page whose source path is `source` links it. Archived lines
 * with no address are left out. Empty when the build isn't versioned.
 */
export function listedLines(siteRoot: string, source: string): ListedLine[] {
  const versioned = versionedBuild(siteRoot);
  if (versioned === undefined) return [];
  return versioned.lines.flatMap((line): ListedLine[] => {
    if (!isLive(line)) {
      return line.url === undefined
        ? []
        : [{ line: line.line, status: line.status, route: line.url }];
    }
    const route = inLine(versioned.pages, source, line.line);
    return [{ line: line.line, status: line.status, ...(route === undefined ? {} : { route }) }];
  });
}
