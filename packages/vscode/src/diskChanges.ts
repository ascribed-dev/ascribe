// Which files changed on disk start a project's server, which a running
// server is told about, and the batching that turns a burst of changes into
// one start per project. No VS Code in this module.
import { readFile } from "node:fs/promises";
import * as path from "node:path";
import { parse } from "smol-toml";
import {
  comparable,
  ownedElsewhere,
  owningProject,
  samePath,
  within,
  type Project,
} from "./projects.js";

/** Where a project's own files are, from its `ascribe.toml`. Absolute paths. */
export interface Layout {
  /** `[project] content-root`. */
  contentRoot: string;
  /** `[project] output-dir`. */
  outputDir: string;
  /** The folder of each `[sources.<name>]` that has a `path`. */
  sources: string[];
}

/**
 * `[project]`'s defaults, as the content model reference gives them (a unit
 * test compares the two).
 */
export const PROJECT_DEFAULTS = { "content-root": "docs", "output-dir": ".ascribe/build" };

/** Folders whose files never start a server, wherever they are. */
const SKIPPED = new Set(["node_modules", ".git"]);

/**
 * A project's layout from the text of its `ascribe.toml`. A model that doesn't
 * parse, or a key of the wrong type, gives the defaults: the server, once
 * started, reports what's wrong with it.
 */
export function layoutOf(project: Project, text: string): Layout {
  let table: Record<string, unknown> = {};
  try {
    table = parse(text);
  } catch {
    // The defaults.
  }
  const settings = section(table, "project");
  const setting = (key: keyof typeof PROJECT_DEFAULTS) => {
    const value = settings[key];
    return path.resolve(project.folder, typeof value === "string" ? value : PROJECT_DEFAULTS[key]);
  };
  const sources = Object.values(section(table, "sources")).flatMap((source) => {
    const folder = isTable(source) ? source["path"] : undefined;
    return typeof folder === "string" ? [path.resolve(project.folder, folder)] : [];
  });
  return { contentRoot: setting("content-root"), outputDir: setting("output-dir"), sources };
}

/** A project's layout, read from disk; the defaults when its `ascribe.toml` can't be read. */
export async function readLayout(project: Project): Promise<Layout> {
  const text = await readFile(project.config, "utf8").catch(() => "");
  return layoutOf(project, text);
}

/**
 * Whether a change to `file` on disk should start `project`'s server: the
 * file is the project's `ascribe.toml`, is under its content root (and not
 * in a project nested there), or is in one of its sources' folders, and
 * isn't in its output directory, `node_modules`, or `.git`. Anything under the content root
 * counts, not just pages: assets are checked too, and deleting a folder of
 * pages is reported as one change to the folder.
 */
export function startsServer(
  file: string,
  project: Project,
  layout: Layout,
  projects: readonly Project[],
): boolean {
  if (samePath(file, project.config)) return true;
  if (within(file, layout.outputDir)) return false;
  const inContent =
    within(file, layout.contentRoot) &&
    owningProject(file, projects)?.config === project.config &&
    !skipped(file, layout.contentRoot);
  return inContent || inASource(file, layout.sources);
}

/**
 * Whether a change to `file` makes a check of every build of `project` out of
 * date (`EveryBuildChecks.isCurrent`): any file of the project, not just
 * those under the content root, since the check reads `ascribe.nav`,
 * `ascribe.permalinks`, `ascribe.lock`, the memos, and the templates beside
 * `ascribe.toml` too; and any file in one of its sources' folders. A file of a
 * nested project, or in the output directory, the project's `.ascribe`
 * folder (which the check itself may write), `node_modules`, or `.git`
 * doesn't count. It errs toward out of date: a change the check doesn't read
 * costs a check, where one it reads and this missed would show a stale count
 * as current.
 */
export function changesCheck(
  file: string,
  project: Project,
  layout: Layout,
  projects: readonly Project[],
): boolean {
  if (samePath(file, project.config)) return true;
  if (within(file, layout.outputDir)) return false;
  const own =
    owningProject(file, projects)?.config === project.config &&
    !skipped(file, project.folder) &&
    !within(file, path.join(project.folder, ".ascribe"));
  return own || inASource(file, layout.sources);
}

/**
 * Whether a running server of `project` is told that `file` changed on disk
 * (`didChangeWatchedFile` in `scope.ts`): a file of its own, or one in its
 * sources' folders wherever they are, in no project or in another's. The
 * server answers its lenses and This page's rows again on these, and reads
 * a source's observations and `ascribe.examples.toml` from its folder.
 * `sources` are the folders of the sources the server loaded.
 */
export function toldOfChange(
  file: string,
  project: Project,
  sources: readonly string[],
  projects: readonly Project[],
): boolean {
  return !ownedElsewhere(project, file, projects) || inASource(file, sources);
}

/** Whether `file` is in one of `sources`' folders, and not in its `node_modules` or `.git`. */
export function inASource(file: string, sources: readonly string[]): boolean {
  return sources.some((folder) => within(file, folder) && !skipped(file, folder));
}

/** Whether a path below `folder` passes through `node_modules` or `.git`. */
function skipped(file: string, folder: string): boolean {
  const below = comparable(file).slice(comparable(folder).length);
  return below.split("/").some((segment) => SKIPPED.has(segment));
}

/** How long a burst of changes must go quiet before it's acted on. */
export const QUIET_MS = 250;
/** The longest a change waits, however long the burst. */
export const MAX_WAIT_MS = 1_000;

/**
 * Gathers changed files and hands them over together: once no change has
 * come for `QUIET_MS`, or `MAX_WAIT_MS` after the first, whichever is sooner.
 * A `git checkout` that touches 400 files is one call.
 */
export class ChangeBatch {
  private readonly files = new Map<string, string>();
  private quiet: ReturnType<typeof setTimeout> | undefined;
  private longest: ReturnType<typeof setTimeout> | undefined;

  constructor(private readonly flush: (files: string[]) => void) {}

  add(file: string): void {
    this.files.set(comparable(file), file);
    clearTimeout(this.quiet);
    this.quiet = setTimeout(() => this.flushNow(), QUIET_MS);
    this.longest ??= setTimeout(() => this.flushNow(), MAX_WAIT_MS);
  }

  dispose(): void {
    this.clear();
    this.files.clear();
  }

  private flushNow(): void {
    this.clear();
    const files = [...this.files.values()];
    this.files.clear();
    if (files.length > 0) this.flush(files);
  }

  private clear(): void {
    clearTimeout(this.quiet);
    clearTimeout(this.longest);
    this.quiet = undefined;
    this.longest = undefined;
  }
}

/** A table's subtable, or an empty one when it's missing or isn't a table. */
export function section(table: Record<string, unknown>, name: string): Record<string, unknown> {
  const value = table[name];
  return isTable(value) ? value : {};
}

export function isTable(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
