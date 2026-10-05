// A review session: one pull request's threads, placed on pages, and the
// viewer's pending review that new comments go into.
import { readFileSync, realpathSync, statSync } from "node:fs";
import path from "node:path";
import { parse } from "smol-toml";
import { formatSource, parseSource, type Anchor } from "../place/anchor.js";
import { lineMap, parseHunks, type Hunk } from "../place/lines.js";
import {
  locateThreads,
  placeOnPage,
  type LocatedThread,
  type PageRef,
  type PlacedThreads,
} from "../place/place.js";
import { ReviewError } from "../shared/errors.js";
import { gitMaybe } from "../shared/git.js";
import type { Thread, ThreadComment } from "../shared/types.js";
import { formatSection, joinSections, PLACEHOLDER, withoutPlaceholder } from "./marker.js";
import {
  ADD_REPLY,
  ADD_REVIEW,
  ADD_THREAD,
  DELETE_COMMENT,
  DELETE_REVIEW,
  FIND_PULL_REQUEST,
  RESOLVE,
  SUBMIT_REVIEW,
  UNRESOLVE,
  UPDATE_REVIEW,
  VIEWER,
} from "./queries.js";
import {
  markedThreads,
  readConversation,
  readFiles,
  readRestOfComments,
  readReviews,
  readReviewThreads,
  toComment,
  toThread,
  type PullRequestKey,
  type RawReview,
  type RawThread,
} from "./read.js";
import { readCheckout, type CheckoutInfo, type RepositoryRef } from "./repository.js";
import { ghTransport, type GitHubTransport } from "./transport.js";

/** How the local `HEAD` relates to the pull request's head commit. */
export type LocalState =
  /** `HEAD` is the pull request's head commit. */
  | "same"
  /** The pull request has commits `HEAD` doesn't: pull them. */
  | "behind"
  /** `HEAD` has commits the pull request doesn't: push them. */
  | "ahead"
  /** Both have commits the other doesn't. */
  | "diverged"
  /** The pull request's head commit isn't in the local repository: fetch it. */
  | "missing";

/** The pull request for the checkout. */
export interface PullRequestInfo {
  /** Its GraphQL node id. */
  id: string;
  number: number;
  url: string;
  repository: RepositoryRef;
  baseRefName: string;
  baseOid: string;
  headRefName: string;
  headOid: string;
  local: LocalState;
}

/** What's in the viewer's pending review: comments only they can see until they submit. */
export interface PendingReview {
  /** The review's node id, or `undefined` when there's no pending review. */
  id: string | undefined;
  /** New threads. */
  threads: Thread[];
  /** Replies to existing threads. */
  replies: { threadId: string; comment: ThreadComment }[];
  /** Comments on blocks GitHub can't anchor, which go on the conversation when submitted. */
  conversation: Thread[];
  /** How many comments there are in all. */
  count: number;
}

/** How a review ends: as a comment, an approval, or a request for changes. */
export type ReviewEvent = "COMMENT" | "APPROVE" | "REQUEST_CHANGES";

/** How a comment on a block would go, worked out before it's made. */
export type CommentTarget =
  /** As a review thread on the block's lines. */
  | { kind: "thread" }
  /**
   * In the pending review's body, since GitHub can't anchor it: the file
   * isn't in the pull request (`"file"`), or the lines are away from its
   * changes (`"lines"`).
   */
  | { kind: "summary"; reason: "file" | "lines" }
  /** Not until the block's lines are pushed (or fetched): `message` says which. */
  | { kind: "push-first"; message: string };

