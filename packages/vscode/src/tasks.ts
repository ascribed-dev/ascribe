// The `ascribe` task type: Build and Sync Agent Files, each run in the
// terminal by the binary the project's server runs, in the project's folder,
// with `--config` naming its `ascribe.toml` when the project isn't the
// workspace folder. What each task runs and where, worked out from plain
// values so the unit tests can check it without VS Code; `ui/tasks.ts`
// provides the tasks.

import * as path from "node:path";
import { samePath, type Project } from "./projects.js";

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
} as const;

export type TaskName = keyof typeof TASKS;

/** What a task is in `tasks.json`. */
export interface AscribeTaskDefinition {
  type: typeof TASK_TYPE;
  task: TaskName;
  /** The project's folder, relative to the workspace folder; the workspace folder's own when left out. */
  project?: string;
  /** What else `tasks.json` gives it, which VS Code keeps and the task doesn't read. */
  readonly [name: string]: unknown;
}

/** Whether a value is a task's definition this extension can run. */
export function isAscribeTask(definition: {
  readonly type: string;
  readonly [name: string]: unknown;
}): definition is AscribeTaskDefinition {
  const { task, project } = definition;
  return (
    definition.type === TASK_TYPE &&
    typeof task === "string" &&
    Object.hasOwn(TASKS, task) &&
    (project === undefined || typeof project === "string")
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
  task: TaskName,
  project: Project,
  workspaceFolder: string,
): TaskCommand {
  const config = samePath(project.folder, workspaceFolder) ? [] : ["--config", project.config];
  return { cwd: project.folder, args: [...TASKS[task].args, ...config] };
}

/** A task's label: its name, then the project's when there's more than one to tell apart. */
export function taskLabel(task: TaskName, project?: string): string {
  return project === undefined ? TASKS[task].label : `${TASKS[task].label} (${project})`;
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

/** The project a task's definition names, from the folder of its workspace folder. */
export function definedProject(
  definition: AscribeTaskDefinition,
  workspaceFolder: string,
  projects: readonly Project[],
): Project | undefined {
  const folder = path.resolve(workspaceFolder, definition.project ?? ".");
  return projects.find((project) => samePath(project.folder, folder));
}
