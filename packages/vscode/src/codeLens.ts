// Which files a project's server is asked for lenses in, and what a lens
// says when more than one project's server answers in a file. No VS Code in
// this module, so the rules are unit-tested as they are.
//
// A server registers its lenses for every Markdown file in its folder and
// for its sources' files (`shown.rs` in crates/ascribe-lsp). A code file
// isn't sent to any server, so a server's lens in one is asked for whichever
// project's folder the file is in, or none: a source may be a folder outside
// the project, or inside another project's. A Markdown file is asked for
// from its own project, and from any project whose sources hold it, which a
// `@snippet` may show; a nested project's page, which the registration also
// covers, isn't. Ascribe's own files go only to their project.

import * as path from "node:path";
import { inASource } from "./diskChanges.js";
import { ownedElsewhere, type Project } from "./projects.js";

/**
 * Ascribe's own files that aren't pages: the model, the permalinks list, the
 * navigation file, a source's examples config.
 */
const ASCRIBE_FILES = [
  "ascribe.toml",
  "ascribe.permalinks",
  "ascribe.nav",
  "ascribe.examples.toml",
];

/** Whether a file is one of Ascribe's own: the model, the permalinks list, and the rest. */
function isAscribeFile(file: string): boolean {
  return ASCRIBE_FILES.includes(path.basename(file));
}

/** Whether a file is a code file: not a page (Markdown) and not one of Ascribe's own files. */
export function isCodeFile(file: string, languageId: string): boolean {
  return languageId !== "markdown" && !isAscribeFile(file);
}

/**
 * Whether `project`'s server is asked for a file's lenses: every code file its
 * registration covers, a Markdown file in one of `sources`' folders, and its
 * own pages and files. `sources` are the folders of the sources the server
 * loaded.
 */
export function asksForLenses(
  project: Project,
  file: string,
  languageId: string,
  sources: readonly string[],
  projects: readonly Project[],
): boolean {
  return (
    !ownedElsewhere(project, file, projects) ||
    isCodeFile(file, languageId) ||
    (!isAscribeFile(file) && inASource(file, sources))
  );
}

/**
 * A lens's title, named for its project when more than one project's server
 * answers in the file with a lens: `docs: shown in 3 pages`.
 */
export function lensTitle(title: string, project: string, answering: number): string {
  if (answering < 2) return title;
  return `${project}: ${title.charAt(0).toLowerCase()}${title.slice(1)}`;
}
