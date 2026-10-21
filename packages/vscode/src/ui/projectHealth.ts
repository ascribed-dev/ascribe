// What the Project view draws, worked out from plain values so the unit
// tests can check it without VS Code: the head with its server's buttons,
// its build switch, and its settings buttons; Health's counts with when each
// is from; the actions, in their groups; and About. `projectView.ts` gathers
// the values and sends the answer; `../webview/project.ts` draws it. In
// Restricted Mode the counts are replaced by a sentence saying so, and every
// action is off.
//
// A count is shown as current only when it is. One that's out of date, or
// from a run that hasn't happened, shows a dash and says why, with the
// action that brings it current beside it; a count of zero shows `0`,
// dimmed. So "none" and "not checked" never look alike.

import type { Counted } from "../everyBuild.js";
import type { ServerState } from "../serverState.js";
import type { InventoryResult, Summary } from "../shapes.js";
import { TASKS, type TaskName } from "../tasks.js";
import type {
  ProjectAboutRow,
  ProjectAction,
  ProjectActionGroup,
  ProjectAnswer,
  ProjectButton,
  ProjectCount,
  ProjectHead,
} from "../webview/viewProtocol.js";
import type { AgentFilesResult } from "./agentFiles.js";
import type { EveryBuildResult } from "./checkEveryBuild.js";
import { choices, lineDescription, type ListedBuild } from "./chosenBuild.js";
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
  /** Review and agents: the same commands the Command Palette runs, for the view's project. */
  startReview: "startReview",
  promptAgent: "promptAgent",
  /** The head's buttons: the server's, and the two settings. */
  showOutput: "showOutput",
  restart: "restart",
  projectSettings: "projectSettings",
  editorSettings: "editorSettings",
  /** Restricted Mode's button: VS Code's Workspace Trust editor. */
  manageTrust: "manageTrust",
} as const;

/**
 * What the build switch sends, before the build's name: `chooseBuild/site@3.4`.
 * It sets the same choice the status bar's Switch Build and the preview's
 * picker set (`ChosenBuilds`).
 */
export const CHOOSE_BUILD = "chooseBuild/";

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
  /** Its builds, as its server lists them; empty until it has. */
  builds: readonly ListedBuild[];
  /** The build you're looking at (`ChosenBuilds.shown`). */
  build: string | undefined;
  /** What About says. */
  about: AboutFacts;
}

/** What the Project view's About is made from. */
export interface AboutFacts {
  /** The content root, from the project's folder; `undefined` until the server has said. */
  contentRoot: string | undefined;
  /** The content types, each with how many pages have it; `undefined` until the server has said. */
  types: readonly { key: string; pages: number | null }[] | undefined;
  /**
   * The sources, as `ascribe model` answers: each with its folder, and
   * whether it has tested examples. `asking` while it hasn't answered,
   * `failed` when it couldn't, as for a content model with errors.
   */
  sources: readonly { name: string; path: string; tested: boolean }[] | "asking" | "failed";
}

/** What the Project view says with no project to show. */
export const NO_PROJECT = "Open a page of a project to see the project.";

/** What the Project view says in Restricted Mode, in place of the counts. */
export const RESTRICTED =
  "This folder is in Restricted Mode, so Ascribe isn't running in it: nothing is counted, and no action runs until you trust the folder.";

const CHECK_EVERY_BUILD: ProjectAction = {
  key: PROJECT_KEYS.checkEveryBuild,
  label: "Check Every Build",
  icon: "checklist",
  caption: "Runs ascribe check and puts every build's problems in the Problems panel.",
  off: false,
};

const START_REVIEW: ProjectAction = {
  key: PROJECT_KEYS.startReview,
  label: "Start Review",
  icon: "git-compare",
  caption:
    "Marks what changed in the preview, against a base it asks for, as Ascribe: Start Review does.",
  off: false,
};

const PROMPT_AGENT: ProjectAction = {
  key: PROJECT_KEYS.promptAgent,
  label: "Prompt Agent to Fix This Project",
  icon: "sparkle",
  caption:
    "Builds a prompt about the project's problems for your agent, and puts it where ascribe.agents.promptTarget says, without sending it.",
  off: false,
};

/** The head's buttons beside the server's state. */
const SERVER_BUTTONS: ProjectButton[] = [
  {
    key: PROJECT_KEYS.showOutput,
    label: "Show Output",
    icon: "output",
    tooltip: "Show Output: the language server's log",
  },
  {
    key: PROJECT_KEYS.restart,
    label: "Restart",
    icon: "debug-restart",
    tooltip: "Restart the language server",
  },
];

/** The head's buttons beside the project's name. */
const SETTINGS_BUTTONS: ProjectButton[] = [
  {
    key: PROJECT_KEYS.projectSettings,
    label: "Project Settings",
    icon: "settings-gear",
    tooltip: "Project Settings: the project's ascribe.toml, which everyone shares",
  },
  {
    key: PROJECT_KEYS.editorSettings,
    label: "Ascribe's Settings in VS Code",
    icon: "settings",
    tooltip: "Ascribe's Settings in VS Code: your editor's, which nobody else gets",
  },
];

