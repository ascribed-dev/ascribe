// The messages between the toolbar app (in the page) and the dev server,
// over Astro's dev toolbar channel. Types only: both sides import them.
//
// The page sends `ascribe:review:request` with `{ tab, id, method, params }`;
// the server answers that page with `ascribe:review:result` and the same
// `tab` and `id`. When a page's request changes the threads, or review stops,
// the server tells every page with `ascribe:review:changed`, naming the
// `from` tab, which already knows.

import type { BaseInfo, PageDiff } from "../shapes.js";
import type { FormattedPiece } from "@ascribed/review/overlay";

/** `ascribe diff --format json`'s types: a changed page, its block changes, and the base. */
export type { BaseInfo, Change, Counts, PageDiff } from "../shapes.js";

/** The toolbar app's id, also the prefix of its events. */
export const APP_ID = "ascribe:review";
export const REQUEST_EVENT = "ascribe:review:request";
export const RESULT_EVENT = "ascribe:review:result";
export const CHANGED_EVENT = "ascribe:review:changed";

export interface Request {
  /** The page's own id, made when it loads. */
  tab: string;
  id: number;
  method: string;
  params: Record<string, unknown>;
}

export interface Result {
  tab: string;
  id: number;
  result?: unknown;
  error?: { message: string; code?: string };
}

/**
 * A changed page in the list: without its changes, and with its title when
 * the build has it, formatted when its field sets `inline = "code"`.
 */
export type ChangedPage = Omit<PageDiff, "changes"> & {
  title: string | null;
  formatted_title: FormattedPiece[] | null;
};

/** How the checkout's `HEAD` relates to the pull request's head commit. */
export type LocalState = "same" | "behind" | "ahead" | "diverged" | "missing";

/** The review threads: on, or why not, with a sentence to show. */
export type ThreadsState =
  | {
      state: "on";
      pullRequest: { number: number; url: string; baseRefName: string };
      local: { state: LocalState; behind: number; ahead: number };
    }
  /** No pull request, no GitHub CLI, no sign-in, a channel other pages could use, or a failure. */
  | {
      state: "no-pull-request" | "gh-missing" | "signed-out" | "unprotected" | "error";
      message: string;
    };

/** `status`'s answer. */
export interface Status {
  on: boolean;
}

/** `page`'s answer: what the app shows for the page at a route. */
export interface PageView {
  /** `"page"` for a page of the Ascribe build, `"not-page"` for the site's own routes. */
  kind: "page" | "not-page";
  /** The page's content path; `null` for a route that isn't one. */
  path: string | null;
  /** The page's changes; `null` when it didn't change. */
  page: PageDiff | null;
  base: BaseInfo | null;
  /** Why the changes couldn't be read, when they couldn't. */
  problem: string | null;
  /** How many errors `ascribe check` finds in the working tree for the build. */
  errors: number;
  threads: ThreadsState;
  /** The build's changed pages, in path order. */
  changedPages: ChangedPage[];
  /** The content root, absolute, and the platform's path separator: for opening a source file. */
  contentRoot: string;
  separator: string;
}
