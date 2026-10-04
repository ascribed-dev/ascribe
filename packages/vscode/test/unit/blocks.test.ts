// The line-to-block mapping the preview scrolls by (`src/preview/blocks.ts`).
import { describe, expect, it } from "vitest";
import { blockAt, linesInPage, type Lines } from "../../src/preview/blocks.js";

describe("linesInPage", () => {
  it("reads a block written in the page, from 0", () => {
    expect(linesInPage("guides/install.md:12-14", null, "guides/install.md")).toEqual({
      first: 11,
      last: 13,
    });
  });

  it("puts a block from a fragment at the page's include line", () => {
    expect(
      linesInPage(
        "_fragments/nested.md:1-3",
        "guides/install.md:20 _fragments/prereqs.md:6",
        "guides/install.md",
      ),
    ).toEqual({ first: 19, last: 19 });
  });

  it("decodes the path", () => {
    expect(linesInPage("my%20guide/%C3%A9t%C3%A9.md:3-3", undefined, "my guide/été.md")).toEqual({
      first: 2,
      last: 2,
    });
  });

  it("has nothing for another file or a malformed anchor", () => {
    expect(linesInPage("other.md:1-2", null, "page.md")).toBeUndefined();
    expect(linesInPage("page.md:1-2", "other.md:4", "page.md")).toBeUndefined();
    for (const source of [
      "page.md",
      "page.md:2",
      "page.md:3-2",
      "page.md:0-1",
      ":1-1",
      "%E0.md:1-1",
    ]) {
      expect(linesInPage(source, null, "page.md"), source).toBeUndefined();
    }
  });
});

describe("blockAt", () => {
  // A heading, a list holding two items (the second with a paragraph and a
  // code block), and a paragraph after a gap.
  const blocks: (Lines | undefined)[] = [
    { first: 0, last: 0 }, // 0: heading
    { first: 2, last: 8 }, // 1: list
    { first: 2, last: 2 }, // 2: item
    { first: 3, last: 8 }, // 3: item
    { first: 3, last: 3 }, // 4: paragraph in it
    { first: 5, last: 8 }, // 5: code block in it
    undefined, // 6: a block from another file
    { first: 12, last: 12 }, // 7: paragraph
  ];

  it("picks the innermost block that holds the line", () => {
    expect(blockAt(blocks, 0)).toBe(0);
    expect(blockAt(blocks, 2)).toBe(2);
    expect(blockAt(blocks, 3)).toBe(4);
    expect(blockAt(blocks, 6)).toBe(5);
    expect(blockAt(blocks, 4)).toBe(3);
  });

  it("picks the nearest block before a line no block holds", () => {
    expect(blockAt(blocks, 1)).toBe(0);
    // The list, its item, and the code block all end on line 8: the innermost.
    expect(blockAt(blocks, 10)).toBe(5);
    expect(blockAt(blocks, 40)).toBe(7);
  });

  it("picks the first block for a line before every block", () => {
    expect(blockAt([undefined, { first: 4, last: 5 }], 1)).toBe(1);
  });

  it("has nothing when no block has lines", () => {
    expect(blockAt([], 3)).toBeUndefined();
    expect(blockAt([undefined], 3)).toBeUndefined();
  });

  it("picks the first of the blocks one include brought in", () => {
    const included = [
      { first: 7, last: 7 },
      { first: 7, last: 7 },
      { first: 9, last: 9 },
    ];
    expect(blockAt(included, 7)).toBe(0);
    expect(blockAt(included, 8)).toBe(0);
  });
});
