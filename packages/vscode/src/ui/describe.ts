// What the status bar item and Switch Project say about a project, worked
// out from plain values so the unit tests can check it without VS Code.

import * as path from "node:path";
import type { ResolvedBinary } from "../binary.js";
import { owningProject, samePath, within, type Project } from "../projects.js";
import type { ServerState } from "../serverState.js";
import { formatVersion } from "../version.js";

/** A project as the status bar and Switch Project see it. */
export interface ProjectInfo extends Project {
  /** Its name for people (`ProjectRegistry.name`). */
  name: string;
  state: ServerState;
  /** The binary its server uses, once one has been found. */
  binary: ResolvedBinary | undefined;
  /** Its builds, from `ascribe/targets`; empty until its server has answered. */
  builds: readonly { name: string; editor: boolean }[];
  /** The build you're looking at (`ChosenBuilds.shown`). */
  build: string | undefined;
  /** Whether the build lens is on: the editor dims what `build` leaves out. */
  lens: boolean;
  /**
   * Whether the Problems panel holds a check of every build
   * (**Check Every Build**) that nothing in the project has changed since.
   * Without one, it holds the editor build's problems alone.
   */
  everyBuild: boolean;
}

/**
 * The context keys about the workspace's projects, from how many the
 * registry has (those `ascribe.projects.exclude` leaves out and those past
 * the cap aren't among them): `ascribe.active` with one or more, and
 * `ascribe.multipleProjects`, which shows Switch Project, with two or more.
 */
export function projectKeys(count: number): { active: boolean; multipleProjects: boolean } {
  return { active: count > 0, multipleProjects: count >= 2 };
}

/** A server's state in words. */
export const STATE_NAMES: Record<ServerState, string> = {
  stopped: "not started",
  starting: "starting",
  running: "running",
  failed: "failed",
};

/** The codicon for a server's state, as the status bar shows it. */
const STATE_ICONS: Record<ServerState, string> = {
  stopped: "book",
  starting: "sync~spin",
  running: "book",
  failed: "warning",
};

/** The file in the active editor, if any. */
export interface ActiveFile {
  /** Its URI's scheme. */
  scheme: string;
  /** Its path, for a `file:` URI. */
  path: string;
  languageId: string;
}

/** What the status bar item shows when it isn't about the active file's project. */
export interface Notice {
  text: string;
  tooltip: string;
  /** What a screen reader says. */
  label: string;
}

/** What the status bar item shows. */
export interface StatusText extends Notice {
  /** The project it's about. */
  project: ProjectInfo;
}

/**
 * The status bar item in Restricted Mode, where the extension runs nothing:
 * without it, the Problems panel's "No problems" reads as a clean project.
 */
export const RESTRICTED_NOTICE: Notice = {
  text: "$(shield) Ascribe: not checking",
  tooltip:
    "This folder is in Restricted Mode, so Ascribe doesn't run or check its projects. Trust the folder to check them.",
  label: "Ascribe isn't checking this folder: it's in Restricted Mode",
};

/**
 * The status bar item while no page of a project is active and some project's
 * server hasn't started: a server starts when one of its project's pages
 * opens, so until then nothing has checked that project.
 */
export function waitingNotice(projects: readonly ProjectInfo[]): Notice | undefined {
  const waiting = projects.filter((p) => p.state === "stopped").map((p) => p.name);
  if (waiting.length === 0) return undefined;
  const what = waiting.length === 1 ? waiting[0] : `${waiting.length} projects`;
  return {
    text: `$(circle-slash) ${what}: not checked yet`,
    tooltip: [
      `Not checked yet: ${waiting.join(", ")}`,
      "Ascribe checks a project once one of its pages is open.",
    ].join("\n"),
    label: `Ascribe hasn't checked ${waiting.join(", ")} yet: it checks a project once one of its pages is open`,
  };
}

/**
 * The status bar item for the active file: shown only while it's a page
 * (Markdown) or an `ascribe.toml` in a project, and about the project that
 * owns it, which for a nested project's file is the nested project.
 */
export function statusFor(
  file: ActiveFile | undefined,
  projects: readonly ProjectInfo[],
): StatusText | undefined {
  if (!file || file.scheme !== "file") return undefined;
  if (file.languageId !== "markdown" && path.basename(file.path) !== "ascribe.toml") {
    return undefined;
  }
  const owner = owningProject(file.path, projects);
  const project = projects.find((p) => p.config === owner?.config);
  if (!project) return undefined;
  const build = project.state === "running" ? project.build : undefined;
  // While the lens dims by the build, the eye says so.
  const lens = build !== undefined && project.lens;
  const icon = lens ? "eye" : STATE_ICONS[project.state];
  const alone = build === undefined ? undefined : coveredAlone(project);
  const covers = alone === undefined ? "" : alone === build ? "this build only" : `${alone} only`;
  const text = `$(${icon}) ${project.name}${build ? ` · ${build}` : ""}${covers ? ` · problems: ${covers}` : ""}`;
  const state = STATE_NAMES[project.state];
  return {
    project,
    text,
    tooltip: tooltipLines(project).join("\n"),
    label: `Ascribe project ${project.name}, server ${state}${build ? `, build ${build}` : ""}${lens ? ", dimming what it leaves out" : ""}${alone ? `, problems for ${alone} only` : ""}`,
  };
}

