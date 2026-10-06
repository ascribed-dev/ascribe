// A fake GitHub GraphQL API, and builders for its responses in the API's
// shapes (`PullRequestReviewThread`, `PullRequestReviewComment`, and so on).
import type { GitHubTransport } from "../../src/github/transport.js";

export interface Call {
  operation: string;
  query: string;
  variables: Record<string, unknown>;
}

type Handler = (variables: Record<string, unknown>) => unknown;

/** A transport that answers each operation with its handler and records every call. */
export class FakeGitHub implements GitHubTransport {
  calls: Call[] = [];
  handlers: Record<string, Handler> = {};

  on(operation: string, handler: Handler): this {
    this.handlers[operation] = handler;
    return this;
  }

  async graphql<T>(query: string, variables: Record<string, unknown>): Promise<T> {
    const operation = /\b(?:query|mutation) (\w+)/.exec(query)?.[1] ?? "?";
    this.calls.push({ operation, query, variables });
    const handler = this.handlers[operation];
    if (handler === undefined) throw new Error(`no handler for ${operation}`);
    return structuredClone(await handler(variables)) as T;
  }

  /** The operations called, in order. */
  operations(): string[] {
    return this.calls.map((call) => call.operation);
  }

  mutations(): Call[] {
    return this.calls.filter((call) => /^\s*mutation\b/.test(call.query));
  }
}

let next = 0;

export interface CommentInit {
  id?: string;
  body?: string;
  login?: string;
  state?: "PENDING" | "SUBMITTED";
  diffHunk?: string;
  commit?: string;
  originalCommit?: string;
}

export function comment(init: CommentInit = {}) {
  const id = init.id ?? `PRRC_${++next}`;
  return {
    id,
    body: init.body ?? "A comment",
    createdAt: "2026-10-01T12:00:00Z",
    url: `https://github.com/acme/docs/pull/7#discussion_r${next}`,
    state: init.state ?? "SUBMITTED",
    diffHunk: init.diffHunk ?? "@@ -1,1 +1,1 @@\n line",
    author: { login: init.login ?? "reviewer", avatarUrl: "https://avatars.example/u" },
    commit: { oid: init.commit ?? "head" },
    originalCommit: { oid: init.originalCommit ?? init.commit ?? "head" },
  };
}

export interface ThreadInit {
  id?: string;
  path: string;
  line?: number | null;
  startLine?: number | null;
  originalLine?: number | null;
  originalStartLine?: number | null;
  side?: "LEFT" | "RIGHT";
  resolved?: boolean;
  outdated?: boolean;
  subject?: "LINE" | "FILE";
  comments?: ReturnType<typeof comment>[];
  moreComments?: string | null;
}

export function thread(init: ThreadInit) {
  const line = init.line === undefined ? 1 : init.line;
  return {
    id: init.id ?? `PRRT_${++next}`,
    path: init.path,
    line,
    startLine: init.startLine ?? null,
    originalLine: init.originalLine === undefined ? line : init.originalLine,
    originalStartLine: init.originalStartLine ?? init.startLine ?? null,
    diffSide: init.side ?? "RIGHT",
    startDiffSide:
      init.startLine === undefined || init.startLine === null ? null : (init.side ?? "RIGHT"),
    isResolved: init.resolved ?? false,
    isOutdated: init.outdated ?? false,
    subjectType: init.subject ?? "LINE",
    viewerCanResolve: true,
    viewerCanUnresolve: true,
    viewerCanReply: true,
    comments: {
      pageInfo: {
        hasNextPage: init.moreComments !== undefined && init.moreComments !== null,
        endCursor: init.moreComments ?? null,
      },
      nodes: init.comments ?? [comment()],
    },
  };
}

/** One page of a pull request connection, wrapped as `repository.pullRequest.<field>`. */
function page<T>(field: string, nodes: T[], endCursor: string | null = null) {
  return {
    repository: {
      pullRequest: {
        [field]: { pageInfo: { hasNextPage: endCursor !== null, endCursor }, nodes },
      },
    },
  };
}

/** Serves a connection's items 100 at a time, with cursors. */
export function paged<T>(field: string, items: T[]) {
  return (variables: Record<string, unknown>) => {
    const start = typeof variables["after"] === "string" ? Number(variables["after"]) : 0;
    const end = start + 100;
    return page(field, items.slice(start, end), end < items.length ? String(end) : null);
  };
}

export function review(init: {
  id?: string;
  state?: "PENDING" | "COMMENTED" | "APPROVED";
  body?: string;
  mine?: boolean;
}) {
  return {
    id: init.id ?? `PRR_${++next}`,
    state: init.state ?? "COMMENTED",
    body: init.body ?? "",
    createdAt: "2026-10-01T12:00:00Z",
    url: "https://github.com/acme/docs/pull/7#pullrequestreview-1",
    viewerDidAuthor: init.mine ?? false,
    author: { login: init.mine === true ? "me" : "reviewer", avatarUrl: null },
  };
}
