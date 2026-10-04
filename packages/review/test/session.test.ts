import { afterAll, beforeAll, beforeEach, describe, expect, test } from "vitest";
import { createSession, findPullRequest, openReview } from "../src/github/session.js";
import type { PullRequestInfo, ReviewSession } from "../src/github/session.js";
import { readCheckout } from "../src/github/repository.js";
import { ReviewError } from "../src/shared/errors.js";
import type { PageRef } from "../src/place/place.js";
import { comment, FakeGitHub, paged, review, thread } from "./helpers/github.js";
import { numbered, tempRepo, type TempRepo } from "./helpers/repo.js";

// The repository: a project in `site/` whose content root is `site/content/`.
//
// At the diff's base (B), guides/install.md has 33 lines: `line 1`..`line 9`,
// then `old 1`..`old 3`, then `line 10`..`line 30`. The pull request's head
// (H) removes the three `old` lines. The working tree (W) inserts two lines
// after line 1 and deletes `line 20`, so H's line n is W's n + 2 for
// 2 <= n <= 19, gone for 20, and n + 1 above it.
const PREFIX = "site/content/";
const INSTALL = `${PREFIX}guides/install.md`;
const FRAGMENT = `${PREFIX}_fragments/prereqs.md`;

let repo: TempRepo;
let base: string;
let head: string;

beforeAll(() => {
  repo = tempRepo();
  const h = numbered(30);
  const b = [...h.slice(0, 9), "old 1", "old 2", "old 3", ...h.slice(9)];
  repo.write("README.md", "Outside the project\n");
  repo.write("site/ascribe.toml", '[project]\ncontent-root = "content"\n');
  repo.write(INSTALL, `${b.join("\n")}\n`);
  repo.write(FRAGMENT, "p1\np2\np3\n");
  repo.write(`${PREFIX}guides/other.md`, "# Other\n");
  base = repo.commit("base");
  repo.git("checkout", "--quiet", "-b", "feature");
  repo.write(INSTALL, `${h.join("\n")}\n`);
  head = repo.commit("head");
  const w = [...h];
  w.splice(19, 1);
  w.splice(1, 0, "new a", "new b");
  repo.write(INSTALL, `${w.join("\n")}\n`);
  repo.git("remote", "add", "origin", "https://github.com/acme/docs.git");
  repo.git("config", "branch.feature.remote", "origin");
  repo.git("config", "branch.feature.merge", "refs/heads/feature");
});

afterAll(() => repo.remove());

function pullRequest(overrides: Partial<PullRequestInfo> = {}): PullRequestInfo {
  return {
    id: "PR_7",
    number: 7,
    url: "https://github.com/acme/docs/pull/7",
    repository: { host: "github.com", owner: "acme", name: "docs" },
    baseRefName: "main",
    baseOid: base,
    headRefName: "feature",
    headOid: head,
    local: "same",
    ...overrides,
  };
}

/** A fake API serving these threads, conversation comments, reviews, and files. */
function github(
  options: {
    threads?: ReturnType<typeof thread>[];
    conversation?: { id: string; body: string; createdAt: string; url: string; author: null }[];
    reviews?: ReturnType<typeof review>[];
    files?: string[];
  } = {},
) {
  return new FakeGitHub()
    .on("ReviewThreads", paged("reviewThreads", options.threads ?? []))
    .on("Conversation", paged("comments", options.conversation ?? []))
    .on("Reviews", paged("reviews", options.reviews ?? []))
    .on(
      "Files",
      paged(
        "files",
        (options.files ?? [INSTALL]).map((path) => ({ path })),
      ),
    );
}

function session(fake: FakeGitHub, overrides: Partial<PullRequestInfo> = {}): ReviewSession {
  return createSession({
    root: repo.root,
    contentPrefix: PREFIX,
    pullRequest: pullRequest(overrides),
    transport: fake,
    mutationInterval: 0,
  });
}

const installPage: PageRef = {
  build: "site",
  path: "guides/install.md",
  anchors: [
    { source: "guides/install.md:1-1", via: [] },
    { source: "guides/install.md:4-30", via: [] },
    { source: "guides/install.md:7-8", via: [] },
    { source: "guides/install.md:14-16", via: [] },
    { source: "_fragments/prereqs.md:1-3", via: ["guides/install.md:25"] },
    { source: "_fragments/prereqs.md:1-3", via: ["guides/install.md:28"] },
  ],
  removed: [{ source: "guides/install.md:10-12", via: [] }],
};