/** One pull request's review, for a host and the overlay behind it. */
export interface ReviewSession {
  pullRequest: PullRequestInfo;
  /** The page's threads. Cached until `refresh()` or a change made through the session. */
  threads(page: PageRef): Promise<PlacedThreads>;
  /** Every thread on the pull request's files in the content root, located in the working tree. */
  allThreads(): Promise<LocatedThread[]>;
  /** The signed-in viewer's login. */
  viewer(): Promise<string>;
  /** How a comment on a block would go: as a thread, in the review's summary, or not yet. */
  commentTarget(anchor: Anchor): Promise<CommentTarget>;
  /** Comments on a block, in the pending review. */
  /**
   * Comments on a block. `quote` is the block's text as the page shows it,
   * which a comment held in the review's summary quotes; without it, the
   * comment quotes the block's source.
   */
  comment(anchor: Anchor, body: string, page: PageRef, quote?: string): Promise<Thread>;
  /** Replies to a thread: sent at once (`"now"`), or held in the pending review. */
  reply(threadId: string, body: string, when: "now" | "withReview"): Promise<ThreadComment>;
  /** Resolves or reopens a thread, at once. */
  resolve(threadId: string, resolved: boolean): Promise<void>;
  pending(): Promise<PendingReview>;
  /** Submits the pending review, making its comments visible. */
  submit(event: ReviewEvent, body?: string): Promise<void>;
  /** Deletes the pending review and everything in it. */
  discard(): Promise<void>;
  /** Forgets what's cached, so the next call reads from GitHub again. */
  refresh(): Promise<void>;
}

/** Options for `openReview`. */
export interface OpenReviewOptions {
  /** The directory holding the project's `ascribe.toml`. */
  projectDir: string;
  /**
   * The content root's path in the repository, with a trailing `/`. By
   * default it's worked out from `ascribe.toml`'s `content-root`.
   */
  contentPrefix?: string;
  /** Makes the transport for a host. By default, `ghTransport`. */
  transport?: (host: string) => GitHubTransport;
  /**
   * The least time between two requests that change something, in
   * milliseconds, to stay under GitHub's secondary rate limit. Default 1000.
   */
  mutationInterval?: number;
}

/**
 * Opens a review of the pull request for the project's checkout: the open
 * pull request whose head is the current branch. Resolves with `undefined`
 * when there's none, or `HEAD` is detached.
 */
export async function openReview(options: OpenReviewOptions): Promise<ReviewSession | undefined> {
  const checkout = await readCheckout(options.projectDir);
  const transportFor = options.transport ?? ((host: string) => ghTransport({ host }));
  const found = await findPullRequest(checkout, transportFor);
  if (found === undefined) return undefined;
  const contentPrefix = options.contentPrefix ?? contentPrefixOf(checkout.root, options.projectDir);
  return createSession({
    root: checkout.root,
    contentPrefix,
    pullRequest: found,
    transport: transportFor(found.repository.host),
    mutationInterval: options.mutationInterval ?? 1000,
  });
}

/** The content root's path in the repository, from the project's `ascribe.toml`. */
export function contentPrefixOf(root: string, projectDir: string): string {
  const configPath = path.join(projectDir, "ascribe.toml");
  let contentRoot = "docs";
  try {
    const table = parse(readFileSync(configPath, "utf8"));
    const project = table["project"];
    if (typeof project === "object" && project !== null && !Array.isArray(project)) {
      const value = (project as Record<string, unknown>)["content-root"];
      if (typeof value === "string") contentRoot = value;
    }
  } catch (error) {
    throw new ReviewError("not-found", `Couldn't read ${configPath}.`, { cause: error });
  }
  const real = (p: string) => {
    try {
      return realpathSync.native(p);
    } catch {
      return path.resolve(p);
    }
  };
  const relative = path.relative(real(root), real(path.resolve(projectDir, contentRoot)));
  if (relative === "") return "";
  return `${relative.split(path.sep).join("/")}/`;
}

interface FoundPullRequest {
  id: string;
  number: number;
  url: string;
  baseRefName: string;
  baseRefOid: string;
  headRefName: string;
  headRefOid: string;
  headRepositoryOwner: { login: string } | null;
  headRepository: { name: string } | null;
}

/**
 * The open pull request whose head is the checkout's branch, looked for in an
 * `upstream` remote's repository and then in the one the branch is pushed to.
 */
