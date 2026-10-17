// Which files a project's server is asked for lenses in, and what a lens in
// a code file says when more than one project's server answers for it. No
// VS Code in this module, so the rules are unit-tested as they are.
//
// A server registers its lenses for its pages and for its sources' files
// (`shown.rs` in crates/ascribe-lsp). A code file isn't sent to any server,
// so a server's lens in one is asked for whichever project's folder the file
// is in, or none: a source may be a folder outside the project, or inside
// another project's. Pages and Ascribe's own files go only to their project.

import * as path from "node:path";
import { ownedElsewhere, type Project } from "./projects.js";

/** Ascribe's own files that aren't pages: the model, the permalinks list, a source's examples config. */
const ASCRIBE_FILES = ["ascribe.toml", "ascribe.permalinks", "ascribe.examples.toml"];

/** Whether a file is a code file: not a page (Markdown) and not one of Ascribe's own files. */
export function isCodeFile(file: string, languageId: string): boolean {
  return languageId !== "markdown" && !ASCRIBE_FILES.includes(path.basename(file));
}

/**
 * Whether `project`'s server is asked for a file's lenses: every code file its
 * registration covers, and its own pages and files.
 */
export function asksForLenses(
  project: Project,
  file: string,
  languageId: string,
  projects: readonly Project[],
): boolean {
  return isCodeFile(file, languageId) || !ownedElsewhere(project, file, projects);
}

/**
 * A lens's title in a code file, named for its project when more than one
 * project's server answers for the file: `docs: shown in 3 pages`.
 */
export function lensTitle(title: string, project: string, answering: number): string {
  if (answering < 2) return title;
  return `${project}: ${title.charAt(0).toLowerCase()}${title.slice(1)}`;
}