/**
 * The build whose problems the Problems panel holds alone: the editor
 * build, while the project has other builds and no check of every build
 * is current. `undefined` when the panel covers every build.
 */
function coveredAlone(project: ProjectInfo): string | undefined {
  if (project.state !== "running" || project.everyBuild || project.builds.length < 2) {
    return undefined;
  }
  return project.builds.find((b) => b.editor)?.name;
}

/** The tooltip's lines: where the project is, the binary, the state, and the build. */
function tooltipLines(project: ProjectInfo): string[] {
  const lines = [
    `Ascribe project ${project.name}`,
    `Folder: ${project.folder}`,
    `Content model: ${project.config}`,
  ];
  if (project.binary) lines.push(`Binary: ${binaryText(project.binary)}`);
  lines.push(`Server: ${STATE_NAMES[project.state]}`);
  if (project.state === "running" && project.build) {
    const editor = project.builds.find((b) => b.editor)?.name;
    lines.push(
      `Build: ${project.build}${editor === project.build ? " (the editor build)" : editor ? ` (the editor build is ${editor})` : ""}`,
    );
    if (project.lens) lines.push(`Dimming what ${project.build} leaves out`);
    const alone = coveredAlone(project);
    if (alone !== undefined) {
      lines.push(
        `Problems: ${alone} only. Ascribe: Check Every Build adds the problems of the others.`,
      );
    } else if (project.everyBuild) {
      lines.push("Problems: every build, from Ascribe: Check Every Build");
    }
  }
  return lines;
}

/** A binary as people read it: its path, where it was found, and its version. */
function binaryText(binary: ResolvedBinary): string {
  return `${binary.path} (${binary.source}, ${formatVersion(binary.version)})`;
}

/** A project as Switch Project lists it. */
export interface SwitchItem {
  /** The project's `ascribe.toml`, which names it. */
  config: string;
  folder: string;
  /** Its name, after a check for the sidebar's project. */
  label: string;
  /** Its folder. */
  description: string;
  /** Its server's state, and its binary once one has been found. */
  detail: string;
  /** Whether it's the sidebar's project. */
  current: boolean;
  /** Its item buttons: Restart only on a server that is running or has failed. */
  buttons: SwitchButton[];
}

/** A button on one of Switch Project's items. */
export type SwitchButton = "showOutput" | "restart";

/**
 * Switch Project's items: every project, the sidebar's first and marked,
 * then the rest in path order, each with its folder (as `show` writes it)
 * and its server's state.
 */
export function switchItems(
  projects: readonly ProjectInfo[],
  current: string | undefined,
  show: (folder: string) => string,
): SwitchItem[] {
  const isCurrent = (project: ProjectInfo) =>
    current !== undefined && samePath(project.config, current);
  const ordered = [...projects.filter(isCurrent), ...projects.filter((p) => !isCurrent(p))];
  return ordered.map((project) => {
    const here = isCurrent(project);
    const detail = [switchState(project)];
    if (project.binary) detail.push(binaryText(project.binary));
    return {
      config: project.config,
      folder: project.folder,
      label: `${here ? "$(check)" : "$(blank)"} ${project.name}`,
      description: show(project.folder),
      detail: detail.join(" · "),
      current: here,
      buttons:
        project.state === "running" || project.state === "failed"
          ? ["showOutput", "restart"]
          : ["showOutput"],
    };
  });
}

/** A server's state as Switch Project says it: running with its build, or why it isn't. */
function switchState(project: ProjectInfo): string {
  switch (project.state) {
    case "running":
      return project.build ? `running, ${project.build} build` : "running";
    case "starting":
      return "starting";
    case "stopped":
      return "not checked yet";
    case "failed":
      return "failed";
  }
}

/** The item of the project in `folder`, compared as the platform compares paths. */
export function switchItemAt<T extends { folder: string }>(
  items: readonly T[],
  folder: string,
): T | undefined {
  return items.find((item) => samePath(item.folder, folder));
}

/** A folder with the home directory written `~`. */
export function tildeFolder(folder: string, home: string): string {
  if (home === "" || !within(folder, home)) return folder;
  const base = home.replace(/[\\/]+$/, "");
  return `~${folder.slice(base.length)}`;
}
