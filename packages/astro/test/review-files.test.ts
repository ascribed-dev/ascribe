// The files review reads and writes: `ascribe diff`'s JSON, the JSON
// output's routes, and `.ascribe/dev.json`.
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { devFilePath, removeDevFile, siteUrl, writeDevFile } from "../src/review/devfile.js";
import { parseDiff } from "../src/review/diff.js";
import { normalizeRoute, readRoutes } from "../src/review/routes.js";

function temp(): string {
  return mkdtempSync(path.join(tmpdir(), "ascribe-review-"));
}

describe("parseDiff", () => {
  it("takes the build's pages and the base", () => {
    const json = JSON.stringify({
      schema_version: 1,
      base: { requested: "main", commit: "abc", merge_base: "def" },
      builds: [
        { build: "cloud", pages: [{ path: "x.md" }] },
        { build: "site", pages: [{ path: "a.md" }] },
      ],
    });
    expect(parseDiff(json, "site")).toEqual({
      base: { requested: "main", commit: "abc", merge_base: "def" },
      pages: [{ path: "a.md" }],
    });
    expect(parseDiff(json, "other").pages).toEqual([]);
    expect(() => parseDiff("{}", "site")).toThrow();
  });
});

describe("routes", () => {
  it("compares routes decoded and without a trailing slash", () => {
    expect(normalizeRoute("/docs/guides/my-setup/")).toBe("/docs/guides/my-setup");
    expect(normalizeRoute("/docs/caf%C3%A9?x=1#top")).toBe("/docs/café");
    expect(normalizeRoute("/")).toBe("/");
    expect(normalizeRoute("")).toBe("/");
  });

  it("reads every page's route from the JSON output", async () => {
    const build = temp();
    const json = path.join(build, "json");
    mkdirSync(path.join(json, "Guides"), { recursive: true });
    writeFileSync(
      path.join(build, "json.manifest.json"),
      JSON.stringify({
        files: [
          { path: "Guides/My Setup.json", kind: "page" },
          { path: "index.json", kind: "page" },
          { path: "weave.png", kind: "asset" },
          { path: "missing.json", kind: "page" },
        ],
      }),
    );
    writeFileSync(
      path.join(json, "Guides", "My Setup.json"),
      JSON.stringify({
        schemaVersion: 1,
        path: "Guides/My Setup.md",
        route: "/docs/guides/my-setup",
        title: 'My "setup"',
        frontmatter: { title: 'My "setup"' },
      }),
    );
    // A route past the part read first, and no title: read the whole file.
    writeFileSync(
      path.join(json, "index.json"),
      JSON.stringify({ padding: "x".repeat(5000), path: "index.md", route: "/docs/" }),
    );
    const routes = await readRoutes(json);
    expect([...routes.entries()].sort(([a], [b]) => a.localeCompare(b))).toEqual([
      ["/docs", { path: "index.md", title: null }],
      ["/docs/guides/my-setup", { path: "Guides/My Setup.md", title: 'My "setup"' }],
    ]);
    expect((await readRoutes(path.join(temp(), "json"))).size).toBe(0);
  });
});

describe("dev.json", () => {
  it("names the site's address, with a loopback or wildcard host as localhost", () => {
    const at = (address: string, port = 4321) => ({ address, port, family: "IPv4" });
    expect(siteUrl(at("::1"), { https: false, base: "/docs" })).toBe("http://localhost:4321/docs/");
    expect(siteUrl(at("0.0.0.0"), { https: false, base: "/" })).toBe("http://localhost:4321/");
    expect(siteUrl(at("127.0.0.1"), { https: true, base: "/docs/" })).toBe(
      "https://127.0.0.1:4321/docs/",
    );
    expect(siteUrl(at("fe80::1"), { https: false, base: "/" })).toBe("http://[fe80::1]:4321/");
  });

  it("is written, and removed only by the process that wrote it", () => {
    const project = temp();
    writeDevFile(project, { url: "http://localhost:4321/", build: "site", pid: 1 });
    expect(JSON.parse(readFileSync(devFilePath(project), "utf8"))).toEqual({
      url: "http://localhost:4321/",
      build: "site",
      pid: 1,
    });
    removeDevFile(project, 2);
    expect(existsSync(devFilePath(project))).toBe(true);
    removeDevFile(project, 1);
    expect(existsSync(devFilePath(project))).toBe(false);
    // Nothing to remove: nothing happens.
    removeDevFile(project, 1);
  });
});
