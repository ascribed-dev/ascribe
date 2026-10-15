import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { lineNotice, linesOf, listedLines, olderLines, pageLines } from "../src/versions.js";

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

  it("follows a rewritten page to the page that takes its place in a line, and back", () => {
    const site = output({
      ...VERSIONS,
      builds: {
        site: {
          ...VERSIONS.builds.site,
          pages: {
            "configure.md": {
              lines: { "3.5": "/docs/configure/" },
              replaces: "configure-legacy.md",
            },
            "configure-legacy.md": {
              lines: { "3.4": "/docs/3.4/configure-legacy/" },
              "replaced-by": "configure.md",
            },
          },
        },
      },
    });
    const both = [
      { line: "3.5", current: true, route: "/docs/configure/" },
      { line: "3.4", current: false, route: "/docs/3.4/configure-legacy/" },
    ];
    expect(pageLines(site, "configure.md")).toEqual(both);
    expect(pageLines(site, "configure-legacy.md")).toEqual(both);
  });

  it("says what an older line is and where the page is in the current one", () => {
    const site = output({
      ...VERSIONS,
      builds: {
        site: {
          ...VERSIONS.builds.site,
          label: "Self-managed",
          lines: [
            VERSIONS.builds.site.lines[0],
            { ...VERSIONS.builds.site.lines[1], ends: "2027-01-31" },
          ],
          pages: {
            ...VERSIONS.builds.site.pages,
            "old.md": { lines: { "3.4": "/docs/3.4/old/" } },
          },
        },
      },
    });
    expect(lineNotice(site, "install.md", "3.4", "/docs/")).toEqual({
      label: "Self-managed",
      line: "3.4",
      ends: "2027-01-31",
      current: "3.5",
      route: "/docs/install/",
      samePage: true,
    });
    // The current line's home, when it has no such page.
    expect(lineNotice(site, "old.md", "3.4", "/docs/")).toMatchObject({
      route: "/docs/",
      samePage: false,
    });
    // Nothing on the current line.
    expect(lineNotice(site, "install.md", "3.5", "/docs/")).toBeUndefined();
    // The target's name, and no date, from a file that doesn't say.
    expect(lineNotice(output(VERSIONS), "install.md", "3.4", "/docs/")).toEqual({
      label: "self-managed",
      line: "3.4",
      current: "3.5",
      route: "/docs/install/",
      samePage: true,
    });
  });

  it("lists every line for a page: those without it unlinked, and archived ones with an address", () => {
    const site = output({
      ...VERSIONS,
      builds: {
        site: {
          ...VERSIONS.builds.site,
          lines: [
            ...VERSIONS.builds.site.lines,
            { line: "3.3", status: "archived", url: "https://archive.example.com/3.3/" },
            { line: "3.2", status: "archived" },
          ],
        },
      },
    });
    expect(listedLines(site, "upgrade.md")).toEqual([
      { line: "3.5", status: "current", route: "/docs/upgrade/" },
      { line: "3.4", status: "live" },
      { line: "3.3", status: "archived", route: "https://archive.example.com/3.3/" },
    ]);
    expect(listedLines(output(undefined), "upgrade.md")).toEqual([]);
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
