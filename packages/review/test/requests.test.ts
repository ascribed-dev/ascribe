// Answering the overlay's requests from a review session
// (`src/github/requests.ts`), for the hosts that pass them on as messages.
import { describe, expect, it } from "vitest";
import {
  answerRequest,
  pagesShowing,
  pullRequestBase,
  type ChangedPageRef,
  type RequestContext,
} from "../src/github/requests.js";
import type { PendingReview, ReviewSession } from "../src/github/session.js";
import type { LocatedThread, PageRef } from "../src/place/place.js";

function changed(
  path: string,
  because: string[] = [],
  status: ChangedPageRef["status"] = "changed",
): ChangedPageRef {
  return { path, status, because, title: path.toUpperCase() };
}

describe("pagesShowing", () => {
  it("lists the file's own page first, then the pages that include it, never removed ones", () => {
    const code = [{ type: "code", value: "a" } as const];
    const pages = [
      { ...changed("a.md", ["_f/note.md"]), formatted_title: code },
      changed("_f/note.md"),
      changed("gone.md", ["_f/note.md"], "removed"),
    ];
    expect(pagesShowing("_f/note.md", pages)).toEqual([
      { path: "_f/note.md", title: "_F/NOTE.MD", formatted_title: null },
      { path: "a.md", title: "A.MD", formatted_title: code },
    ]);
    expect(pagesShowing("other.md", pages)).toEqual([]);
  });
});

describe("pullRequestBase", () => {
  const repo = "github.com/acme/quill";
  it("uses the remote that is the pull request's repository, upstream first", () => {
    expect(
      pullRequestBase(
        [
          { name: "origin", repository: "github.com/me/quill" },
          { name: "upstream", repository: repo },
        ],
        repo,
        "main",
      ),
    ).toBe("upstream/main");
    expect(pullRequestBase([{ name: "origin", repository: repo }], repo, "main")).toBe(
      "origin/main",
    );
  });

  it("falls back to the branch's name with no such remote", () => {
    expect(pullRequestBase([{ name: "origin", repository: undefined }], repo, "dev")).toBe("dev");
  });
});