export async function findPullRequest(
  checkout: CheckoutInfo,
  transportFor: (host: string) => GitHubTransport,
): Promise<PullRequestInfo | undefined> {
  const head = checkout.head;
  if (checkout.headRef === undefined || head === undefined) return undefined;
  for (const base of checkout.bases) {
    const data: {
      repository: { pullRequests: { nodes: (FoundPullRequest | null)[] } } | null;
    } = await transportFor(base.host).graphql(FIND_PULL_REQUEST, {
      owner: base.owner,
      name: base.name,
      head: checkout.headRef,
    });
    const match = data.repository?.pullRequests.nodes.find(
      (node) =>
        node !== null &&
        node.headRepositoryOwner?.login.toLowerCase() === head.owner.toLowerCase() &&
        node.headRepository?.name.toLowerCase() === head.name.toLowerCase(),
    );
    if (match === undefined || match === null) continue;
    return {
      id: match.id,
      number: match.number,
      url: match.url,
      repository: base,
      baseRefName: match.baseRefName,
      baseOid: match.baseRefOid,
      headRefName: match.headRefName,
      headOid: match.headRefOid,
      local: await localState(checkout.root, checkout.headOid, match.headRefOid),
    };
  }
  return undefined;
}

async function localState(root: string, local: string, head: string): Promise<LocalState> {
  if (local === head) return "same";
  if ((await gitMaybe(root, ["cat-file", "-e", `${head}^{commit}`])) === undefined) {
    return "missing";
  }
  if ((await gitMaybe(root, ["merge-base", "--is-ancestor", local, head])) !== undefined) {
    return "behind";
  }
  if ((await gitMaybe(root, ["merge-base", "--is-ancestor", head, local])) !== undefined) {
    return "ahead";
  }
  return "diverged";
}

/** What a session needs, for hosts and tests that find the pull request themselves. */
export interface SessionOptions {
  root: string;
  contentPrefix: string;
  pullRequest: PullRequestInfo;
  transport: GitHubTransport;
  mutationInterval: number;
  /**
   * How long to wait before reading again when GitHub fails a read with its
   * generic error, in milliseconds. Default 2000.
   */
  retryDelay?: number;
}

interface State {
  raw: RawThread[];
  reviews: RawReview[];
  threads: Thread[];
  /** The threads located in the working tree, as its files were at `stamp`. */
  located: Promise<LocatedThread[]>;
  stamp: string;
  pendingReview: RawReview | undefined;
}

// GitHub's generic failure, which it gives for a moment after some changes
// (deleting a pending review): worth one more try.
const TRANSIENT = /something went wrong/i;

/** The most of a block's shown text a held comment quotes: a review's body can't be longer than 65,536 characters. */
const QUOTE_LIMIT = 2000;

// GitHub's errors when a line can't take a review comment.
const CANT_ANCHOR =
  /could not be resolved|must be part of the diff|is not part of the diff|line.*outside/i;

/**
 * When each file the threads are on last changed in the working tree, as one
 * string: a different one means the threads need locating again.
 */
function worktreeStamp(root: string, threads: readonly Thread[]): string {
  const files = [...new Set(threads.map((thread) => thread.repositoryPath))].sort();
  return files
    .map((file) => {
      try {
        const stat = statSync(path.join(root, ...file.split("/")));
        return `${file}\0${stat.mtimeMs}\0${stat.size}`;
      } catch {
        return `${file}\0-`;
      }
    })
    .join("\n");
}

