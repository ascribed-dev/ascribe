import { describe, expect, it } from "vitest";
import type { InventoryResult, PageResult } from "../../src/shapes.js";
import {
  modelChildren,
  modelLook,
  modelRoots,
  pagesChildren,
  pagesLook,
  pagesPath,
  pagesRoots,
  thisPageChildren,
  thisPageLook,
  thisPageRoots,
  type Place,
  type ThisPageNode,
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

describe("the This page view", () => {
  const place = (path: string, line: number, text: string): Place => ({
    uri: `file:///p/docs/${path}`,
    path,
    range: range(line),
    text,
  });
  const about = {
    path: "keys.md",
    fragment: false,
    title: "API keys",
    type: "page",
    address: "/keys/",
    markdownAddress: "/keys.md",
  };
  const page: PageResult = {
    about,
    heading: { id: "rotate-keys", text: "Rotate keys", range: range(11) },
    linkedFrom: [
      place("b.md", 3, "See [rotate](keys.md#rotate-keys)."),
      place("a.md", 7, "- [Rotate](keys.md#rotate-keys)"),
      place("b.md", 9, "Again: [a](keys.md#rotate-keys)."),
    ],
    includedBy: [],
  };

  /** Every item as `label (description)`, depth first. */
  function flat(nodes: ThisPageNode[], depth = 0): string[] {
    return nodes.flatMap((node) => {
      const look = thisPageLook(node);
      return [
        `${"  ".repeat(depth)}${look.label}${look.description ? ` (${look.description})` : ""}`,
        ...flat(thisPageChildren(node), depth + 1),
      ];
    });
  }

  it("says what the page is, then what links to the heading, by file", () => {
    expect(flat(thisPageRoots(page))).toEqual([
      "About",
      "  Title (API keys)",
      "  Type (page)",
      "  Address (/keys/)",
      "  Markdown (/keys.md)",
      "Connections (to “Rotate keys”)",
      "  Linked from (3)",
      "    a.md (a.md)",
      "      - [Rotate](keys.md#rotate-keys) (line 8)",
      "    b.md (b.md)",
      "      See [rotate](keys.md#rotate-keys). (line 4)",
      "      Again: [a](keys.md#rotate-keys). (line 10)",
      "  Included by (0)",
    ]);
    const roots = thisPageRoots(page);
    expect(roots.map((n) => thisPageLook(n).expanded)).toEqual([true, true]);
    const included = nth(thisPageChildren(nth(roots, 1)), 1);
    expect(thisPageLook(included)).toMatchObject({ dimmed: true, expandable: false });
  });

  it("is about the whole page outside any heading, and says what isn't published", () => {
    const whole = thisPageRoots({
      ...page,
      about: { ...about, markdownAddress: null },
      heading: null,
      linkedFrom: [],
      includedBy: [place("guide.md", 2, "@include: keys.md")],
    });
    expect(flat(whole)).toEqual([
      "About",
      "  Title (API keys)",
      "  Type (page)",
      "  Address (/keys/)",
      "  Markdown (not published)",
      "Connections (to the page)",
      "  Linked from (0)",
      "  Included by (1)",
      "    guide.md (guide.md)",
      "      @include: keys.md (line 3)",
    ]);
  });

  it("says a fragment is published only where it's included", () => {
    const fragment = thisPageRoots({
      ...page,
      about: {
        path: "_setup.md",
        fragment: true,
        title: null,
        type: null,
        address: null,
        markdownAddress: null,
      },
      heading: null,
    });
    expect(flat([nth(fragment, 0)])).toEqual([
      "About",
      "  Title (_setup.md)",
      "  Type (fragment)",
      "  Address (only where it's included)",
    ]);
  });

  it("has nothing to show for a file that isn't a page", () => {
    expect(thisPageRoots({ about: null, heading: null, linkedFrom: [], includedBy: [] })).toEqual(
      [],
    );
  });
});