const otherPage: PageRef = {
  build: "site",
  path: "guides/other.md",
  anchors: [
    { source: "guides/other.md:1-1", via: [] },
    { source: "_fragments/prereqs.md:1-3", via: ["guides/other.md:3"] },
  ],
};

function threadIds(placed: { threads: { id: string }[] }[]): Record<string, string[]> {
  return Object.fromEntries(
    placed.map((block) => [
      `${(block as unknown as { anchor: { source: string; via: string[] } }).anchor.source} ${(block as unknown as { anchor: { via: string[] } }).anchor.via.join(" ")}`.trim(),
      block.threads.map((t) => t.id),
    ]),
  );
}

describe("placing threads", () => {
  // Built in each test: the commits exist only once `beforeAll` has run.
  let threads: ReturnType<typeof thread>[] = [];
  beforeEach(() => {
    threads = fixtureThreads();
  });
  const fixtureThreads = () => [
    thread({ id: "right", path: INSTALL, line: 5 }),
    thread({ id: "left", path: INSTALL, line: 11, side: "LEFT" }),
    thread({ id: "multi", path: INSTALL, startLine: 12, line: 14 }),
    thread({ id: "fragment", path: FRAGMENT, line: 2 }),
    thread({
      id: "outdated",
      path: INSTALL,
      line: null,
      originalLine: 15,
      outdated: true,
      comments: [comment({ commit: head, originalCommit: base })],
    }),
    thread({
      id: "outdated-gone",
      path: INSTALL,
      line: null,
      originalLine: 11,
      outdated: true,
      comments: [comment({ commit: head, originalCommit: base })],
    }),
    thread({ id: "removed-locally", path: INSTALL, line: 20 }),
    thread({ id: "outside", path: "README.md", line: 1 }),
    thread({ id: "whole-file", path: INSTALL, line: null, subject: "FILE" }),
  ];

  test("puts each thread on its block, and keeps detached ones", async () => {
    const placed = await session(github({ threads })).threads(installPage);
    expect(threadIds(placed.blocks)).toEqual({
      // H line 5 is W line 7.
      "guides/install.md:7-8": ["right"],
      // H lines 12-14 are W lines 14-16.
      // And the outdated thread, at B line 15, which is H line 12 and W line 14.
      "guides/install.md:14-16": ["multi", "outdated"],
      "_fragments/prereqs.md:1-3 guides/install.md:25": ["fragment"],
      "_fragments/prereqs.md:1-3 guides/install.md:28": ["fragment"],
    });
    expect(threadIds(placed.removed)).toEqual({ "guides/install.md:10-12": ["left"] });
    expect(placed.detached.map((t) => [t.id, t.detached, t.quote])).toEqual([
      ["outdated-gone", "line-gone", "old 2"],
      ["removed-locally", "line-gone", "line 20"],
      ["whole-file", "file", undefined],
    ]);
  });

  test("marks outdated threads, with their original text", async () => {
    const placed = await session(github({ threads })).threads(installPage);
    const outdated = placed.blocks.flatMap((b) => b.threads).find((t) => t.id === "outdated");
    expect(outdated).toMatchObject({
      outdated: true,
      lines: { first: 14, last: 14 },
      quote: "line 12",
    });
  });

  test("keeps a thread on reworded text in its place, marked outdated, with what it said", async () => {
    const original = repo.read(INSTALL);
    try {
      // H line 5 is W line 7.
      repo.write(INSTALL, original.replace("line 5\n", "line five\n"));
      const placed = await session(
        github({ threads: [threads[0] as ReturnType<typeof thread>] }),
      ).threads(installPage);
      expect(placed.blocks.flatMap((b) => b.threads)).toMatchObject([
        { id: "right", outdated: true, lines: { first: 7, last: 7 }, quote: "line 5" },
      ]);
    } finally {
      repo.write(INSTALL, original);
    }
  });

  test("shows a fragment's thread on every page that includes it", async () => {
    const placed = await session(github({ threads })).threads(otherPage);
    expect(threadIds(placed.blocks)).toEqual({
      "_fragments/prereqs.md:1-3 guides/other.md:3": ["fragment"],
    });
    expect(placed.detached).toEqual([]);
  });

  test("leaves out files outside the content root", async () => {
    const placed = await session(github({ threads })).threads(installPage);
    const all = [...placed.blocks, ...placed.removed].flatMap((b) => b.threads);
    expect([...all, ...placed.detached].some((t) => t.id === "outside")).toBe(false);
  });

  test("detaches a thread no block on the page holds", async () => {
    const sparse: PageRef = {
      ...installPage,
      anchors: [{ source: "guides/install.md:1-1", via: [] }],
    };
    const placed = await session(
      github({ threads: [threads[0] as ReturnType<typeof thread>] }),
    ).threads(sparse);
    expect(placed.detached.map((t) => [t.id, t.detached])).toEqual([["right", "no-block"]]);
  });

  test("detaches every line thread when the head commit isn't fetched", async () => {
    const placed = await session(
      github({ threads: [thread({ id: "right", path: INSTALL, line: 5 })] }),
      {
        headOid: "f".repeat(40),
        local: "missing",
      },
    ).threads(installPage);
    expect(placed.detached.map((t) => [t.id, t.detached])).toEqual([["right", "line-gone"]]);
  });

  test("places conversation comments by their marker, for their build only", async () => {
    const body =
      "> p2\n\nWhy here?\n\n<sub>On [x](https://example.com)</sub>\n<!-- ascribe:anchor _fragments/prereqs.md:2-2 build=site -->";
    const conversation = [
      { id: "IC_1", body, createdAt: "2026-10-01T12:00:00Z", url: "u", author: null },
      {
        id: "IC_2",
        body: "Other build\n\n<!-- ascribe:anchor guides/install.md:7-7 build=cloud -->",
        createdAt: "2026-10-01T12:00:00Z",
        url: "u",
        author: null,
      },
    ];
    const placed = await session(github({ conversation })).threads(installPage);
    expect(threadIds(placed.blocks)).toEqual({
      "_fragments/prereqs.md:1-3 guides/install.md:25": ["IC_1"],
      "_fragments/prereqs.md:1-3 guides/install.md:28": ["IC_1"],
    });
    const [ic] = placed.blocks[0]?.threads ?? [];
    expect(ic).toMatchObject({
      kind: "conversation",
      canResolve: false,
      quote: "p2",
      comments: [{ body: "Why here?", pending: false }],
    });
  });

  test("reads every page of threads and of a thread's comments", async () => {
    const many = Array.from({ length: 150 }, (_, i) =>
      thread({ id: `t${i}`, path: INSTALL, line: 5 }),
    );
    many[3] = thread({ id: "t3", path: INSTALL, line: 5, moreComments: "c1" });
    const fake = github({ threads: many }).on("ThreadComments", () => ({
      node: {
        comments: {
          pageInfo: { hasNextPage: false, endCursor: null },
          nodes: [comment({ id: "later", body: "Comment 101" })],
        },
      },
    }));
    const placed = await session(fake).threads(installPage);
    const onBlock = placed.blocks.find((b) => b.anchor.source === "guides/install.md:7-8");
    expect(onBlock?.threads).toHaveLength(150);
    expect(onBlock?.threads.find((t) => t.id === "t3")?.comments.map((c) => c.id)).toContain(
      "later",
    );
    expect(fake.operations().filter((op) => op === "ReviewThreads")).toHaveLength(2);
    expect(fake.calls.find((c) => c.operation === "ThreadComments")?.variables).toEqual({
      id: "t3",
      after: "c1",
    });
  });

  test("caches until refresh", async () => {
    const fake = github({ threads });
    const s = session(fake);
    await s.threads(installPage);
    await s.threads(otherPage);
    expect(fake.operations().filter((op) => op === "ReviewThreads")).toHaveLength(1);
    await s.refresh();
    await s.threads(installPage);
    expect(fake.operations().filter((op) => op === "ReviewThreads")).toHaveLength(2);
  });
});

