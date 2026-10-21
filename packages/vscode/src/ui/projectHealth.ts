// What the Project view draws, worked out from plain values so the unit
// tests can check it without VS Code: the head, Health's counts with when
// each is from, and the actions. `projectView.ts` gathers the values and
// sends the answer; `../webview/project.ts` draws it.
//
// A count is shown as current only when it is. One that's out of date, or
// from a run that hasn't happened, shows a dash and says why, with the
// action that brings it current beside it; a count of zero shows `0`,
// dimmed. So "none" and "not checked" never look alike.

import type { Counted } from "../everyBuild.js";
import type { ServerState } from "../serverState.js";
import type { InventoryResult, Summary } from "../shapes.js";
import { TASKS, type TaskName } from "../tasks.js";
import type { ProjectAction, ProjectAnswer, ProjectCount } from "../webview/viewProtocol.js";
import type { AgentFilesResult } from "./agentFiles.js";
import type { EveryBuildResult } from "./checkEveryBuild.js";
import { STATE_NAMES } from "./describe.js";

/** The keys of what the Project view's items name, which a choice in it sends back. */
export const PROJECT_KEYS = {
  /** The problems in every build: opens the Problems panel. */
  problems: "problems",
  /** Tested examples changed since they were tested, or never tested: the Problems panel, at the first. */
  examples: "examples",
  /** Reviews overdue: the Problems panel, at the first. */
  reviews: "reviews",
  /** A permalinks list that's behind: the Problems panel, at it. */
  permalinks: "permalinks",
  /** Pages nothing links to or includes: Pages, with Orphans Only. */
  orphans: "orphans",
  /** Content model entries nothing uses: Content model, with Unused Only. */
  unused: "unused",
  /** Agent files out of date: Sync Agent Files, with the files it would change. */
  agentFiles: "agentFiles",
  /** Check Every Build, of the view's project. */
  checkEveryBuild: "checkEveryBuild",
  /** The edits to the whole project, through the language server. */
  applyEverySafeFix: "applyEverySafeFix",
  formatEveryPage: "formatEveryPage",
  /** `ascribe agents sync --check`, of the view's project. */
  checkAgentFiles: "checkAgentFiles",
  /** The tasks, by their names in `tasks.json`. */
  build: "build",
  syncAgentFiles: "syncAgentFiles",
  observeExamples: "observeExamples",
} as const;

/** What `ascribe/inventory` counts, from the view's project's server. */
export interface InventoryCounts {
  /** Pages nothing links to or includes. */
  orphans: number;
  /** Content model entries nothing uses. */
  unused: number;
}

/** What the Project view counts of an inventory. */
export function inventoryCounts(inventory: InventoryResult): InventoryCounts {
  return {
    orphans: inventory.orphans.length,
    // An entry whose uses aren't counted (`null`) isn't unused.
    unused: inventory.model.filter((entry) => entry.uses === 0).length,
  };
}

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
  /** What its server's inventory counts, while it has one. */
  inventory: InventoryCounts | undefined;
  /** Its check of its agent files. */
  agentFiles: AgentFilesResult;
  /** Now, in milliseconds since the epoch: overdue reviews are counted for a day. */
  now: number;
  /** A time of day, as the view says when something ran. */
  time: (at: number) => string;
  /** A day, as the view says which day overdue reviews are for: the same text for the same day. */
  day: (at: number) => string;
}

/** What the Project view says with no project to show. */
export const NO_PROJECT = "Open a page of a project to see the project.";

const CHECK_EVERY_BUILD: ProjectAction = {
  key: PROJECT_KEYS.checkEveryBuild,
  label: "Check Every Build",
  icon: "checklist",
  caption: "Runs ascribe check and puts every build's problems in the Problems panel.",
};

/** The project edits, by the kind `ascribe/projectEdit` names. */
export const PROJECT_EDITS = {
  fixAll: {
    key: PROJECT_KEYS.applyEverySafeFix,
    label: "Apply Every Safe Fix",
    icon: "lightbulb-autofix",
    caption:
      "Applies the safe fixes of the problems the editor shows, the editor build's, in every file, as one edit left unsaved. Every build's are ascribe check --fix's.",
  },
  format: {
    key: PROJECT_KEYS.formatEveryPage,
    label: "Format Every Page",
    icon: "edit",
    caption: "Formats every page as ascribe fmt does, as one edit left unsaved.",
  },
} as const satisfies Record<ProjectEditKind, ProjectAction>;

/** Which edit to the whole project `ascribe/projectEdit` makes. */
export type ProjectEditKind = "fixAll" | "format";

/**
 * What the extension asks before applying a project edit, naming its counts:
 * "Apply 6 safe fixes in 4 files? They're left unsaved, to read before you
 * save them." `fixes` is `null` for formatting.
 */
