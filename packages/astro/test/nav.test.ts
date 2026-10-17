import { describe, expect, it } from "vitest";
import {
  entryId,
  navFor,
  navigationGroups,
  navProblems,
  pageRoute,
  sidebarGroups,
  type Navigation,
} from "../src/nav.js";

describe("entryId", () => {
  it("is Astro's: each segment slugged, without the extension or a final index", () => {
    expect(entryId("Guides/My Setup.md")).toBe("guides/my-setup");
    expect(entryId("index.md")).toBe("index");
    expect(entryId("guides/index.md")).toBe("guides");
  });
});

describe("pageRoute", () => {
  it("is the base path plus the entry id, as Ascribe's links are", () => {
    expect(pageRoute("guides/my-setup", "/docs/", "never")).toBe("/docs/guides/my-setup");
    expect(pageRoute("guides/my-setup", "/docs/", "always")).toBe("/docs/guides/my-setup/");
  });

  it("puts the root page at the base path itself", () => {
    expect(pageRoute("index", "/docs/", "never")).toBe("/docs");
    expect(pageRoute("index", "/docs/", "always")).toBe("/docs/");
    expect(pageRoute("index", "/", "never")).toBe("/");
  });
});

const entries = [{ id: "index" }, { id: "guides/my-setup" }];

describe("navProblems", () => {
  it("finds nothing when the list names each page once", () => {
    const nav = [
      { label: "Start", pages: ["index.md"] },
      { label: "Guides", pages: ["Guides/My Setup.md"] },
    ];
    expect(navProblems(nav, entries)).toEqual([]);
  });

  it("names a page the list leaves out, a file that isn't a page, and a page named twice", () => {
    const nav = [{ label: "Start", pages: ["index.md", "gone.md", "index.md"] }];
    expect(navProblems(nav, entries, (entry) => `<${entry.id}>`)).toEqual([
      "<guides/my-setup> isn't in the sidebar's list. Add it to a group.",
      "the sidebar's list names gone.md, which isn't a page of the site.",
      "the sidebar's list names index.md twice.",
    ]);
  });
});

describe("sidebarGroups", () => {
  it("gives each listed page its entry and address, in the list's order", () => {
    const nav = [{ label: "All", pages: ["Guides/My Setup.md", "index.md"] }];
    const groups = sidebarGroups(
      nav,
      entries,
      (id) => `/docs/${id}`,
      undefined,
      undefined,
      (e) => (e.id === "index" ? "Home" : undefined),
    );
    expect(groups).toEqual([
      {
        label: "All",
        pages: [
          {
            entry: { id: "guides/my-setup" },
            href: "/docs/guides/my-setup",
            label: "guides/my-setup",
          },
          { entry: { id: "index" }, href: "/docs/index", label: "Home" },
        ],
        sections: [],
      },
    ]);
  });

  it("throws, naming each problem, when the list and the pages disagree", () => {
    const nav = [{ label: "Start", pages: ["index.md"] }];
    expect(() => sidebarGroups(nav, entries, (id) => id)).toThrow(
      /src\/nav\.ts.*\n- guides\/my-setup isn't in the sidebar's list/,
    );
  });
});

describe("sidebarGroups with lines", () => {
  // A versioned build's collection: 3.5's pages, one only it has, and 3.4's, with the line in front.
  const lined = [
    { id: "index", line: "3.5" },
    { id: "guides/my-setup", line: "3.5" },
    { id: "guides/upgrade", line: "3.5" },
    { id: "3.4", line: "3.4" },
    { id: "3.4/guides/my-setup", line: "3.4" },
  ];
  const id = (entry: { id: string; line: string }) =>
    entry.line === "3.5" ? entry.id : entry.id === "3.4" ? "index" : entry.id.slice(4);
  const nav = [{ label: "All", pages: ["index.md", "Guides/My Setup.md", "Guides/Upgrade.md"] }];

  it("shows one line's pages, each listed page that line has", () => {
    const groups = sidebarGroups(nav, lined, (to) => `/docs/${to}`, undefined, {
      id,
      shown: (entry) => entry.line === "3.4",
    });
    expect(groups[0]?.pages.map((page) => page.href)).toEqual([
      "/docs/3.4",
      "/docs/3.4/guides/my-setup",
    ]);
  });

  it("checks the list against every line's pages, a problem in several said once", () => {
    const short = [{ label: "All", pages: ["index.md", "Guides/Upgrade.md"] }];
    expect(() =>
      sidebarGroups(short, lined, (to) => to, undefined, { id, shown: () => true }),
    ).toThrow(/\n- guides\/my-setup isn't in the sidebar's list\. Add it to a group\.$/);
  });
});