describe("posting", () => {
  /**
   * A fake that posts the way GitHub does: a line away from the diff's changes
   * gets no thread and no error, and a pending review created without a body
   * can't be given one.
   */
  function postingFake(reviews: ReturnType<typeof review>[] = [], diffLines = [1, 5, 6]) {
    const bodies = new Map(reviews.map((r) => [r.id, r.body]));
    return github({ reviews })
      .on("AddReview", (v) => {
        bodies.set("PRR_new", (v["body"] as string | undefined) ?? "");
        return { addPullRequestReview: { pullRequestReview: { id: "PRR_new" } } };
      })
      .on("AddThread", (v) => ({
        addPullRequestReviewThread: {
          thread: diffLines.includes(Number(v["line"]))
            ? thread({
                id: "PRRT_new",
                path: String(v["path"]),
                line: Number(v["line"]),
                startLine: v["startLine"] as number | null,
                comments: [comment({ body: String(v["body"]), state: "PENDING" })],
              })
            : null,
        },
      }))
      .on("UpdateReview", (v) => {
        const id = String(v["reviewId"]);
        if (bodies.get(id) === "") {
          throw new ReviewError(
            "refused",
            "GitHub refused the request: Could not edit a review with a missing body.",
          );
        }
        bodies.set(id, String(v["body"]));
        return { updatePullRequestReview: { pullRequestReview: { id } } };
      });
  }

  test("a comment on a block becomes a thread at its lines at the head commit, in a new pending review", async () => {
    const fake = postingFake();
    const created = await session(fake).comment(
      { source: "guides/install.md:7-8", via: [] },
      "Unclear.",
      installPage,
    );
    expect(fake.mutations().map((c) => [c.operation, c.variables])).toEqual([
      ["AddReview", { pullRequestId: "PR_7", commit: head, body: "<!-- ascribe:review -->" }],
      [
        "AddThread",
        { reviewId: "PRR_new", path: INSTALL, body: "Unclear.", line: 6, startLine: 5 },
      ],
    ]);
    expect(created).toMatchObject({ id: "PRRT_new", comments: [{ pending: true }] });
  });

  test("a one-line block has no start line, and an existing pending review is reused", async () => {
    const fake = postingFake([review({ id: "PRR_mine", state: "PENDING", mine: true })]);
    await session(fake).comment(
      { source: "guides/install.md:1-1", via: [] },
      "Title?",
      installPage,
    );
    expect(fake.mutations().map((c) => [c.operation, c.variables])).toEqual([
      [
        "AddThread",
        { reviewId: "PRR_mine", path: INSTALL, body: "Title?", line: 1, startLine: null },
      ],
    ]);
  });

  test("refuses a block whose lines aren't pushed", async () => {
    const fake = postingFake();
    await expect(
      session(fake).comment({ source: "guides/install.md:2-3", via: [] }, "x", installPage),
    ).rejects.toMatchObject({ code: "push-first" });
    expect(fake.mutations()).toEqual([]);
  });

  test("a block in a file the pull request doesn't change goes to the conversation, held in the review", async () => {
    const fake = postingFake([
      review({ id: "PRR_mine", state: "PENDING", mine: true, body: "Earlier" }),
    ]);
    const created = await session(fake).comment(
      { source: "_fragments/prereqs.md:1-3", via: ["guides/install.md:25"] },
      "Should this mention Windows?",
      installPage,
    );
    const [update] = fake.mutations();
    expect(update?.operation).toBe("UpdateReview");
    expect(update?.variables["reviewId"]).toBe("PRR_mine");
    const body = String(update?.variables["body"]);
    expect(body.startsWith("Earlier\n\n> p1\n> p2\n> p3\n\nShould this mention Windows?")).toBe(
      true,
    );
    expect(body).toContain(
      `(https://github.com/acme/docs/blob/${head}/site/content/_fragments/prereqs.md#L1-L3)`,
    );
    expect(body.endsWith("<!-- ascribe:anchor _fragments/prereqs.md:1-3 build=site -->")).toBe(
      true,
    );
    expect(created).toMatchObject({
      kind: "conversation",
      marker: { source: "_fragments/prereqs.md:1-3", build: "site" },
      comments: [{ body: "Should this mention Windows?", pending: true }],
    });
  });

  test("falls back to the conversation when GitHub can't anchor the line", async () => {
    const fake = postingFake().on("AddThread", () => {
      throw new ReviewError("refused", "GitHub refused the request: Line could not be resolved");
    });
    const created = await session(fake).comment(
      { source: "guides/install.md:7-8", via: [] },
      "Hm.",
      installPage,
    );
    expect(fake.mutations().map((c) => c.operation)).toEqual([
      "AddReview",
      "AddThread",
      "UpdateReview",
    ]);
    expect(created.kind).toBe("conversation");
  });

  test("falls back to the conversation when GitHub returns no thread for a line away from the diff", async () => {
    const fake = postingFake();
    const created = await session(fake).comment(
      { source: "guides/install.md:4-30", via: [] },
      "Long.",
      installPage,
    );
    const mutations = fake.mutations();
    expect(mutations.map((c) => c.operation)).toEqual(["AddReview", "AddThread", "UpdateReview"]);
    // The placeholder the review was created with is replaced by the comment.
    const body = String(mutations[2]?.variables["body"]);
    expect(body).not.toContain("ascribe:review");
    expect(body.endsWith("<!-- ascribe:anchor guides/install.md:4-30 build=site -->")).toBe(true);
    expect(created).toMatchObject({ kind: "conversation", comments: [{ pending: true }] });
  });

  test("a pending review started on GitHub without a body can't hold a comment", async () => {
    const fake = postingFake([review({ id: "PRR_web", state: "PENDING", mine: true })]);
    await expect(
      session(fake).comment(
        { source: "_fragments/prereqs.md:1-3", via: ["guides/install.md:25"] },
        "x",
        installPage,
      ),
    ).rejects.toMatchObject({ code: "cant-hold" });
  });

  test("refuses a block with lines reworded locally", async () => {
    const fake = postingFake();
    const original = repo.read(INSTALL);
    try {
      repo.write(INSTALL, original.replace("line 4\n", "line 4, reworded\n"));
      await expect(
        session(fake).comment({ source: "guides/install.md:6-6", via: [] }, "x", installPage),
      ).rejects.toMatchObject({ code: "push-first" });
    } finally {
      repo.write(INSTALL, original);
    }
    expect(fake.mutations()).toEqual([]);
  });

  test("held conversation comments are read back and counted as pending", async () => {
    const body = "Q?\n\n<!-- ascribe:anchor _fragments/prereqs.md:1-3 build=site -->";
    const fake = github({
      threads: [
        thread({ id: "new", path: INSTALL, line: 5, comments: [comment({ state: "PENDING" })] }),
        thread({
          id: "old",
          path: INSTALL,
          line: 5,
          comments: [comment(), comment({ id: "my-reply", state: "PENDING" })],
        }),
      ],
      reviews: [review({ id: "PRR_mine", state: "PENDING", mine: true, body })],
    });
    const pending = await session(fake).pending();
    expect(pending.id).toBe("PRR_mine");
    expect(pending.threads.map((t) => t.id)).toEqual(["new"]);
    expect(pending.replies.map((r) => [r.threadId, r.comment.id])).toEqual([["old", "my-reply"]]);
    expect(pending.conversation.map((t) => t.comments[0]?.body)).toEqual(["Q?"]);
    expect(pending.count).toBe(3);
    const placed = await session(fake).threads(otherPage);
    expect(placed.blocks[0]?.threads.map((t) => [t.id, t.comments[0]?.pending])).toEqual([
      ["PRR_mine", true],
    ]);
  });

  test("a review holding only the placeholder holds nothing, and submits without it", async () => {
    const fake = github({
      reviews: [
        review({ id: "PRR_mine", state: "PENDING", mine: true, body: "<!-- ascribe:review -->" }),
      ],
    }).on("SubmitReview", () => ({
      submitPullRequestReview: { pullRequestReview: { id: "PRR_mine", state: "COMMENTED" } },
    }));
    const s = session(fake);
    expect((await s.pending()).count).toBe(0);
    await s.submit("COMMENT");
    expect(fake.mutations().map((c) => [c.operation, c.variables])).toEqual([
      ["SubmitReview", { reviewId: "PRR_mine", event: "COMMENT", body: null }],
    ]);
  });

  test("another reviewer's pending review isn't the viewer's", async () => {
    const fake = github({ reviews: [review({ state: "PENDING", mine: false })] });
    expect((await session(fake).pending()).count).toBe(0);
  });

  test("replies go into the review, or at once", async () => {
    const replied: Record<string, unknown>[] = [];
    const fake = postingFake()
      .on("ReviewThreads", paged("reviewThreads", [thread({ id: "T", path: INSTALL, line: 5 })]))
      .on("AddReply", (v) => {
        replied.push(v);
        return {
          addPullRequestReviewThreadReply: {
            comment: comment({ state: v["reviewId"] === null ? "SUBMITTED" : "PENDING" }),
          },
        };
      });
    const s = session(fake);
    expect((await s.reply("T", "Later", "withReview")).pending).toBe(true);
    expect((await s.reply("T", "Now", "now")).pending).toBe(false);
    expect(replied).toEqual([
      { threadId: "T", body: "Later", reviewId: "PRR_new" },
      { threadId: "T", body: "Now", reviewId: null },
    ]);
  });

  test("a reply sent at once that GitHub holds in the review is taken back", async () => {
    const fake = github({ threads: [thread({ id: "T", path: INSTALL, line: 5 })] })
      .on("AddReply", () => ({
        addPullRequestReviewThreadReply: { comment: comment({ id: "held", state: "PENDING" }) },
      }))
      .on("DeleteComment", () => ({ deletePullRequestReviewComment: { clientMutationId: null } }));
    await expect(session(fake).reply("T", "Now", "now")).rejects.toMatchObject({
      code: "reply-held",
    });
    expect(fake.mutations().map((c) => [c.operation, c.variables["id"]])).toEqual([
      ["AddReply", undefined],
      ["DeleteComment", "held"],
    ]);
  });

  test("resolve and reopen act at once; conversation comments can't be resolved", async () => {
    const body = "Q?\n\n<!-- ascribe:anchor guides/install.md:7-7 build=site -->";
    const fake = github({
      threads: [thread({ id: "T", path: INSTALL, line: 5 })],
      conversation: [{ id: "IC", body, createdAt: "", url: "", author: null }],
    })
      .on("Resolve", () => ({ resolveReviewThread: { thread: { id: "T", isResolved: true } } }))
      .on("Unresolve", () => ({
        unresolveReviewThread: { thread: { id: "T", isResolved: false } },
      }));
    const s = session(fake);
    await s.resolve("T", true);
    await s.resolve("T", false);
    expect(fake.mutations().map((c) => [c.operation, c.variables])).toEqual([
      ["Resolve", { threadId: "T" }],
      ["Unresolve", { threadId: "T" }],
    ]);
    await expect(s.resolve("IC", true)).rejects.toMatchObject({ code: "refused" });
    await expect(s.resolve("nope", true)).rejects.toMatchObject({ code: "not-found" });
  });

  test("submit sends the held comments with the body; discard deletes the review", async () => {
    const held = "Q?\n\n<!-- ascribe:anchor guides/install.md:7-7 build=site -->";
    const fake = github({
      reviews: [review({ id: "PRR_mine", state: "PENDING", mine: true, body: held })],
    })
      .on("SubmitReview", () => ({
        submitPullRequestReview: { pullRequestReview: { id: "PRR_mine", state: "COMMENTED" } },
      }))
      .on("DeleteReview", () => ({ deletePullRequestReview: { clientMutationId: null } }));
    const s = session(fake);
    await s.submit("APPROVE", "Looks good.");
    await s.discard();
    expect(fake.mutations().map((c) => [c.operation, c.variables])).toEqual([
      ["SubmitReview", { reviewId: "PRR_mine", event: "APPROVE", body: `${held}\n\nLooks good.` }],
      ["DeleteReview", { reviewId: "PRR_mine" }],
    ]);
  });

  test("discard does nothing without a pending review", async () => {
    const fake = github();
    await session(fake).discard();
    expect(fake.mutations()).toEqual([]);
  });

  test("changes go one at a time, spaced out", async () => {
    const fake = github({ threads: [thread({ id: "T", path: INSTALL, line: 5 })] });
    const times: number[] = [];
    fake.on("Resolve", () => {
      times.push(Date.now());
      return { resolveReviewThread: { thread: { id: "T", isResolved: true } } };
    });
    const s = createSession({
      root: repo.root,
      contentPrefix: PREFIX,
      pullRequest: pullRequest(),
      transport: fake,
      mutationInterval: 60,
    });
    await Promise.all([s.resolve("T", true), s.resolve("T", true), s.resolve("T", true)]);
    expect(times).toHaveLength(3);
    for (let i = 1; i < times.length; i++) {
      expect((times[i] ?? 0) - (times[i - 1] ?? 0)).toBeGreaterThanOrEqual(55);
    }
  });

  test("a refusal reaches the caller and the next change still runs", async () => {
    let first = true;
    const fake = github({ threads: [thread({ id: "T", path: INSTALL, line: 5 })] }).on(
      "Resolve",
      () => {
        if (first) {
          first = false;
          throw new ReviewError("rate-limited", "slow down", { retryAfter: 30 });
        }
        return { resolveReviewThread: { thread: { id: "T", isResolved: true } } };
      },
    );
    const s = session(fake);
    await expect(s.resolve("T", true)).rejects.toMatchObject({
      code: "rate-limited",
      retryAfter: 30,
    });
    await expect(s.resolve("T", true)).resolves.toBeUndefined();
  });
});

