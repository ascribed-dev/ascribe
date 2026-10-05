// The messages between the toolbar app (in the page) and the dev server,
// over Astro's dev toolbar channel. Types only: both sides import them.
//
// The page sends `ascribe:review:request` with `{ tab, id, method, params }`;
// the server answers that page with `ascribe:review:result` and the same
// `tab` and `id`. When a page's request changes the threads, or review stops,
// the server tells every page with `ascribe:review:changed`, naming the
// `from` tab, which already knows.

import type { Change } from "@ascribed/review/marks";

/** A block change, as `ascribe diff --format json` reports it and the marks take it. */
export type { Change };

/** The toolbar app's id, also the prefix of its events. */
export const APP_ID = "ascribe:review";
export const REQUEST_EVENT = "ascribe:review:request";
export const RESULT_EVENT = "ascribe:review:result";
export const CHANGED_EVENT = "ascribe:review:changed";

/** What the page asks for, besides the overlay's own requests (`OverlayMethod`). */
export type AppMethod = "status" | "start" | "stop" | "page" | "refresh";

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

export interface Counts {
  changed: number;
  added: number;
  removed: number;
  moved: number;
}

/** A changed page, as `ascribe diff --format json` lists it. */
export interface DiffPage {
  path: string;
  route: string;
  status: "added" | "removed" | "changed";
  own_file_changed: boolean;
  because: string[];
  page_changed: string[];
  counts: Counts;
  changes: Change[];
}

/** The base compared with, as `ascribe diff` reports it. */
export interface DiffBase {
  requested: string;
  commit: string;
  merge_base: string | null;
}

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
  page: DiffPage | null;
  base: DiffBase | null;
  /** Why the changes couldn't be read, when they couldn't. */
  problem: string | null;
  /** How many errors `ascribe check` finds in the working tree for the build. */
  errors: number;
  threads: ThreadsState;
  /** The build's changed pages, in path order. */
  changedPages: Omit<DiffPage, "changes">[];
  /** The content root, absolute, and the platform's path separator: for opening a source file. */
  contentRoot: string;
  separator: string;
}
