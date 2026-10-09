import { describe, expect, it } from "vitest";
import type { InventoryResult } from "../../src/shapes.js";
import {
  isHeadingLine,
  modelChildren,
  modelLook,
  modelRoots,
  pagesChildren,
  pagesLook,
  pagesPath,
  pagesRoots,
  usedByChildren,
  usedByLook,
  usedByRoots,
  type Place,
} from "../../src/ui/sidebar.js";

/** The item at `i`, which the test expects to be there. */
function nth<T>(list: readonly T[], i: number): T {
  const item = list[i];
  if (item === undefined) throw new Error(`no item ${i}`);
  return item;
}

const range = (line: number) => ({
  start: { line, character: 0 },
  end: { line, character: 5 },
});

const inventory: InventoryResult = {
  modelUri: "file:///p/ascribe.toml",
  contentUri: "file:///p/docs",
  pages: [
    { path: "guides/install.md", title: "Install", type: "guide", incoming: 3 },
    { path: "index.md", title: "Home", type: "page", incoming: 0 },
    { path: "old.md", title: null, type: null, incoming: 0 },
    { path: "ref/api.md", title: "API", type: "reference", incoming: 1 },
  ],
  fragments: [
    { path: "_setup.md", includedBy: ["guides/install.md", "index.md"] },
    { path: "_unused.md", includedBy: [] },
  ],
  orphans: ["old.md"],
  model: [
    { kind: "phrase", key: "product", label: "Quill", uses: 14, declaration: range(4) },
    { kind: "phrase", key: "old", label: "Old", uses: 0, declaration: range(5) },
    { kind: "note", key: "note", label: "Note", uses: 2, declaration: null },
    { kind: "build", key: "site", label: null, uses: null, declaration: range(9) },
  ],
};

describe("the Pages view", () => {
  it("groups pages by type, then fragments, then orphans", () => {
    const roots = pagesRoots(inventory);
    expect(roots.map((n) => pagesLook(n).label)).toEqual([
      "guide",
      "page",
      "reference",
      "no type",
      "Fragments",
      "Orphans",
    ]);
    expect(roots.map((n) => pagesLook(n).description)).toEqual(["1", "1", "1", "1", "2", "1"]);
  });

  it("shows a page's title as its label and its path as the description", () => {
    const guides = nth(pagesRoots(inventory), 0);
    const install = nth(pagesChildren(guides), 0);
    expect(pagesLook(install)).toMatchObject({
      label: "Install",
      description: "guides/install.md",
      icon: "file",
    });
    expect(pagesPath(install)).toBe("guides/install.md");
    // A page without a title shows its file name.
    const untyped = nth(pagesRoots(inventory), 3);
    expect(pagesLook(nth(pagesChildren(untyped), 0)).label).toBe("old.md");
  });

  it("lists what includes each fragment, and marks one nothing includes", () => {
    const fragments = nth(pagesRoots(inventory), 4);
    const children = pagesChildren(fragments);
    const setup = nth(children, 0);
    const unused = nth(children, 1);
    expect(pagesLook(setup)).toMatchObject({ description: "2 files", expandable: true });
    expect(pagesChildren(setup).map((n) => pagesPath(n))).toEqual([
      "guides/install.md",
      "index.md",
    ]);
    expect(pagesLook(unused)).toMatchObject({
      description: "not included",
      dimmed: true,
      expandable: false,
    });
  });

  it("leaves out groups with nothing in them", () => {
    const roots = pagesRoots({ ...inventory, fragments: [], orphans: [] });
    expect(roots.map((n) => n.kind)).toEqual(["type", "type", "type", "type"]);
  });
});

describe("the Content model view", () => {
  it("has a node per kind with a count, in a fixed order", () => {
    const roots = modelRoots(inventory);
    expect(roots.map((n) => [modelLook(n).label, modelLook(n).description])).toEqual([
      ["Phrases", "2"],
      ["Note types", "1"],
      ["Builds", "1"],
    ]);
    expect(modelLook(nth(roots, 0)).tooltip).toBe("2 entries, 1 unused");
  });

  it("shows each entry's uses, and marks one nothing uses", () => {
    const roots = modelRoots(inventory);
    const phrases = nth(roots, 0);
    const notes = nth(roots, 1);
    const builds = nth(roots, 2);
    const entries = modelChildren(phrases);
    const product = nth(entries, 0);
    const old = nth(entries, 1);
    expect(modelLook(product)).toMatchObject({
      label: "product",
      description: "14 uses",
      icon: "symbol-string",
      dimmed: false,
    });
    expect(modelLook(old)).toMatchObject({ description: "unused", dimmed: true });
    expect(modelLook(nth(modelChildren(notes), 0)).tooltip).toContain("Built in");
    // Pages don't name builds: no count.
    expect(modelLook(nth(modelChildren(builds), 0))).toMatchObject({
      description: "",
      dimmed: false,
    });
  });
});

describe("the Used by view", () => {
  const place = (path: string, line: number, text: string): Place => ({
    uri: `file:///p/docs/${path}`,
    path: `docs/${path}`,
    range: range(line),
    text,
  });
  const places = [
    place("b.md", 3, "See [setup](a.md#setup)."),
    place("a2.md", 7, "- [Install](a.md)"),
    place("b.md", 9, "Again: [a](a.md)."),
    place("guide.md", 2, "@include: _a.md"),
    place("list.md", 4, "> - @include: _a.md#x"),
  ];

  it("lists links grouped by file, then includes", () => {
    const roots = usedByRoots(places);
    expect(roots.map((n) => [usedByLook(n).label, usedByLook(n).description])).toEqual([
      ["Linked from", "3"],
      ["Included by", "2"],
    ]);
    const files = usedByChildren(nth(roots, 0));
    expect(files.map((n) => usedByLook(n).description)).toEqual(["docs/a2.md", "docs/b.md"]);
    const inB = usedByChildren(nth(files, 1));
    expect(inB.map((n) => [usedByLook(n).label, usedByLook(n).description])).toEqual([
      ["See [setup](a.md#setup).", "line 4"],
      ["Again: [a](a.md).", "line 10"],
    ]);
  });

  it("has nothing to show when nothing uses the page", () => {
    expect(usedByRoots([])).toEqual([]);
  });

  it("asks about a heading only on a heading's line", () => {
    expect(isHeadingLine("## Install")).toBe(true);
    expect(isHeadingLine("   # Top")).toBe(true);
    expect(isHeadingLine("#")).toBe(true);
    expect(isHeadingLine("    # code")).toBe(false);
    expect(isHeadingLine("#hashtag")).toBe(false);
    expect(isHeadingLine("Text")).toBe(false);
  });
});