/** A session that records what it's asked, standing in for GitHub. */
function fakeSession(): { session: ReviewSession; calls: unknown[][] } {
  const calls: unknown[][] = [];
  const thread = { id: "T1", path: "a.md" } as LocatedThread;
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
    pullRequest: { number: 128, url: "https://github.com/acme/quill/pull/128" },
    threads: record("threads", { blocks: [], removed: [], detached: [] }),
    allThreads: record("allThreads", [thread]),
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

function context(session: ReviewSession, lastPage?: PageRef): RequestContext {
  return {
    session,
    page: { build: "site", path: "a.md" },
    lastPage,
    changedPages: async () => [changed("a.md")],
  };
}

describe("answerRequest", () => {
  it("reads a page's threads with its anchors, and keeps the page for comments", async () => {
    const { session, calls } = fakeSession();
    const anchors = [{ source: "a.md:1-2", via: [] }];
    const removed = [{ source: "a.md:4-4", via: [] }];
    const answer = await answerRequest(context(session), "load", { anchors, removed });
    const page = { build: "site", path: "a.md", anchors, removed };
    expect(calls[0]).toEqual(["threads", page]);
    expect(answer).toEqual({
      result: {
        pullRequest: { number: 128, url: "https://github.com/acme/quill/pull/128" },
        threads: { blocks: [], removed: [], detached: [] },
        pending: { id: undefined, threads: [], replies: [], conversation: [], count: 0 },
        viewer: "kyle",
      },
      page,
      changed: false,
    });
  });

  it("comments on the page last read, and says the threads changed", async () => {
    const { session, calls } = fakeSession();
    const page = { build: "cloud", path: "a.md", anchors: [] };
    const anchor = { source: "a.md:1-2", via: [] };
    const answer = await answerRequest(context(session, page), "comment", {
      anchor,
      body: "Why?",
      quote: "Shown text",
    });
    expect(calls).toEqual([["comment", anchor, "Why?", page, "Shown text"]]);
    expect(answer.changed).toBe(true);
  });

  it("replies, resolves, submits, and discards, each changing the threads", async () => {
    const { session, calls } = fakeSession();
    const ctx = context(session);
    const answers = [
      await answerRequest(ctx, "reply", { threadId: "T1", body: "Done", when: "now" }),
      await answerRequest(ctx, "reply", { threadId: "T1", body: "Later", when: "withReview" }),
      await answerRequest(ctx, "resolve", { threadId: "T1", resolved: true }),
      await answerRequest(ctx, "submit", { event: "APPROVE" }),
      await answerRequest(ctx, "submit", { event: "COMMENT", body: "Thanks" }),
      await answerRequest(ctx, "discard", {}),
    ];
    expect(answers.every((a) => a.changed)).toBe(true);
    expect(calls).toEqual([
      ["reply", "T1", "Done", "now"],
      ["reply", "T1", "Later", "withReview"],
      ["resolve", "T1", true],
      ["submit", "APPROVE", undefined],
      ["submit", "COMMENT", "Thanks"],
      ["discard"],
    ]);
  });

  it("lists every thread with the changed pages that show it", async () => {
    const { session } = fakeSession();
    const answer = await answerRequest(context(session), "allThreads", {});
    expect(answer.result).toEqual([
      {
        thread: { id: "T1", path: "a.md" },
        pages: [{ path: "a.md", title: "A.MD", formatted_title: null }],
      },
    ]);
  });

  it("moves blocks from the editor's unsaved lines to the file's, and back", async () => {
    const { session, calls } = fakeSession();
    // Two lines added at the top of a.md in the editor, and the third line
    // (the second block's) reworded.
    const saved = "Title\n\nFirst.\n\nSecond.\n";
    const current = "New.\n\nTitle\n\nFirst!\n\nSecond.\n";
    const placed = {
      blocks: [{ anchor: { source: "a.md:5-5", via: [] }, threads: [{ id: "T1" }] }],
      removed: [],
      detached: [],
    };
    (session as unknown as { threads: unknown }).threads = async (page: PageRef) => {
      calls.push(["threads", page]);
      return placed;
    };
    const ctx: RequestContext = {
      ...context(session),
      unsaved: async (file) => (file === "a.md" ? { saved, current } : undefined),
    };
    const shown = [
      { source: "a.md:1-1", via: [] },
      { source: "a.md:3-3", via: [] },
      { source: "a.md:5-5", via: [] },
      { source: "a.md:7-7", via: [] },
      { source: "b.md:2-2", via: ["a.md:7"] },
    ];
    const answer = await answerRequest(ctx, "load", { anchors: shown, removed: [] });
    // The new paragraph has no lines on disk; the rest move up by two.
    expect((calls[0]?.[1] as PageRef | undefined)?.anchors).toEqual([
      { source: "a.md:1-1", via: [] },
      { source: "a.md:3-3", via: [] },
      { source: "a.md:5-5", via: [] },
      { source: "b.md:2-2", via: ["a.md:5"] },
    ]);
    // The thread on disk's line 5 is on the block the editor has at line 7.
    expect((answer.result as { threads: typeof placed }).threads.blocks[0]?.anchor).toEqual({
      source: "a.md:7-7",
      via: [],
    });
    // A block with unsaved changes can't be commented on; a saved one can, at its lines on disk.
    await expect(
      answerRequest(ctx, "commentTarget", { anchor: { source: "a.md:5-5", via: [] } }),
    ).resolves.toMatchObject({
      result: { kind: "push-first", message: expect.stringContaining("Save the file") },
    });
    await expect(
      answerRequest(ctx, "comment", { anchor: { source: "a.md:1-1", via: [] }, body: "x" }),
    ).rejects.toThrow("Save the file");
    await answerRequest(ctx, "comment", { anchor: { source: "a.md:7-7", via: [] }, body: "Why?" });
    expect(calls.at(-1)?.slice(0, 2)).toEqual(["comment", { source: "a.md:5-5", via: [] }]);
  });

  it("refuses a request it can't read, before reaching the session", async () => {
    const { session, calls } = fakeSession();
    await expect(answerRequest(context(session), "submit", { event: "MERGE" })).rejects.toThrow(
      "can't be submitted",
    );
    await expect(answerRequest(context(session), "comment", { body: "x" })).rejects.toThrow(
      "missing a block",
    );
    await expect(
      answerRequest({ ...context(session), page: undefined }, "load", {}),
    ).rejects.toThrow("no page");
    expect(calls).toEqual([]);
  });
});
