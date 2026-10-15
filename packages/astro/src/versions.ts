// A versioned build's lines, from `<output-dir>/_ascribe/versions.json`
// (the output layout contract's versions file): which lines a build has, the
// site output of each, and the lines that publish each page, with its route
// in each. The compiler writes the file with every build of a project that
// has `[releases]`, whichever build was asked for, so the current line's
// build is enough to know the others.
import { readFileSync, statSync } from "node:fs";
import path from "node:path";

/** The version of the file this integration reads. */
const VERSION = 1;

/** A line of a versioned build, as the file has it. */
interface Line {
  /** The line, as `[releases]` writes it: `3.4`. */
  line: string;
  /** `current` for the newest line, `live` for the others. */
  status: "current" | "live";
  /** The build that writes the line's output: the declared build, or `<build>@<line>`. */
  build: string;
  /** The path the line's pages are under, from the host's root, ending in `/`. */
  base: string;
}

/** A line with where its site output is. */
export interface LineOutput extends Line {
  /** The line's site output root: `<output-dir>/<build>/site`. */
  siteRoot: string;
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

/**
 * The lines of the build whose site output is at `siteRoot`, newest first,
 * each with its site output, or `undefined` when the build isn't versioned.
 * `siteRoot` is the declared build's, which is the current line's.
 */
export function linesOf(siteRoot: string): LineOutput[] | undefined {
  const { outputRoot, build } = outputOf(siteRoot);
  const versioned = readVersions(outputRoot)?.builds[build];
  if (versioned === undefined) return undefined;
  return versioned.lines.map((line) => ({
    ...line,
    siteRoot: path.join(outputRoot, line.build, "site"),
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
  const { outputRoot, build } = outputOf(siteRoot);
  const versioned = readVersions(outputRoot)?.builds[build];
  if (versioned?.pages[source] === undefined) return [];
  const pages = versioned.pages;
  return versioned.lines.flatMap((line) => {
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