export function projectEditQuestion(files: number, fixes: number | null): string {
  const unsaved =
    files === 1
      ? "It's left unsaved, to read before you save it."
      : "They're left unsaved, to read before you save them.";
  const asked =
    fixes === null
      ? `Format ${count(files, "file")}?`
      : `Apply ${count(fixes, "safe fix", "safe fixes")} in ${count(files, "file")}?`;
  return `${asked} ${unsaved}`;
}

/** What the extension says when a project edit would change nothing in the project `name`. */
export function nothingToEdit(kind: ProjectEditKind, name: string): string {
  return kind === "fixAll"
    ? `Ascribe: none of the problems the editor shows in ${name} has a safe fix.`
    : `Ascribe: every page of ${name} is formatted already.`;
}
const CHECK_AGENT_FILES = { key: PROJECT_KEYS.checkAgentFiles, label: "Check Agent Files" };

const TASK_ICONS: Record<TaskName, string> = {
  build: "package",
  syncAgentFiles: "sync",
  observeExamples: "beaker",
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
  const health = [
    problemsCount(project),
    examplesCount(project),
    reviewsCount(project),
    permalinksCount(project),
    orphansCount(project),
    unusedCount(project),
    agentFilesCount(project),
  ];
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
    actions: [
      CHECK_EVERY_BUILD,
      PROJECT_EDITS.fixAll,
      PROJECT_EDITS.format,
      taskAction("build"),
      taskAction("syncAgentFiles"),
      taskAction("observeExamples"),
    ],
  };
}

type Base = Pick<ProjectCount, "key" | "label">;

/** A count, current, of `value`: `0`, dimmed, for none, which opens nothing. */
function counted(base: Base, n: number, value: string, note: string): ProjectCount {
  return {
    ...base,
    state: n === 0 ? "none" : "current",
    value: n === 0 ? "0" : value,
    note,
    opens: n > 0,
    action: null,
  };
}

/** A count that isn't current: a dash, why, and the action that brings it current. */
function dash(
  base: Base,
  state: "outOfDate" | "unknown" | "running",
  note: string,
  action: ProjectCount["action"],
): ProjectCount {
  return { ...base, state, value: "—", note, opens: false, action };
}

/**
 * A count from the last Check Every Build: a dash until it has run, while it
 * runs, once it's out of date, or when it couldn't finish; else `current`.
 */
function fromCheck(
  base: Base,
  { everyBuild, time }: ProjectFacts,
  current: (made: Extract<EveryBuildResult, { kind: "made" }>, at: string) => ProjectCount,
): ProjectCount {
  const check = { key: CHECK_EVERY_BUILD.key, label: CHECK_EVERY_BUILD.label };
  switch (everyBuild.kind) {
    case "none":
      return dash(base, "unknown", "Needs every build checked", check);
    case "running":
      return dash(base, "running", "Checking every build…", null);
    case "made": {
      const at = time(everyBuild.at);
      if (everyBuild.error !== null) {
        return dash(
          base,
          "outOfDate",
          `The check at ${at} couldn't finish: ${everyBuild.error}`,
          check,
        );
      }
      if (!everyBuild.current) {
        return dash(
          base,
          "outOfDate",
          `Out of date: the project changed after the check at ${at}`,
          check,
        );
      }
      return current(everyBuild, at);
    }
  }
}

/** Problems in every build, by severity, from the last Check Every Build. */
export function problemsCount(facts: ProjectFacts): ProjectCount {
  const base = { key: PROJECT_KEYS.problems, label: "Problems in every build" };
  return fromCheck(base, facts, (made, at) => {
    const value = bySeverity(made.summary);
    const unsaved =
      made.unsaved === 0
        ? ""
        : `, ${count(made.unsaved, "file")} with unsaved changes read as saved`;
    return counted(base, value === "" ? 0 : 1, value, `Checked at ${at}${unsaved}`);
  });
}

/** One kind of problem the last Check Every Build found, as a count of `noun`. */
function checkedKind(
  facts: ProjectFacts,
  base: Base,
  kind: Counted,
  noun: string,
  note = (at: string) => `Checked at ${at}`,
): ProjectCount {
  return fromCheck(base, facts, (made, at) => {
    const n = made.counted[kind].count;
    return counted(base, n, count(n, noun), note(at));
  });
}

/** Tested examples changed since they were tested, or never tested. */
export function examplesCount(facts: ProjectFacts): ProjectCount {
  const base = { key: PROJECT_KEYS.examples, label: "Tested examples changed or never tested" };
  return checkedKind(facts, base, "examples", "example");
}

/**
 * Reviews overdue. They depend on the day as well as the files, so the count
 * says the day it's for, and goes out of date when the day changes, whatever
 * `isCurrent` says.
 */
