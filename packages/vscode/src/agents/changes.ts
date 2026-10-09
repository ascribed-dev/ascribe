// The changed pages review compares, for the `ascribe_review_changes` tool.
// Apart from VS Code, so it can be tested by itself.

import * as path from "node:path";
import type { BaseInfo, ChangedPage, ChangesResult, Counts } from "../shapes.js";
import { relativePath, shellWord } from "./problems.js";

/** The most pages the tool lists. */
export const MAX_PAGES = 100;

/** A changed page, for an agent. */
interface AgentChangedPage {
  /** The page's file, relative to the project's folder, with `/`. */
  file: string;
  /** Its content path, as `because` names files. */
  path: string;
  title: string | null;
  route: string;
  status: ChangedPage["status"];
  own_file_changed: boolean;
  because: string[];
  page_changed: string[];
  counts: Counts;
}

/** What the tool answers. */
export interface ChangesReport {
  /** The build whose pages are compared. */
  build: string;
  /** The base the review compares with. */
  base: BaseInfo | null;
  /** The pull request's number, when review reads one. */
  pull_request: number | null;
  /** The content root, relative to the project's folder: where the content paths are. */
  content_root: string | null;
  pages: AgentChangedPage[];
  truncated: boolean;
  shown: number;
  total: number;
  /** The command that lists every changed page, when `truncated`; `null` otherwise. */
  next_command: string | null;
}

/** The answer to `ascribe/review/changes`, for an agent: each page with its file. */
export function changesReport(
  result: ChangesResult,
  root: string,
  pullRequest: number | undefined,
): ChangesReport {
  const contentRoot = result.contentRoot;
  const pages = result.pages.slice(0, MAX_PAGES).map((page): AgentChangedPage => ({
    file:
      contentRoot === null
        ? page.path
        : relativePath(root, path.join(contentRoot, ...page.path.split("/"))),
    path: page.path,
    title: page.title,
    route: page.route,
    status: page.status,
    own_file_changed: page.own_file_changed,
    because: page.because,
    page_changed: page.page_changed,
    counts: page.counts,
  }));
  const truncated = pages.length < result.pages.length;
  const base = result.base;
  return {
    build: result.build,
    base,
    pull_request: pullRequest ?? null,
    content_root: contentRoot === null ? null : relativePath(root, contentRoot) || ".",
    pages,
    truncated,
    shown: pages.length,
    total: result.pages.length,
    next_command:
      truncated && base
        ? `ascribe diff --base ${shellWord(base.requested)} --build ${shellWord(result.build)} --pages-only --format json`
        : null,
  };
}
