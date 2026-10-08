// What review says and where it points, apart from VS Code so it can be
// tested by itself: the base's name, a changed page's line in the list, the
// files a page changed through, and which changed page comes next.

import * as path from "node:path";
import { countsText } from "./counts.js";
import type { BaseInfo, ChangedPage, PageDiff } from "./protocol.js";

/** The content model's file, which `because` names last when it's a cause. */
const MODEL_FILE = "ascribe.toml";

/**
 * The base as the header and the status bar name it: the revision asked for,
 * or, when that's the pull request's base on a remote (`origin/main`), the
 * branch's name (`main`), as GitHub names it.
 */
export function baseName(base: BaseInfo, pullRequestBase?: string): string {
  const name = pullRequestBase;
  return name !== undefined && base.requested.endsWith(`/${name}`) ? name : base.requested;
}

/** The commit compared with, shortened: where the branch left the base. */
export function baseCommit(base: BaseInfo): string {
  return (base.merge_base ?? base.commit).slice(0, 7);
}

/** Whether two answers name the same base at the same commits. */
export function sameBase(a: BaseInfo, b: BaseInfo): boolean {
  return a.requested === b.requested && a.commit === b.commit && a.merge_base === b.merge_base;
}

/**
 * A changed page's label in the list: its title, or its path when it has
 * none. A quick pick shows text only, so a formatted title's code spans are
 * written as Markdown writes them, between backticks.
 */
export function pageLabel(page: Pick<ChangedPage, "path" | "title" | "formatted_title">): string {
  if (!page.formatted_title || page.formatted_title.length === 0) return page.title ?? page.path;
  return page.formatted_title
    .map((piece) => (piece.type === "code" ? codeSpan(piece.value) : piece.value))
    .join("");
}

/** `value` as a Markdown code span: fenced by one more backtick than its longest run. */
function codeSpan(value: string): string {
  const longest = Math.max(0, ...(value.match(/`+/g) ?? []).map((run) => run.length));
  const fence = "`".repeat(longest + 1);
  const pad = value.startsWith("`") || value.endsWith("`") ? " " : "";
  return `${fence}${pad}${value}${pad}${fence}`;
}

/** A changed page's detail in the list: its counts, and what it changed through. */
export function pageDetail(page: Omit<PageDiff, "changes">): string {
  let text: string;
  if (page.status === "added") text = "New page";
  else if (page.status === "removed") text = "Removed";
  else {
    text = countsText(page.counts);
    if (page.page_changed.length > 0) {
      const what = `${page.page_changed.join(", ")} changed`;
      text = text ? `${text} · ${what}` : what;
    }
    if (!text) text = "Changed";
  }
  if (!page.own_file_changed && page.because.length > 0) {
    text += ` · via ${page.because.join(", ")}`;
  }
  return text;
}

/**
 * The files a page changed through, when its own file didn't change: each
 * cause's content path (or `ascribe.toml`) and its absolute path.
 */
export function causes(
  page: Omit<PageDiff, "changes"> | null,
  roots: { projectRoot: string | null; contentRoot: string | null },
): { label: string; path: string }[] {
  if (!page || page.own_file_changed || page.status !== "changed") return [];
  const out: { label: string; path: string }[] = [];
  for (const cause of page.because) {
    const root = cause === MODEL_FILE ? roots.projectRoot : roots.contentRoot;
    if (root === null) continue;
    out.push({ label: cause, path: fromContentPath(root, cause) });
  }
  return out;
}

/** An absolute path from a root and a `/`-separated path below it. */
export function fromContentPath(root: string, contentPath: string): string {
  return path.join(root, ...contentPath.split("/"));
}

/**
 * The changed page to offer after `current` (a content path): the next one
 * in path order, or, past the last, the first (`first`). Removed pages are
 * skipped, since there's nothing to open. `null` when no other page changed.
 */
export function nextChangedPage(
  pages: readonly ChangedPage[],
  current: string,
): { page: ChangedPage; first: boolean } | null {
  const open = pages.filter((p) => p.status !== "removed" && p.path !== current);
  if (open.length === 0) return null;
  const after = open.find((p) => p.path > current);
  return after ? { page: after, first: false } : { page: open[0] as ChangedPage, first: true };
}

/** A block's anchor source, `<path>:<first>-<last>`, as a content path and lines from 0. */
export function parseSource(
  source: string,
): { path: string; first: number; last: number } | undefined {
  const colon = source.lastIndexOf(":");
  if (colon <= 0) return undefined;
  const match = /^(\d+)-(\d+)$/.exec(source.slice(colon + 1));
  if (!match) return undefined;
  let file: string;
  try {
    file = decodeURIComponent(source.slice(0, colon));
  } catch {
    return undefined;
  }
  const first = Number(match[1]);
  const last = Number(match[2]);
  if (first < 1 || last < first) return undefined;
  return { path: file, first: first - 1, last: last - 1 };
}
