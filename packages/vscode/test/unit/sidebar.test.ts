import { describe, expect, it } from "vitest";
import type { InventoryResult, PageAbout, PageListing, PageResult } from "../../src/shapes.js";
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
  NEXT_PLACEHOLDER,
  SHOW_PROBLEMS,
  SWITCH_BUILD,
  type Place,
  type ThisPageNode,
  type ThisPageReview,
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
    { path: "guides/install.md", title: "Install", type: "guide", incoming: 3, builds: ["site"] },
    { path: "index.md", title: "Home", type: "page", incoming: 0, builds: ["site"] },
    { path: "old.md", title: null, type: null, incoming: 0, builds: ["site"] },
    { path: "ref/api.md", title: "API", type: "reference", incoming: 1, builds: ["site"] },
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
  /** What the server says beyond what the page is, when it has nothing to say. */
  const quiet = {
    description: null,
    review: null,
    availability: null,
    builds: [],
    listing: null,
    permalinks: [],
    template: null,
  };
  const about: PageAbout = {
    path: "keys.md",
    fragment: false,
    title: "API keys",
    type: "page",
    address: "/keys/",
    markdownAddress: "/keys.md",
    ...quiet,
  };
  const health = { size: null, examples: [] };
  const page: PageResult = {
    about,
    heading: { id: "rotate-keys", text: "Rotate keys", range: range(11) },
    linkedFrom: [
      place("a.md", 7, "- [Rotate](keys.md#rotate-keys)"),
      place("b.md", 3, "See [rotate](keys.md#rotate-keys)."),
      place("b.md", 9, "Again: [a](keys.md#rotate-keys)."),
    ],
    includedBy: [],
    memos: [],
    health,
    includes: [],
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

  it("says what the page is, then what links to the heading, by file in the server's order", () => {
    expect(flat(thisPageRoots(page))).toEqual([
      "About",
      "  Title (API keys)",
      "  Type (page)",
      "  Address (/keys/)",
      "  Markdown (/keys.md)",
      "Health",
      "Connections (to “Rotate keys”)",
      "  Linked from (3)",
      "    a.md (a.md)",
      "      - [Rotate](keys.md#rotate-keys) (line 8)",
      "    b.md (b.md)",
      "      See [rotate](keys.md#rotate-keys). (line 4)",
      "      Again: [a](keys.md#rotate-keys). (line 10)",
      "  Included by (0)",
      "  Includes (0)",
      "Memos (none)",
    ]);
    const roots = thisPageRoots(page);
    expect(roots.map((n) => thisPageLook(n).expanded)).toEqual([true, true, true, true]);
    expect(thisPageLook(nth(roots, 3))).toMatchObject({ dimmed: true, expandable: false });
    const included = nth(thisPageChildren(nth(roots, 2)), 1);
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
      "Health",
      "Connections (to the page)",
      "  Linked from (0)",
      "  Included by (1)",
      "    guide.md (guide.md)",
      "      @include: keys.md (line 3)",
      "  Includes (0)",
      "Memos (none)",
    ]);
  });

  it("lists the memos about the heading, personal and outdated ones marked", () => {
    const memo = (path: string, firstLine: string, state: string, personal: boolean) => ({
      uri: `file:///work/${path}`,
      path,
      firstLine,
      state,
      personal,
      body: range(4),
    });
    const roots = thisPageRoots({
      ...page,
      memos: [
        memo("memos/why.md", "Why keys rotate.", "placed", false),
        memo("memos/old.md", "", "outdated", false),
        memo(".ascribe/memos/mine.md", "Ask Sam.", "placed", true),
      ],
    });
    const memos = nth(roots, 3);
    expect(flat([memos])).toEqual([
      "Memos (about “Rotate keys”)",
      "  Why keys rotate. (memos/why.md)",
      "  old.md (memos/old.md · outdated)",
      "  Ask Sam. (.ascribe/memos/mine.md · personal)",
    ]);
    expect(thisPageLook(memos)).toMatchObject({ dimmed: false, expandable: true });
    expect(thisPageLook(nth(thisPageChildren(memos), 2)).icon).toBe("lock");
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
        ...quiet,
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

  describe("while review is on", () => {
    const changed: ThisPageReview["page"] = {
      path: "keys.md",
      route: "/keys/",
      status: "changed",
      own_file_changed: false,
      because: ["_setup.md"],
      page_changed: [],
      counts: { changed: 2, added: 1, removed: 0, moved: 0 },
      title: "API keys",
      formatted_title: null,
    };
    const thread = (id: string, path: string, line: number | undefined) => ({
      id,
      path,
      line,
      author: "octocat",
      excerpt: `Comment ${id}`,
    });

    it("adds Review after About: what the branch changed, and the open threads", () => {
      const review: ThisPageReview = {
        base: "main",
        page: changed,
        threads: [thread("a", "_setup.md", 3), thread("b", "keys.md", undefined)],
      };
      const roots = thisPageRoots({ ...page, linkedFrom: [] }, review);
      expect(flat(roots)).toEqual([
        "About",
        "  Title (API keys)",
        "  Type (page)",
        "  Address (/keys/)",
        "  Markdown (/keys.md)",
        "Review (against main)",
        "  Changed (2 changed · 1 added · via _setup.md)",
        "  Open threads (2)",
        "    Comment a (octocat · _setup.md:3)",
        "    Comment b (octocat · keys.md)",
        "Health",
        "Connections (to “Rotate keys”)",
        "  Linked from (0)",
        "  Included by (0)",
        "  Includes (0)",
        "Memos (none)",
      ]);
      const reviewNode = nth(roots, 1);
      expect(thisPageLook(reviewNode).expanded).toBe(true);
      expect(thisPageLook(nth(thisPageChildren(reviewNode), 1)).expanded).toBe(true);
    });

    it("says when the branch doesn't change the page, and dims no open threads", () => {
      const roots = thisPageRoots(page, { base: "origin/release", page: null, threads: [] });
      const reviewNode = nth(roots, 1);
      expect(flat([reviewNode])).toEqual([
        "Review (against origin/release)",
        "  Unchanged (this branch doesn't change it)",
        "  Open threads (0)",
      ]);
      const threads = nth(thisPageChildren(reviewNode), 1);
      expect(thisPageLook(threads)).toMatchObject({ dimmed: true, expandable: false });
    });

    it("leaves the threads out when the pull request's aren't known, and calls a new page added", () => {
      const roots = thisPageRoots(page, {
        base: "main",
        page: { ...changed, status: "added", own_file_changed: true, because: [] },
        threads: undefined,
      });
      expect(flat([nth(roots, 1)])).toEqual(["Review (against main)", "  Added (New page)"]);
    });
  });

  describe("About's and Health's rows", () => {
    const uri = "file:///p/docs/keys.md";
    const listing: PageListing = {
      build: "site",
      listing: "unlisted",
      from: "folder",
      rule: '"internal/" = "unlisted"',
      uri: "file:///p/ascribe.toml",
      range: range(20),
    };
    const fullAbout: PageAbout = {
      ...about,
      description: { field: "description", length: 312, limit: 300, range: range(3) },
      review: {
        field: "review-by",
        date: "2026-10-03",
        days: -14,
        overdue: true,
        reported: true,
        range: range(4),
      },
      availability: "self-managed 3.5",
      builds: [
        { name: "site", leftOut: null },
        { name: "agents", leftOut: null },
        { name: "cloud", leftOut: "availability" },
      ],
      listing,
      permalinks: [
        { name: "keys", heading: null, range: range(5) },
        { name: "rotate-keys", heading: "Rotate keys", range: range(12) },
      ],
      template: { type: "how-to", left: 2 },
    };
    const full: PageResult = {
      ...page,
      about: fullAbout,
      health: {
        size: { characters: 31_200, build: "site", limit: 50_000 },
        examples: [
          {
            address: "code:greet.ts#greet",
            state: "tested",
            tested: "Node 22 on Linux",
            range: range(30),
          },
          { address: "code:old.ts", state: "stale", tested: null, range: range(40) },
        ],
      },
      includes: [
        {
          kind: "include",
          written: "_setup.md",
          path: "_setup.md",
          uri: "file:///p/docs/_setup.md",
          range: range(8),
        },
        {
          kind: "snippet",
          written: "code:greet.ts#greet",
          path: "code/greet.ts",
          uri: "file:///p/code/greet.ts",
          range: range(30),
        },
      ],
    };
    const roots = thisPageRoots(full, undefined, {
      uri,
      problems: { errors: 0, warnings: 1, advice: 2 },
    });

    it("say whether the page is described, due, published, listed, and unfinished", () => {
      expect(flat([nth(roots, 0)]).slice(5)).toEqual([
        "  Description (312 of 300 characters, too long)",
        "  Review (overdue since 3 October)",
        "  Availability (self-managed 3.5)",
        "  Builds (Published by site, agents; left out by cloud)",
        "  Listing (Unlisted, by `[listing.folders]`)",
        "  Permalink (keys)",
        "  Permalink (rotate-keys)",
        "  Template (how-to template, 2 placeholders left)",
      ]);
      expect(flat([nth(roots, 1)])).toEqual([
        "Health (2 things to do)",
        "  Problems (1 warning, 2 advice)",
        "  Size (31,200 of 50,000 characters, in site)",
        "  Example (code:greet.ts#greet, tested)",
        "  Example (code:old.ts, changed since it was tested)",
      ]);
    });

    it("open the place each is set, or run the command that acts on it", () => {
      const rows = (node: ThisPageNode) =>
        thisPageChildren(node).flatMap((n) => (n.kind === "row" ? [n.row] : []));
      const about = rows(nth(roots, 0));
      const opened = (label: string) => about.find((row) => row.label === label)?.opens;
      expect(opened("Description")).toEqual({ kind: "place", uri, range: range(3) });
      expect(opened("Listing")).toEqual({
        kind: "place",
        uri: "file:///p/ascribe.toml",
        range: range(20),
      });
      expect(opened("Builds")).toMatchObject({ kind: "command", command: SWITCH_BUILD });
      expect(opened("Template")).toMatchObject({ kind: "command", command: NEXT_PLACEHOLDER });
      expect(opened("Availability")).toBeUndefined();
      const health = rows(nth(roots, 1));
      expect(health[0]?.opens).toMatchObject({ kind: "command", command: SHOW_PROBLEMS });
      // Size goes nowhere: its tooltip says which build and the limit.
      expect(health[1]?.opens).toBeUndefined();
      expect(health[1]?.tooltip).toContain("50,000");
      expect(health[2]?.opens).toEqual({ kind: "place", uri, range: range(30) });
    });

    it("list what the page includes under Connections", () => {
      const connections = nth(roots, 2);
      expect(flat([nth(thisPageChildren(connections), 2)])).toEqual([
        "Includes (2)",
        "  _setup.md (_setup.md)",
        "  code:greet.ts#greet (code/greet.ts)",
      ]);
    });

    it("warn of an overdue review only where review-overdue reports it", () => {
      const warned = (reported: boolean) => {
        const roots = thisPageRoots({
          ...full,
          about: {
            ...fullAbout,
            review: {
              field: "review-by",
              date: "2026-10-03",
              days: -14,
              overdue: true,
              reported,
              range: range(4),
            },
          },
        });
        return thisPageChildren(nth(roots, 0))
          .flatMap((n) => (n.kind === "row" ? [n.row] : []))
          .find((row) => row.label === "Review");
      };
      expect(warned(true)).toMatchObject({ value: "overdue since 3 October", warn: true });
      expect(warned(false)).toMatchObject({ value: "overdue since 3 October", warn: false });
    });

    it("say when a review is due, and that a short description is fine", () => {
      const later = thisPageRoots({
        ...full,
        about: {
          ...fullAbout,
          description: { field: "description", length: 120, limit: null, range: range(3) },
          review: {
            field: "review-by",
            date: "2027-01-03",
            days: 12,
            overdue: false,
            reported: false,
            range: range(4),
          },
          builds: [{ name: "site", leftOut: null }],
          listing: {
            ...listing,
            listing: "listed",
            from: "default",
            rule: null,
            uri: null,
            range: null,
          },
          template: null,
        },
      });
      expect(flat([nth(later, 0)]).slice(5, 9)).toEqual([
        "  Description (120 characters)",
        "  Review (due in 12 days)",
        "  Availability (self-managed 3.5)",
        "  Builds (Published by site)",
      ]);
      expect(flat([nth(later, 0)])).toContain("  Listing (Listed (the default))");
    });
  });

  it("has nothing to show for a file that isn't a page", () => {
    expect(
      thisPageRoots({
        about: null,
        heading: null,
        linkedFrom: [],
        includedBy: [],
        memos: [],
        health,
        includes: [],
      }),
    ).toEqual([]);
  });
});
