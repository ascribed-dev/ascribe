import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { readHelp } from "../src/help-file.js";
import { helpAddresses, helpMarkdown, helpPaths, helpPatterns } from "../src/help.js";

/** An output directory with each build's files, by build and path under `_ascribe/`; the `site` build's site output root. */
function output(files: Record<string, Record<string, unknown>>): string {
  const root = mkdtempSync(path.join(tmpdir(), "ascribe-help-"));
  for (const [build, byName] of Object.entries(files)) {
    const dir =
      build === "" ? path.join(root, "_ascribe") : path.join(root, build, "site", "_ascribe");
    mkdirSync(dir, { recursive: true });
    for (const [name, contents] of Object.entries(byName)) {
      writeFileSync(path.join(dir, name), JSON.stringify(contents));
    }
  }
  return path.join(root, "site", "site");
}

const connect = (line: string) => ({
  title: "Connect to Quill",
  summary: `Connect the ${line} agent.`,
  text: `Connect the ${line} agent.`,
  address: "https://docs.quill.dev/docs/go/connect/",
  source: "install-agent.md",
});

const PERMALINKS = {
  version: 1,
  path: "/docs/go/",
  permalinks: {
    connect: { route: "/docs/install-agent/", fragment: "connect", source: "install-agent.md" },
  },
};

/** A versioned build with live lines 3.4 and 3.5, as `ascribe build` writes it. */
function versioned(): string {
  return output({
    "": {
      "versions.json": {
        version: 1,
        builds: {
          site: {
            target: "self-managed",
            lines: [
              { line: "3.5", status: "current", build: "site", base: "/docs/" },
              { line: "3.4", status: "live", build: "site@3.4", base: "/docs/3.4/" },
              { line: "3.3", status: "archived", url: "https://archive.quill.dev/3.3/" },
            ],
            pages: {},
          },
        },
      },
    },
    site: {
      "permalinks.json": PERMALINKS,
      "help.json": { version: 1, build: "site", line: "3.5", help: { connect: connect("3.5") } },
    },
    "site@3.4": {
      // An older line's permalinks are the current line's addresses.
      "permalinks.json": PERMALINKS,
      "help.json": { version: 1, build: "site", line: "3.4", help: { connect: connect("3.4") } },
    },
  });
}

/** What `GET pathname` answers on a site with `base`, as Astro routes it: the body and its type, or `undefined` for none. */
function answer(
  siteRoot: string,
  base: string,
  pathname: string,
): ReturnType<typeof helpPaths>[number]["props"] | undefined {
  const root = base.endsWith("/") ? base : `${base}/`;
  for (const pattern of helpPatterns([siteRoot], base)) {
    for (const { params, props } of helpPaths([siteRoot], base, pattern)) {
      const route = `${root}${pattern.slice(1).replace("[id]", params.id)}`;
      if (route === pathname) return props;
    }
  }
  return undefined;
}

/** What `GET pathname` answers as JSON, parsed, or `undefined` for none. */
function get(siteRoot: string, base: string, pathname: string): unknown {
  const props = answer(siteRoot, base, pathname);
  return props === undefined ? undefined : JSON.parse(props.body);
}

describe("a help entry's address", () => {
  it("answers with the current line's entry beside its permalink, and an older line's under its number", () => {
    const site = versioned();
    expect(get(site, "/", "/docs/go/connect.json")).toEqual({
      version: 1,
      build: "site",
      line: "3.5",
      help: { connect: connect("3.5") },
    });
    expect(get(site, "/", "/docs/3.4/go/connect.json")).toEqual({
      version: 1,
      build: "site",
      line: "3.4",
      help: { connect: connect("3.4") },
    });
    expect(get(site, "/", "/docs/3.3/go/connect.json")).toBeUndefined();
  });

  it("is one route for each line, under Astro's base", () => {
    expect(helpPatterns([versioned()], "/")).toEqual([
      "/docs/go/[id].json",
      "/docs/go/[id].md",
      "/docs/3.4/go/[id].json",
      "/docs/3.4/go/[id].md",
    ]);
    expect(helpPatterns([versioned()], "/docs")).toEqual([
      "/go/[id].json",
      "/go/[id].md",
      "/3.4/go/[id].json",
      "/3.4/go/[id].md",
    ]);
    expect(get(versioned(), "/docs", "/docs/3.4/go/connect.json")).toMatchObject({ line: "3.4" });
  });

  it("answers for each name the file has, alias and all, with no line for a build that isn't versioned", () => {
    const site = output({
      site: {
        "permalinks.json": { ...PERMALINKS, path: "/go/" },
        "help.json": {
          version: 1,
          build: "site",
          help: { connect: connect("3.5"), link: connect("3.5") },
        },
      },
    });
    expect([...helpAddresses([site]).keys()]).toEqual(["/go/connect.json", "/go/link.json"]);
    expect(get(site, "/", "/go/link.json")).toEqual({
      version: 1,
      build: "site",
      help: { link: connect("3.5") },
    });
  });

  it("answers with the section as Markdown beside the JSON, which names it as its alternate", () => {
    const site = versioned();
    expect(answer(site, "/", "/docs/go/connect.md")).toEqual({
      body: "# Connect to Quill\n\nConnect the 3.5 agent.\n",
      type: "text/markdown",
    });
    expect(answer(site, "/", "/docs/3.4/go/connect.md")?.body).toBe(
      "# Connect to Quill\n\nConnect the 3.4 agent.\n",
    );
    expect(answer(site, "/", "/docs/go/connect.json")).toMatchObject({
      type: "application/json",
      alternate: "/docs/go/connect.md",
    });
  });

  it("writes a section with no text as its heading alone", () => {
    const help = (text: string) => ({
      version: 1,
      build: "site",
      help: { start: { title: "Start", text, source: "index.md" } },
    });
    expect(helpMarkdown(help(""))).toBe("# Start\n");
    expect(helpMarkdown(help("Text.\n\n"))).toBe("# Start\n\nText.\n");
  });

  it("is no route without permalinks, and none without the file", () => {
    const empty = output({
      site: {
        "permalinks.json": { ...PERMALINKS, permalinks: {} },
        "help.json": { version: 1, build: "site", help: {} },
      },
    });
    expect(helpPatterns([empty], "/")).toEqual([]);
    expect(helpPatterns([output({ site: { "permalinks.json": PERMALINKS } })], "/")).toEqual([]);
  });

  it("serves several projects' entries, the first's for a name two give", () => {
    const first = output({
      site: {
        "permalinks.json": PERMALINKS,
        "help.json": { version: 1, build: "site", help: { connect: connect("first") } },
      },
    });
    const second = output({
      site: {
        "permalinks.json": PERMALINKS,
        "help.json": { version: 1, build: "site", help: { connect: connect("second") } },
      },
    });
    expect(helpAddresses([first, second]).get("/docs/go/connect.json")?.help).toEqual({
      connect: connect("first"),
    });
  });

  it("refuses a file in a format it doesn't read", () => {
    const site = output({ site: { "help.json": { version: 2, build: "site", help: {} } } });
    expect(() => readHelp(site)).toThrow(/version 2 of the help file/);
  });
});
