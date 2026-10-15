import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { linesOf, olderLines, pageLines } from "../src/versions.js";

/** An output directory with `versions` as its `_ascribe/versions.json`, and the site build's root. */
function output(versions: unknown): string {
  const root = mkdtempSync(path.join(tmpdir(), "ascribe-versions-"));
  if (versions !== undefined) {
    mkdirSync(path.join(root, "_ascribe"));
    writeFileSync(path.join(root, "_ascribe", "versions.json"), JSON.stringify(versions));
  }
  return path.join(root, "site", "site");
}

const VERSIONS = {
  version: 1,
  builds: {
    site: {
      target: "self-managed",
      lines: [
        { line: "3.5", status: "current", build: "site", base: "/docs/" },
        { line: "3.4", status: "live", build: "site@3.4", base: "/docs/3.4/" },
      ],
      pages: {
        "install.md": { lines: { "3.5": "/docs/install/", "3.4": "/docs/3.4/install/" } },
        "upgrade.md": { lines: { "3.5": "/docs/upgrade/" } },
      },
    },
  },
};

describe("versions.json", () => {
  it("gives a versioned build's lines, newest first, each with its site output", () => {
    const site = output(VERSIONS);
    const outputRoot = path.dirname(path.dirname(site));
    expect(linesOf(site)?.map((line) => [line.line, line.siteRoot])).toEqual([
      ["3.5", site],
      ["3.4", path.join(outputRoot, "site@3.4", "site")],
    ]);
    expect(olderLines(site).map((line) => line.base)).toEqual(["/docs/3.4/"]);
  });

  it("gives the lines that publish a page, with its route in each", () => {
    const site = output(VERSIONS);
    expect(pageLines(site, "install.md")).toEqual([
      { line: "3.5", current: true, route: "/docs/install/" },
      { line: "3.4", current: false, route: "/docs/3.4/install/" },
    ]);
    expect(pageLines(site, "upgrade.md")).toEqual([
      { line: "3.5", current: true, route: "/docs/upgrade/" },
    ]);
    expect(pageLines(site, "gone.md")).toEqual([]);
  });

  it("has nothing for a build that isn't versioned, or a project with no file", () => {
    const other = path.join(path.dirname(path.dirname(output(VERSIONS))), "cloud", "site");
    expect(linesOf(other)).toBeUndefined();
    expect(olderLines(output(undefined))).toEqual([]);
    expect(pageLines(output(undefined), "install.md")).toEqual([]);
  });

  it("refuses a version of the file it doesn't read", () => {
    expect(() => linesOf(output({ ...VERSIONS, version: 2 }))).toThrow(
      /is version 2 of the versions file, and this version of the integration reads version 1/,
    );
  });
});
