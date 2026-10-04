// A review session: one pull request's threads, placed on pages, and the
// viewer's pending review that new comments go into.
import { readFileSync, realpathSync } from "node:fs";
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
  comment(anchor: Anchor, body: string, page: PageRef): Promise<Thread>;
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
}

interface State {
  raw: RawThread[];
  reviews: RawReview[];
  located: LocatedThread[];
  pendingReview: RawReview | undefined;
}

// GitHub's errors when a line can't take a review comment.
const CANT_ANCHOR =
  /could not be resolved|must be part of the diff|is not part of the diff|line.*outside/i;

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

  const load = (): Promise<State> => {
    state ??= (async () => {
      const raw = await readReviewThreads(transport, key);
      const conversation = await readConversation(transport, key);
      const reviews = await readReviews(transport, key);
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
      const located = await locateThreads(threads, {
        root,
        contentPrefix,
        headOid: pullRequest.headOid,
      });
      return { raw, reviews, located, pendingReview };
    })();
    state.catch(() => {
      state = undefined;
    });
    return state;
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

  const ensureReview = async (): Promise<{ id: string; body: string }> => {
    const pendingReview = (await load()).pendingReview;
    if (pendingReview !== undefined) return pendingReview;
    if (known !== undefined) return known;
    const data = await mutate<{ addPullRequestReview: { pullRequestReview: { id: string } } }>(
      ADD_REVIEW,
      { pullRequestId: pullRequest.id, commit: pullRequest.headOid, body: PLACEHOLDER },
    );
    changed();
    known = { id: data.addPullRequestReview.pullRequestReview.id, body: PLACEHOLDER };
    return known;
  };

  const findThread = async (threadId: string): Promise<LocatedThread | undefined> =>
    (await load()).located.find((thread) => thread.id === threadId);

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
      return placeOnPage(page, (await load()).located);
    },

    async allThreads() {
      return (await load()).located;
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

    async comment(anchor, body, page) {
      const { range, repositoryPath, first, last } = await atHead(anchor);
      if ((await changedFiles()).has(repositoryPath)) {
        const review = await ensureReview();
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
          changed();
          // GitHub answers a line away from the diff's changes with no thread
          // and no error: it can't anchor the comment there.
          const thread = data.addPullRequestReviewThread.thread;
          if (thread !== null) {
            await readRestOfComments(transport, thread);
            return toThread(thread);
          }
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
      const text = readFileSync(path.join(root, ...repositoryPath.split("/")), "utf8");
      const quote = text
        .split(/\r?\n/)
        .slice(range.first - 1, range.last)
        .join("\n");
      const source = formatSource(range);
      const section = formatSection({
        source,
        build: page.build,
        body,
        quote,
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
      const data = await mutate<{
        addPullRequestReviewThreadReply: { comment: Parameters<typeof toComment>[0] | null };
      }>(ADD_REPLY, { threadId, body, reviewId: reviewId ?? null });
      changed();
      const raw = data.addPullRequestReviewThreadReply.comment;
      if (raw === null) throw new ReviewError("refused", "GitHub didn't add the reply.");
      if (when === "now" && raw.state === "PENDING") {
        // GitHub put the reply in the viewer's pending review instead of
        // sending it. Take it back out, so nothing changes without asking.
        await mutate(DELETE_COMMENT, { id: raw.id });
        throw new ReviewError(
          "reply-held",
          "GitHub adds replies to your unsent review while you have one. Add the reply to the review, or submit or discard the review first.",
        );
      }
      return toComment(raw);
    },

    async resolve(threadId, resolved) {
      await requireReviewThread(threadId);
      await mutate(resolved ? RESOLVE : UNRESOLVE, { threadId });
      changed();
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