export function reviewsCount(facts: ProjectFacts): ProjectCount {
  const base = { key: PROJECT_KEYS.reviews, label: "Reviews overdue" };
  const { everyBuild, day, now } = facts;
  if (
    everyBuild.kind === "made" &&
    everyBuild.error === null &&
    everyBuild.current &&
    day(everyBuild.at) !== day(now)
  ) {
    return dash(
      base,
      "outOfDate",
      `Out of date: checked on ${day(everyBuild.at)}, and reviews fall due by the day`,
      {
        key: CHECK_EVERY_BUILD.key,
        label: CHECK_EVERY_BUILD.label,
      },
    );
  }
  return checkedKind(
    facts,
    base,
    "reviews",
    "page",
    (at) => `Checked at ${at}, for ${everyBuild.kind === "made" ? day(everyBuild.at) : ""}`,
  );
}

/** Permalinks the content declares that `ascribe.permalinks` doesn't list as they are. */
export function permalinksCount(facts: ProjectFacts): ProjectCount {
  const base = { key: PROJECT_KEYS.permalinks, label: "Permalinks the list is behind on" };
  return checkedKind(facts, base, "permalinks", "permalink");
}

/**
 * A count from `ascribe/inventory`, which the server answers from its
 * snapshot, unsaved text included: current as you type, while the server
 * runs and has answered.
 */
function fromInventory(
  base: Base,
  facts: ProjectFacts,
  n: (counts: InventoryCounts) => number,
  noun: string,
  plural: string,
): ProjectCount {
  if (!facts.inventory) {
    return facts.state === "running" || facts.state === "starting"
      ? dash(base, "running", "Counting…", null)
      : dash(
          base,
          "unknown",
          `Counted while the language server runs: it's ${STATE_NAMES[facts.state]}`,
          null,
        );
  }
  const value = n(facts.inventory);
  return counted(
    base,
    value,
    `${value} ${value === 1 ? noun : plural}`,
    "As the pages are now, unsaved changes too",
  );
}

/** Pages nothing links to or includes, from `ascribe/inventory`'s orphans. */
export function orphansCount(facts: ProjectFacts): ProjectCount {
  const base = { key: PROJECT_KEYS.orphans, label: "Pages nothing links to or includes" };
  return fromInventory(base, facts, (counts) => counts.orphans, "page", "pages");
}

/** Content model entries nothing uses, from `ascribe/inventory`'s uses. */
export function unusedCount(facts: ProjectFacts): ProjectCount {
  const base = { key: PROJECT_KEYS.unused, label: "Content model entries nothing uses" };
  return fromInventory(base, facts, (counts) => counts.unused, "entry", "entries");
}

/**
 * Agent files out of date, from `ascribe agents sync --check`, which runs
 * again after each save of a file of the project: the count says it's from
 * the last save, or that it's running.
 */
export function agentFilesCount({ agentFiles, time }: ProjectFacts): ProjectCount {
  const base = { key: PROJECT_KEYS.agentFiles, label: "Agent files out of date" };
  switch (agentFiles.kind) {
    case "none":
      return dash(base, "unknown", "Needs the agent files checked", CHECK_AGENT_FILES);
    case "running":
      return dash(base, "running", "Checking the agent files…", null);
    case "made": {
      const at = time(agentFiles.at);
      if (agentFiles.error !== null) {
        return dash(
          base,
          "outOfDate",
          `The check at ${at} couldn't finish: ${agentFiles.error}`,
          CHECK_AGENT_FILES,
        );
      }
      if (!agentFiles.current) {
        return dash(
          base,
          "outOfDate",
          `Out of date: a file changed on disk after the check at ${at}`,
          CHECK_AGENT_FILES,
        );
      }
      const n = agentFiles.stale.length;
      return counted(base, n, count(n, "file"), `Checked at ${at}, after the last save`);
    }
  }
}

/** The files `ascribe agents sync --check` lists as `stale`, in its order. */
export function staleFiles(stdout: string): string[] {
  return stdout
    .split(/\r?\n/)
    .flatMap((line) => (line.startsWith("stale ") ? [line.slice("stale ".length).trim()] : []));
}

/** What Sync Agent Files asks before it runs. */
export function syncQuestion(
  project: string,
  stale: readonly string[],
): { message: string; detail: string } {
  const files = stale.length === 1 ? "1 agent file is" : `${stale.length} agent files are`;
  return {
    message: `${files} out of date in ${project}. Sync Agent Files?`,
    detail: `ascribe agents sync would change:\n${stale.join("\n")}`,
  };
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

function count(n: number, noun: string, nouns = `${noun}s`): string {
  return `${n} ${n === 1 ? noun : nouns}`;
}
