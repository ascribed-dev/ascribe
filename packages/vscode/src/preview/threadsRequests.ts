// Answering the overlay's requests from a review session, apart from VS Code
// so it can be tested with a fake session.

import { ReviewError, type ReviewEvent, type ReviewSession } from "@ascribed/review/github";
import type { Anchor, PageRef } from "@ascribed/review/place";
import type { ChangedPage, ThreadsMethod } from "./protocol.js";
import { pagesShowing } from "./threadsText.js";

/** What answering a request needs. */
export interface RequestContext {
  session: ReviewSession;
  /** The page the preview shows: its build and content path. */
  page: { build: string; path: string } | undefined;
  /** The page as the overlay last read it, with its anchors: comments are made on it. */
  lastPage: PageRef | undefined;
  /** The project's changed pages, for which pages show a thread. */
  changedPages(): Promise<ChangedPage[]>;
}

/** An answer: the result, the page read (for `load`), and whether the request changed the threads. */
export interface Answer {
  result: unknown;
  page?: PageRef;
  changed: boolean;
}

/** Answers one of the overlay's requests (`OverlayHost`). Rejects with a `ReviewError`. */
export async function answerThreads(
  context: RequestContext,
  method: ThreadsMethod,
  params: Record<string, unknown>,
): Promise<Answer> {
  const { session } = context;
  switch (method) {
    case "load": {
      const page = context.page;
      if (!page) throw new ReviewError("not-found", "There's no page in the preview.");
      const ref: PageRef = {
        build: page.build,
        path: page.path,
        anchors: anchors(params["anchors"]),
        removed: anchors(params["removed"]),
      };
      const [threads, pending, viewer] = await Promise.all([
        session.threads(ref),
        session.pending(),
        session.viewer(),
      ]);
      const pr = session.pullRequest;
      return {
        result: { pullRequest: { number: pr.number, url: pr.url }, threads, pending, viewer },
        page: ref,
        changed: false,
      };
    }
    case "commentTarget":
      return { result: await session.commentTarget(anchor(params["anchor"])), changed: false };
    case "allThreads": {
      const [threads, pages] = await Promise.all([
        session.allThreads(),
        context.changedPages().catch(() => []),
      ]);
      return {
        result: threads.map((thread) => ({ thread, pages: pagesShowing(thread.path, pages) })),
        changed: false,
      };
    }
    case "comment": {
      const target = anchor(params["anchor"]);
      const page = context.lastPage ?? {
        build: context.page?.build ?? "",
        path: context.page?.path ?? "",
        anchors: [target],
      };
      return { result: await session.comment(target, text(params["body"]), page), changed: true };
    }
    case "reply":
      return {
        result: await session.reply(
          text(params["threadId"]),
          text(params["body"]),
          params["when"] === "now" ? "now" : "withReview",
        ),
        changed: true,
      };
    case "resolve":
      await session.resolve(text(params["threadId"]), params["resolved"] === true);
      return { result: null, changed: true };
    case "submit": {
      const body = typeof params["body"] === "string" ? params["body"] : undefined;
      await session.submit(event(params["event"]), body);
      return { result: null, changed: true };
    }
    case "discard":
      await session.discard();
      return { result: null, changed: true };
  }
}

const EVENTS: readonly ReviewEvent[] = ["COMMENT", "APPROVE", "REQUEST_CHANGES"];

function event(value: unknown): ReviewEvent {
  if (EVENTS.includes(value as ReviewEvent)) return value as ReviewEvent;
  throw new ReviewError("refused", `A review can't be submitted as ${String(value)}.`);
}

function text(value: unknown): string {
  if (typeof value !== "string") throw new ReviewError("refused", "The request is missing text.");
  return value;
}

function anchor(value: unknown): Anchor {
  const record = value as Partial<Anchor> | null;
  if (typeof record?.source !== "string") {
    throw new ReviewError("refused", "The request is missing a block.");
  }
  const via = Array.isArray(record.via) ? record.via.filter((v) => typeof v === "string") : [];
  return { source: record.source, via };
}

function anchors(value: unknown): Anchor[] {
  return Array.isArray(value) ? value.map(anchor) : [];
}
