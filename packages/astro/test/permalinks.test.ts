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
      dimensions: { pm: ["npm", "pnpm"] },
      permalinks: {
        "cli-check": {
          route: "/docs/reference/cli/",
          fragment: "cli-check",
          source: "reference/cli.md",
        },
        overview: { route: "/docs/overview/", source: "overview.md" },
      },
    });
    const dimensions = { pm: ["npm", "pnpm"] };
    expect(forwardingPaths(root)).toEqual([
      {
        params: { id: "cli-check" },
        props: { to: "/docs/reference/cli/#cli-check", dimensions },
      },
      { params: { id: "overview" }, props: { to: "/docs/overview/", dimensions } },
    ]);
  });

  it("takes no parameters from a list that names no dimensions", () => {
    const root = siteRoot({
      version: 1,
      path: "/go/",
      permalinks: { overview: { route: "/overview/", source: "overview.md" } },
    });
    expect(forwardingPaths(root)).toEqual([
      { params: { id: "overview" }, props: { to: "/overview/", dimensions: {} } },
    ]);
  });

  it("leads an alias and a retired permalink like any other, and says why one was removed", () => {
    const root = siteRoot({
      version: 1,
      path: "/go/",
      permalinks: {
        settings: { route: "/settings/", source: "settings.md" },
        "agent-settings": { route: "/settings/", source: "settings.md", alias: "settings" },
        "rotate-keys": {
          route: "/security/",
          fragment: "keys",
          source: "security.md",
          retired: "Merged into the security guide",
        },
      },
      removed: { "legacy-sync": { reason: "The sync agent was removed in <b>4.0</b>" } },
    });
    expect(forwardingPaths(root)).toEqual([
      { params: { id: "settings" }, props: { to: "/settings/", dimensions: {} } },
      { params: { id: "agent-settings" }, props: { to: "/settings/", dimensions: {} } },
      { params: { id: "rotate-keys" }, props: { to: "/security/#keys", dimensions: {} } },
      {
        params: { id: "legacy-sync" },
        props: { reason: "The sync agent was removed in <b>4.0</b>" },
      },
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
