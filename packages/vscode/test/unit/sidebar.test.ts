import { describe, expect, it } from "vitest";
import type {
  InventoryResult,
  InventorySection,
  PageAbout,
  PageListing,
  PageResult,
} from "../../src/shapes.js";
import {
  blocksMove,
  contentPathOf,
  followedProject,
  labelRename,
  leftOut,
  modelChildren,
  modelDeclaration,
  modelLook,
  modelRoots,
  newPathProblem,
  NEXT_PLACEHOLDER,
  pagesChildren,
  pagesDrop,
  pagesFind,
  pagesLook,
  pagesParent,
  pagesPath,
  pagesResource,
  pagesRoots,
  SHOW_PROBLEMS,
  shownPage,
  SWITCH_BUILD,
  thisPageChildren,
  thisPageItems,
  thisPageLook,
  thisPageRoots,
  viewsNotice,
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
    {
      path: "guides/install.md",
      title: "Install",
      label: "Install",
      navLabel: null,
      type: "guide",
      incoming: 3,
      builds: ["site"],
    },
    {
      path: "index.md",
      title: "Home",
      label: "Home",
      navLabel: null,
      type: "page",
      incoming: 0,
      builds: ["site"],
    },
    {
      path: "old.md",
      title: null,
      label: "old",
      navLabel: null,
      type: null,
      incoming: 0,
      builds: ["site"],
    },
    {
      path: "ref/api.md",
      title: "API",
      label: "API",
      navLabel: null,
      type: "reference",
      incoming: 1,
      builds: ["site"],
    },
  ],
  fragments: [
    { path: "_setup.md", includedBy: ["guides/install.md", "index.md"] },
    { path: "_unused.md", includedBy: [] },
  ],
  orphans: ["old.md"],
  navigation: null,
  model: [
    { kind: "phrase", key: "product", label: "Quill", uses: 14, declaration: range(4), page: null },
    { kind: "phrase", key: "old", label: "Old", uses: 0, declaration: range(5), page: null },
    { kind: "note", key: "note", label: "Note", uses: 2, declaration: null, page: null },
    { kind: "build", key: "site", label: null, uses: null, declaration: range(9), page: null },
    {
      kind: "term",
      key: "api-key",
      label: "API key",
      uses: 3,
      declaration: range(6),
      page: "ref/api.md",
    },
  ],
};

/** A section of the navigation, with what the test gives it. */
function section(
  folder: string,
  label: string,
  more: Partial<InventorySection> = {},
): InventorySection {
  return {
    folder,
    label,
    labelSet: null,
    page: null,
    pages: [],
    sections: [],
    listed: false,
    listsPages: false,
    ...more,
  };
}

const page = (path: string, placed = true, unlisted = false) => ({ path, placed, unlisted });

/**
 * Start with its home page; Guides with a nested section that sits where
 * its page beside the folder does, and a page left out of its list;
 * Reference.
 */
const deep = section("guides/deep/", "Deep", {
  page: "guides/deep.md",
  pages: [page("guides/deep/part.md")],
});
const guides = section("guides/", "Guides", {
  listed: true,
  listsPages: true,
  labelSet: "Guides",
  pages: [page("guides/install.md"), page("guides/deep.md"), page("guides/old.md", false)],
  sections: [deep],
});
const reference = section("ref/", "Reference", { pages: [page("ref/api.md")] });
const navigated: InventoryResult = {
  ...inventory,
  pages: [
    ...inventory.pages,
    {
      path: "guides/deep.md",
      title: "Deep",
      label: "Deep",
      navLabel: null,
      type: "guide",
      incoming: 1,
      builds: ["site"],
    },
    {
      path: "guides/deep/part.md",
      title: "Part",
      label: "Part",
      navLabel: null,
      type: "guide",
      incoming: 1,
      builds: ["site"],
    },
    {
      path: "guides/old.md",
      title: "Old",
      label: "Old",
      navLabel: null,
      type: "guide",
      incoming: 0,
      builds: ["site"],
    },
  ],
  fragments: [
    { path: "_setup.md", includedBy: ["guides/install.md", "index.md"] },
    { path: "_parts/_unused.md", includedBy: [] },
  ],
  navigation: section("", "Start", {
    page: "index.md",
    sections: [guides, reference],
  }),
};

