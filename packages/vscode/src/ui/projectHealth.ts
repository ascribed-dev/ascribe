// What the Project view draws, worked out from plain values so the unit
// tests can check it without VS Code: the head, Health's counts with when
// each is from, and the actions. `projectView.ts` gathers the values and
// sends the answer; `../webview/project.ts` draws it.
//
// A count is shown as current only when it is. One that's out of date, or
// from a run that hasn't happened, shows a dash and says why, with the
// action that brings it current beside it; a count of zero shows `0`,
// dimmed. So "none" and "not checked" never look alike.

import type { ServerState } from "../serverState.js";
import type { Summary } from "../shapes.js";
import { TASKS, type TaskName } from "../tasks.js";
import type { ProjectAction, ProjectAnswer, ProjectCount } from "../webview/viewProtocol.js";
import type { EveryBuildResult } from "./checkEveryBuild.js";
import { STATE_NAMES } from "./describe.js";

/** The keys of what the Project view's items name, which a choice in it sends back. */
export const PROJECT_KEYS = {
  /** The problems in every build: opens the Problems panel. */
  problems: "problems",
  /** Check Every Build, of the view's project. */
  checkEveryBuild: "checkEveryBuild",
  /** The tasks, by their names in `tasks.json`. */
  build: "build",
  syncAgentFiles: "syncAgentFiles",
} as const;

/** The project the view shows, as plain values. */
export interface ProjectFacts {
  name: string;
  /** Its folder, as the workspace names it. */
  folder: string;
  state: ServerState;
  /** The version of the binary its server runs, once one is found. */
  version: string | undefined;
  /** Its check of every build. */
  everyBuild: EveryBuildResult;
  /** A time of day, as the view says when something ran. */
  time: (at: number) => string;
}

/** What the Project view says with no project to show. */
export const NO_PROJECT = "Open a page of a project to see the project.";

const CHECK_EVERY_BUILD: ProjectAction = {
  key: PROJECT_KEYS.checkEveryBuild,
  label: "Check Every Build",
  icon: "checklist",
  caption: "Runs ascribe check and puts every build's problems in the Problems panel.",
};

const TASK_ICONS: Record<TaskName, string> = {
  build: "package",
  syncAgentFiles: "sync",
};

/** A task, as an action: it runs in the terminal. */
function taskAction(name: TaskName): ProjectAction {
  return {
    key: name,
    label: TASKS[name].label,
    icon: TASK_ICONS[name],
    caption: TASKS[name].caption,
  };
}

/** The Project view's answer for `project`, or its message when there's none. */
export function projectAnswer(project: ProjectFacts | undefined): ProjectAnswer {
  if (!project) return { message: NO_PROJECT, head: null, toDo: 0, health: [], actions: [] };
  const health = [problemsCount(project)];
  return {
    message: null,
    head: {
      name: project.name,
      folder: project.folder,
      state: STATE_NAMES[project.state],
      version: project.version ?? null,
    },
    toDo: health.filter((count) => count.state !== "none" && count.state !== "running").length,
    health,
    actions: [CHECK_EVERY_BUILD, taskAction("build"), taskAction("syncAgentFiles")],
  };
}

/** Problems in every build, by severity, from the last Check Every Build. */
export function problemsCount({ everyBuild, time }: ProjectFacts): ProjectCount {
  const base = { key: PROJECT_KEYS.problems, label: "Problems in every build" };
  const check = { key: CHECK_EVERY_BUILD.key, label: CHECK_EVERY_BUILD.label };
  switch (everyBuild.kind) {
    case "none":
      return {
        ...base,
        state: "unknown",
        value: "—",
        note: "Needs every build checked",
        opens: false,
        action: check,
      };
    case "running":
      return {
        ...base,
        state: "running",
        value: "—",
        note: "Checking every build…",
        opens: false,
        action: null,
      };
    case "made": {
      const at = time(everyBuild.at);
      if (everyBuild.error !== null) {
        return {
          ...base,
          state: "outOfDate",
          value: "—",
          note: `The check at ${at} couldn't finish: ${everyBuild.error}`,
          opens: false,
          action: check,
        };
      }
      if (!everyBuild.current) {
        return {
          ...base,
          state: "outOfDate",
          value: "—",
          note: `Out of date: the project changed after the check at ${at}`,
          opens: false,
          action: check,
        };
      }
      const value = bySeverity(everyBuild.summary);
      const unsaved =
        everyBuild.unsaved === 0
          ? ""
          : `, ${count(everyBuild.unsaved, "file")} with unsaved changes read as saved`;
      return {
        ...base,
        state: value === "" ? "none" : "current",
        value: value || "0",
        note: `Checked at ${at}${unsaved}`,
        opens: value !== "",
        action: null,
      };
    }
  }
}

/** "2 errors, 1 warning, 3 advice", leaving out a severity there's none of; empty for none at all. */
function bySeverity({ errors, warnings, advice }: Summary): string {
  return [
    errors > 0 ? count(errors, "error") : "",
    warnings > 0 ? count(warnings, "warning") : "",
    advice > 0 ? `${advice} advice` : "",
  ]
    .filter(Boolean)
    .join(", ");
}

function count(n: number, noun: string): string {
  return `${n} ${noun}${n === 1 ? "" : "s"}`;
}
