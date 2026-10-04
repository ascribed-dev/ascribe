// Answering the overlay's requests (`OverlayHost`) from a review session, for
// a host that keeps GitHub to itself and passes the overlay's requests on as
// messages: the page preview's extension, or the site preview's dev server.
// The parameters arrive as untyped messages, so each is checked here.
import type { Anchor } from "../place/anchor.js";
import type { PageRef } from "../place/place.js";
import { ReviewError } from "../shared/errors.js";
import { gitMaybe } from "../shared/git.js";
import { parseRemote } from "./repository.js";
import type { PullRequestInfo, ReviewEvent, ReviewSession } from "./session.js";

/** The overlay's requests, one per `OverlayHost` method that reads or writes threads. */
export type OverlayMethod =
  "load" | "commentTarget" | "comment" | "reply" | "allThreads" | "resolve" | "submit" | "discard";

export const OVERLAY_METHODS: readonly OverlayMethod[] = [
  "load",
  "commentTarget",
  "comment",
  "reply",
  "allThreads",
  "resolve",
  "submit",
  "discard",
];

/** A changed page as `ascribe diff` lists it, with its title when the host knows it. */
export interface ChangedPageRef {
  path: string;
  status: "added" | "removed" | "changed";
  because: readonly string[];
  title?: string | null;
}

/** What answering a request needs. */
export interface RequestContext {
  session: ReviewSession;
  /** The page the overlay is on: its build and content path. */
  page: { build: string; path: string } | undefined;
  /** The page as the overlay last read it, with its anchors: comments are made on it. */
  lastPage: PageRef | undefined;
  /** The changed pages, for which pages show a thread. */
  changedPages(): Promise<readonly ChangedPageRef[]>;
}

/** An answer: the result, the page read (for `load`), and whether the request changed the threads. */
export interface Answer {
  result: unknown;
  page?: PageRef;
  changed: boolean;
}

/** Answers one of the overlay's requests. Rejects with a `ReviewError`. */
export async function answerRequest(
  context: RequestContext,
  method: OverlayMethod,
  params: Record<string, unknown>,
): Promise<Answer> {
  const { session } = context;
  switch (method) {
    case "load": {
      const page = context.page;
      if (!page) throw new ReviewError("not-found", "There's no page to review here.");
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

/**
 * The changed pages that show a file (a content path): the file's own page,
 * and the pages that changed through it (an include). `[]` when none does.
 */
export function pagesShowing(
  file: string,
  pages: readonly ChangedPageRef[],
): { path: string; title: string | null }[] {
  return pages
    .filter((page) => page.status !== "removed")
    .filter((page) => page.path === file || page.because.includes(file))
    .sort((a, b) => Number(b.path === file) - Number(a.path === file))
    .map((page) => ({ path: page.path, title: page.title ?? null }));
}

/**
 * The git revision to compare with for a pull request's base: the remote
 * branch, from the remote whose URL is the pull request's repository, when
 * there is one; else the branch's name.
 */
export function pullRequestBase(
  remotes: readonly { name: string; repository: string | undefined }[],
  repository: string,
  baseRefName: string,
): string {
  const remote =
    remotes.find((r) => r.repository === repository && r.name === "upstream") ??
    remotes.find((r) => r.repository === repository && r.name === "origin") ??
    remotes.find((r) => r.repository === repository);
  return remote ? `${remote.name}/${baseRefName}` : baseRefName;
}

/**
 * The git revision for the pull request's base in the checkout at `root`:
 * `origin/main` when that remote branch exists, else the branch's name.
 */
export async function baseRevision(root: string, pr: PullRequestInfo): Promise<string> {
  const out = (await gitMaybe(root, ["remote", "-v"]).catch(() => undefined)) ?? "";
  const remotes = out
    .split("\n")
    .map((line) => /^(\S+)\s+(\S+)\s+\(fetch\)$/.exec(line))
    .filter((match) => match !== null)
    .map((match) => {
      const repository = parseRemote(match[2] ?? "");
      return { name: match[1] ?? "", repository: repository && repoKey(repository) };
    });
  const revision = pullRequestBase(remotes, repoKey(pr.repository), pr.baseRefName);
  if (revision !== pr.baseRefName) {
    const exists = await gitMaybe(root, ["rev-parse", "--verify", "--quiet", revision]).catch(
      () => undefined,
    );
    if (exists !== undefined) return revision;
  }
  return pr.baseRefName;
}

function repoKey(repository: { host: string; owner: string; name: string }): string {
  return `${repository.host}/${repository.owner}/${repository.name}`.toLowerCase();
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