/** The project edits, by the kind `ascribe/projectEdit` names. */
export const PROJECT_EDITS = {
  fixAll: {
    key: PROJECT_KEYS.applyEverySafeFix,
    label: "Apply Every Safe Fix",
    icon: "lightbulb-autofix",
    caption:
      "Applies the safe fixes of the problems the editor shows, the editor build's, in every file, as one edit left unsaved. Every build's are ascribe check --fix's.",
    off: false,
  },
  format: {
    key: PROJECT_KEYS.formatEveryPage,
    label: "Format Every Page",
    icon: "edit",
    caption: "Formats every page as ascribe fmt does, as one edit left unsaved.",
    off: false,
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
    off: false,
  };
}

/** The actions, in their groups, as the proposal orders them. */
function actionGroups(): ProjectActionGroup[] {
  return [
    { label: "Check", actions: [CHECK_EVERY_BUILD] },
    {
      label: "Write",
      actions: [
        PROJECT_EDITS.fixAll,
        PROJECT_EDITS.format,
        taskAction("build"),
        taskAction("syncAgentFiles"),
        taskAction("observeExamples"),
      ],
    },
    { label: "Review and agents", actions: [START_REVIEW, PROMPT_AGENT] },
  ];
}

/**
 * The Project view's answer in Restricted Mode, where the extension runs
 * nothing: a sentence in place of the counts, with Manage Workspace Trust,
 * and every action off.
 */
export function restrictedAnswer(): ProjectAnswer {
  return {
    message: RESTRICTED,
    notice: [{ key: PROJECT_KEYS.manageTrust, label: "Manage Workspace Trust" }],
    head: null,
    toDo: 0,
    health: [],
    actions: actionGroups().map((group) => ({
      ...group,
      actions: group.actions.map((action) => ({ ...action, off: true })),
    })),
    about: [],
  };
}

/** The Project view's answer for `project`, or its message when there's none. */
export function projectAnswer(project: ProjectFacts | undefined): ProjectAnswer {
  if (!project) {
    return {
      message: NO_PROJECT,
      notice: [],
      head: null,
      toDo: 0,
      health: [],
      actions: [],
      about: [],
    };
  }
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
    notice: [],
    head: projectHead(project),
    toDo: health.filter((count) => count.state !== "none" && count.state !== "running").length,
    health,
    actions: actionGroups(),
    about: aboutRows(project),
  };
}

/** The head: the project, its server with its buttons, the build switch, and the settings buttons. */
function projectHead(project: ProjectFacts): ProjectHead {
  const offered = choices(project.builds);
  const shown = project.build ?? offered.find((choice) => choice.editor)?.name;
  return {
    name: project.name,
    folder: project.folder,
    state: STATE_NAMES[project.state],
    version: project.version ?? null,
    // Show Output always has something to show; Restart, once the server has started.
    server: SERVER_BUTTONS.filter(
      (button) => button.key !== PROJECT_KEYS.restart || project.state !== "stopped",
    ),
    settings: SETTINGS_BUTTONS,
    build:
      offered.length === 0 || shown === undefined
        ? null
        : {
            shown,
            choices: offered.map((choice) => ({
              name: choice.name,
              description: [lineDescription(choice), choice.editor ? "editor build" : ""]
                .filter(Boolean)
                .join(", "),
            })),
          },
  };
}

/** What About says while the language server hasn't said. */
const UNTIL_RUNNING = "Known once the language server runs";

/** About: the content root, the builds, the release lines, the sources, and the content types. */
export function aboutRows({ about, builds }: ProjectFacts): ProjectAboutRow[] {
  const known = builds.length > 0;
  const versioned = builds.filter((build) => (build.lines ?? []).length > 0);
  const { sources } = about;
  return [
    { label: "Content root", value: about.contentRoot ?? UNTIL_RUNNING },
    {
      label: "Builds",
      value: known
        ? builds
            .map((build) => (build.editor ? `${build.name} (editor build)` : build.name))
            .join(", ")
        : UNTIL_RUNNING,
    },
    {
      label: "Release lines",
      value: !known
        ? UNTIL_RUNNING
        : versioned.length === 0
          ? "None: no build is versioned"
          : versioned
              .map(
                (build) =>
                  `${build.name}: ${(build.lines ?? [])
                    .map((line) => (line.current ? `${line.line} (current)` : line.line))
                    .join(", ")}`,
              )
              .join("; "),
    },
    {
      label: "Sources",
      value:
        sources === "asking"
          ? "Asking ascribe model…"
          : sources === "failed"
            ? "Not known: ascribe model couldn't answer"
            : sources.length === 0
              ? "None"
              : sources
                  .map(
                    (source) =>
                      `${source.name} (${source.path}${source.tested ? ", tested examples" : ""})`,
                  )
                  .join(", "),
    },
    {
      label: "Content types",
      value:
        about.types === undefined
          ? UNTIL_RUNNING
          : about.types
              .map((type) =>
                type.pages === null ? type.key : `${type.key} (${count(type.pages, "page")})`,
              )
              .join(", "),
    },
  ];
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
