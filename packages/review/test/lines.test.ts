import { afterAll, beforeAll, describe, expect, test } from "vitest";
import { lineHunks, lineMap, linesAt, parseHunks, shiftLine } from "../src/place/lines.js";
import { locateThreads } from "../src/place/place.js";
import type { Thread } from "../src/shared/types.js";
import { numbered, tempRepo, type TempRepo } from "./helpers/repo.js";

describe("hunks between two texts", () => {
  test("match git diff --unified=0's numbering", () => {
    expect(lineHunks("a\nb\nc", "a\nb\nc")).toEqual([]);
    // An insertion after line 1, a replacement of line 3, a deletion of line 5.
    expect(lineHunks("a\nb\nc\nd\ne\nf", "a\nX\nb\nC\nd\nf")).toEqual([
      { oldStart: 1, oldCount: 0, newStart: 2, newCount: 1 },
      { oldStart: 3, oldCount: 1, newStart: 4, newCount: 1 },
      { oldStart: 5, oldCount: 1, newStart: 5, newCount: 0 },
    ]);
    const hunks = lineHunks("a\nb\nc\nd\ne\nf", "a\nX\nb\nC\nd\nf");
    expect(shiftLine(hunks, 2)).toEqual({ line: 3, replaced: false });
    expect(shiftLine(hunks, 5)).toBeUndefined();
    expect(shiftLine(hunks, 6)).toEqual({ line: 6, replaced: false });
  });
});

describe("shifting a line through hunks", () => {
  const hunks = parseHunks(
    [
      "diff --git a/f b/f",
      "@@ -1,0 +2,2 @@",
      "+new a",
      "+new b",
      "@@ -20 +21,0 @@",
      "-line 20",
      "@@ -25,2 +26,3 @@",
    ].join("\n"),
  );

  test("parse counts, with 1 when omitted", () => {
    expect(hunks).toEqual([
      { oldStart: 1, oldCount: 0, newStart: 2, newCount: 2 },
      { oldStart: 20, oldCount: 1, newStart: 21, newCount: 0 },
      { oldStart: 25, oldCount: 2, newStart: 26, newCount: 3 },
    ]);
  });

  const kept = (line: number) => ({ line, replaced: false });
  const replaced = (line: number) => ({ line, replaced: true });

  test("lines before, between, and after hunks move by what came before them", () => {
    expect(shiftLine(hunks, 1)).toEqual(kept(1));
    expect(shiftLine(hunks, 2)).toEqual(kept(4));
    expect(shiftLine(hunks, 19)).toEqual(kept(21));
    expect(shiftLine(hunks, 21)).toEqual(kept(22));
    expect(shiftLine(hunks, 27)).toEqual(kept(29));
  });

  test("deleted lines are gone, and replaced lines map onto their replacement", () => {
    expect(shiftLine(hunks, 20)).toBeUndefined();
    expect(shiftLine(hunks, 25)).toEqual(replaced(26));
    expect(shiftLine(hunks, 26)).toEqual(replaced(27));
  });

  test("a replaced line past the end of a shorter replacement maps onto its last line", () => {
    const shorter = parseHunks("@@ -5,3 +5 @@");
    expect([5, 6, 7, 8].map((l) => shiftLine(shorter, l))).toEqual([
      replaced(5),
      replaced(5),
      replaced(5),
      kept(6),
    ]);
  });
});

describe("telling a deleted line from a reworded one in the same hunk", () => {
  const before = [
    "# Options",
    "",
    "`retries` sets how many times a failed request is tried again.",
    "",
    "`timeout` sets how long, in seconds, to wait for an answer.",
    "",
    "End.",
  ];
  const after = [
    "# Options",
    "",
    "`timeout` sets how many seconds to wait for an answer before giving up.",
    "",
    "End.",
  ];
  const sides = { old: before, new: after };

  test("a deleted line is gone, and its reworded neighbour maps onto its rewording", () => {
    const hunks = lineHunks(before.join("\n"), after.join("\n"));
    expect(shiftLine(hunks, 3, sides)).toBeUndefined();
    expect(shiftLine(hunks, 5, sides)).toEqual({ line: 3, replaced: true });
    expect(shiftLine(hunks, 7, sides)).toEqual({ line: 5, replaced: false });
  });

  test("the same in a hunk that spans both lines, as git reports it", () => {
    const hunks = parseHunks("@@ -3,3 +3 @@");
    expect(shiftLine(hunks, 3, sides)).toBeUndefined();
    expect(shiftLine(hunks, 4, sides)).toBeUndefined();
    expect(shiftLine(hunks, 5, sides)).toEqual({ line: 3, replaced: true });
    // Without the text, lines pair by position.
    expect(shiftLine(hunks, 3)).toEqual({ line: 3, replaced: true });
  });

  test("lines rewritten as unlike ones, or as many new lines as old, pair by position", () => {
    const rewritten = { old: ["TODO write this.", "TBD."], new: ["Real words here."] };
    expect(shiftLine(parseHunks("@@ -1,2 +1 @@"), 2, rewritten)).toEqual({
      line: 1,
      replaced: true,
    });
    const swapped = { old: ["alpha beta", "gamma delta"], new: ["gamma delta x", "alpha beta y"] };
    expect(shiftLine(parseHunks("@@ -1,2 +1,2 @@"), 1, swapped)).toEqual({
      line: 1,
      replaced: true,
    });
  });

  test("text without spaces compares by pairs of characters, and lines without words by characters", () => {
    const japanese = {
      old: ["再試行回数を設定します。", "タイムアウトを秒で設定します。", "削除された行です。"],
      new: ["再試行の回数を設定します。", "タイムアウトを秒単位で設定します。"],
    };
    const hunks = parseHunks("@@ -1,3 +1,2 @@");
    expect(shiftLine(hunks, 1, japanese)).toEqual({ line: 1, replaced: true });
    expect(shiftLine(hunks, 2, japanese)).toEqual({ line: 2, replaced: true });
    expect(shiftLine(hunks, 3, japanese)).toBeUndefined();
    const code = { old: ["  return x;", "}", "// gone entirely"], new: ["  return y;", "});"] };
    expect(shiftLine(hunks, 2, code)).toEqual({ line: 2, replaced: true });
  });

  test("one line replaced by one keeps its place however much it changed", () => {
    const hunks = parseHunks("@@ -2 +2 @@");
    const one = {
      old: ["a", "Completely different.", "c"],
      new: ["a", "Nothing alike here.", "c"],
    };
    expect(shiftLine(hunks, 2, one)).toEqual({ line: 2, replaced: true });
  });
});

