// The site preview's review on the dev server's side, with a fake channel, a
// fake `ascribe diff`, and a fake review session standing in for GitHub.
import type { PendingReview, ReviewSession } from "@ascribed/review/github";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { DiffResult } from "../src/review/diff.js";
import type { Connection } from "../src/review/github.js";
import {
  CHANGED_EVENT,
  REQUEST_EVENT,
  RESULT_EVENT,
  type DiffPage,
  type PageView,
  type Result,
} from "../src/review/protocol.js";
import {
  channelProblem,
  ReviewServer,
  type ChannelClient,
  type ReviewServerOptions,
} from "../src/review/server.js";

/** The toolbar's channel: requests in, answers and broadcasts recorded. */
class FakeChannel {
  private listener: ((data: unknown, client?: ChannelClient) => void) | undefined;
  readonly broadcasts: { event: string; payload: unknown }[] = [];
  on(event: string, callback: (data: unknown, client?: ChannelClient) => void): void {
    if (event === REQUEST_EVENT) this.listener = callback;
  }
  send(event: string, payload: unknown): void {
    this.broadcasts.push({ event, payload });
  }
  /** Sends a request as a page would, and resolves with the answer it gets. */
  request(method: string, params: Record<string, unknown> = {}, tab = "t1"): Promise<Result> {
    return new Promise((resolve) => {
      const client: ChannelClient = {
        send: (event, payload) => {
          if (event === RESULT_EVENT) resolve(payload as Result);
        },
      };
      this.listener?.({ tab, id: 1, method, params }, client);
    });
  }
}

function page(path: string, route: string, init: Partial<DiffPage> = {}): DiffPage {
  return {
    path,
    route,
    status: "changed",
    own_file_changed: true,
    because: [],
    page_changed: [],
    counts: { changed: 1, added: 0, removed: 0, moved: 0 },
    changes: [{ kind: "changed", now: { source: `${path}:3-3`, via: [] } }],
    ...init,
  };
}

const DIFF: DiffResult = {
  base: { requested: "origin/main", commit: "abc1234def", merge_base: "abc1234def" },
  pages: [page("guide.md", "/docs/guide"), page("gone.md", "/docs/gone", { status: "removed" })],
};

/** A session that records what it's asked. */
function fakeSession(): { session: ReviewSession; calls: unknown[][] } {
  const calls: unknown[][] = [];
  const pending: PendingReview = {
    id: undefined,
    threads: [],
    replies: [],
    conversation: [],
    count: 0,
  };
  const record =
    (name: string, value?: unknown) =>
    async (...args: unknown[]) => {
      calls.push([name, ...args]);
      return value;
    };
  const session = {
    pullRequest: {
      number: 7,
      url: "https://github.com/acme/docs/pull/7",
      baseRefName: "main",
      local: "same",
    },
    threads: record("threads", { blocks: [], removed: [], detached: [] }),
    allThreads: record("allThreads", []),
    viewer: record("viewer", "kyle"),
    commentTarget: record("commentTarget", { kind: "thread" }),
    comment: record("comment", { id: "T2" }),
    reply: record("reply", { id: "C1" }),
    resolve: record("resolve"),
    pending: record("pending", pending),
    submit: record("submit"),
    discard: record("discard"),
    refresh: record("refresh"),
  } as unknown as ReviewSession;
  return { session, calls };
}

function serve(init: Partial<ReviewServerOptions> = {}) {
  const channel = new FakeChannel();
  const { session, calls } = fakeSession();
  const diff = vi.fn(async (_base: string | undefined) => DIFF);
  const connect = vi.fn(async (): Promise<Connection> => ({
    state: "on",
    session,
    base: "origin/main",
    local: { state: "same", behind: 0, ahead: 0 },
  }));
  const writeRoutes = vi.fn(async () => undefined);
  const readRoutes = vi.fn(
    async () =>
      new Map([
        ["/docs/plain", { path: "plain.md", title: "Plain" }],
        ["/docs/guide", { path: "guide.md", title: "The guide" }],
      ]),
  );
  const server = new ReviewServer({
    channel,
    logger: { info: () => undefined, warn: () => undefined },
    build: "site",
    contentRoot: "/p/content",
    channelProblem: undefined,
    diff,
    connect,
    writeRoutes,
    readRoutes,
    debounceMs: 0,
    ...init,
  });
  return { channel, server, calls, diff, connect, writeRoutes, readRoutes };
}

afterEach(() => {
  vi.useRealTimers();
});

