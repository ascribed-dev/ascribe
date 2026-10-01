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
function comparable(file: string): string {
  const windows = /^[a-zA-Z]:|^\\\\/.test(file);
  const normal = windows
    ? path.win32.normalize(file).replace(/\\/g, "/").toLowerCase()
    : path.posix.normalize(file);
  return normal.length > 1 && normal.endsWith("/") ? normal.slice(0, -1) : normal;
}

/** Whether `file` is `folder` or lies below it. */
function within(file: string, folder: string): boolean {
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
