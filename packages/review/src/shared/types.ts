// The shapes threads take, shared by `./github` and `./place`.

/** A comment's author: `null` when GitHub reports none (a deleted account). */
export interface Author {
  login: string;
  avatarUrl: string | undefined;
}

/** A comment in a thread. `body` is Markdown from other people, unrendered. */
export interface ThreadComment {
  id: string;
  author: Author | null;
  body: string;
  createdAt: string;
  url: string;
  /** Whether it's in the viewer's pending review: only the viewer sees it. */
  pending: boolean;
}

/** Which side of the pull request's diff a thread is on: new text, or removed text. */
export type Side = "RIGHT" | "LEFT";

/**
 * A review thread, or a conversation comment with a hidden anchor marker
 * (`kind: "conversation"`), which GitHub can't resolve.
 */
export interface Thread {
  id: string;
  kind: "review" | "conversation";
  /** The file's path in the repository, with `/`. */
  repositoryPath: string;
  /** `"file"` for a thread on a whole file, which has no lines. */
  subject: "line" | "file";
  side: Side;
  /** The thread's last line at the pull request's head, or `null` when it's outdated. */
  line: number | null;
  /** Its first line, for a multi-line thread. */
  startLine: number | null;
  /** Its last line at `originalCommit`. */
  originalLine: number | null;
  originalStartLine: number | null;
  /** The commit `line` is counted at. */
  commit: string | undefined;
  /** The commit the thread was made at. */
  originalCommit: string | undefined;
  resolved: boolean;
  outdated: boolean;
  canResolve: boolean;
  canUnresolve: boolean;
  canReply: boolean;
  comments: ThreadComment[];
  /** GitHub's diff hunk ending at the thread's line. */
  diffHunk: string | undefined;
  /** For a conversation comment: the block source and build its marker names. */
  marker: { source: string; build: string | undefined } | undefined;
  /** The text the thread is on, when known: quoted by a conversation comment, or read from git. */
  quote: string | undefined;
}

/**
 * What **Prompt agent** asks a host for: a prompt about a thread, every open
 * thread, the page's changes, or the pages a changed fragment (a content
 * path) shows on.
 */
export type PromptRequest =
  | { kind: "thread"; threadId: string }
  | { kind: "open-threads" }
  | { kind: "page-changes" }
  | { kind: "fragment-reach"; fragment: string };