describe("finding the pull request", () => {
  const found = {
    id: "PR_7",
    number: 7,
    url: "https://github.com/acme/docs/pull/7",
    baseRefName: "main",
    baseRefOid: "b".repeat(40),
    headRefName: "feature",
    headRepositoryOwner: { login: "acme" },
    headRepository: { name: "docs" },
  };

  test("finds the open pull request for the branch, and how HEAD relates to it", async () => {
    const fake = new FakeGitHub().on("FindPullRequest", () => ({
      repository: {
        pullRequests: {
          nodes: [
            {
              ...found,
              number: 6,
              headRepositoryOwner: { login: "someone-else" },
              headRefOid: head,
            },
            { ...found, headRefOid: head },
          ],
        },
      },
    }));
    const pr = await findPullRequest(await readCheckout(repo.root), () => fake);
    expect(pr).toMatchObject({
      number: 7,
      headOid: head,
      local: "same",
      repository: { owner: "acme" },
    });
    expect(fake.calls[0]?.variables).toEqual({ owner: "acme", name: "docs", head: "feature" });
  });

  test("says whether HEAD is behind, ahead, or missing the head commit", async () => {
    const respond = (oid: string) =>
      new FakeGitHub().on("FindPullRequest", () => ({
        repository: { pullRequests: { nodes: [{ ...found, headRefOid: oid }] } },
      }));
    const checkout = await readCheckout(repo.root);
    expect(
      (await findPullRequest({ ...checkout, headOid: base }, () => respond(head)))?.local,
    ).toBe("behind");
    expect((await findPullRequest(checkout, () => respond(base)))?.local).toBe("ahead");
    expect((await findPullRequest(checkout, () => respond("e".repeat(40))))?.local).toBe("missing");
  });

  test("no pull request is an answer, not an error", async () => {
    const fake = new FakeGitHub().on("FindPullRequest", () => ({
      repository: { pullRequests: { nodes: [] } },
    }));
    expect(
      await openReview({ projectDir: `${repo.root}/site`, transport: () => fake }),
    ).toBeUndefined();
  });

  test("opens a session with the content prefix from ascribe.toml", async () => {
    const fake = github({ threads: [thread({ id: "right", path: INSTALL, line: 5 })] }).on(
      "FindPullRequest",
      () => ({ repository: { pullRequests: { nodes: [{ ...found, headRefOid: head }] } } }),
    );
    const hosts: string[] = [];
    const s = await openReview({
      projectDir: `${repo.root}/site`,
      transport: (host) => {
        hosts.push(host);
        return fake;
      },
    });
    expect(hosts).toEqual(["github.com", "github.com"]);
    const placed = await s?.threads(installPage);
    expect(placed?.blocks.map((b) => b.anchor.source)).toEqual(["guides/install.md:7-8"]);
  });
});

describe("pages that use one fragment twice", () => {
  test("get the thread on both places, once each", async () => {
    const fake = github({ threads: [thread({ id: "f", path: FRAGMENT, line: 2 })], files: [] });
    const placed = await session(fake).threads(installPage);
    expect(placed.blocks.map((b) => b.anchor.via[0])).toEqual([
      "guides/install.md:25",
      "guides/install.md:28",
    ]);
  });
});