/** The view as lines: each item's label and description, its children indented. */
function outline(nodes: ReturnType<typeof pagesRoots>, depth = 0): string[] {
  return nodes.flatMap((node) => {
    const look = pagesLook(node);
    return [
      `${"  ".repeat(depth)}${look.label}${look.description ? ` (${look.description})` : ""}`,
      ...outline(pagesChildren(node), depth + 1),
    ];
  });
}

/** The node of a section, by its folder. */
function sectionOf(roots: ReturnType<typeof pagesRoots>, folder: string) {
  const find = (
    nodes: ReturnType<typeof pagesRoots>,
  ): ReturnType<typeof pagesRoots>[number] | undefined => {
    for (const node of nodes) {
      if (node.kind === "section" && node.section.folder === folder) return node;
      const found = find(pagesChildren(node));
      if (found) return found;
    }
    return undefined;
  };
  const found = find(roots);
  if (!found) throw new Error(`no section ${folder}`);
  return found;
}

function pageOf(roots: ReturnType<typeof pagesRoots>, path: string) {
  const found = pagesFind(roots, path);
  if (!found) throw new Error(`no page ${path}`);
  return found;
}

describe("the Pages view", () => {
  it("draws the navigation: Start, each section in order with its pages, then fragments by folder", () => {
    expect(outline(pagesRoots(navigated))).toEqual([
      "Start",
      "  Home (index.md)",
      "Guides (guides/)",
      "  Install (guides/install.md)",
      // A section whose page sits beside its folder sits where that page does.
      "  Deep (guides/deep/)",
      "    Deep (guides/deep.md)",
      "    Part (guides/deep/part.md)",
      "  Old (not placed)",
      "Reference (ref/)",
      "  API (ref/api.md)",
      "Fragments (2)",
      "  the content root (1)",
      "    _setup.md (2 files)",
      "      guides/install.md (includes it)",
      "      index.md (includes it)",
      "  _parts/ (1)",
      "    _unused.md (not included)",
    ]);
  });

  it("marks a page the file leaves out, offers Place Here on it, and Rename Label… on a section", () => {
    const roots = pagesRoots(navigated);
    const old = pageOf(roots, "guides/old.md");
    expect(pagesLook(old)).toMatchObject({ icon: "warning", context: "ascribe.pages.unplaced" });
    expect(pagesLook(old).tooltip).toContain("Place Here appends it");
    expect(pagesLook(pageOf(roots, "guides/install.md")).context).toBe("ascribe.pages.page");
    // A section's own page doesn't move, so it isn't offered Place Here.
    expect(pagesLook(pageOf(roots, "index.md")).context).toBe("ascribe.pages.own");
    expect(pagesLook(sectionOf(roots, "guides/")).context).toBe("ascribe.pages.section");
    expect(pagesPath(pageOf(roots, "guides/install.md"))).toBe("guides/install.md");
  });

  it("marks an unlisted page as unlisted, not as one to place", () => {
    const secret = {
      path: "guides/secret.md",
      title: "Secret",
      label: "Secret",
      navLabel: null,
      type: "guide",
      incoming: 0,
      builds: ["site"],
    };
    const roots = pagesRoots({
      ...navigated,
      pages: [...navigated.pages, secret],
      navigation: section("", "Start", {
        page: "index.md",
        sections: [{ ...guides, pages: [...guides.pages, page("guides/secret.md", true, true)] }],
      }),
    });
    const look = pagesLook(pageOf(roots, "guides/secret.md"));
    expect(look).toMatchObject({
      description: "unlisted",
      icon: "eye-closed",
      context: "ascribe.pages.page",
    });
    expect(look.tooltip).toContain("no sidebar shows it");
    expect(look.tooltip).not.toContain("Place Here");
  });

  it("draws a page by its nav-label, and Rename Label… offers it, or a section's label", () => {
    const labelled: InventoryResult = {
      ...navigated,
      pages: navigated.pages.map((page) =>
        page.path === "guides/install.md"
          ? { ...page, label: "Setup", navLabel: "Setup" }
          : page.path === "index.md"
            ? { ...page, label: "Home" }
            : page,
      ),
    };
    const roots = pagesRoots(labelled);
    const install = pageOf(roots, "guides/install.md");
    expect(pagesLook(install).label).toBe("Setup");
    expect(pagesLook(install).description).toBe("guides/install.md");
    expect(pagesLook(install).tooltip).toContain("Its nav-label shortens its title, Install");
    expect(pagesLook(pageOf(roots, "ref/api.md")).tooltip).not.toContain("nav-label");
    expect(labelRename(install)).toMatchObject({
      action: "setPageLabel",
      page: "guides/install.md",
      value: "Setup",
    });
    // With none, its label as the navigation shows it: its title.
    expect(labelRename(pageOf(roots, "guides/old.md"))).toMatchObject({
      action: "setPageLabel",
      page: "guides/old.md",
      value: "Old",
    });
    // A section's own page has a label of its own, in its neighbors' links.
    expect(labelRename(pageOf(roots, "index.md"))).toMatchObject({
      action: "setPageLabel",
      value: "Home",
    });
    expect(labelRename(sectionOf(roots, "guides/"))).toMatchObject({
      action: "setSectionLabel",
      folder: "guides/",
      value: "Guides",
    });
    expect(labelRename(sectionOf(roots, "ref/"))).toMatchObject({ value: "Reference" });
    expect(labelRename(pageOf(roots, "_setup.md"))).toBeUndefined();
    expect(labelRename(nth(roots, roots.length - 1))).toBeUndefined();
  });

  it("offers a page's title as written, code spans and all, when it has no nav-label", () => {
    const coded: InventoryResult = {
      ...navigated,
      pages: navigated.pages.map((page) =>
        page.path === "ref/api.md"
          ? { ...page, title: "The `loom` API", label: "The loom API" }
          : page,
      ),
    };
    const api = pageOf(pagesRoots(coded), "ref/api.md");
    expect(pagesLook(api).label).toBe("The loom API");
    expect(labelRename(api)).toMatchObject({
      action: "setPageLabel",
      page: "ref/api.md",
      value: "The `loom` API",
    });
  });

  it("names a page with no title by its file, as the navigation does", () => {
    const roots = pagesRoots({
      ...navigated,
      pages: [
        ...navigated.pages,
        {
          path: "ref/limits.md",
          title: null,
          label: "limits",
          navLabel: null,
          type: null,
          incoming: 0,
          builds: [],
        },
      ],
      navigation: section("", "Start", {
        sections: [section("ref/", "Reference", { pages: [page("ref/limits.md")] })],
      }),
    });
    expect(pagesLook(pageOf(roots, "ref/limits.md")).label).toBe("limits");
  });

  it("knows each item's parent, so the open page can be revealed", () => {
    const roots = pagesRoots(navigated);
    const part = pageOf(roots, "guides/deep/part.md");
    const up: string[] = [];
    for (let node = pagesParent(part); node; node = pagesParent(node)) {
      up.push(pagesLook(node).label);
    }
    expect(up).toEqual(["Deep", "Guides"]);
    expect(pagesParent(sectionOf(roots, "guides/"))).toBeUndefined();
  });

  it("starts with every section closed, so revealing the open page shows its path alone", () => {
    const sections: string[] = [];
    const visit = (nodes: ReturnType<typeof pagesRoots>): void => {
      for (const node of nodes) {
        if (node.kind === "section") {
          expect(pagesLook(node)).toMatchObject({ expandable: true, expanded: false });
          sections.push(node.section.folder);
        }
        visit(pagesChildren(node));
      }
    };
    visit(pagesRoots(navigated));
    // Start, a top section, a section placed among its parent's pages, and another.
    expect(sections).toEqual(["", "guides/", "guides/deep/", "ref/"]);
  });

  it("is each page's and fragment's file, which VS Code colors by its problems, and nothing else's", () => {
    const roots = pagesRoots(navigated);
    expect(pagesResource(pageOf(roots, "guides/install.md"))).toBe("guides/install.md");
    expect(pagesResource(pageOf(roots, "_setup.md"))).toBe("_setup.md");
    expect(pagesLook(pageOf(roots, "_setup.md")).context).toBe("ascribe.pages.fragment");
    expect(pagesResource(sectionOf(roots, "guides/"))).toBeUndefined();
    // An includer opens the file that includes the fragment, but isn't it.
    const includer = nth(pagesChildren(pageOf(roots, "_setup.md")), 0);
    expect(includer.kind).toBe("includer");
    expect(pagesResource(includer)).toBeUndefined();
  });

  it("dims a page the build the status bar shows leaves out, and says so", () => {
    const roots = pagesRoots({
      ...navigated,
      pages: navigated.pages.map((p) =>
        p.path === "guides/install.md" ? { ...p, builds: ["site", "cloud"] } : p,
      ),
    });
    const install = pageOf(roots, "guides/install.md");
    const api = pageOf(roots, "ref/api.md");
    expect(pagesLook(api, "cloud")).toMatchObject({ dimmed: true, description: "Not in cloud" });
    expect(pagesLook(api, "cloud").tooltip).toContain("The build cloud");
    expect(pagesLook(install, "cloud")).toMatchObject({
      dimmed: false,
      description: "guides/install.md",
    });
    // No build known: nothing is dimmed.
    expect(pagesLook(api)).toMatchObject({ dimmed: false });
  });

  it("dims by a line of a versioned build as by any build", () => {
    const page = {
      path: "a.md",
      title: null,
      label: "a",
      navLabel: null,
      type: null,
      incoming: 0,
      builds: ["site@3.4"],
    };
    expect(leftOut(page, "site@3.4")).toBe(false);
    expect(leftOut(page, "site")).toBe(true);
    expect(leftOut(page, "cloud")).toBe(true);
    expect(leftOut(page, undefined)).toBe(false);
    expect(leftOut(undefined, "cloud")).toBe(false);
  });

  it("lists only the pages and fragments nothing links to or includes, in their sections, with Orphans Only", () => {
    const orphaned: InventoryResult = { ...navigated, orphans: ["guides/old.md", "ref/api.md"] };
    const roots = pagesRoots(orphaned, true);
    expect(outline(roots)).toEqual([
      "Guides (guides/)",
      "  Old (not placed)",
      "Reference (ref/)",
      "  API (ref/api.md)",
      "Fragments (1)",
      "  _parts/ (1)",
      "    _unused.md (not included)",
    ]);
    // Each kept item's parent is the kept section, so revealing it works.
    const old = pageOf(roots, "guides/old.md");
    expect(pagesParent(old)).toBe(sectionOf(roots, "guides/"));
    expect(pagesRoots({ ...orphaned, orphans: [], fragments: [] }, true)).toEqual([]);
    // Without the filter, everything.
    expect(outline(pagesRoots(orphaned))).toEqual(outline(pagesRoots(navigated)));
  });

  it("shows only fragments when there's no navigation", () => {
    const roots = pagesRoots(inventory);
    expect(roots.map((n) => n.kind)).toEqual(["fragments"]);
    expect(pagesRoots({ ...inventory, fragments: [] })).toEqual([]);
  });
});

