import * as path from "node:path";

/** One Ascribe project: the folder of an `ascribe.toml`. */
export interface Project {
  /** Absolute path of `ascribe.toml`. */
  readonly config: string;
  /** Its directory. */
  readonly folder: string;
}

/**
 * A path in a form that compares with `===` and nests with `startsWith`:
 * normalised, forward slashes, no trailing separator. Windows paths (a drive
 * letter or a UNC prefix) are also case-folded, whatever platform this runs
 * on, so the answer doesn't depend on the host.
 */
export function comparable(file: string): string {
  const windows = /^[a-zA-Z]:|^\\\\/.test(file);
  const normal = windows
    ? path.win32.normalize(file).replace(/\\/g, "/").toLowerCase()
    : path.posix.normalize(file);
  return normal.length > 1 && normal.endsWith("/") ? normal.slice(0, -1) : normal;
}

/** Whether `file` is `folder` or lies below it. */
export function within(file: string, folder: string): boolean {
  const f = comparable(file);
  const d = comparable(folder);
  return f === d || f.startsWith(d.endsWith("/") ? d : `${d}/`);
}

/** The project that owns a file: the nearest ancestor `ascribe.toml`. */
export function owningProject(file: string, projects: readonly Project[]): Project | undefined {
  let owner: Project | undefined;
  for (const project of projects) {
    if (!within(file, project.folder)) continue;
    if (!owner || comparable(project.folder).length > comparable(owner.folder).length) {
      owner = project;
    }
  }
  return owner;
}

/**
 * Whether a file belongs to some other project than `project`, or to none: a
 * nested project, a sibling, or a file outside every project. A project's
 * server never needs these files, since a project's files lie inside its folder.
 */
export function ownedElsewhere(
  project: Project,
  file: string,
  projects: readonly Project[],
): boolean {
  return owningProject(file, projects)?.config !== project.config;
}

/** The projects nested inside `project`, whose files it doesn't own. */
export function nestedProjects(project: Project, projects: readonly Project[]): Project[] {
  const folder = comparable(project.folder);
  return projects.filter(
    (other) => comparable(other.folder) !== folder && within(other.folder, project.folder),
  );
}

/** Whether two paths name the same file or folder. */
export function samePath(a: string, b: string): boolean {
  return comparable(a) === comparable(b);
}

/**
 * A path under a folder's real path (git gives real paths), written under the
 * folder as it was opened instead: through the same links, so it matches the
 * URIs of the editors VS Code opens in it (macOS's `/var` is a link to
 * `/private/var`). A path outside the folder is reached from it with `..`.
 */
export function throughFolder(real: string, folder: string, realFolder: string): string {
  return path.resolve(folder, path.relative(realFolder, real));
}

