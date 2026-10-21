// What's kept of a project's last Check Every Build between sessions, and
// whether the project changed since it started: by the files' modification
// times and which files there are, never their contents. No VS Code in this
// module.
import { createHash } from "node:crypto";
import { readdir, stat } from "node:fs/promises";
import * as path from "node:path";
import { changesCheck, sourceUses, type Layout } from "./diskChanges.js";
import type { Counted, Found, PanelProblem } from "./everyBuild.js";
import { comparable, samePath, within, type Project } from "./projects.js";
import type { Summary } from "./shapes.js";

/** The shape of a kept check, which changes with the extension's version. */
export interface KeptCheck {
  /** The extension's version that kept it: one from another is ignored. */
  version: string;
  /** When the check started, in milliseconds since the epoch. */
  started: number;
  summary: Summary;
  /** Why the project couldn't be checked, or `null`. */
  error: string | null;
  /** How many of the project's files had unsaved changes, which it read as saved. */
  unsaved: number;
  counted: Record<Counted, Found>;
  /** The problems it added to the Problems panel; none once it isn't current. */
  problems: PanelProblem[];
  /** Whether nothing had changed since it started when it was last kept. */
  current: boolean;
  /** Which files the project had once it was made (`ProjectFiles.list`). */
  files: string;
}

/**
 * A kept check, if `value` is one the extension at `version` kept; anything
 * else, from another version or not readable, is `undefined`.
 */
export function readKept(value: unknown, version: string): KeptCheck | undefined {
  if (typeof value !== "object" || value === null) return undefined;
  const kept = value as Partial<KeptCheck>;
  const summary = kept.summary as Partial<Summary> | undefined;
  const counted = kept.counted as Partial<Record<Counted, Found>> | undefined;
  const readable =
    kept.version === version &&
    typeof kept.started === "number" &&
    typeof summary === "object" &&
    summary !== null &&
    typeof summary.errors === "number" &&
    typeof summary.warnings === "number" &&
    typeof summary.advice === "number" &&
    (kept.error === null || typeof kept.error === "string") &&
    typeof kept.unsaved === "number" &&
    typeof counted === "object" &&
    counted !== null &&
    (["examples", "reviews", "permalinks"] as const).every(
      (of) => typeof counted[of]?.count === "number",
    ) &&
    Array.isArray(kept.problems) &&
    typeof kept.current === "boolean" &&
    typeof kept.files === "string";
  return readable ? (kept as KeptCheck) : undefined;
}

/** The files a check of every build follows, as they are on disk now. */
export interface ProjectFiles {
  /** Which they are: a digest of their paths, which a file added or removed changes. */
  list: string;
  /** How many there are. */
  count: number;
  /** The latest of their modification times, in milliseconds since the epoch; 0 for none. */
  newest: number;
}

/** Folders never walked, as `changesCheck` passes over their files. */
const SKIPPED = new Set(["node_modules", ".git"]);

/**
 * The files of `project` a change to which makes its check out of date, as
 * `changesCheck` follows them, with when each was last modified: its own
 * files, apart from a nested project's, its output directory and `.ascribe`,
 * and its sources' files, those a source reads (`sourceUses`) rather than
 * the whole of each folder, since a source's folder can be a whole
 * repository, with its build's output. A link is followed to a file, and
 * counted but not walked to a folder.
 */
export async function projectFiles(
  project: Project,
  layout: Layout,
  projects: readonly Project[],
): Promise<ProjectFiles> {
  const found = new Map<string, number>();
  const nested = projects.filter((other) => other.config !== project.config);
  const ownData = path.join(project.folder, ".ascribe");
  await walk(
    project.folder,
    (folder) =>
      !within(folder, layout.outputDir) &&
      !samePath(folder, ownData) &&
      !nested.some((other) => samePath(folder, other.folder)),
    (file) => changesCheck(file, project, layout, projects),
    found,
  );
  for (const source of layout.sources) {
    await walk(
      source.folder,
      (folder) => sourceUses(folder, source) && !within(folder, layout.outputDir),
      (file) => sourceUses(file, source),
      found,
    );
  }
  const files = [...found.keys()].sort();
  let newest = 0;
  for (const modified of found.values()) newest = Math.max(newest, modified);
  const list = createHash("sha256").update(files.join("\n")).digest("hex");
  return { list, count: files.length, newest };
}

/**
 * Whether the project changed after a check kept as `kept`: a file it
 * follows was modified once the check had started, or one was added or
 * removed. A branch switch rewrites the files it changes, so it's a change.
 */
export function changedSince(
  kept: Pick<KeptCheck, "started" | "files">,
  now: ProjectFiles,
): boolean {
  return now.list !== kept.files || now.newest >= kept.started;
}

/** Adds the files below `folder` that `keepFile` takes to `found`, through the folders `keepFolder` takes. */
async function walk(
  folder: string,
  keepFolder: (folder: string) => boolean,
  keepFile: (file: string) => boolean,
  found: Map<string, number>,
): Promise<void> {
  // Outside FileSystem: the editor's own walk, of names and modification times only.
  const entries = await readdir(folder, { withFileTypes: true }).catch(() => []);
  await Promise.all(
    entries.map(async (entry) => {
      const full = path.join(folder, entry.name);
      if (entry.isDirectory()) {
        if (!SKIPPED.has(entry.name) && keepFolder(full)) {
          await walk(full, keepFolder, keepFile, found);
        }
        return;
      }
      if (!keepFile(full)) return;
      const status = await stat(full).catch(() => undefined);
      if (status === undefined) return;
      // A link to a folder counts by its name alone.
      found.set(comparable(full), status.isFile() ? status.mtimeMs : 0);
    }),
  );
}
