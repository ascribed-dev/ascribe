import { describe, expect, it } from "vitest";
import { entryId, navFor, navProblems, pageRoute, sidebarGroups } from "../src/nav.js";

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
    const groups = sidebarGroups(nav, entries, (id) => `/docs/${id}`);
    expect(groups).toEqual([
      {
        label: "All",
        pages: [
          { entry: { id: "guides/my-setup" }, href: "/docs/guides/my-setup" },
          { entry: { id: "index" }, href: "/docs/index" },
        ],
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