/** A folder as a glob pattern: forward slashes, and glob characters escaped. */
export function globFolder(folder: string): string {
  return folder.replace(/\\/g, "/").replace(/[*?[{]/g, "[$&]");
}

/**
 * The folders never searched for projects, wherever they are and whether or
 * not git ignores them: Ascribe's own outputs and caches, dependencies, and
 * build output and caches whose names are only ever that. Not `build`, `out`,
 * or `bin`, which are also folders people write in; git's ignore rules leave
 * those out where they're output. The setting's description and the editor's
 * settings page list them too, and a unit test holds the three together.
 */
export const LEFT_OUT_FOLDERS = [
  ".ascribe",
  "node_modules",
  "bower_components",
  "vendor",
  ".venv",
  "venv",
  "__pypackages__",
  "target",
  "dist",
  ".output",
  ".next",
  ".nuxt",
  ".svelte-kit",
  ".astro",
  ".turbo",
  ".vercel",
  ".netlify",
  ".cache",
  "coverage",
  ".gradle",
  ".vscode-test",
  ".tox",
  ".nox",
] as const;

/**
 * The pattern that leaves folders out of the search for projects: every
 * folder of `LEFT_OUT_FOLDERS`, each pattern `files.exclude` turns on
 * (`true`, not a `when` clause), and each of `ascribe.projects.exclude`, as
 * one brace group. VS Code's globs don't nest braces, so each pattern's own
 * are expanded first.
 */
export function projectExclude(
  filesExclude: Readonly<Record<string, unknown>>,
  projectsExclude: readonly unknown[],
): string {
  const patterns = [
    ...LEFT_OUT_FOLDERS.map((folder) => `**/${folder}/**`),
    ...Object.entries(filesExclude)
      .filter(([, on]) => on === true)
      .map(([pattern]) => pattern),
    ...projectsExclude.filter((pattern): pattern is string => typeof pattern === "string"),
  ];
  const all = [
    ...new Set(
      patterns
        .flatMap(expandBraces)
        .map((pattern) => pattern.trim())
        .filter((pattern) => pattern !== ""),
    ),
  ];
  const [only] = all;
  return all.length === 1 && only !== undefined ? only : `{${all.join(",")}}`;
}

/**
 * Whether an `ascribe.toml` is in a project's copies of a source in another
 * repository, `sources/<name>/` in the folder of another of `configs` (SPEC
 * §7.4): that repository's own project, which the project shows as code.
 */
export function inSourceCopies(config: string, configs: readonly string[]): boolean {
  const file = comparable(config);
  return configs.some((other) => {
    const folder = path.posix.dirname(comparable(other));
    const copies = folder.endsWith("/") ? `${folder}sources/` : `${folder}/sources/`;
    // Below `sources/<name>/`, not `sources/` itself.
    return file.startsWith(copies) && file.slice(copies.length).includes("/");
  });
}

/**
 * A glob pattern's alternatives, one pattern each: `*.{md,mdx}` is `*.md`
 * and `*.mdx`. Groups nest, and a brace with no partner is left as it is.
 */
export function expandBraces(pattern: string): string[] {
  const open = pattern.indexOf("{");
  if (open < 0) return [pattern];
  let depth = 0;
  const commas: number[] = [];
  for (let i = open; i < pattern.length; i++) {
    const c = pattern[i];
    if (c === "{") depth++;
    else if (c === "}") {
      depth--;
      if (depth === 0) {
        const before = pattern.slice(0, open);
        const after = pattern.slice(i + 1);
        const starts = [open, ...commas];
        const ends = [...commas, i];
        const options = starts.map((start, n) => pattern.slice(start + 1, ends[n] ?? i));
        return options.flatMap((option) => expandBraces(before + option + after));
      }
    } else if (c === "," && depth === 1) commas.push(i);
  }
  return [pattern];
}

/** A path with forward slashes, if it's a Windows path. */
function slashed(file: string): string {
  return /^[a-zA-Z]:|^\\\\/.test(file) ? file.replace(/\\/g, "/") : file;
}

/** A workspace folder: its path and its name. */
export interface WorkspaceFolder {
  path: string;
  name: string;
}

/**
 * A project's name for people: its folder relative to its workspace folder,
 * with forward slashes, or the workspace folder's name for a project at its
 * root, or the whole folder for a project outside every workspace folder.
 */
export function projectName(project: Project, workspace: WorkspaceFolder | undefined): string {
  if (!workspace || !within(project.folder, workspace.path)) return slashed(project.folder);
  const base = slashed(workspace.path).replace(/\/+$/, "");
  const relative = slashed(project.folder)
    .slice(base.length)
    .replace(/^\/+|\/+$/g, "");
  return relative === "" ? workspace.name : relative;
}

/**
 * The output channel's name: `Ascribe` for a workspace with one project, and
 * otherwise `Ascribe (<project name>)`.
 */
export function channelName(
  project: Project,
  workspace: WorkspaceFolder | undefined,
  only: boolean,
): string {
  return only ? "Ascribe" : `Ascribe (${projectName(project, workspace)})`;
}