describe("line maps between a commit and the working tree", () => {
  // One repository and one commit for every test: each maps a file of its
  // own, and edits only that file in the working tree.
  let repo: TempRepo;
  let head: string;
  const lines = numbered(30);
  beforeAll(() => {
    repo = tempRepo();
    repo.write("docs/a b.md", `${lines.join("\n")}\n`);
    repo.write("docs/same.md", "same\n");
    repo.write(
      "Options.md",
      "# Options\n\nretries sets how many times a request is tried again.\n\ntimeout sets how long to wait, in seconds.\n",
    );
    repo.write(
      "wrapped.md",
      "# A\n\nThis paragraph wraps\nacross four lines and\nthen ends with the\ncaller.\n",
    );
    repo.write("removed.md", "a\n");
    head = repo.commit("head");
  });
  afterAll(() => repo.remove());

  test("follow local edits both ways, and know an unchanged file", async () => {
    const edited = [...lines];
    edited.splice(19, 1);
    edited.splice(1, 0, "new a", "new b");
    repo.write("docs/a b.md", `${edited.join("\n")}\n`);

    const forward = await lineMap(repo.root, head, "docs/a b.md", "to-worktree");
    expect(forward.identity).toBe(false);
    expect([1, 2, 19, 20, 21].map((l) => forward.map(l)?.line)).toEqual([1, 4, 21, undefined, 22]);

    const back = await lineMap(repo.root, head, "docs/a b.md", "to-commit");
    expect([1, 2, 3, 4, 22].map((l) => back.map(l)?.line)).toEqual([
      1,
      undefined,
      undefined,
      2,
      21,
    ]);

    const same = await lineMap(repo.root, head, "docs/same.md", "to-worktree");
    expect(same.identity).toBe(true);
    expect(same.map(1)).toEqual({ line: 1, replaced: false });

    expect(await linesAt(repo.root, head, "docs/a b.md", 19, 20)).toBe("line 19\nline 20");
  });

  test("leave a line deleted beside a reworded one off the rewording", async () => {
    repo.write("Options.md", "# Options\n\ntimeout sets how many seconds to wait.\n");
    const forward = await lineMap(repo.root, head, "Options.md", "to-worktree");
    expect(forward.map(3)).toBeUndefined();
    expect(forward.map(5)).toEqual({ line: 3, replaced: true });
    // Back the other way, a hunk with more new lines than old pairs by position.
    const back = await lineMap(repo.root, head, "Options.md", "to-commit");
    expect(back.map(3)).toEqual({ line: 3, replaced: true });
  });

  test("a thread whose last line went keeps the lines that stayed", async () => {
    repo.write(
      "wrapped.md",
      "# A\n\nThis paragraph wraps across\nfour lines and then ends\nwith the caller.\n",
    );
    const thread: Thread = {
      id: "T",
      kind: "review",
      repositoryPath: "wrapped.md",
      subject: "line",
      side: "RIGHT",
      line: 6,
      startLine: 3,
      originalLine: 6,
      originalStartLine: 3,
      commit: head,
      originalCommit: head,
      resolved: false,
      outdated: false,
      canResolve: false,
      canUnresolve: false,
      canReply: false,
      comments: [],
      diffHunk: undefined,
      marker: undefined,
      quote: undefined,
    };
    const [located] = await locateThreads([thread], {
      root: repo.root,
      contentPrefix: "",
      headOid: head,
    });
    expect(located).toMatchObject({ detached: undefined, outdated: true });
    expect(located?.lines?.first).toBe(3);
  });

  test("map nothing for a missing commit or a file missing on either side", async () => {
    repo.write("new.md", "n\n");
    const missingCommit = await lineMap(repo.root, "0".repeat(40), "removed.md", "to-worktree");
    expect(missingCommit.map(1)).toBeUndefined();
    expect((await lineMap(repo.root, head, "new.md", "to-commit")).map(1)).toBeUndefined();
    repo.git("rm", "--quiet", "removed.md");
    expect((await lineMap(repo.root, head, "removed.md", "to-worktree")).map(1)).toBeUndefined();
  });
});