describe("ReviewServer", () => {
  it("runs nothing until review starts", async () => {
    const { channel, diff, connect, writeRoutes } = serve();
    expect((await channel.request("status")).result).toEqual({ on: false });
    expect((await channel.request("page", { route: "/docs/guide" })).error?.message).toBe(
      "Review is off.",
    );
    expect(diff).not.toHaveBeenCalled();
    expect(connect).not.toHaveBeenCalled();
    expect(writeRoutes).not.toHaveBeenCalled();
  });

  it("starts by finding the pull request and comparing with its base", async () => {
    const { channel, server, diff, connect, writeRoutes } = serve();
    expect((await channel.request("start")).result).toEqual({ on: true });
    expect(server.active).toBe(true);
    expect(connect).toHaveBeenCalledTimes(1);
    expect(diff).toHaveBeenCalledWith("origin/main");
    expect(writeRoutes).toHaveBeenCalledTimes(1);
    expect((await channel.request("status")).result).toEqual({ on: true });
  });

  it("answers a changed page by its route, with the pull request and the changed pages", async () => {
    const { channel } = serve();
    await channel.request("start");
    const view = (await channel.request("page", { route: "/docs/guide/", path: "guide.md" }))
      .result as PageView;
    expect(view.kind).toBe("page");
    expect(view.path).toBe("guide.md");
    expect(view.page?.changes).toHaveLength(1);
    expect(view.base?.requested).toBe("origin/main");
    expect(view.threads).toEqual({
      state: "on",
      pullRequest: { number: 7, url: "https://github.com/acme/docs/pull/7", baseRefName: "main" },
      local: { state: "same", behind: 0, ahead: 0 },
    });
    expect(view.changedPages.map((p) => [p.path, p.title])).toEqual([
      ["guide.md", "The guide"],
      // A removed page isn't in the build, so it has no title.
      ["gone.md", null],
    ]);
    expect(view.changedPages[0]).not.toHaveProperty("changes");
    expect(view.contentRoot).toBe("/p/content");
  });

  it("takes an unchanged page from its blocks, or from the routes, and says when a route isn't a page", async () => {
    const { channel, readRoutes } = serve();
    await channel.request("start");
    const fromBlocks = (await channel.request("page", { route: "/docs/other", path: "other.md" }))
      .result as PageView;
    expect(fromBlocks).toMatchObject({ kind: "page", path: "other.md", page: null });
    // No anchors arrived: the route says which page it is.
    const fromRoutes = (await channel.request("page", { route: "/docs/plain", path: null }))
      .result as PageView;
    expect(fromRoutes).toMatchObject({ kind: "page", path: "plain.md", page: null });
    const notPage = (await channel.request("page", { route: "/docs/changelog", path: null }))
      .result as PageView;
    expect(notPage).toMatchObject({ kind: "not-page", path: null, page: null });
    // A removed page's old route isn't a page any more.
    const gone = (await channel.request("page", { route: "/docs/gone", path: null }))
      .result as PageView;
    expect(gone.kind).toBe("not-page");
    // The routes are read once, until the next rebuild.
    expect(readRoutes).toHaveBeenCalledTimes(1);
  });

  it("compares again after a rebuild, once", async () => {
    const { channel, server, diff } = serve();
    await channel.request("start");
    expect(diff).toHaveBeenCalledTimes(1);
    server.rebuilt();
    server.rebuilt();
    await channel.request("page", { route: "/docs/guide", path: "guide.md" });
    expect(diff).toHaveBeenCalledTimes(2);
    await new Promise((resolve) => setTimeout(resolve, 5));
    await channel.request("page", { route: "/docs/guide", path: "guide.md" });
    expect(diff).toHaveBeenCalledTimes(2);
  });

  it("passes the overlay's requests to the session, for the page they name, and tells other pages", async () => {
    const { channel, calls } = serve();
    await channel.request("start");
    const anchors = [{ source: "guide.md:3-3", via: [] }];
    const load = await channel.request("load", { path: "guide.md", anchors, removed: [] });
    expect(load.result).toMatchObject({ pullRequest: { number: 7 }, viewer: "kyle" });
    expect(calls[0]).toEqual([
      "threads",
      { build: "site", path: "guide.md", anchors, removed: [] },
    ]);
    await channel.request(
      "comment",
      { path: "guide.md", anchor: anchors[0], body: "Why?", quote: "Shown text" },
      "t2",
    );
    expect(calls.find((c) => c[0] === "comment")).toEqual([
      "comment",
      anchors[0],
      "Why?",
      { build: "site", path: "guide.md", anchors, removed: [] },
      "Shown text",
    ]);
    expect(channel.broadcasts).toContainEqual({ event: CHANGED_EVENT, payload: { from: "t2" } });
  });

  it("answers a failure with its message and code", async () => {
    const { channel, calls } = serve();
    await channel.request("start");
    const { session } = fakeSession();
    void session;
    const answer = await channel.request("submit", { path: "guide.md", event: "MERGE" });
    expect(answer.error?.message).toContain("can't be submitted");
    expect(answer.error?.code).toBe("refused");
    expect((await channel.request("nonsense")).error?.message).toBe("Unknown request: nonsense.");
    expect(calls.some((c) => c[0] === "submit")).toBe(false);
  });

  it("shows changes only without a pull request, compared with the default branch", async () => {
    const { channel, diff } = serve({
      connect: async () => ({ state: "no-pull-request", message: "No pull request." }),
    });
    await channel.request("start");
    expect(diff).toHaveBeenCalledWith(undefined);
    const view = (await channel.request("page", { route: "/docs/guide", path: "guide.md" }))
      .result as PageView;
    expect(view.threads).toEqual({ state: "no-pull-request", message: "No pull request." });
    expect((await channel.request("load", { path: "guide.md" })).error?.message).toBe(
      "Review comments aren't on.",
    );
  });

  it("keeps GitHub off when other pages could use the channel", async () => {
    const { channel, connect } = serve({
      channelProblem:
        "other web pages could talk to this dev server: `server.cors` lets any origin read its pages",
    });
    await channel.request("start");
    expect(connect).not.toHaveBeenCalled();
    const view = (await channel.request("page", { route: "/docs/guide", path: "guide.md" }))
      .result as PageView;
    expect(view.threads.state).toBe("unprotected");
    expect(view.page?.changes).toHaveLength(1);
  });

  it("keeps GitHub off when the server listens on the network", async () => {
    const { server, channel, connect } = serve();
    server.listening("127.0.0.1");
    server.listening("::1");
    server.listening("::");
    await channel.request("start");
    expect(connect).not.toHaveBeenCalled();
    const view = (await channel.request("page", { route: "/docs/guide", path: "guide.md" }))
      .result as PageView;
    expect(view.threads).toMatchObject({ state: "unprotected" });
    expect(JSON.stringify(view.threads)).toContain("listening on the network");
    expect(view.page?.changes).toHaveLength(1);
  });

  it("reports a failed comparison and still answers", async () => {
    const { channel } = serve({
      diff: async () => {
        throw new Error("unknown revision origin/main");
      },
    });
    await channel.request("start");
    const view = (await channel.request("page", { route: "/docs/guide", path: "guide.md" }))
      .result as PageView;
    expect(view.problem).toBe("unknown revision origin/main");
    expect(view.page).toBeNull();
  });

  it("refreshes the session, or tries to connect again, and stops", async () => {
    let connected = false;
    const { session, calls } = fakeSession();
    const { channel, diff } = serve({
      connect: async () =>
        connected
          ? {
              state: "on",
              session,
              base: "origin/main",
              local: { state: "same", behind: 0, ahead: 0 },
            }
          : { state: "signed-out", message: "Sign in." },
    });
    await channel.request("start");
    connected = true;
    await channel.request("refresh");
    expect(diff).toHaveBeenLastCalledWith("origin/main");
    await channel.request("refresh");
    expect(calls).toContainEqual(["refresh"]);
    expect((await channel.request("stop")).result).toEqual({ on: false });
    expect((await channel.request("status")).result).toEqual({ on: false });
  });
});

describe("channelProblem", () => {
  it("accepts Vite's defaults", () => {
    expect(channelProblem({ server: {} })).toBeUndefined();
    expect(
      channelProblem({ server: { cors: { origin: /localhost/ }, allowedHosts: [] } }),
    ).toBeUndefined();
  });

  it("names a setting that lets other pages in", () => {
    expect(channelProblem({ server: { cors: true } })).toContain("server.cors");
    expect(channelProblem({ server: { cors: { origin: "*" } } })).toContain("server.cors");
    expect(channelProblem({ server: { allowedHosts: true } })).toContain("allowedHosts");
    expect(channelProblem({ server: {}, legacy: { skipWebSocketTokenCheck: true } })).toContain(
      "skipWebSocketTokenCheck",
    );
  });

  it("names a server that listens on the network", () => {
    for (const host of [undefined, false, "localhost", "127.0.0.1", "::1", "[::1]"]) {
      expect(channelProblem({ server: { host } }), String(host)).toBeUndefined();
    }
    for (const host of [true, "0.0.0.0", "::", "192.168.0.41", "example.local"]) {
      expect(channelProblem({ server: { host } }), String(host)).toContain("--host");
    }
  });
});