/**
 * A top section whose own page is beside its folder: `guides.md` beside
 * `guides/`, sorting between the root's other pages. The navigation is as
 * the server gives it, and as `nav.json` has it for the same project: the
 * page among the root's pages too, and the sections in `[[sections]]`
 * order, Reference listed first.
 */
const besideTop: InventoryResult = {
  ...inventory,
  pages: [
    ...inventory.pages,
    ...[
      ["a.md", "Alpha"],
      ["guides.md", "Guides"],
      ["z.md", "Zeta"],
    ].map(([path, title]) => ({
      path: path ?? "",
      title: title ?? null,
      label: title ?? "",
      navLabel: null,
      type: "guide",
      incoming: 1,
      builds: ["site"],
    })),
  ],
  fragments: [],
  navigation: section("", "Start", {
    page: "index.md",
    pages: [page("a.md"), page("guides.md"), page("z.md")],
    sections: [
      reference,
      section("guides/", "Guides", {
        page: "guides.md",
        pages: [page("guides/install.md")],
      }),
    ],
  }),
};

describe("a top section beside its page in the Pages view", () => {
  const roots = pagesRoots(besideTop);

  it("draws it after Start, in the navigation's order, its page heading it", () => {
    expect(outline(roots)).toEqual([
      "Start",
      "  Home (index.md)",
      "  Alpha (a.md)",
      "  Zeta (z.md)",
      "Reference (ref/)",
      "  API (ref/api.md)",
      "Guides (guides/)",
      "  Guides (guides.md)",
      "  Install (guides/install.md)",
    ]);
    const top = roots.flatMap((node) => (node.kind === "section" ? [node.section.folder] : []));
    expect(top.slice(1)).toEqual(besideTop.navigation?.sections.map((s) => s.folder));
  });

  it("moves it among the top sections when it's dragged", () => {
    const guidesNode = sectionOf(roots, "guides/");
    const refNode = sectionOf(roots, "ref/");
    expect(pagesDrop(guidesNode, refNode)).toEqual({
      kind: "edit",
      action: "moveSection",
      args: { folder: "guides/", before: "ref/" },
    });
    expect(pagesDrop(refNode, guidesNode)).toEqual({
      kind: "edit",
      action: "moveSection",
      args: { folder: "ref/", before: null },
    });
    // Its page heads it, and doesn't move among Start's pages.
    expect(pagesDrop(pageOf(roots, "guides.md"), pageOf(roots, "a.md"))).toEqual({ kind: "none" });
  });
});

