import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import type { PermalinkList } from "../src/permalinks.js";
import { readProject } from "../src/project.js";
import {
  basePathClashes,
  entryId,
  nameProblems,
  pagesUnderOtherPaths,
  permalinkProblems,
  sitePages,
  type ResolvedEntry,
} from "../src/projects.js";

/** A project with `base-path`, read. */
function project(basePath: string) {
  const dir = mkdtempSync(path.join(tmpdir(), "ascribe-astro-projects-"));
  writeFileSync(path.join(dir, "ascribe.toml"), `[consumer]\nbase-path = "${basePath}"\n`);
  return readProject(dir);
}

function entry(name: string, info: ResolvedEntry["project"], build = "site"): ResolvedEntry {
  return { name, build, project: info };
}

function list(names: string[], at = "/go/", removed: string[] = []): PermalinkList {
  return {
    version: 1,
    path: at,
    permalinks: Object.fromEntries(names.map((n) => [n, { route: "/x/", source: "x.md" }])),
    removed: Object.fromEntries(removed.map((n) => [n, { reason: "gone" }])),
  };
}

describe("nameProblems", () => {
  it("accepts names a collection can have", () => {
    expect(
      nameProblems([
        { name: "docs", build: "site" },
        { name: "v3-4_x", build: "3.4" },
      ]),
    ).toEqual([]);
  });

  it("refuses a name used twice, a name a folder or collection can't have, and no entries", () => {
    expect(
      nameProblems([
        { name: "docs", build: "site" },
        { name: "docs", build: "pdf" },
      ]).join("\n"),
    ).toMatch(/two entries are named "docs"/);
    expect(nameProblems([{ name: "../docs", build: "site" }]).join("\n")).toMatch(/"\.\.\/docs"/);
    expect(nameProblems([{ name: "", build: "site" }])).toHaveLength(1);
    expect(nameProblems([])).toHaveLength(1);
  });
});

describe("basePathClashes", () => {
  it("refuses two projects at one base path, naming both", () => {
    const problems = basePathClashes([
      entry("docs", project("/docs")),
      entry("api", project("/docs/")),
    ]);
    expect(problems).toHaveLength(1);
    expect(problems[0]).toMatch(/"docs" and "api" both have \[consumer\] base-path "\/docs\/"/);
  });

  it("accepts one project beneath another, and two builds of one project", () => {
    const docs = project("/docs/");
    expect(basePathClashes([entry("home", project("/")), entry("docs", docs)])).toEqual([]);
    expect(basePathClashes([entry("docs", docs), entry("all", docs, "all")])).toEqual([]);
  });
});

describe("entryId", () => {
  it("is Astro's: each segment slugged, without the extension or a final index", () => {
    expect(entryId("Guides/My Setup.md")).toBe("guides/my-setup");
    expect(entryId("index.md")).toBe("index");
    expect(entryId("guides/index.md")).toBe("guides");
  });
});

describe("sitePages", () => {
  it("reads the pages from the site output's manifest", () => {
    const info = project("/");
    const buildDir = path.join(info.dir, ".ascribe", "build", "site");
    mkdirSync(buildDir, { recursive: true });
    writeFileSync(
      path.join(buildDir, "site.manifest.json"),
      JSON.stringify({
        files: [
          { path: "index.md", kind: "page", source: "index.md" },
          { path: "img/a.png", kind: "asset", source: "img/a.png" },
          { path: "_ascribe/schema.ts", kind: "generated" },
        ],
      }),
    );
    expect(sitePages(info, "site")).toEqual(["index.md"]);
    expect(sitePages(info, "nope")).toEqual([]);
  });
});

describe("pagesUnderOtherPaths", () => {
  const home = entry("home", project("/"));
  const docs = entry("docs", project("/docs/"));

  it("refuses a page of one project under another's base path, naming both", () => {
    const pages = (e: ResolvedEntry) =>
      e === home ? ["index.md", "Docs/Setup.md", "docs.md", "documents.md"] : ["index.md"];
    const problems = pagesUnderOtherPaths([home, docs], pages);
    expect(problems).toHaveLength(2);
    expect(problems[0]).toMatch(
      /the page Docs\/Setup\.md of "home" is at \/docs\/setup\/, under the base path of "docs", \/docs\//,
    );
    expect(problems[1]).toMatch(/the page docs\.md of "home" is at \/docs\//);
  });

  it("accepts pages beside the other project's path", () => {
    expect(pagesUnderOtherPaths([home, docs], () => ["index.md", "why.md"])).toEqual([]);
  });
});

describe("permalinkProblems", () => {
  const docs = entry("docs", project("/docs/"));
  const home = entry("home", project("/"));

  it("refuses a name two projects declare, a removed one included", () => {
    expect(permalinkProblems([docs, home], [list(["setup"]), list(["setup"])]).join("\n")).toMatch(
      /"docs" and "home" both declare the permalink "setup"/,
    );
    expect(
      permalinkProblems([docs, home], [list([], "/go/", ["old"]), list(["old"])]),
    ).toHaveLength(1);
  });

  it("refuses lists at two paths, since one route serves them", () => {
    expect(
      permalinkProblems([docs, home], [list(["a"], "/docs/go/"), list(["b"], "/go/")]).join("\n"),
    ).toMatch(/"docs" publishes its permalinks at \/docs\/go\/, and "home" at \/go\//);
  });

  it("lets an empty list be at any path", () => {
    expect(permalinkProblems([docs, home], [list(["a"], "/docs/go/"), list([], "/go/")])).toEqual(
      [],
    );
  });

  it("accepts a name in two builds of one project, and a build without a list", () => {
    const all = entry("all", docs.project, "all");
    expect(
      permalinkProblems([docs, all, home], [list(["setup"]), list(["setup"]), undefined]),
    ).toEqual([]);
  });
});
