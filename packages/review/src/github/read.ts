// Reading a pull request's threads, reviews, and files, page by page.
import { parseSource } from "../place/anchor.js";
import type { Author, Thread, ThreadComment } from "../shared/types.js";
import { parseSections } from "./marker.js";
import { CONVERSATION, FILES, REVIEW_THREADS, REVIEWS, THREAD_COMMENTS } from "./queries.js";
import type { GitHubTransport } from "./transport.js";

/** A pull request, by repository and number. */
export interface PullRequestKey {
  owner: string;
  name: string;
  number: number;
}

interface Page<T> {
  pageInfo: { hasNextPage: boolean; endCursor: string | null };
  nodes: (T | null)[];
}

interface PullRequestData<K extends string, T> {
  repository: { pullRequest: Record<K, Page<T>> | null } | null;
}

interface RawAuthor {
  login: string;
  avatarUrl?: string | null;
}

interface RawComment {
  id: string;
  body: string;
  createdAt: string;
  url: string;
  state: "PENDING" | "SUBMITTED";
  diffHunk: string;
  author: RawAuthor | null;
  commit: { oid: string } | null;
  originalCommit: { oid: string } | null;
}

/** A review thread as the API returns it. */
export interface RawThread {
  id: string;
  path: string;
  line: number | null;
  startLine: number | null;
  originalLine: number | null;
  originalStartLine: number | null;
  diffSide: "LEFT" | "RIGHT";
  startDiffSide: "LEFT" | "RIGHT" | null;
  isResolved: boolean;
  isOutdated: boolean;
  subjectType: "LINE" | "FILE";
  viewerCanResolve: boolean;
  viewerCanUnresolve: boolean;
  viewerCanReply: boolean;
  comments: Page<RawComment>;
}

interface RawConversationComment {
  id: string;
  body: string;
  createdAt: string;
  url: string;
  author: RawAuthor | null;
}

/** A review, as the API returns it. */
export interface RawReview extends RawConversationComment {
  state: "PENDING" | "COMMENTED" | "APPROVED" | "CHANGES_REQUESTED" | "DISMISSED";
  viewerDidAuthor: boolean;
}

/** Reads every page of one of the pull request's connections. */
async function readAll<K extends string, T>(
  transport: GitHubTransport,
  query: string,
  key: PullRequestKey,
  field: K,
): Promise<T[]> {
  const items: T[] = [];
  let after: string | null = null;
  for (;;) {
    const data: PullRequestData<K, T> = await transport.graphql(query, { ...key, after });
    const page = data.repository?.pullRequest?.[field];
    if (page === undefined) break;
    for (const node of page.nodes) if (node !== null) items.push(node);
    if (!page.pageInfo.hasNextPage || page.pageInfo.endCursor === null) break;
    after = page.pageInfo.endCursor;
  }
  return items;
}

/** Every review thread, each with all its comments. */
export async function readReviewThreads(
  transport: GitHubTransport,
  key: PullRequestKey,
): Promise<RawThread[]> {
  const threads = await readAll<"reviewThreads", RawThread>(
    transport,
    REVIEW_THREADS,
    key,
    "reviewThreads",
  );
  for (const thread of threads) await readRestOfComments(transport, thread);
  return threads;
}

/** Reads the comments past a thread's first page into it. */
export async function readRestOfComments(
  transport: GitHubTransport,
  thread: RawThread,
): Promise<void> {
  let page = thread.comments;
  while (page.pageInfo.hasNextPage && page.pageInfo.endCursor !== null) {
    const data: { node: { comments: Page<RawComment> } | null } = await transport.graphql(
      THREAD_COMMENTS,
      { id: thread.id, after: page.pageInfo.endCursor },
    );
    if (data.node === null) break;
    page = data.node.comments;
    thread.comments.nodes.push(...page.nodes);
  }
  thread.comments.pageInfo = page.pageInfo;
}

/** The pull request's conversation comments. */
export function readConversation(
  transport: GitHubTransport,
  key: PullRequestKey,
): Promise<RawConversationComment[]> {
  return readAll(transport, CONVERSATION, key, "comments");
}

/** The pull request's reviews, the viewer's pending one included. */
export function readReviews(transport: GitHubTransport, key: PullRequestKey): Promise<RawReview[]> {
  return readAll(transport, REVIEWS, key, "reviews");
}

/** The repository paths of the files the pull request changes. */
export async function readFiles(
  transport: GitHubTransport,
  key: PullRequestKey,
): Promise<Set<string>> {
  const files = await readAll<"files", { path: string }>(transport, FILES, key, "files");
  return new Set(files.map((file) => file.path));
}

function author(raw: RawAuthor | null): Author | null {
  return raw === null ? null : { login: raw.login, avatarUrl: raw.avatarUrl ?? undefined };
}

/** A comment of a review thread. */
export function toComment(raw: RawComment): ThreadComment {
  return {
    id: raw.id,
    author: author(raw.author),
    body: raw.body,
    createdAt: raw.createdAt,
    url: raw.url,
    pending: raw.state === "PENDING",
  };
}

/** A review thread. */
export function toThread(raw: RawThread): Thread {
  const comments = raw.comments.nodes.filter((node) => node !== null);
  const first = comments[0];
  return {
    id: raw.id,
    kind: "review",
    repositoryPath: raw.path,
    subject: raw.subjectType === "FILE" ? "file" : "line",
    side: raw.diffSide,
    line: raw.line,
    startLine: raw.startLine,
    originalLine: raw.originalLine,
    originalStartLine: raw.originalStartLine,
    commit: first?.commit?.oid,
    originalCommit: first?.originalCommit?.oid,
    resolved: raw.isResolved,
    outdated: raw.isOutdated,
    canResolve: raw.viewerCanResolve,
    canUnresolve: raw.viewerCanUnresolve,
    canReply: raw.viewerCanReply,
    comments: comments.map(toComment),
    diffHunk: first?.diffHunk,
    marker: undefined,
    quote: undefined,
  };
}

/**
 * The anchored comments in a conversation comment's or a review's body, as
 * threads. `contentPrefix` is the content root's path in the repository.
 */
export function markedThreads(
  raw: RawConversationComment,
  contentPrefix: string,
  pending: boolean,
): Thread[] {
  const threads: Thread[] = [];
  const { sections } = parseSections(raw.body);
  sections.forEach((section, index) => {
    const range = parseSource(section.source);
    if (range === undefined) return;
    threads.push({
      id: sections.length === 1 ? raw.id : `${raw.id}#${index}`,
      kind: "conversation",
      repositoryPath: `${contentPrefix}${range.path}`,
      subject: "line",
      side: "RIGHT",
      line: range.last,
      startLine: range.first === range.last ? null : range.first,
      originalLine: range.last,
      originalStartLine: range.first === range.last ? null : range.first,
      commit: undefined,
      originalCommit: undefined,
      resolved: false,
      outdated: false,
      canResolve: false,
      canUnresolve: false,
      canReply: false,
      comments: [
        {
          id: sections.length === 1 ? raw.id : `${raw.id}#${index}`,
          author: author(raw.author),
          body: section.body,
          createdAt: raw.createdAt,
          url: raw.url,
          pending,
        },
      ],
      diffHunk: undefined,
      marker: { source: section.source, build: section.build },
      quote: section.quote,
    });
  });
  return threads;
}
