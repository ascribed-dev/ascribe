// The `ascribe` task type: Build and Sync Agent Files, each run in the
// terminal by the binary the project's server runs, in the project's folder,
// with `--config` naming its `ascribe.toml` when the project isn't the
// workspace folder; and Observe Examples, run in the folder of each of the
// project's sources with tested examples, where `ascribe observe` shows the
// commands its `ascribe.examples.toml` names and asks before running them.
// Which sources those are is the binary's answer (`ascribe model`), so the
// extension reads no TOML of its own. What each task runs and where, worked
// out from plain values so the unit tests can check it without VS Code;
// `ui/tasks.ts` provides the tasks.

import * as path from "node:path";
import { samePath, type Project } from "./projects.js";
import type { ModelReport } from "./shapes.js";

/** The task type, which `tasks.json` names. */
export const TASK_TYPE = "ascribe";

/** The tasks, by the name `tasks.json` gives each as `task`. */
export const TASKS = {
  build: {
    label: "Build",
    args: ["build"],
    caption: "A task: runs ascribe build in the terminal.",
  },
  syncAgentFiles: {
    label: "Sync Agent Files",
    args: ["agents", "sync"],
    caption: "A task: runs ascribe agents sync in the terminal.",
  },
  observeExamples: {
    label: "Observe Examples",
    args: ["observe"],
    caption:
      "A task: runs ascribe observe in the terminal, in a source's folder. It shows the commands ascribe.examples.toml names, and asks before running them.",
  },
} as const;

export type TaskName = keyof typeof TASKS;

/** The tasks run in the project's folder; Observe Examples runs in a source's. */
export type ProjectTaskName = Exclude<TaskName, "observeExamples">;

/** A source with tested examples, which Observe Examples runs in. */
export interface TestedSource {
  /** Its name in `ascribe.toml`, which `tasks.json` names as `source`. */
  name: string;
  /** Its folder, relative to the project's folder, as `ascribe.toml` writes it. */
  path: string;
}

/** What a task is in `tasks.json`. */
export interface AscribeTaskDefinition {
  type: typeof TASK_TYPE;
  task: TaskName;
  /** The project's folder, relative to the workspace folder; the workspace folder's own when left out. */
  project?: string;
  /** Observe Examples' source, by its name; the project's only source with tested examples when left out. */
  source?: string;
  /** What else `tasks.json` gives it, which VS Code keeps and the task doesn't read. */
  readonly [name: string]: unknown;
}

/** Whether a value is a task's definition this extension can run. */
export function isAscribeTask(definition: {
  readonly type: string;
  readonly [name: string]: unknown;
}): definition is AscribeTaskDefinition {
  const { task, project, source } = definition;
  return (
    definition.type === TASK_TYPE &&
    typeof task === "string" &&
    Object.hasOwn(TASKS, task) &&
    (project === undefined || typeof project === "string") &&
    (source === undefined || typeof source === "string")
  );
}

/** A program to run: its arguments after the binary, and the folder it runs in. */
export interface TaskCommand {
  cwd: string;
  args: string[];
}

/**
 * What a task runs for `project`: in its folder, with `--config` naming its
 * `ascribe.toml` when the project isn't `workspaceFolder`.
 */
export function projectCommand(
  task: ProjectTaskName,
  project: Project,
  workspaceFolder: string,
): TaskCommand {
  const config = samePath(project.folder, workspaceFolder) ? [] : ["--config", project.config];
  return { cwd: project.folder, args: [...TASKS[task].args, ...config] };
}

/**
 * What Observe Examples runs for `source` of `project`: `ascribe observe` in
 * the source's folder, which runs only the `ascribe.examples.toml` there,
 * and takes no `--config`.
 */
export function sourceCommand(project: Project, source: TestedSource): TaskCommand {
  return { cwd: path.resolve(project.folder, source.path), args: [...TASKS.observeExamples.args] };
}

/**
 * The sources with tested examples in `ascribe model --section sources
 * --format json`'s report: each with an `ascribe.examples.toml`, and in this
 * repository. A source in another repository's are copies, which `ascribe
 * observe` refuses to run.
 */
export function testedSources(report: Pick<ModelReport, "sources">): TestedSource[] {
  return (report.sources ?? [])
    .filter((source) => source.examples !== undefined && source.git === null)
    .map(({ name, path }) => ({ name, path }));
}

/**
 * A task's label: its name, then the project's and the source's, each when
 * there's more than one to tell apart.
 */
export function taskLabel(task: TaskName, project?: string, source?: string): string {
  const parts = [project, source].filter((part) => part !== undefined);
  return parts.length === 0 ? TASKS[task].label : `${TASKS[task].label} (${parts.join(", ")})`;
}

/**
 * The project's folder as a task's definition names it: relative to the
 * workspace folder, with forward slashes; `undefined` for the workspace
 * folder's own project, or a project outside it.
 */
export function projectProperty(project: Project, workspaceFolder: string): string | undefined {
  const relative = path.relative(workspaceFolder, project.folder);
  if (relative === "" || relative.startsWith("..") || path.isAbsolute(relative)) return undefined;
  return relative.split(path.sep).join("/");
}

/**
 * The source a task's definition names, among the names of the project's
 * sources with tested examples: its own, or the only one when it names none.
 */
export function definedSource(
  definition: AscribeTaskDefinition,
  sources: readonly string[],
): string | undefined {
  if (definition.source === undefined) return sources.length === 1 ? sources[0] : undefined;
  return sources.find((source) => source === definition.source);
}

/** The project a task's definition names, from the folder of its workspace folder. */
export function definedProject(
  definition: AscribeTaskDefinition,
  workspaceFolder: string,
  projects: readonly Project[],
): Project | undefined {
  const folder = path.resolve(workspaceFolder, definition.project ?? ".");
  return projects.find((project) => samePath(project.folder, folder));
}
