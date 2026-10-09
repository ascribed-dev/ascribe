// What the status bar item and the Projects view say about a project, worked
// out from plain values so the unit tests can check it without VS Code.

import * as path from "node:path";
import type { ResolvedBinary } from "../binary.js";
import { owningProject, within, type Project } from "../projects.js";
import type { ServerState } from "../serverState.js";
import { formatVersion } from "../version.js";

/** A project as the status bar and the Projects view see it. */
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
}

/** A server's state in words. */
export const STATE_NAMES: Record<ServerState, string> = {
  stopped: "not started",
  starting: "starting",
  running: "running",
  failed: "failed",
};

/** The codicon for a server's state, as the status bar and the Projects view show it. */
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

/** What the status bar item shows. */
export interface StatusText {
  /** The project it's about. */
  project: ProjectInfo;
  text: string;
  tooltip: string;
  /** What a screen reader says. */
  label: string;
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
  const text = `$(${STATE_ICONS[project.state]}) ${project.name}${build ? ` · ${build}` : ""}`;
  const state = STATE_NAMES[project.state];
  return {
    project,
    text,
    tooltip: tooltipLines(project).join("\n"),
    label: `Ascribe project ${project.name}, server ${state}${build ? `, build ${build}` : ""}`,
  };
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
  }
  return lines;
}

/** A binary as people read it: its path, where it was found, and its version. */
function binaryText(binary: ResolvedBinary): string {
  return `${binary.path} (${binary.source}, ${formatVersion(binary.version)})`;
}

/** An item of the Projects view. */
export type ProjectNode =
  | { kind: "project"; project: ProjectInfo }
  | { kind: "model"; project: ProjectInfo }
  | { kind: "build"; project: ProjectInfo; build: string }
  | { kind: "binary"; project: ProjectInfo; binary: ResolvedBinary };

/** How an item of the Projects view looks. */
export interface NodeLook {
  label: string;
  description: string;
  tooltip: string;
  /** A codicon, or `undefined` for the file's own icon. */
  icon: string | undefined;
  /** Whether it has children. */
  expandable: boolean;
  /** What the view's menus match on (`viewItem`). */
  contextValue: string;
}

/** The Projects view's top level: every project, started or not, in path order. */
export function projectNodes(projects: readonly ProjectInfo[]): ProjectNode[] {
  return projects.map((project) => ({ kind: "project", project }));
}

/** A project's children: its `ascribe.toml`, and while it's running its editor build and binary. */
export function childNodes(node: ProjectNode): ProjectNode[] {
  if (node.kind !== "project") return [];
  const { project } = node;
  const children: ProjectNode[] = [{ kind: "model", project }];
  if (project.state !== "running") return children;
  const editor = project.builds.find((b) => b.editor)?.name;
  if (editor !== undefined) children.push({ kind: "build", project, build: editor });
  if (project.binary) children.push({ kind: "binary", project, binary: project.binary });
  return children;
}

/**
 * How an item looks. A project's `contextValue` names its state,
 * `ascribe.project.<state>`, so Restart is offered only for a running or
 * failed server.
 */
export function nodeLook(node: ProjectNode, show: (folder: string) => string): NodeLook {
  switch (node.kind) {
    case "project":
      return {
        label: node.project.name,
        description: show(node.project.folder),
        tooltip: tooltipLines(node.project).join("\n"),
        icon: STATE_ICONS[node.project.state],
        expandable: true,
        contextValue: `ascribe.project.${node.project.state}`,
      };
    case "model":
      return {
        label: path.basename(node.project.config),
        description: "content model",
        tooltip: node.project.config,
        icon: undefined,
        expandable: false,
        contextValue: "ascribe.model",
      };
    case "build":
      return {
        label: node.build,
        description: "editor build",
        tooltip: `The editor build, [editor] build in ascribe.toml, which decides the diagnostics`,
        icon: "package",
        expandable: false,
        contextValue: "ascribe.build",
      };
    case "binary":
      return {
        label: `ascribe ${formatVersion(node.binary.version)}`,
        description: node.binary.source,
        tooltip: binaryText(node.binary),
        icon: "terminal",
        expandable: false,
        contextValue: "ascribe.binary",
      };
  }
}

/** A folder with the home directory written `~`. */
export function tildeFolder(folder: string, home: string): string {
  if (home === "" || !within(folder, home)) return folder;
  const base = home.replace(/[\\/]+$/, "");
  return `~${folder.slice(base.length)}`;
}
