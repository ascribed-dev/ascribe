// What the preview keeps and says for each project: the build chosen in it,
// and why there is no page when there isn't one.

import type { ServerState } from "../client.js";
import { comparable, within } from "../projects.js";
import type { PreviewResult, ShownProblem } from "./protocol.js";

/**
 * The build chosen in the preview for each project, by project folder. A
 * project with no choice previews its editor's build.
 */
export class BuildChoices {
  private readonly chosen = new Map<string, string>();

  get(folder: string): string | undefined {
    return this.chosen.get(comparable(folder));
  }

  /** Records a project's choice; `undefined` goes back to its editor's build. */
  set(folder: string, build: string | undefined): void {
    if (build === undefined) this.chosen.delete(comparable(folder));
    else this.chosen.set(comparable(folder), build);
  }
}

/** Where the previewed file stands, for choosing what the preview says. */
export interface PreviewSituation {
  /** The previewed file's path; `undefined` before an Ascribe file has been active. */
  file: string | undefined;
  /** The name of the project that owns the file; `undefined` when none does. */
  project: string | undefined;
  /** The state of that project's server. */
  state: ServerState | undefined;
  /** The server's answer, if it gave one. */
  result: PreviewResult | undefined;
  /** How a path is shown: relative to the workspace when it's inside it. */
  show: (path: string) => string;
}

/**
 * What the preview says about a file: the server's problems when it answered
 * for a page, and otherwise why there is no page. A file in its project's
 * folder but outside the content root gets a note that names both, in place
 * of the server's note about it (an info).
 */
export function previewProblems(situation: PreviewSituation): ShownProblem[] {
  const { file, project, state, result, show } = situation;
  if (file === undefined) return [info("Open an Ascribe page to preview it.")];
  if (project === undefined) {
    return [info("This file isn't part of an Ascribe project (no ascribe.toml above it).")];
  }
  if (state === "failed") {
    return [
      {
        severity: "error",
        message: `The language server for ${project} failed, so there is nothing to preview. Its output says why.`,
        action: "showOutput",
      },
    ];
  }
  if (!result) {
    return [
      {
        severity: "info",
        message: `The language server for ${project} isn't running, so there is nothing to preview.`,
        action: "showOutput",
      },
    ];
  }
  const contentRoot = result.contentRoot;
  if (result.page === null && contentRoot !== null && !within(file, contentRoot)) {
    return [
      ...result.problems.filter((problem) => problem.severity !== "info"),
      info(
        `${show(file)} is in the project ${project}, but outside its content root ` +
          `(${show(contentRoot)}), so there is no page to preview.`,
      ),
    ];
  }
  return result.problems;
}

function info(message: string): ShownProblem {
  return { severity: "info", message };
}
