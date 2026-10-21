// The `ascribe` task type: Build, Sync Agent Files, and Observe Examples,
// each run in the terminal by the binary the project's server runs. What
// each task runs and where, worked out from plain values so the unit tests
// can check it without VS Code; `ui/tasks.ts` provides the tasks.
//
// Build and Sync Agent Files run in the project's folder, with `--config`
// naming its `ascribe.toml` when the project isn't the workspace folder.
// Observe Examples runs `ascribe observe` in a source's folder, the one its
// `ascribe.examples.toml` is in: observe runs only the config in the folder
// it's run from, and takes no `--config`.

import * as path from "node:path";
import { parse } from "smol-toml";
import { isTable, section } from "./diskChanges.js";
import { samePath, type Project } from "./projects.js";

/** The task type, which `tasks.json` names. */
export const TASK_TYPE = "ascribe";

/** The file `ascribe observe` runs the commands of, in the folder it's run from. */
export const EXAMPLES_FILE = "ascribe.examples.toml";

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
      "A task: runs ascribe observe in the terminal, which runs the test commands ascribe.examples.toml names, after showing them and asking.",
  },
} as const;

export type TaskName = keyof typeof TASKS;

/** What a task is in `tasks.json`. */
export interface AscribeTaskDefinition {
  type: typeof TASK_TYPE;
  task: TaskName;
  /** The project's folder, relative to the workspace folder; the workspace folder's own when left out. */
  project?: string;
  /** Observe Examples only: the source whose folder to observe; the project's only one when left out. */
  source?: string;
  /** What else `tasks.json` gives it, which VS Code keeps and the task doesn't read. */
  readonly [name: string]: unknown;
}

/** Whether a value is a task's definition this extension can run. */
export function isAscribeTask(definition: {
  readonly type: string;
  readonly [name: string]: unknown;
}): definition is AscribeTaskDefinition {
  const { task, project, source } = definition as Partial<AscribeTaskDefinition>;
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
 * Build or Sync Agent Files for `project`: in its folder, with `--config`
 * naming its `ascribe.toml` when the project isn't `workspaceFolder`.
 */
export function projectCommand(
  task: Exclude<TaskName, "observeExamples">,
  project: Project,
  workspaceFolder: string,
): TaskCommand {
  const config = samePath(project.folder, workspaceFolder) ? [] : ["--config", project.config];
  return { cwd: project.folder, args: [...TASKS[task].args, ...config] };
}

/** Observe Examples for a source: `ascribe observe`, in the source's folder. */
export function observeCommand(source: ExamplesSource): TaskCommand {
  return { cwd: source.folder, args: [...TASKS.observeExamples.args] };
}

/** A source in the project's own repository, which may have tested examples. */
export interface ExamplesSource {
  /** Its name in `ascribe.toml`: `[sources.<name>]`. */
  name: string;
  /** Its folder, absolute. */
  folder: string;
}

/**
 * The sources of `ascribe.toml`'s text that are folders of this repository:
 * those with a `path` and no `git`. A source in another repository is a copy,
 * and its tests are run there. A model that doesn't parse gives none.
 */
export function localSources(project: Project, text: string): ExamplesSource[] {
  let table: Record<string, unknown>;
  try {
    table = parse(text);
  } catch {
    return [];
  }
  return Object.entries(section(table, "sources")).flatMap(([name, source]) => {
    if (!isTable(source) || source["git"] !== undefined) return [];
    const folder = source["path"];
    return typeof folder === "string"
      ? [{ name, folder: path.resolve(project.folder, folder) }]
      : [];
  });
}

/**
 * The commands an `ascribe.examples.toml`'s text names, in the order
 * `ascribe observe` runs them: `test`, then each `[environment]` entry's
 * `command`. `undefined` when the text isn't TOML; observe says what's wrong.
 */
export function examplesCommands(text: string): string[] | undefined {
  let table: Record<string, unknown>;
  try {
    table = parse(text);
  } catch {
    return undefined;
  }
  const environment = Object.values(section(table, "environment")).flatMap((entry) => {
    const command = isTable(entry) ? entry["command"] : undefined;
    return typeof command === "string" ? [command] : [];
  });
  const test = table["test"];
  return [...(typeof test === "string" ? [test] : []), ...environment];
}

/** What Observe Examples asks before it runs, each time. */
export interface ObserveQuestion {
  message: string;
  /** The commands, one a line. */
  detail: string;
}

/**
 * The question for observing `file`, an `ascribe.examples.toml` as the
 * workspace names it, with the commands it names; `undefined` commands when
 * they couldn't be read.
 */
export function observeQuestion(file: string, commands: string[] | undefined): ObserveQuestion {
  const message = `Observe Examples runs the commands ${file} names, as you. Run them?`;
  if (commands === undefined) {
    return {
      message,
      detail:
        "Ascribe couldn't read its commands: it isn't TOML. ascribe observe will say what's wrong.",
    };
  }
  if (commands.length === 0) return { message, detail: "It names no command." };
  return { message, detail: commands.join("\n") };
}

/** A task's label: its name, then the project's and the source's when there's more than one. */
export function taskLabel(task: TaskName, project?: string, source?: string): string {
  const which = [project, source].filter((part) => part !== undefined);
  return which.length === 0 ? TASKS[task].label : `${TASKS[task].label} (${which.join(", ")})`;
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