describe("dropping in the Pages view", () => {
  const roots = pagesRoots(navigated);
  const drop = (dragged: string, target: string) => {
    const node = (name: string) =>
      name.endsWith("/") || name === "" ? sectionOf(roots, name) : pageOf(roots, name);
    return pagesDrop(node(dragged), node(target));
  };

  it("puts a section before one above it, and after one below it", () => {
    expect(drop("ref/", "guides/")).toEqual({
      kind: "edit",
      action: "moveSection",
      args: { folder: "ref/", before: "guides/" },
    });
    expect(drop("guides/", "ref/")).toEqual({
      kind: "edit",
      action: "moveSection",
      args: { folder: "guides/", before: null },
    });
    expect(drop("ref/", "guides/deep/")).toMatchObject({ kind: "none" });
  });

  it("puts a page among its section's, and first when dropped on its section", () => {
    expect(drop("guides/old.md", "guides/install.md")).toEqual({
      kind: "edit",
      action: "movePage",
      args: { page: "guides/old.md", before: "guides/install.md" },
    });
    expect(drop("guides/install.md", "guides/old.md")).toEqual({
      kind: "edit",
      action: "movePage",
      args: { page: "guides/install.md", before: null },
    });
    // A section beside its page moves as its page does.
    expect(drop("guides/deep/", "guides/install.md")).toEqual({
      kind: "edit",
      action: "movePage",
      args: { page: "guides/deep.md", before: "guides/install.md" },
    });
    expect(drop("guides/deep/", "guides/old.md")).toEqual({
      kind: "edit",
      action: "movePage",
      args: { page: "guides/deep.md", before: null },
    });
    // A section beside its page moves as its page does, never as a section.
    expect(drop("guides/deep/", "guides/")).toMatchObject({ kind: "edit", action: "movePage" });
    expect(drop("guides/old.md", "guides/")).toEqual({
      kind: "edit",
      action: "movePage",
      args: { page: "guides/old.md", before: "guides/install.md" },
    });
  });

  it("moves a page's file when it's dropped on another section, asked first", () => {
    expect(drop("guides/install.md", "ref/")).toEqual({
      kind: "move",
      page: "guides/install.md",
      folder: "ref/",
      label: "Reference",
    });
    expect(drop("guides/install.md", "ref/api.md")).toMatchObject({ kind: "move", folder: "ref/" });
    expect(drop("index.md", "ref/")).toEqual({ kind: "none" });
  });
});

