import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { forwardingPaths, permalinkPattern, readPermalinks } from "../src/permalinks.js";

function siteRoot(list: unknown): string {
  const root = mkdtempSync(path.join(tmpdir(), "ascribe-permalinks-"));
  if (list !== undefined) {
    mkdirSync(path.join(root, "_ascribe"));
    writeFileSync(path.join(root, "_ascribe", "permalinks.json"), JSON.stringify(list));
  }
  return root;
}

describe("permalinkPattern", () => {
  it("is the list's path under Astro's base, with the id as a parameter", () => {
    expect(permalinkPattern("/go/", "/")).toBe("/go/[id]");
    expect(permalinkPattern("/docs/go/", "/docs")).toBe("/go/[id]");
    expect(permalinkPattern("/docs/go/", "/docs/")).toBe("/go/[id]");
    expect(permalinkPattern("/docs/", "/docs")).toBe("/[id]");
  });

  it("refuses a path outside the base, which Astro can't serve", () => {
    expect(() => permalinkPattern("/go/", "/docs")).toThrow(/isn't under Astro's `base`, "\/docs"/);
    expect(() => permalinkPattern("/documents/go/", "/docs")).toThrow(/permalink-path/);
  });
});

describe("forwardingPaths", () => {
  it("is one page for each permalink, leading to its route and anchor", () => {
    const root = siteRoot({
      version: 1,
      path: "/docs/go/",
      permalinks: {
        "cli-check": {
          route: "/docs/reference/cli/",
          fragment: "cli-check",
          source: "reference/cli.md",
        },
        overview: { route: "/docs/overview/", source: "overview.md" },
      },
    });
    expect(forwardingPaths(root)).toEqual([
      { params: { id: "cli-check" }, props: { to: "/docs/reference/cli/#cli-check" } },
      { params: { id: "overview" }, props: { to: "/docs/overview/" } },
    ]);
  });

  it("is no pages without a list, or with an empty one", () => {
    expect(forwardingPaths(siteRoot(undefined))).toEqual([]);
    expect(forwardingPaths(siteRoot({ version: 1, path: "/go/", permalinks: {} }))).toEqual([]);
  });

  it("refuses a list in a format it doesn't read", () => {
    const root = siteRoot({ version: 2, path: "/go/", permalinks: {} });
    expect(() => readPermalinks(root)).toThrow(/version 2/);
  });
});
