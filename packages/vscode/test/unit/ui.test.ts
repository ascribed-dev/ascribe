import * as path from "node:path";
import { describe, expect, it } from "vitest";
import type { ResolvedBinary } from "../../src/binary.js";
import {
  childNodes,
  nodeLook,
  projectNodes,
  statusFor,
  tildeFolder,
  waitingNotice,
  type ActiveFile,
  type ProjectInfo,
} from "../../src/ui/describe.js";

const root = path.resolve("/work/monorepo");
const binary: ResolvedBinary = {
  path: "/usr/local/bin/ascribe",
  source: "setting",
  version: { parts: [0, 2, 0], prerelease: undefined },
};
const builds = [
  { name: "site", editor: true },
  { name: "cloud", editor: false },
];

/** A project of a fake registry, in `root`. */
function project(folder: string, name: string, extra: Partial<ProjectInfo> = {}): ProjectInfo {
  const full = path.join(root, folder);
  return {
    folder: full,
    config: path.join(full, "ascribe.toml"),
    name,
    state: "stopped",
    binary: undefined,
    builds: [],
    build: undefined,
    lens: false,
    everyBuild: false,
    ...extra,
  };
}

const running = { state: "running", binary, builds, build: "site" } as const;

/** The monorepo fixture: docs, handbook, and a project nested in the handbook's pages. */
const registry = (): ProjectInfo[] => [
  project("docs", "docs", { ...running, build: "cloud" }),
  project("handbook", "handbook", running),
  project(path.join("handbook", "pages", "nested"), "handbook/pages/nested", {
    state: "starting",
  }),
];

const file = (relative: string, languageId = "markdown"): ActiveFile => ({
  scheme: "file",
  path: path.join(root, relative),
  languageId,
});

describe("the status bar item", () => {
  it("names the project that owns the active page, and the build you're looking at", () => {
    expect(statusFor(file("docs/docs/index.md"), registry())?.text).toBe(
      "$(book) docs · cloud · problems: site only",
    );
    expect(statusFor(file("handbook/pages/index.md"), registry())?.text).toBe(
      "$(book) handbook · site · problems: this build only",
    );
  });

  it("says when the Problems panel holds the editor build's problems alone", () => {
    // One build: the panel covers every build.
    const one = [project("docs", "docs", { ...running, builds: [{ name: "site", editor: true }] })];
    const single = statusFor(file("docs/docs/index.md"), one);
    expect(single?.text).toBe("$(book) docs · site");
    expect(single?.tooltip).not.toContain("Problems:");
    // A check of every build that's current: the panel covers them all.
    const checked = [project("docs", "docs", { ...running, build: "cloud", everyBuild: true })];
    const all = statusFor(file("docs/docs/index.md"), checked);
    expect(all?.text).toBe("$(book) docs · cloud");
    expect(all?.label).toBe("Ascribe project docs, server running, build cloud");
    expect(all?.tooltip.split("\n").at(-1)).toBe(
      "Problems: every build, from Ascribe: Check Every Build",
    );
    // Otherwise, the editor build's alone.
    const alone = statusFor(file("docs/docs/index.md"), registry());
    expect(alone?.label).toBe(
      "Ascribe project docs, server running, build cloud, problems for site only",
    );
    expect(alone?.tooltip.split("\n").at(-1)).toBe(
      "Problems: site only. Ascribe: Check Every Build adds the problems of the others.",
    );
    // A server that isn't running has no problems to cover.
    const stopped = [project("docs", "docs", { builds, build: "cloud" })];
    expect(statusFor(file("docs/docs/index.md"), stopped)?.text).toBe("$(book) docs");
  });

  it("shows an eye while the build lens dims by the build", () => {
    const projects = [
      project("docs", "docs", { ...running, build: "cloud", lens: true, everyBuild: true }),
    ];
    const status = statusFor(file("docs/docs/index.md"), projects);
    expect(status?.text).toBe("$(eye) docs · cloud");
    expect(status?.tooltip).toContain("Dimming what cloud leaves out");
    expect(status?.label).toBe(
      "Ascribe project docs, server running, build cloud, dimming what it leaves out",
    );
    // A server that isn't running shows its state: there's nothing to dim by.
    const stopped = [project("docs", "docs", { build: "cloud", lens: true })];
    expect(statusFor(file("docs/docs/index.md"), stopped)?.text).toBe("$(book) docs");
  });

  it("names a nested project for its own files, and its state", () => {
    const status = statusFor(file("handbook/pages/nested/content/index.md"), registry());
    expect(status?.text).toBe("$(sync~spin) handbook/pages/nested");
    expect(status?.project.name).toBe("handbook/pages/nested");
  });

  it("shows a failed server, and no build for a server that isn't running", () => {
    const projects = [project("docs", "docs", { ...running, state: "failed" })];
    expect(statusFor(file("docs/docs/index.md"), projects)?.text).toBe("$(warning) docs");
    const stopped = [project("docs", "docs", { build: "cloud" })];
    expect(statusFor(file("docs/docs/index.md"), stopped)?.text).toBe("$(book) docs");
  });

  it("is shown for a project's ascribe.toml", () => {
    expect(statusFor(file("handbook/ascribe.toml", "toml"), registry())?.project.name).toBe(
      "handbook",
    );
    expect(
      statusFor(file("handbook/pages/nested/ascribe.toml", "toml"), registry())?.project.name,
    ).toBe("handbook/pages/nested");
  });

  it("is hidden for other files, a file in no project, and no file", () => {
    expect(statusFor(file("docs/docs/script.js", "javascript"), registry())).toBeUndefined();
    expect(statusFor(file("code/README.md"), registry())).toBeUndefined();
    expect(statusFor(undefined, registry())).toBeUndefined();
    expect(
      statusFor({ ...file("docs/docs/index.md"), scheme: "untitled" }, registry()),
    ).toBeUndefined();
  });

  it("says which projects aren't checked yet, while no page of theirs has started a server", () => {
    const one = [project("docs", "docs"), project("handbook", "handbook", running)];
    expect(waitingNotice(one)?.text).toBe("$(circle-slash) docs: not checked yet");
    expect(waitingNotice(one)?.tooltip).toBe(
      "Not checked yet: docs\nAscribe checks a project once one of its pages is open.",
    );
    const both = [project("docs", "docs"), project("handbook", "handbook")];
    expect(waitingNotice(both)?.text).toBe("$(circle-slash) 2 projects: not checked yet");
    expect(waitingNotice(both)?.label).toContain("docs, handbook");
    // Starting, running, or failed: each says something else, so nothing's waiting.
    expect(waitingNotice(registry())).toBeUndefined();
    expect(waitingNotice([])).toBeUndefined();
  });

  it("says where the project is, the binary, the state, and the build", () => {
    const docs = statusFor(file("docs/docs/index.md"), registry());
    expect(docs?.tooltip.split("\n")).toEqual([
      "Ascribe project docs",
      `Folder: ${path.join(root, "docs")}`,
      `Content model: ${path.join(root, "docs", "ascribe.toml")}`,
      "Binary: /usr/local/bin/ascribe (setting, 0.2.0)",
      "Server: running",
      "Build: cloud (the editor build is site)",
      "Problems: site only. Ascribe: Check Every Build adds the problems of the others.",
    ]);
    const handbook = statusFor(file("handbook/pages/index.md"), registry());
    expect(handbook?.tooltip.split("\n").at(-2)).toBe("Build: site (the editor build)");
  });
});