describe("Rename or Move… on a page", () => {
  it("takes a new path from the content root, ending in .md", () => {
    expect(newPathProblem("keys.md", "guides/keys.md")).toBeUndefined();
    expect(newPathProblem("keys.md", "keys.md")).toMatch(/its path now/);
    expect(newPathProblem("keys.md", "keys.txt")).toMatch(/ends in \.md/);
    expect(newPathProblem("keys.md", "/keys2.md")).toMatch(/from the content root/);
    expect(newPathProblem("keys.md", "../keys2.md")).toMatch(/from the content root/);
    expect(newPathProblem("keys.md", "guides//keys.md")).toMatch(/from the content root/);
    expect(newPathProblem("keys.md", "guides\\keys.md")).toMatch(/from the content root/);
    expect(newPathProblem("keys.md", "Keys.md")).toBeUndefined();
  });

  it("lets a rename that changes only case past the file it finds there", () => {
    expect(blocksMove("keys.md", "Keys.md")).toBe(false);
    expect(blocksMove("guides/keys.md", "Guides/KEYS.md")).toBe(false);
    expect(blocksMove("keys.md", "guides/keys.md")).toBe(true);
    expect(blocksMove("keys.md", "keys2.md")).toBe(true);
  });
});

const modeled: InventoryResult = {
  ...inventory,
  model: [
    ...inventory.model,
    { kind: "widget", key: "quill-lab", label: null, uses: 1, declaration: range(12), page: null },
    {
      kind: "dimension",
      key: "deployment",
      label: null,
      uses: 4,
      declaration: range(14),
      page: null,
      values: [
        { value: "cloud", label: "Quill Cloud", declaration: range(15) },
        { value: "self-managed", label: null, declaration: null },
      ],
    },
    {
      kind: "type",
      key: "guide",
      label: "Teach a task",
      uses: 2,
      declaration: range(20),
      page: null,
    },
    { kind: "type", key: "reference", label: null, uses: 0, declaration: range(24), page: null },
  ],
};

