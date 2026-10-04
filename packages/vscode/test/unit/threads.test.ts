// The pull request's threads, apart from VS Code: what the header says about
// them (`src/preview/threadsText.ts`) and how the overlay's requests are
// answered from a session (`src/preview/threadsRequests.ts`).
import type { PendingReview, ReviewSession } from "@ascribed/review/github";
import type { LocatedThread, PageRef } from "@ascribed/review/place";
import { describe, expect, it } from "vitest";
import type { ChangedPage, ThreadsView } from "../../src/preview/protocol.js";
import { answerThreads, type RequestContext } from "../../src/preview/threadsRequests.js";
import {
  againstText,
  pagesShowing,
  pullRequestBase,
  showSourceComments,
  threadsNotice,
} from "../../src/preview/threadsText.js";

const ON: ThreadsView = {
  state: "on",
  pullRequest: { number: 128, url: "https://github.com/acme/quill/pull/128", baseRefName: "main" },
  local: { state: "same", behind: 0, ahead: 0 },
  gh: false,
  message: null,
  goTo: null,
};

describe("threadsNotice", () => {
  it("says nothing while the checkout is the pull request's head", () => {
    expect(threadsNotice(ON)).toBeUndefined();
  });

  it("offers a sign-in when signed out, and the GitHub CLI when it's signed in", () => {
    const view: ThreadsView = { ...ON, state: "signed-out", pullRequest: null, local: null };
    expect(threadsNotice(view)).toEqual({
      text: "Showing changes only. Comments need GitHub.",
      actions: [{ label: "Sign in to see comments", message: "signIn" }],
    });
    expect(threadsNotice({ ...view, gh: true })?.actions.map((a) => a.label)).toEqual([
      "Sign in to see comments",
      "Use GitHub CLI",
    ]);
  });

  it("says why the comments couldn't be read, and offers to try again", () => {
    expect(
      threadsNotice({ ...ON, state: "error", message: "Couldn't reach github.com: offline" }),
    ).toEqual({
      text: "Couldn't read the comments. Couldn't reach github.com: offline",
      actions: [{ label: "Try again", message: "refresh" }],
    });
  });

  it("says how the checkout differs from the pull request, with what helps", () => {
    const local = (state: ThreadsView["local"]) => threadsNotice({ ...ON, local: state });
    expect(local({ state: "behind", behind: 1, ahead: 0 })).toEqual({
      text: "Your checkout is 1 commit behind #128, so some comments may be on lines you don't have.",
      actions: [{ label: "Pull", message: "pull" }],
    });
    expect(local({ state: "ahead", behind: 0, ahead: 2 })).toEqual({
      text: "You have 2 commits that aren't pushed. You can comment only on lines that are on GitHub.",
      actions: [{ label: "Push", message: "push" }],
    });
    expect(local({ state: "ahead", behind: 0, ahead: 1 })?.text).toContain("1 commit that isn't");
    expect(local({ state: "missing", behind: 0, ahead: 0 })?.actions).toEqual([
      { label: "Fetch", message: "fetch" },
    ]);
    expect(local({ state: "diverged", behind: 3, ahead: 1 })?.text).toContain(
      "(3 behind, 1 ahead)",
    );
  });
});

describe("againstText", () => {
  it("names the pull request when there is one", () => {
    expect(againstText(ON)).toBe("#128 against ");
    expect(againstText(null)).toBe("Against ");
    expect(againstText({ ...ON, state: "signed-out", pullRequest: null })).toBe("Against ");
  });
});

function changed(path: string, because: string[] = [], status: ChangedPage["status"] = "changed") {
  return {
    path,
    route: `/${path}`,
    status,
    own_file_changed: because.length === 0,
    because,
    page_changed: [],
    counts: { changed: 1, added: 0, removed: 0, moved: 0 },
    title: path.toUpperCase(),
  } satisfies ChangedPage;
}

describe("pagesShowing", () => {
  it("lists the file's own page first, then the pages that include it, never removed ones", () => {
    const pages = [
      changed("a.md", ["_f/note.md"]),
      changed("_f/note.md"),
      changed("gone.md", ["_f/note.md"], "removed"),
    ];
    expect(pagesShowing("_f/note.md", pages)).toEqual([
      { path: "_f/note.md", title: "_F/NOTE.MD" },
      { path: "a.md", title: "A.MD" },
    ]);
    expect(pagesShowing("other.md", pages)).toEqual([]);
  });
});

describe("showSourceComments", () => {
  it("shows threads on source lines unless the GitHub Pull Requests extension does, with auto", () => {
    expect(showSourceComments("auto", false)).toBe(true);
    expect(showSourceComments("auto", true)).toBe(false);
    expect(showSourceComments(undefined, true)).toBe(false);
    expect(showSourceComments("on", true)).toBe(true);
    expect(showSourceComments("off", false)).toBe(false);
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

describe("answerThreads", () => {
  it("reads a page's threads with its anchors, and keeps the page for comments", async () => {
    const { session, calls } = fakeSession();
    const anchors = [{ source: "a.md:1-2", via: [] }];
    const removed = [{ source: "a.md:4-4", via: [] }];
    const answer = await answerThreads(context(session), "load", { anchors, removed });
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
    const answer = await answerThreads(context(session, page), "comment", {
      anchor,
      body: "Why?",
    });
    expect(calls).toEqual([["comment", anchor, "Why?", page]]);
    expect(answer.changed).toBe(true);
  });

  it("replies, resolves, submits, and discards, each changing the threads", async () => {
    const { session, calls } = fakeSession();
    const ctx = context(session);
    const answers = [
      await answerThreads(ctx, "reply", { threadId: "T1", body: "Done", when: "now" }),
      await answerThreads(ctx, "reply", { threadId: "T1", body: "Later", when: "withReview" }),
      await answerThreads(ctx, "resolve", { threadId: "T1", resolved: true }),
      await answerThreads(ctx, "submit", { event: "APPROVE" }),
      await answerThreads(ctx, "submit", { event: "COMMENT", body: "Thanks" }),
      await answerThreads(ctx, "discard", {}),
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
    const answer = await answerThreads(context(session), "allThreads", {});
    expect(answer.result).toEqual([
      { thread: { id: "T1", path: "a.md" }, pages: [{ path: "a.md", title: "A.MD" }] },
    ]);
  });

  it("refuses a request it can't read, before reaching the session", async () => {
    const { session, calls } = fakeSession();
    await expect(answerThreads(context(session), "submit", { event: "MERGE" })).rejects.toThrow(
      "can't be submitted",
    );
    await expect(answerThreads(context(session), "comment", { body: "x" })).rejects.toThrow(
      "missing a block",
    );
    await expect(
      answerThreads({ ...context(session), page: undefined }, "load", {}),
    ).rejects.toThrow("no page");
    expect(calls).toEqual([]);
  });
});
