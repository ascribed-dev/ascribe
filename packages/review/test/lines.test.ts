import { afterEach, describe, expect, test } from "vitest";
import { lineHunks, lineMap, linesAt, parseHunks, shiftLine } from "../src/place/lines.js";
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

describe("line maps between a commit and the working tree", () => {
  let repo: TempRepo;
  afterEach(() => repo.remove());

  test("follow local edits both ways, and know an unchanged file", async () => {
    repo = tempRepo();
    const lines = numbered(30);
    repo.write("docs/a b.md", `${lines.join("\n")}\n`);
    repo.write("docs/same.md", "same\n");
    const head = repo.commit("head");
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

  test("map nothing for a missing commit or a file missing on either side", async () => {
    repo = tempRepo();
    repo.write("a.md", "a\n");
    const head = repo.commit("head");
    repo.write("new.md", "n\n");
    const missingCommit = await lineMap(repo.root, "0".repeat(40), "a.md", "to-worktree");
    expect(missingCommit.map(1)).toBeUndefined();
    expect((await lineMap(repo.root, head, "new.md", "to-commit")).map(1)).toBeUndefined();
    repo.git("rm", "--quiet", "a.md");
    expect((await lineMap(repo.root, head, "a.md", "to-worktree")).map(1)).toBeUndefined();
  });
});