/** A session for a pull request already found. */
export function createSession(options: SessionOptions): ReviewSession {
  const { root, contentPrefix, pullRequest, transport } = options;
  const key: PullRequestKey = {
    owner: pullRequest.repository.owner,
    name: pullRequest.repository.name,
    number: pullRequest.number,
  };
  let state: Promise<State> | undefined;
  let files: Promise<Set<string>> | undefined;
  // The pending review this session made or wrote to, until a read shows it.
  let known: { id: string; body: string } | undefined;
  let viewer: Promise<string> | undefined;
  let lastMutation = 0;
  let queue: Promise<unknown> = Promise.resolve();
  // The pending review just discarded. For a moment after, GitHub can still
  // list it and its comments; reads leave them out until one doesn't.
  let discarded: string | undefined;
  const retryDelay = options.retryDelay ?? 2000;

  /** A read, tried once more after a wait when GitHub fails it with its generic error. */
  const read = async <T>(run: () => Promise<T>): Promise<T> => {
    try {
      return await run();
    } catch (error) {
      if (!(
        error instanceof ReviewError &&
        error.code === "refused" &&
        TRANSIENT.test(error.message)
      )) {
        throw error;
      }
      await new Promise((resolve) => setTimeout(resolve, retryDelay));
      return run();
    }
  };

  const load = (): Promise<State> => {
    state ??= (async () => {
      let [raw, conversation, reviews] = await Promise.all([
        read(() => readReviewThreads(transport, key)),
        read(() => readConversation(transport, key)),
        read(() => readReviews(transport, key)),
      ]);
      if (discarded !== undefined) {
        const gone = discarded;
        const stale =
          reviews.some((review) => review.id === gone) ||
          raw.some((thread) => thread.comments.nodes.some((c) => c?.state === "PENDING"));
        if (stale) {
          // The viewer has one pending review at most, so every pending
          // comment was in the one discarded.
          reviews = reviews.filter((review) => review.id !== gone);
          raw = raw
            .map((thread) => ({
              ...thread,
              comments: {
                ...thread.comments,
                nodes: thread.comments.nodes.filter((c) => c !== null && c.state !== "PENDING"),
              },
            }))
            .filter((thread) => thread.comments.nodes.length > 0);
        } else {
          discarded = undefined;
        }
      }
      const pendingReview = reviews.find(
        (review) => review.state === "PENDING" && review.viewerDidAuthor,
      );
      const threads: Thread[] = raw.map(toThread);
      for (const comment of conversation) {
        threads.push(...markedThreads(comment, contentPrefix, false));
      }
      for (const review of reviews) {
        threads.push(...markedThreads(review, contentPrefix, review.state === "PENDING"));
      }
      const stamp = worktreeStamp(root, threads);
      const first = locate(threads);
      await first;
      return { raw, reviews, threads, located: first, stamp, pendingReview };
    })();
    state.catch(() => {
      state = undefined;
    });
    return state;
  };

  const locate = (threads: readonly Thread[]): Promise<LocatedThread[]> =>
    locateThreads(threads, { root, contentPrefix, headOid: pullRequest.headOid });

  // The threads located in the working tree: again once one of their files
  // changed (saved), so they follow edits without reading GitHub again.
  const located = async (): Promise<LocatedThread[]> => {
    const current = await load();
    const stamp = worktreeStamp(root, current.threads);
    if (stamp !== current.stamp) {
      current.stamp = stamp;
      current.located = locate(current.threads);
    }
    return current.located;
  };

  // One change at a time, spaced out to stay under the secondary rate limit.
  const mutate = <T>(query: string, variables: Record<string, unknown>): Promise<T> => {
    const run = queue.then(async () => {
      const wait = lastMutation + options.mutationInterval - Date.now();
      if (wait > 0) await new Promise((resolve) => setTimeout(resolve, wait));
      lastMutation = Date.now();
      try {
        return await transport.graphql<T>(query, variables);
      } finally {
        lastMutation = Date.now();
      }
    });
    queue = run.catch(() => undefined);
    return run;
  };

  const changed = () => {
    state = undefined;
  };

  // After a change to one review thread, the cached threads take GitHub's
  // answer instead of reading every comment again. When a read started
  // since `before`, or the thread isn't in it, the next call reads again.
  const patch = async (
    before: Promise<State> | undefined,
    threadId: string,
    update: (raw: RawThread | undefined) => RawThread | undefined,
  ): Promise<void> => {
    if (before === undefined || state !== before) return changed();
    const current = await before;
    const index = current.raw.findIndex((thread) => thread.id === threadId);
    const raw = update(current.raw[index]);
    if (raw === undefined || state !== before) return changed();
    const thread = toThread(raw);
    if (index === -1) {
      current.raw.push(raw);
      current.threads.push(thread);
    } else {
      current.raw[index] = raw;
      const at = current.threads.findIndex((t) => t.id === threadId);
      if (at === -1) return changed();
      current.threads[at] = thread;
    }
    current.stamp = worktreeStamp(root, current.threads);
    current.located = locate(current.threads);
    current.located.catch(() => {
      if (state === before) state = undefined;
    });
  };

  const ensureReview = async (): Promise<{ id: string; body: string }> => {
    const pendingReview = (await load()).pendingReview;
    if (pendingReview !== undefined) return pendingReview;
    if (known !== undefined) return known;
    const data = await mutate<{ addPullRequestReview: { pullRequestReview: { id: string } } }>(
      ADD_REVIEW,
      { pullRequestId: pullRequest.id, commit: pullRequest.headOid, body: PLACEHOLDER },
    );
    changed();
    // A new pending review: the discarded one is gone from GitHub by now.
    discarded = undefined;
    known = { id: data.addPullRequestReview.pullRequestReview.id, body: PLACEHOLDER };
    return known;
  };

  const findThread = async (threadId: string): Promise<LocatedThread | undefined> =>
    (await located()).find((thread) => thread.id === threadId);

  const requireReviewThread = async (threadId: string): Promise<void> => {
    const state = await load();
    if (state.raw.some((thread) => thread.id === threadId)) return;
    const thread = await findThread(threadId);
    throw thread?.kind === "conversation"
      ? new ReviewError(
          "refused",
          "This is a comment on the pull request's conversation; GitHub can't reply to or resolve it as a thread.",
        )
      : new ReviewError("not-found", "That thread isn't in the pull request.");
  };

  // A block's lines at the head commit: every one of them has to be there, unchanged.
  const atHead = async (anchor: Anchor) => {
    const range = parseSource(anchor.source);
    if (range === undefined) {
      throw new ReviewError("not-found", `${anchor.source} isn't a block's source.`);
    }
    const repositoryPath = `${contentPrefix}${range.path}`;
    const map = await lineMap(root, pullRequest.headOid, repositoryPath, "to-commit");
    const lines: number[] = [];
    for (let line = range.first; line <= range.last; line++) {
      const mapped = map.map(line);
      if (mapped !== undefined && !mapped.replaced) lines.push(mapped.line);
    }
    const first = lines[0];
    const last = lines.at(-1);
    if (
      first === undefined ||
      last === undefined ||
      lines.length !== range.last - range.first + 1
    ) {
      throw new ReviewError(
        "push-first",
        pullRequest.local === "missing"
          ? "The pull request's latest commit isn't in this checkout. Fetch it, then comment."
          : "This block has changes that aren't in the pull request. Push them, then comment.",
      );
    }
    return { range, repositoryPath, first, last };
  };

  const changedFiles = (): Promise<Set<string>> => {
    files ??= readFiles(transport, key);
    files.catch(() => {
      files = undefined;
    });
    return files;
  };

  // The lines of each file GitHub shows in the pull request's diff, and so
  // takes comments on: its changes and three lines around them.
  const shown = new Map<string, Promise<Hunk[] | undefined>>();
  const shownLines = (repositoryPath: string): Promise<Hunk[] | undefined> => {
    let hunks = shown.get(repositoryPath);
    if (hunks === undefined) {
      hunks = gitMaybe(root, [
        "diff",
        "--no-color",
        "--no-ext-diff",
        "--no-renames",
        "--unified=3",
        `${pullRequest.baseOid}...${pullRequest.headOid}`,
        "--",
        `:(literal)${repositoryPath}`,
      ]).then((diff) => (diff === undefined ? undefined : parseHunks(diff)));
      shown.set(repositoryPath, hunks);
    }
    return hunks;
  };

  return {
    pullRequest,

    async threads(page) {
      return placeOnPage(page, await located());
    },

    async allThreads() {
      return located();
    },

    async viewer() {
      viewer ??= transport
        .graphql<{ viewer: { login: string } }>(VIEWER, {})
        .then((data) => data.viewer.login);
      viewer.catch(() => {
        viewer = undefined;
      });
      return viewer;
    },

    async commentTarget(anchor) {
      let lines: Awaited<ReturnType<typeof atHead>>;
      try {
        lines = await atHead(anchor);
      } catch (error) {
        if (error instanceof ReviewError && error.code === "push-first") {
          return { kind: "push-first", message: error.message };
        }
        throw error;
      }
      if (!(await changedFiles()).has(lines.repositoryPath)) {
        return { kind: "summary", reason: "file" };
      }
      const hunks = await shownLines(lines.repositoryPath);
      // Without the base commit there's no telling: try, and hold it if GitHub refuses.
      if (hunks === undefined) return { kind: "thread" };
      const inOne = hunks.some(
        (hunk) => lines.first >= hunk.newStart && lines.last < hunk.newStart + hunk.newCount,
      );
      return inOne ? { kind: "thread" } : { kind: "summary", reason: "lines" };
    },

    async comment(anchor, body, page, shown) {
      const { range, repositoryPath, first, last } = await atHead(anchor);
      if ((await changedFiles()).has(repositoryPath)) {
        const review = await ensureReview();
        const before = state;
        try {
          const data = await mutate<{ addPullRequestReviewThread: { thread: RawThread | null } }>(
            ADD_THREAD,
            {
              reviewId: review.id,
              path: repositoryPath,
              body,
              line: last,
              startLine: first === last ? null : first,
            },
          );
          // GitHub answers a line away from the diff's changes with no thread
          // and no error: it can't anchor the comment there.
          const thread = data.addPullRequestReviewThread.thread;
          if (thread !== null) {
            try {
              await readRestOfComments(transport, thread);
            } catch (error) {
              // The thread is on GitHub: read it with the rest next time.
              changed();
              throw error;
            }
            // The pending review the cache read is the one the thread is in.
            const cached = before && (await before).pendingReview?.id === review.id;
            await patch(cached ? before : undefined, thread.id, (raw) =>
              raw === undefined ? thread : undefined,
            );
            return toThread(thread);
          }
          changed();
        } catch (error) {
          if (!(
            error instanceof ReviewError &&
            error.code === "refused" &&
            CANT_ANCHOR.test(error.message)
          )) {
            throw error;
          }
        }
      }
      // GitHub can't anchor it: hold it in the pending review's body, which
      // goes on the conversation when the review is submitted.
      const review = await ensureReview();
      const text = shown?.trim();
      const quote =
        text === undefined || text === ""
          ? readFileSync(path.join(root, ...repositoryPath.split("/")), "utf8")
              .split(/\r?\n/)
              .slice(range.first - 1, range.last)
              .join("\n")
          : text.length > QUOTE_LIMIT
            ? `${text.slice(0, QUOTE_LIMIT).trimEnd()}…`
            : text;
      const source = formatSource(range);
      const section = formatSection({
        source,
        build: page.build,
        body,
        quote,
        shown: text !== undefined && text !== "",
        link: {
          label: `${range.path}, line${range.first === range.last ? ` ${range.first}` : `s ${range.first}–${range.last}`}`,
          url: blobUrl(pullRequest, repositoryPath, first, last),
        },
      });
      const held = joinSections([withoutPlaceholder(review.body), section], undefined);
      try {
        await mutate(UPDATE_REVIEW, { reviewId: review.id, body: held });
      } catch (error) {
        if (error instanceof ReviewError && /missing body/i.test(error.message)) {
          throw new ReviewError(
            "cant-hold",
            "You have unsent comments started on GitHub, and GitHub can't add this comment to them. Submit or discard them on GitHub, then comment again.",
            { cause: error },
          );
        }
        throw error;
      }
      changed();
      known = { id: review.id, body: held };
      const [thread] = markedThreads(
        {
          id: review.id,
          body: section,
          createdAt: new Date().toISOString(),
          url: pullRequest.url,
          author: null,
        },
        contentPrefix,
        true,
      );
      if (thread === undefined) throw new ReviewError("refused", "Couldn't write the comment.");
      return thread;
    },

    async reply(threadId, body, when) {
      await requireReviewThread(threadId);
      const reviewId = when === "withReview" ? (await ensureReview()).id : undefined;
      const before = state;
      const data = await mutate<{
        addPullRequestReviewThreadReply: { comment: Parameters<typeof toComment>[0] | null };
      }>(ADD_REPLY, { threadId, body, reviewId: reviewId ?? null });
      const raw = data.addPullRequestReviewThreadReply.comment;
      if (raw === null) {
        changed();
        throw new ReviewError("refused", "GitHub didn't add the reply.");
      }
      if (when === "now" && raw.state === "PENDING") {
        // GitHub put the reply in the viewer's pending review instead of
        // sending it. Take it back out, so nothing changes without asking.
        changed();
        await mutate(DELETE_COMMENT, { id: raw.id });
        throw new ReviewError(
          "reply-held",
          "GitHub adds replies to your unsent review while you have one. Add the reply to the review, or submit or discard the review first.",
        );
      }
      // A pending reply counts only once the cache has the review it's in.
      const cached =
        raw.state !== "PENDING" || (before && (await before).pendingReview?.id === reviewId);
      await patch(cached ? before : undefined, threadId, (thread) =>
        thread === undefined
          ? undefined
          : { ...thread, comments: { ...thread.comments, nodes: [...thread.comments.nodes, raw] } },
      );
      return toComment(raw);
    },

    async resolve(threadId, resolved) {
      await requireReviewThread(threadId);
      const before = state;
      const data = await mutate<Record<string, { thread: Partial<RawThread> | null } | undefined>>(
        resolved ? RESOLVE : UNRESOLVE,
        { threadId },
      );
      const answer = data[resolved ? "resolveReviewThread" : "unresolveReviewThread"]?.thread;
      await patch(answer ? before : undefined, threadId, (thread) =>
        thread === undefined || answer == null
          ? undefined
          : {
              ...thread,
              isResolved: answer.isResolved ?? resolved,
              viewerCanResolve: answer.viewerCanResolve ?? !resolved,
              viewerCanUnresolve: answer.viewerCanUnresolve ?? resolved,
            },
      );
    },

    async pending() {
      const state = await load();
      if (state.pendingReview === undefined) {
        return { id: undefined, threads: [], replies: [], conversation: [], count: 0 };
      }
      const threads: Thread[] = [];
      const replies: PendingReview["replies"] = [];
      for (const raw of state.raw) {
        const thread = toThread(raw);
        if (thread.comments.length > 0 && thread.comments.every((c) => c.pending)) {
          threads.push(thread);
        } else {
          for (const comment of thread.comments) {
            if (comment.pending) replies.push({ threadId: thread.id, comment });
          }
        }
      }
      const conversation = markedThreads(state.pendingReview, contentPrefix, true);
      return {
        id: state.pendingReview.id,
        threads,
        replies,
        conversation,
        count:
          threads.reduce((sum, thread) => sum + thread.comments.length, 0) +
          replies.length +
          conversation.length,
      };
    },

    async submit(event, body) {
      const review = await ensureReview();
      await mutate(SUBMIT_REVIEW, {
        reviewId: review.id,
        event,
        body: joinSections([withoutPlaceholder(review.body)], body) || null,
      });
      changed();
      known = undefined;
    },

    async discard() {
      const pendingReview = (await load()).pendingReview ?? known;
      if (pendingReview === undefined) return;
      await mutate(DELETE_REVIEW, { reviewId: pendingReview.id });
      changed();
      known = undefined;
      discarded = pendingReview.id;
    },

    async refresh() {
      state = undefined;
      files = undefined;
      shown.clear();
    },
  };
}

/** A link to lines of a file at the pull request's head commit. */
function blobUrl(pr: PullRequestInfo, file: string, first: number, last: number): string {
  const { host, owner, name } = pr.repository;
  const encoded = file.split("/").map(encodeURIComponent).join("/");
  const lines = first === last ? `L${first}` : `L${first}-L${last}`;
  return `https://${host}/${owner}/${name}/blob/${pr.headOid}/${encoded}#${lines}`;
}
