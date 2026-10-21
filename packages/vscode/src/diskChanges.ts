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
  /** Each `[sources.<name>]` that has a `path`. */
  sources: Source[];
}

/** A source with a folder: where it is, absolute, and which of its files it reads. */
export interface Source {
  folder: string;
  /** `include`: the files it reads, all of them when empty. */
  include: string[];
  /** `ignore`: the files it doesn't, whatever `include` says. */
  ignore: string[];
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
    if (!isTable(source) || typeof source["path"] !== "string") return [];
    return [
      {
        folder: path.resolve(project.folder, source["path"]),
        include: strings(source["include"]),
        ignore: strings(source["ignore"]),
      },
    ];
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
  return inContent || inASource(file, folders(layout.sources));
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
  return own || inASource(file, folders(layout.sources));
}

/**
 * Whether a running server of `project` is told that `file` changed on disk
 * (`didChangeWatchedFile` in `scope.ts`): a file of its own, or one a source
 * can use (`sourceUses`) wherever it is, in no project or in another's.
 * The server answers its lenses and This page's rows again on these, and
 * reads a source's observations and `ascribe.examples.toml` from its folder.
 * The rest of a source's folder, such as a build's `target/`, isn't sent.
 * `sources` are the sources the server loaded.
 */
export function toldOfChange(
  file: string,
  project: Project,
  sources: readonly Source[],
  projects: readonly Project[],
): boolean {
  return (
    !ownedElsewhere(project, file, projects) || sources.some((source) => sourceUses(file, source))
  );
}

/** A source's examples config, and the folder of its observations. */
const EXAMPLES_FILE = "ascribe.examples.toml";
const OBSERVATIONS_FOLDER = "examples.observations";

/**
 * Whether a change to the file or folder `file` can matter to `source`, as
 * `Source::may_use` in crates/ascribe-model decides: a file its `include`
 * patterns cover and `ignore` doesn't, a folder that may hold one, or its
 * `ascribe.examples.toml` and observations. Never its `node_modules` or
 * `.git`. A pattern the content model wouldn't load covers everything for
 * `include` and nothing for `ignore`, as does case: telling the server of a
 * file it doesn't read costs little, where missing one leaves a lens stale.
 */
export function sourceUses(file: string, source: Source): boolean {
  if (!within(file, source.folder) || skipped(file, source.folder)) return false;
  const rest = comparable(file).slice(comparable(source.folder).length);
  const segments = rest.split("/").filter((segment) => segment !== "");
  const [first] = segments;
  if (first === OBSERVATIONS_FOLDER || (first === EXAMPLES_FILE && segments.length === 1)) {
    return true;
  }
  const ignored = source.ignore.some((pattern) => matches(pattern, segments, false) === true);
  const included =
    source.include.length === 0 ||
    source.include.some((pattern) => matches(pattern, segments, true) !== false);
  return !ignored && included;
}

/** The folders of `sources`. */
function folders(sources: readonly Source[]): string[] {
  return sources.map((source) => source.folder);
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

/** A segment of a compiled pattern: `**`, or one segment's glob. */
type Segment = "**" | RegExp;

const compiled = new Map<string, Segment[][] | undefined>();

/**
 * Whether `pattern` matches the path of `segments`, or, when `within`, may
 * match a path below it too. `undefined` for a pattern the content model
 * wouldn't load (`pattern.rs` in crates/ascribe-model).
 */
function matches(
  pattern: string,
  segments: readonly string[],
  within: boolean,
): boolean | undefined {
  if (!compiled.has(pattern)) compiled.set(pattern, compile(pattern));
  const alternatives = compiled.get(pattern);
  return alternatives?.some((alternative) => matchSegments(alternative, segments, within));
}

function matchSegments(
  pattern: readonly Segment[],
  segments: readonly string[],
  within: boolean,
): boolean {
  const [first, ...rest] = pattern;
  if (first === undefined) return segments.length === 0;
  if (within && segments.length === 0) return true;
  if (first === "**") {
    for (let skip = 0; skip <= segments.length; skip++) {
      if (matchSegments(rest, segments.slice(skip), within)) return true;
    }
    return false;
  }
  const [head, ...tail] = segments;
  return head !== undefined && first.test(head) && matchSegments(rest, tail, within);
}

/** A pattern's alternatives, each a list of segments, as `Pattern::new` compiles it. */
function compile(pattern: string): Segment[][] | undefined {
  if (pattern === "" || pattern.startsWith("/")) return undefined;
  const out: Segment[][] = [];
  for (const alternative of expandBraces(pattern) ?? []) {
    const segments: Segment[] = [];
    for (const segment of alternative.split("/")) {
      if (segment === "" || segment === ".." || (segment !== "**" && segment.includes("**"))) {
        return undefined;
      }
      const glob = segment === "**" ? segment : segmentGlob(segment);
      if (glob === undefined) return undefined;
      segments.push(glob);
    }
    out.push(segments);
  }
  return out.length > 0 ? out : undefined;
}

/** `{a,b}` alternatives, which don't nest, with escaped characters kept escaped. */
function expandBraces(pattern: string): string[] | undefined {
  let open: number | undefined;
  for (let i = 0; i < pattern.length; i++) {
    const c = pattern[i];
    if (c === "\\") i++;
    else if (c === "{") {
      if (open !== undefined) return undefined;
      open = i;
    } else if (c === "}") {
      if (open === undefined) return undefined;
      const prefix = pattern.slice(0, open);
      const suffix = pattern.slice(i + 1);
      const out: string[] = [];
      for (const option of splitUnescaped(pattern.slice(open + 1, i))) {
        const rest = expandBraces(option + suffix);
        if (rest === undefined) return undefined;
        out.push(...rest.map((r) => prefix + r));
      }
      return out;
    }
  }
  return open === undefined ? [pattern] : undefined;
}

/** `text` split at each `,` that isn't escaped. */
function splitUnescaped(text: string): string[] {
  const out = [""];
  for (let i = 0; i < text.length; i++) {
    const c = text[i] ?? "";
    if (c === ",") out.push("");
    else {
      const next = c === "\\" ? c + (text[++i] ?? "") : c;
      out[out.length - 1] += next;
    }
  }
  return out;
}

/** One segment's glob: `*` and `?` within the segment, `\` escaping the next character. */
function segmentGlob(segment: string): RegExp | undefined {
  let source = "";
  // By code point, as the content model reads a pattern.
  const chars = Array.from(segment);
  for (let i = 0; i < chars.length; i++) {
    const c = chars[i];
    if (c === "*") source += "[^/]*";
    else if (c === "?") source += "[^/]";
    else {
      const literal = c === "\\" ? chars[++i] : c;
      if (literal === undefined) return undefined;
      source += literal.replace(/[.*+?^${}()|[\]\\/]/g, "\\$&");
    }
  }
  return new RegExp(`^${source}$`, "iu");
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

function section(table: Record<string, unknown>, name: string): Record<string, unknown> {
  const value = table[name];
  return isTable(value) ? value : {};
}

/** The strings of a TOML array; none for anything else. */
function strings(value: unknown): string[] {
  return Array.isArray(value)
    ? value.filter((item): item is string => typeof item === "string")
    : [];
}

function isTable(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