describe("the Projects view", () => {
  const look = (node: Parameters<typeof nodeLook>[0]) => nodeLook(node, (folder) => folder);

  it("lists every project, started or not, with its folder and its state", () => {
    const nodes = projectNodes(registry());
    expect(nodes.map((node) => look(node))).toEqual([
      expect.objectContaining({
        label: "docs",
        description: path.join(root, "docs"),
        icon: "book",
        expandable: true,
        contextValue: "ascribe.project.running",
      }),
      expect.objectContaining({ label: "handbook", contextValue: "ascribe.project.running" }),
      expect.objectContaining({
        label: "handbook/pages/nested",
        icon: "sync~spin",
        contextValue: "ascribe.project.starting",
      }),
    ]);
  });

  it("gives a running project its ascribe.toml, editor build, and binary", () => {
    const [docs] = projectNodes(registry());
    if (!docs) throw new Error("no project");
    expect(childNodes(docs).map((node) => look(node))).toEqual([
      expect.objectContaining({
        label: "ascribe.toml",
        description: "content model",
        icon: undefined,
        expandable: false,
      }),
      expect.objectContaining({ label: "site", description: "editor build", icon: "package" }),
      expect.objectContaining({
        label: "ascribe 0.2.0",
        description: "setting",
        tooltip: "/usr/local/bin/ascribe (setting, 0.2.0)",
      }),
    ]);
  });

  it("gives a project whose server isn't running only its ascribe.toml", () => {
    const nested = projectNodes(registry())[2];
    if (!nested) throw new Error("no project");
    expect(childNodes(nested).map((node) => look(node).label)).toEqual(["ascribe.toml"]);
    const failed = projectNodes([project("docs", "docs", { ...running, state: "failed" })])[0];
    if (!failed) throw new Error("no project");
    expect(childNodes(failed).map((node) => look(node).label)).toEqual(["ascribe.toml"]);
    expect(look(failed)).toEqual(
      expect.objectContaining({ icon: "warning", contextValue: "ascribe.project.failed" }),
    );
  });

  it("has no children below a project's own", () => {
    const [docs] = projectNodes(registry());
    if (!docs) throw new Error("no project");
    for (const child of childNodes(docs)) expect(childNodes(child)).toEqual([]);
  });

  it("writes a folder in the home directory from ~", () => {
    expect(tildeFolder("/home/kim/docs", "/home/kim")).toBe("~/docs");
    expect(tildeFolder("/home/kim", "/home/kim/")).toBe("~");
    expect(tildeFolder("/home/kimberly/docs", "/home/kim")).toBe("/home/kimberly/docs");
    expect(tildeFolder("C:\\Users\\Kim\\docs", "c:\\users\\kim")).toBe("~\\docs");
    expect(tildeFolder("/srv/docs", "")).toBe("/srv/docs");
  });
});
