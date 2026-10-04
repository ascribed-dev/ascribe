import { afterEach, describe, expect, test } from "vitest";
import { lineMap, linesAt, parseHunks, shiftLine } from "../src/place/lines.js";
import { numbered, tempRepo, type TempRepo } from "./helpers/repo.js";

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

  test("lines before, between, and after hunks move by what came before them", () => {
    expect(shiftLine(hunks, 1)).toBe(1);
    expect(shiftLine(hunks, 2)).toBe(4);
    expect(shiftLine(hunks, 19)).toBe(21);
    expect(shiftLine(hunks, 21)).toBe(22);
    expect(shiftLine(hunks, 27)).toBe(29);
  });

  test("removed and replaced lines are gone", () => {
    expect(shiftLine(hunks, 20)).toBeUndefined();
    expect(shiftLine(hunks, 25)).toBeUndefined();
    expect(shiftLine(hunks, 26)).toBeUndefined();
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
    expect([1, 2, 19, 20, 21].map((l) => forward.map(l))).toEqual([1, 4, 21, undefined, 22]);

    const back = await lineMap(repo.root, head, "docs/a b.md", "to-commit");
    expect([1, 2, 3, 4, 22].map((l) => back.map(l))).toEqual([1, undefined, undefined, 2, 21]);

    const same = await lineMap(repo.root, head, "docs/same.md", "to-worktree");
    expect(same.identity).toBe(true);
    expect(same.map(1)).toBe(1);

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