describe("navigationGroups", () => {
  // `_ascribe/nav.json` for a root page, a section with its `index.md`, and
  // one whose own page is beside its folder, nested in it.
  const navigation: Navigation = {
    version: 1,
    root: {
      folder: "",
      label: "Start",
      page: { source: "index.md", label: "Home", type: "page" },
      pages: [],
      sections: [
        {
          folder: "checks/",
          label: "Checks",
          formatted: { label: "<code>check</code>s" },
          page: { source: "checks/index.md", label: "Checks" },
          pages: [
            { source: "checks/diagnostics.md", label: "Diagnostics" },
            { source: "checks/vale.md", label: "Vale" },
          ],
          sections: [
            {
              folder: "checks/diagnostics/",
              label: "Diagnostics",
              page: { source: "checks/diagnostics.md", label: "Diagnostics" },
              pages: [{ source: "checks/diagnostics/pages.md", label: "Pages" }],
              sections: [],
            },
          ],
        },
      ],
    },
  };
  const pages = [
    "index.md",
    "checks/index.md",
    "checks/diagnostics.md",
    "checks/vale.md",
    "checks/diagnostics/pages.md",
  ].map((file) => ({ id: entryId(file), filePath: file }));
  const groups = () =>
    navigationGroups(
      navigation,
      pages,
      (entry) => `/${entry.id}`,
      (entry) => entry.filePath,
    );

  it("puts the root's pages first, under its label, and a group for each top section", () => {
    const [start, checks] = groups();
    expect(start?.label).toBe("Start");
    expect(start?.pages.map((page) => page.href)).toEqual(["/index"]);
    expect(start?.sections).toEqual([]);
    expect(checks?.formatted).toEqual({ label: "<code>check</code>s" });
    // A section's `index.md` comes first among its pages.
    expect(checks?.pages.map((page) => page.label)).toEqual(["Checks", "Diagnostics", "Vale"]);
  });

  it("nests a section, its own page beside the folder left among the parent's pages", () => {
    const nested = groups()[1]?.sections[0];
    expect(nested?.page?.href).toBe("/checks/diagnostics");
    expect(nested?.pages.map((page) => page.href)).toEqual(["/checks/diagnostics/pages"]);
  });

  it("leaves the root out when it has no pages", () => {
    const empty: Navigation = {
      version: 1,
      root: { folder: "", label: "Start", pages: [], sections: navigation.root.sections },
    };
    expect(
      navigationGroups(
        empty,
        pages,
        () => "",
        (e) => e.filePath,
      )[0]?.label,
    ).toBe("Checks");
  });

  it("throws, naming each, on a page the collection has no entry for", () => {
    expect(() =>
      navigationGroups(
        navigation,
        pages.slice(1),
        () => "",
        (e) => e.filePath,
      ),
    ).toThrow(/_ascribe\/nav\.json\) names pages the collection has no entry for:\n- index\.md\n/);
  });
});

describe("navFor", () => {
  it("puts the root's pages under Start, the root page first, then a group per folder", () => {
    expect(
      navFor(["Reference/Options.md", "about.md", "Guides/b.md", "index.md", "Guides/Sub/a.md"]),
    ).toEqual([
      { label: "Start", pages: ["index.md", "about.md"] },
      { label: "Guides", pages: ["Guides/Sub/a.md", "Guides/b.md"] },
      { label: "Reference", pages: ["Reference/Options.md"] },
    ]);
  });

  it("is empty for no pages", () => {
    expect(navFor([])).toEqual([]);
  });
});
