// What the Review view shows, worked out from plain values so the unit tests
// can check it without VS Code: the changed pages by status, their problems,
// the badge, and the source files a review changed.

import { fromContentPath, MODEL_FILE, pageDetail, pageLabel } from "../preview/reviewText.js";
import type { ChangedPage } from "../preview/protocol.js";
import type { ItemLook } from "./sidebar.js";

/** A changed page's status, which is its group. */
type PageStatus = ChangedPage["status"];

/** The groups, in the view's order. */
const STATUSES: readonly PageStatus[] = ["added", "changed", "removed"];

const GROUP_NAMES: Record<PageStatus, string> = {
  added: "Added",
  changed: "Changed",
  removed: "Removed",
};

/** How serious a problem is, as VS Code's diagnostics say: errors and warnings, then advice. */
export type ProblemSeverity = "error" | "warning" | "info" | "hint";

/** A problem on a changed page's file. */
export interface PageProblem {
  /** The page's content path. */
  path: string;
  severity: ProblemSeverity;
  message: string;
  /** Its first line, from 0. */
  line: number;
  character: number;
}

/** An item of the Review view. */
export type ReviewNode =
  | { kind: "group"; status: PageStatus; pages: readonly ReviewPage[] }
  | { kind: "page"; page: ReviewPage }
  | { kind: "problems"; problems: readonly PageProblem[]; advice: boolean }
  | { kind: "problem"; problem: PageProblem };

/** A changed page, and how many open threads show on it. */
export interface ReviewPage {
  page: ChangedPage;
  /** `undefined` when the pull request's threads aren't known: none, or not read. */
  openThreads: number | undefined;
}

/**
 * The view's top level: a group for each status with pages, in the order of
 * `STATUSES`, then the problems on the pages. Nothing when no page changed.
 */
export function reviewRoots(
  pages: readonly ReviewPage[],
  problems: readonly PageProblem[],
  advice: boolean,
): ReviewNode[] {
  if (pages.length === 0) return [];
  const nodes: ReviewNode[] = [];
  for (const status of STATUSES) {
    const of = pages.filter((p) => p.page.status === status);
    if (of.length > 0) nodes.push({ kind: "group", status, pages: of });
  }
  nodes.push({ kind: "problems", problems: shownProblems(problems, advice), advice });
  return nodes;
}

/** An item's children. */
export function reviewChildren(node: ReviewNode): ReviewNode[] {
  switch (node.kind) {
    case "group":
      return node.pages.map((page) => ({ kind: "page", page }));
    case "problems":
      return node.problems.map((problem) => ({ kind: "problem", problem }));
    case "page":
    case "problem":
      return [];
  }
}

/** How an item looks. */
export function reviewLook(node: ReviewNode): ItemLook {
  switch (node.kind) {
    case "group":
      return {
        label: GROUP_NAMES[node.status],
        description: String(node.pages.length),
        tooltip: `${pagesCount(node.pages.length)} ${node.status === "added" ? "added" : node.status === "removed" ? "removed" : "changed"} by this branch`,
        icon:
          node.status === "added"
            ? "diff-added"
            : node.status === "removed"
              ? "diff-removed"
              : "diff-modified",
        dimmed: false,
        expandable: true,
        expanded: true,
      };
    case "page": {
      const description = pageDescription(node.page);
      return {
        label: pageLabel(node.page.page),
        description,
        tooltip:
          node.page.page.title === null ? description : `${node.page.page.path}\n${description}`,
        icon: "file",
        dimmed: false,
        expandable: false,
      };
    }
    case "problems": {
      const count = node.problems.length;
      return {
        label: "Problems on these pages",
        description: count === 0 ? "none" : String(count),
        tooltip: `${node.advice ? "Errors, warnings, and advice" : "Errors and warnings"} on the changed pages, whether this branch made them or they were there before.`,
        icon: "warning",
        dimmed: count === 0,
        expandable: count > 0,
        expanded: true,
      };
    }
    case "problem": {
      const { problem } = node;
      return {
        label: problem.message,
        description: `${problem.path}:${problem.line + 1}`,
        tooltip: `${problem.message}\n${problem.path}:${problem.line + 1}`,
        icon:
          problem.severity === "error"
            ? "error"
            : problem.severity === "warning"
              ? "warning"
              : "info",
        dimmed: false,
        expandable: false,
      };
    }
  }
}

/**
 * A page's line in the view: its counts as Changed Pages says them, what it
 * changed through, and its open threads.
 */
export function pageDescription(page: ReviewPage): string {
  const text = pageDetail(page.page);
  const threads = page.openThreads;
  if (threads === undefined || threads === 0) return text;
  return `${text} · ${threads === 1 ? "1 open thread" : `${threads} open threads`}`;
}

/** The problems the view lists: errors and warnings, and advice when it's shown; worst first, then by place. */
export function shownProblems(problems: readonly PageProblem[], advice: boolean): PageProblem[] {
  const rank: Record<ProblemSeverity, number> = { error: 0, warning: 1, info: 2, hint: 3 };
  return problems
    .filter((p) => advice || p.severity === "error" || p.severity === "warning")
    .sort(
      (a, b) =>
        rank[a.severity] - rank[b.severity] ||
        a.path.localeCompare(b.path) ||
        a.line - b.line ||
        a.character - b.character,
    );
}

/** The badge's tooltip for `unsent` comments; `undefined` for no badge. */
export function badgeTooltip(unsent: number | undefined): string | undefined {
  if (unsent === undefined || unsent <= 0) return undefined;
  return unsent === 1
    ? "1 comment you haven't submitted"
    : `${unsent} comments you haven't submitted`;
}

/** A source file a review changed: its label in the diff, and where it is. */
export interface SourceFile {
  /** Its content path, or `ascribe.toml`. */
  label: string;
  /** Its absolute path. */
  path: string;
  /** Whether it's at the base, and whether it's now: an added page's file isn't at the base. */
  before: boolean;
  after: boolean;
}

/**
 * The changed source files of `pages`: each page's own file when it changed,
 * and the files it changed through, each once, in the order first named. A
 * snippet's code, named by its address, isn't a file of the content root and
 * isn't listed.
 */
export function sourceFiles(
  pages: readonly ChangedPage[],
  roots: { projectRoot: string | null; contentRoot: string | null },
): SourceFile[] {
  const out = new Map<string, SourceFile>();
  const add = (label: string, root: string | null, before: boolean, after: boolean): void => {
    if (root === null) return;
    const path = fromContentPath(root, label);
    if (!out.has(path)) out.set(path, { label, path, before, after });
  };
  for (const page of pages) {
    if (page.own_file_changed || page.status !== "changed") {
      add(page.path, roots.contentRoot, page.status !== "added", page.status !== "removed");
    }
    for (const cause of page.because) {
      if (cause.startsWith(SNIPPET_PREFIX)) continue;
      add(cause, cause === MODEL_FILE ? roots.projectRoot : roots.contentRoot, true, true);
    }
  }
  return [...out.values()];
}

/** How `because` names a snippet whose code changed: by its address. */
const SNIPPET_PREFIX = "code:";

/** The most files VS Code's multi-file diff shows well: past it, it may repeat headers. */
export const DIFF_FILES_LIMIT = 30;

/** The multi-file diff's title. */
export function diffTitle(base: string, files: number): string {
  const title = `Source changes against ${base}`;
  return files > DIFF_FILES_LIMIT
    ? `${title} (${files} files: VS Code's diff may repeat headers past ${DIFF_FILES_LIMIT})`
    : title;
}

function pagesCount(count: number): string {
  return count === 1 ? "1 page" : `${count} pages`;
}