describe("the Content model view", () => {
  it("lists the page types first, each with how many pages it has", () => {
    const roots = modelRoots(modeled);
    const types = nth(roots, 0);
    expect(modelLook(types)).toMatchObject({ label: "Page types", description: "2" });
    expect(modelLook(types).tooltip).toBe("2 types, 1 without pages");
    const guide = nth(modelChildren(types), 0);
    const reference = nth(modelChildren(types), 1);
    expect(modelLook(guide)).toMatchObject({ label: "guide", description: "2 pages" });
    expect(modelLook(reference)).toMatchObject({ description: "no pages", dimmed: true });
    expect(modelDeclaration(guide)).toEqual(range(20));
    // A type is found in the pages that name it, but isn't renamed or inserted from here.
    expect(modelLook(guide).context).toBe("ascribe.model.type references");
  });

  it("lists a dimension's values, each renamed where it's declared", () => {
    const dimensions = modelRoots(modeled).find((n) => modelLook(n).label === "Dimensions");
    const dimension = nth(dimensions ? modelChildren(dimensions) : [], 0);
    expect(modelLook(dimension)).toMatchObject({ label: "deployment", expandable: true });
    expect(modelLook(dimension).context).toBe("ascribe.model.dimension references");
    const cloud = nth(modelChildren(dimension), 0);
    const selfManaged = nth(modelChildren(dimension), 1);
    expect(modelLook(cloud)).toMatchObject({
      label: "cloud",
      description: "Quill Cloud",
      context: "ascribe.model.value rename",
    });
    expect(modelDeclaration(cloud)).toEqual(range(15));
    expect(modelLook(selfManaged).context).toBe("ascribe.model.value");
  });

  it("offers Find All References, Rename…, and Insert at the Cursor where each applies", () => {
    const context = (kind: string, key: string) => {
      for (const root of modelRoots(modeled)) {
        for (const entry of modelChildren(root)) {
          if (entry.kind === "entry" && entry.entry.kind === kind && entry.entry.key === key) {
            return modelLook(entry).context;
          }
        }
      }
      throw new Error(`no ${kind} ${key}`);
    };
    expect(context("phrase", "product")).toBe("ascribe.model.phrase references rename insert");
    expect(context("widget", "quill-lab")).toBe("ascribe.model.widget references insert");
    // A built-in note type has no declaration to ask at; a build isn't used by pages.
    expect(context("note", "note")).toBe("ascribe.model.note");
    expect(context("build", "site")).toBe("ascribe.model.build");
    expect(context("type", "guide")).toBe("ascribe.model.type references");
    // The implicit page type isn't declared, and its pages name no type.
    const implicit = {
      kind: "type",
      key: "page",
      label: null,
      uses: 3,
      declaration: null,
      page: null,
    } as const;
    expect(modelLook({ kind: "entry", entry: implicit }).context).toBe("ascribe.model.type");
  });

  it("lists only the entries nothing uses with Unused Only", () => {
    const roots = modelRoots(modeled, true);
    expect(roots.map((n) => modelLook(n).label)).toEqual(["Page types", "Phrases"]);
    expect(modelChildren(nth(roots, 1)).map((n) => modelLook(n).label)).toEqual(["old"]);
    expect(modelRoots({ ...modeled, model: [] }, true)).toEqual([]);
  });

  it("has a node per kind with a count, in a fixed order", () => {
    const roots = modelRoots(inventory);
    expect(roots.map((n) => [modelLook(n).label, modelLook(n).description])).toEqual([
      ["Phrases", "2"],
      ["Glossary", "1"],
      ["Note types", "1"],
      ["Builds", "1"],
    ]);
    expect(modelLook(nth(roots, 0)).tooltip).toBe("2 entries, 1 unused");
  });

  it("shows each entry's uses, and marks one nothing uses", () => {
    const roots = modelRoots(inventory);
    const phrases = nth(roots, 0);
    const glossary = nth(roots, 1);
    const notes = nth(roots, 2);
    const builds = nth(roots, 3);
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
    // A glossary term says which page it's on.
    expect(modelLook(nth(modelChildren(glossary), 0)).tooltip).toContain("On ref/api.md");
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
    // Only About starts open; the rest say in their headers whether to open them.
    expect(thisPageItems(roots, () => false).items.map((item) => item.open)).toEqual([
      true,
      false,
      false,
      false,
    ]);
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
      const [, reviewItem] = thisPageItems(roots, () => false).items;
      expect(reviewItem).toMatchObject({ key: "review", kind: "section", open: false });
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
      description: {
        field: "description",
        length: 312,
        limit: 300,
        reported: true,
        range: range(3),
      },
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
      // An example opens the file its `@snippet` shows, beside the page; one
      // whose `@snippet` the server lists no file for goes to its line.
      expect(health[2]?.opens).toEqual({ kind: "beside", uri: "file:///p/code/greet.ts" });
      expect(health[3]?.opens).toEqual({ kind: "place", uri, range: range(40) });
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

    it("warn of a long description only where description-too-long reports it", () => {
      const warned = (reported: boolean) => {
        const roots = thisPageRoots({
          ...full,
          about: {
            ...fullAbout,
            description: {
              field: "description",
              length: 312,
              limit: 300,
              reported,
              range: range(3),
            },
          },
        });
        return thisPageChildren(nth(roots, 0))
          .flatMap((n) => (n.kind === "row" ? [n.row] : []))
          .find((row) => row.label === "Description");
      };
      expect(warned(true)).toMatchObject({ value: "312 of 300 characters, too long", warn: true });
      expect(warned(false)).toMatchObject({ value: "312 of 300 characters", warn: false });
    });

    it("say when a review is due, and that a short description is fine", () => {
      const later = thisPageRoots({
        ...full,
        about: {
          ...fullAbout,
          description: {
            field: "description",
            length: 120,
            limit: null,
            reported: false,
            range: range(3),
          },
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

describe("This page's items, as its webview draws them", () => {
  const page: PageResult = {
    about: {
      path: "keys.md",
      fragment: false,
      title: "API keys",
      type: "page",
      address: "/keys/",
      markdownAddress: null,
      description: null,
      review: null,
      availability: null,
      builds: [{ name: "site", leftOut: null }],
      listing: null,
      permalinks: [],
      template: null,
    },
    heading: null,
    health: { size: null, examples: [] },
    linkedFrom: [
      { uri: "file:///p/docs/a.md", path: "a.md", range: range(3), text: "See [](keys.md)." },
    ],
    includedBy: [],
    includes: [],
    memos: [],
  };

  it("keys each section by its kind, and the rest by where they are under it", () => {
    const { items, nodes } = thisPageItems(thisPageRoots(page), () => false);
    expect(items.map((item) => item.key)).toEqual(["about", "health", "connections", "memos"]);
    const connections = nth(items, 2);
    expect(connections.children.map((item) => [item.key, item.kind])).toEqual([
      ["connections/0", "group"],
      ["connections/1", "group"],
      ["connections/2", "group"],
    ]);
    const link = nth(nth(connections.children, 0).children, 0).children[0];
    expect(link).toMatchObject({
      key: "connections/0/0/0",
      kind: "item",
      label: "See [](keys.md).",
    });
    expect(nodes.get("connections/0/0/0")?.kind).toBe("place");
    // About's rows are a label and a value.
    expect(
      nth(items, 0).children.map((item) => `${item.kind} ${item.label}: ${item.description}`),
    ).toEqual([
      "row Title: API keys",
      "row Type: page",
      "row Address: /keys/",
      "row Markdown: not published",
      "row Builds: Published by site",
    ]);
  });

  it("asks the model which items do something when they're chosen", () => {
    const { items } = thisPageItems(
      thisPageRoots(page),
      (node) => node.kind === "place" || (node.kind === "row" && node.row.opens !== undefined),
    );
    const acting = (list: typeof items): string[] =>
      list.flatMap((item) => [...(item.acts ? [item.key] : []), ...acting(item.children)]);
    expect(acting(items)).toEqual(["about/4", "connections/0/0/0"]);
  });

  it("marks a row that says something needs doing, and dims a section with nothing in it", () => {
    const { items } = thisPageItems(
      thisPageRoots({
        ...page,
        health: {
          size: { characters: 60_000, limit: 50_000, build: "site" },
          examples: [],
        },
      }),
      () => false,
    );
    const health = nth(items, 1);
    expect(health).toMatchObject({ description: "1 thing to do", dimmed: false });
    expect(nth(health.children, 0)).toMatchObject({ kind: "row", label: "Size", warn: true });
    expect(nth(items, 3)).toMatchObject({ label: "Memos", description: "none", dimmed: true });
  });
});

describe("the page This page shows", () => {
  // Each page by its document; the open ones are those with a tab, covered or not.
  const isPage = (file: string) => file.endsWith(".md");

  it("is the active editor's, when it's a page's", () => {
    expect(shownPage("keys.md", "quickstart.md", ["keys.md", "quickstart.md"], isPage)).toBe(
      "quickstart.md",
    );
  });

  it("stays on its page while the focus is in the view, a panel, or a file that isn't a page", () => {
    // VS Code reports no active editor while a webview has the focus.
    expect(shownPage("keys.md", undefined, ["keys.md"], isPage)).toBe("keys.md");
    // A file that isn't a page covering the page's tab in its group leaves the tab open.
    expect(shownPage("keys.md", "ascribe.toml", ["keys.md", "ascribe.toml"], isPage)).toBe(
      "keys.md",
    );
  });

  it("isn't moved by a page that's only open in another group", () => {
    expect(shownPage("keys.md", undefined, ["keys.md", "quickstart.md"], isPage)).toBe("keys.md");
  });

  it("shows none once its tab is closed and the active one isn't a page's", () => {
    expect(shownPage("keys.md", "ascribe.toml", ["ascribe.toml"], isPage)).toBeUndefined();
    expect(shownPage("keys.md", undefined, [], isPage)).toBeUndefined();
    expect(shownPage(undefined, undefined, [], isPage)).toBeUndefined();
  });
});

describe("the project the sidebar follows", () => {
  const known = ["docs", "quill"];

  it("takes the active file's project when the active editor changes to it", () => {
    expect(followedProject("quill", "docs", true, known)).toBe("docs");
    expect(followedProject("docs", "docs", true, known)).toBe("docs");
  });

  it("keeps a project Switch Project chose while the views are only drawn again", () => {
    // Its server starting, or the projects changing, isn't a file becoming active.
    expect(followedProject("quill", "docs", false, known)).toBe("quill");
  });

  it("stays on its project for a file of no project", () => {
    expect(followedProject("quill", undefined, true, known)).toBe("quill");
  });

  it("takes the active file's project when it shows none, or one that's gone", () => {
    expect(followedProject(undefined, "docs", false, known)).toBe("docs");
    expect(followedProject("gone", "docs", false, known)).toBe("docs");
  });
});

describe("what the views show while their project's server can't answer", () => {
  it("leaves a failed server's welcome showing, with its links, under the project's name", () => {
    // An empty message is what lets the views' welcome, with Show Output and Restart, show.
    expect(viewsNotice("failed", "quill", undefined)).toEqual({
      message: "",
      description: "quill",
    });
    expect(viewsNotice("failed", "quill", navigated)).toEqual({
      message: "",
      description: "quill",
    });
  });

  it("says a server Switch Project started is starting, until it answers", () => {
    expect(viewsNotice("starting", "docs", undefined)).toEqual({
      message: "Starting the project's language server.",
      description: "docs",
    });
    expect(viewsNotice("starting", "docs", navigated)).toBeUndefined();
  });

  it("leaves the rest to the inventory", () => {
    expect(viewsNotice("running", "docs", undefined)).toBeUndefined();
    expect(viewsNotice("stopped", "docs", undefined)).toBeUndefined();
    expect(viewsNotice(undefined, "docs", undefined)).toBeUndefined();
  });
});

describe("a file's content path, for revealing it in Pages", () => {
  it("is the path below the content root, in the file's own case", () => {
    expect(contentPathOf("/work/docs/Guides/My Setup.md", "/work/docs")).toBe("Guides/My Setup.md");
    expect(contentPathOf("/work/docs/Guides/My Setup.md", "/work/docs/")).toBe(
      "Guides/My Setup.md",
    );
  });

  it("matches a Windows drive in either case, as the server and VS Code write it", () => {
    expect(contentPathOf("c:\\work\\docs\\Guides\\My Setup.md", "C:\\work\\docs")).toBe(
      "Guides/My Setup.md",
    );
    expect(contentPathOf("C:\\Work\\Docs\\index.md", "c:\\work\\docs")).toBe("index.md");
  });

  it("is nothing outside the root, or for the root itself", () => {
    expect(contentPathOf("/work/docs-old/index.md", "/work/docs")).toBeUndefined();
    expect(contentPathOf("/work/docs", "/work/docs")).toBeUndefined();
    expect(contentPathOf("/Work/docs/index.md", "/work/docs")).toBeUndefined();
  });
});
