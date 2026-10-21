// What the Project view draws, worked out from plain values so the unit
// tests can check it without VS Code: the head with its server's buttons,
// its build switch, and its settings buttons; the actions, in their groups;
// Health's counts, in groups by when each is from; and About.
// `projectView.ts` gathers the values and sends the answer;
// `../webview/project.ts` draws it. In Restricted Mode the counts are
// replaced by a sentence saying so, and every action is off.
//
// A count is shown as current only when it is. The counts from one run are
// a group, whose head says once when they're from; while they aren't
// current, each is a dash, and the head says why, with the action that
// brings them current. A count of zero shows `0`, dimmed. So "none" and "not
// checked" never look alike.

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
  ProjectCheck,
  ProjectCount,
  ProjectHead,
  ProjectHealthGroup,
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
const CHECK_AGENT_FILES: ProjectCheck = {
  key: PROJECT_KEYS.checkAgentFiles,
  label: "Check Agent Files",
};

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
  const health = healthGroups(project);
  return {
    message: null,
    notice: [],
    head: projectHead(project),
    // A count that isn't current isn't a thing to do: its group's head says so, with its action.
    toDo: health.flatMap((group) => group.counts).filter((item) => item.state === "current").length,
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

/** The counts, by the group they're in, in the order the view shows them. */
const COUNTS = {
  problems: { key: PROJECT_KEYS.problems, label: "Problems" },
  examples: { key: PROJECT_KEYS.examples, label: "Tested examples" },
  reviews: { key: PROJECT_KEYS.reviews, label: "Reviews overdue" },
  permalinks: { key: PROJECT_KEYS.permalinks, label: "Permalinks behind" },
  orphans: { key: PROJECT_KEYS.orphans, label: "Pages nothing links to" },
  unused: { key: PROJECT_KEYS.unused, label: "Unused model entries" },
  agentFiles: { key: PROJECT_KEYS.agentFiles, label: "Agent files out of date" },
} as const satisfies Record<string, Base>;

const CHECK: ProjectCheck = { key: CHECK_EVERY_BUILD.key, label: CHECK_EVERY_BUILD.label };

/** Health's groups: every build's counts, those as you type, and the agent files'. */
function healthGroups(facts: ProjectFacts): ProjectHealthGroup[] {
  return [everyBuildGroup(facts), asYouTypeGroup(facts), lastSaveGroup(facts)];
}

/** A count, current, of `value`: `0`, dimmed, for none, which opens nothing. */
function counted(base: Base, n: number, value: string): ProjectCount {
  return {
    ...base,
    state: n === 0 ? "none" : "current",
    value: n === 0 ? "0" : value,
    note: null,
    opens: n > 0,
    action: null,
  };
}

/** A group that isn't current: its counts are dashes, and its head says why, with the action that brings it current. */
function notCurrent(
  head: Pick<ProjectHealthGroup, "key" | "label">,
  bases: readonly Base[],
  state: Exclude<ProjectHealthGroup["state"], "current">,
  note: string,
  action: ProjectCheck | null,
): ProjectHealthGroup {
  return {
    ...head,
    state,
    note,
    action,
    counts: bases.map((base) => ({
      ...base,
      state,
      value: "—",
      note: null,
      opens: false,
      action: null,
    })),
  };
}

/**
 * The counts from the last Check Every Build: dashes until it has run, while
 * it runs, once it's out of date, or when it couldn't finish. Overdue reviews
 * depend on the day as well as the files, so a check from another day leaves
 * that count alone a dash, with its own note and the check, whatever
 * `isCurrent` says.
 */
export function everyBuildGroup({ everyBuild, time, day, now }: ProjectFacts): ProjectHealthGroup {
  const head = { key: "everyBuild", label: "Every build" };
  const bases = [COUNTS.problems, COUNTS.examples, COUNTS.reviews, COUNTS.permalinks];
  switch (everyBuild.kind) {
    case "none":
      return notCurrent(head, bases, "unknown", "not checked yet", CHECK);
    case "running":
      return notCurrent(head, bases, "running", "checking…", null);
    case "made": {
      const at = time(everyBuild.at);
      if (everyBuild.error !== null) {
        return notCurrent(
          head,
          bases,
          "outOfDate",
          `the check at ${at} couldn't finish: ${everyBuild.error}`,
          CHECK,
        );
      }
      if (!everyBuild.current) {
        return notCurrent(
          head,
          bases,
          "outOfDate",
          `out of date: the project changed after the check at ${at}`,
          CHECK,
        );
      }
      const severities = bySeverity(everyBuild.summary);
      const kind = (base: Base, of: Counted, noun: string) => {
        const n = everyBuild.counted[of].count;
        return counted(base, n, count(n, noun));
      };
      const checkedOn = day(everyBuild.at);
      const reviews: ProjectCount =
        checkedOn === day(now)
          ? kind(COUNTS.reviews, "reviews", "page")
          : {
              ...COUNTS.reviews,
              state: "outOfDate",
              value: "—",
              note: `checked on ${checkedOn}; reviews fall due by the day`,
              opens: false,
              action: CHECK,
            };
      const unsaved =
        everyBuild.unsaved === 0
          ? ""
          : `, ${count(everyBuild.unsaved, "file")} with unsaved changes read as saved`;
      return {
        ...head,
        state: "current",
        note: `checked at ${at}${unsaved}`,
        action: null,
        counts: [
          counted(COUNTS.problems, severities === "" ? 0 : 1, severities),
          kind(COUNTS.examples, "examples", "example"),
          reviews,
          kind(COUNTS.permalinks, "permalinks", "permalink"),
        ],
      };
    }
  }
}

/**
 * The counts from `ascribe/inventory`, which the server answers from its
 * snapshot, unsaved text included: current as you type, while the server
 * runs and has answered.
 */
export function asYouTypeGroup({ inventory, state }: ProjectFacts): ProjectHealthGroup {
  const head = { key: "asYouType", label: "As you type" };
  const bases = [COUNTS.orphans, COUNTS.unused];
  if (!inventory) {
    return state === "running" || state === "starting"
      ? notCurrent(head, bases, "running", "counting…", null)
      : notCurrent(
          head,
          bases,
          "unknown",
          `counted while the language server runs: it's ${STATE_NAMES[state]}`,
          null,
        );
  }
  const { orphans, unused } = inventory;
  return {
    ...head,
    state: "current",
    note: "",
    action: null,
    counts: [
      counted(COUNTS.orphans, orphans, count(orphans, "page")),
      counted(COUNTS.unused, unused, count(unused, "entry", "entries")),
    ],
  };
}

/**
 * The agent files out of date, from `ascribe agents sync --check`, which runs
 * again after each save of a file of the project: the head says the time of
 * the last, or that it's running.
 */
export function lastSaveGroup({ agentFiles, time }: ProjectFacts): ProjectHealthGroup {
  const head = { key: "lastSave", label: "At the last save" };
  const bases = [COUNTS.agentFiles];
  switch (agentFiles.kind) {
    case "none":
      return notCurrent(head, bases, "unknown", "not checked yet", CHECK_AGENT_FILES);
    case "running":
      return notCurrent(head, bases, "running", "checking…", null);
    case "made": {
      const at = time(agentFiles.at);
      if (agentFiles.error !== null) {
        return notCurrent(
          head,
          bases,
          "outOfDate",
          `the check at ${at} couldn't finish: ${agentFiles.error}`,
          CHECK_AGENT_FILES,
        );
      }
      if (!agentFiles.current) {
        return notCurrent(
          head,
          bases,
          "outOfDate",
          `out of date: a file changed on disk after the check at ${at}`,
          CHECK_AGENT_FILES,
        );
      }
      const n = agentFiles.stale.length;
      return {
        ...head,
        state: "current",
        note: at,
        action: null,
        counts: [counted(COUNTS.agentFiles, n, count(n, "file"))],
      };
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
