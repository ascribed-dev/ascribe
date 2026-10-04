// What review says and where it points (`src/preview/reviewText.ts`).
import * as path from "node:path";
import { describe, expect, it } from "vitest";
import type { ChangedPage } from "../../src/preview/protocol.js";
import {
  baseCommit,
  causes,
  nextChangedPage,
  pageDetail,
  parseSource,
  sameBase,
} from "../../src/preview/reviewText.js";

function page(over: Partial<ChangedPage> = {}): ChangedPage {
  return {
    path: "guides/install.md",
    route: "/guides/install/",
    status: "changed",
    own_file_changed: true,
    because: [],
    page_changed: [],
    counts: { changed: 5, added: 3, removed: 1, moved: 1 },
    title: "Install",
    ...over,
  };
}

describe("pageDetail", () => {
  it("counts each kind of change, leaving out kinds with none", () => {
    expect(pageDetail(page())).toBe("5 changed · 3 added · 1 removed · 1 moved");
    expect(pageDetail(page({ counts: { changed: 1, added: 1, removed: 0, moved: 0 } }))).toBe(
      "1 changed · 1 added",
    );
  });

  it("names what the page changed through when its own file didn't", () => {
    expect(
      pageDetail(
        page({
          own_file_changed: false,
          because: ["_fragments/prereqs.md"],
          counts: { changed: 1, added: 0, removed: 0, moved: 0 },
        }),
      ),
    ).toBe("1 changed · via _fragments/prereqs.md");
  });

  it("says a page is new or removed, and what about the page itself changed", () => {
    expect(pageDetail(page({ status: "added" }))).toBe("New page");
    expect(pageDetail(page({ status: "removed" }))).toBe("Removed");
    expect(
      pageDetail(
        page({ counts: { changed: 0, added: 0, removed: 0, moved: 0 }, page_changed: ["title"] }),
      ),
    ).toBe("title changed");
  });
});

describe("causes", () => {
  const roots = {
    projectRoot: path.join("/w", "docs-project"),
    contentRoot: path.join("/w", "docs-project", "docs"),
  };

  it("points each cause at its file: a content path in the content root, ascribe.toml in the project", () => {
    expect(
      causes(
        page({ own_file_changed: false, because: ["_fragments/prereqs.md", "ascribe.toml"] }),
        roots,
      ),
    ).toEqual([
      {
        label: "_fragments/prereqs.md",
        path: path.join("/w", "docs-project", "docs", "_fragments", "prereqs.md"),
      },
      { label: "ascribe.toml", path: path.join("/w", "docs-project", "ascribe.toml") },
    ]);
  });

  it("is empty when the page's own file changed", () => {
    expect(causes(page({ because: ["_fragments/prereqs.md"] }), roots)).toEqual([]);
    expect(causes(null, roots)).toEqual([]);
  });
});

describe("nextChangedPage", () => {
  const pages = [
    page({ path: "a.md", title: "A" }),
    page({ path: "b.md", title: "B", status: "removed" }),
    page({ path: "c.md", title: "C" }),
  ];

  it("is the next page in path order, skipping removed ones", () => {
    expect(nextChangedPage(pages, "a.md")).toMatchObject({ page: { path: "c.md" }, first: false });
    expect(nextChangedPage(pages, "b2.md")).toMatchObject({ page: { path: "c.md" }, first: false });
  });

  it("goes back to the first past the last, and is null when no other page changed", () => {
    expect(nextChangedPage(pages, "c.md")).toMatchObject({ page: { path: "a.md" }, first: true });
    expect(nextChangedPage([page({ path: "a.md" })], "a.md")).toBeNull();
  });
});

describe("parseSource", () => {
  it("reads a block's anchor as a content path and lines from 0", () => {
    expect(parseSource("guides/my%20install.md:12-14")).toEqual({
      path: "guides/my install.md",
      first: 11,
      last: 13,
    });
  });

  it("has nothing for a malformed anchor", () => {
    expect(parseSource("guides/install.md:12")).toBeUndefined();
    expect(parseSource("guides/install.md:0-1")).toBeUndefined();
    expect(parseSource("guides/install.md:5-4")).toBeUndefined();
    expect(parseSource("%E0%A4%A:1-1")).toBeUndefined();
  });
});

describe("the base's commit", () => {
  const base = { requested: "main", commit: "c".repeat(40), merge_base: "1a2b3c4d5e" };

  it("is where the branch left the base, shortened", () => {
    expect(baseCommit(base)).toBe("1a2b3c4");
    expect(baseCommit({ ...base, merge_base: null })).toBe("ccccccc");
  });

  it("tells a base that moved from one that didn't", () => {
    expect(sameBase(base, { ...base })).toBe(true);
    expect(sameBase(base, { ...base, merge_base: "9f8e7d6" })).toBe(false);
    expect(sameBase(base, { ...base, commit: "d".repeat(40) })).toBe(false);
  });
});
