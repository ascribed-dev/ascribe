import { readFileSync } from "node:fs";
import * as path from "node:path";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  ChangeBatch,
  changesCheck,
  layoutOf,
  MAX_WAIT_MS,
  PROJECT_DEFAULTS,
  QUIET_MS,
  startsServer,
  toldOfChange,
  type Layout,
  type Source,
} from "../../src/diskChanges.js";
import type { Project } from "../../src/projects.js";

const project = (folder: string): Project => ({
  folder,
  config: path.join(folder, "ascribe.toml"),
});

describe("layoutOf", () => {
  const docs = project(path.resolve("/repo/docs"));

  it("reads the content root, the output directory, and the sources' folders", () => {
    const layout = layoutOf(
      docs,
      [
        'spec = "0.1"',
        "[project]",
        'content-root = "pages"',
        'output-dir = "../out"',
        "[sources.code]",
        'path = ".."',
        'include = ["crates/**", 3]',
        'ignore = ["**/node_modules/**"]',
        "[sources.api]",
        'git = "https://github.com/acme/api.git"',
      ].join("\n"),
    );
    expect(layout).toEqual({
      contentRoot: path.resolve("/repo/docs/pages"),
      outputDir: path.resolve("/repo/out"),
      sources: [
        {
          folder: path.resolve("/repo"),
          include: ["crates/**"],
          ignore: ["**/node_modules/**"],
        },
      ],
      from: [],
    });
  });

  it("names the files `[from]` takes from, without reading them", () => {
    const layout = layoutOf(
      docs,
      ['spec = "0.1"', '[from."../site/ascribe.toml"]', 'take = ["sources.code"]'].join("\n"),
    );
    expect(layout.from).toEqual([path.resolve("/repo/site/ascribe.toml")]);
    expect(layout.sources).toEqual([]);
  });

  it("gives the defaults for what's missing, and for a model that doesn't parse", () => {
    const defaults: Layout = {
      contentRoot: path.resolve("/repo/docs/docs"),
      outputDir: path.resolve("/repo/docs/.ascribe/build"),
      sources: [],
      from: [],
    };
    expect(layoutOf(docs, 'spec = "0.1"\n')).toEqual(defaults);
    expect(layoutOf(docs, "[project\ncontent-root = ")).toEqual(defaults);
    expect(layoutOf(docs, "[project]\ncontent-root = 3\nsources = 1\n")).toEqual(defaults);
  });

  it("has the content model reference's defaults", () => {
    const reference = readFileSync(
      new URL("../../../../docs/content/_generated/content-model-project.md", import.meta.url),
      "utf8",
    );
    for (const [key, value] of Object.entries(PROJECT_DEFAULTS)) {
      expect(reference).toMatch(
        new RegExp(`^\\| \`${key}\` \\| string \\(path\\) \\| \`"${value}"\` \\|`, "m"),
      );
    }
  });
});

describe("startsServer", () => {
  const repo = path.resolve("/repo");
  const handbook = project(path.join(repo, "handbook"));
  const nested = project(path.join(repo, "handbook", "pages", "nested"));
  const projects = [handbook, nested];
  const layout: Layout = {
    contentRoot: path.join(repo, "handbook", "pages"),
    outputDir: path.join(repo, "handbook", ".ascribe", "build"),
    sources: [{ folder: path.join(repo, "code"), include: [], ignore: [] }],
    from: [path.join(repo, "docs", "ascribe.toml")],
  };
  const starts = (...segments: string[]) =>
    startsServer(path.join(repo, ...segments), handbook, layout, projects);

  it("starts for a file under the content root, page or not, and for the folder itself", () => {
    expect(starts("handbook", "pages", "index.md")).toBe(true);
    expect(starts("handbook", "pages", "guides", "new.md")).toBe(true);
    expect(starts("handbook", "pages", "images", "logo.png")).toBe(true);
    expect(starts("handbook", "pages", "guides")).toBe(true);
  });

  it("starts for the project's ascribe.toml, and not a nested project's", () => {
    expect(starts("handbook", "ascribe.toml")).toBe(true);
    expect(starts("handbook", "pages", "nested", "ascribe.toml")).toBe(false);
  });

  it("starts for a file in a source's folder", () => {
    expect(starts("code", "src", "main.rs")).toBe(true);
  });

  it("starts for a file `[from]` takes from, in another project", () => {
    expect(starts("docs", "ascribe.toml")).toBe(true);
    expect(starts("docs", "index.md")).toBe(false);
  });

  it("doesn't start for a file of a nested project, or outside the content root", () => {
    expect(starts("handbook", "pages", "nested", "content", "index.md")).toBe(false);
    expect(starts("handbook", "README.md")).toBe(false);
    expect(starts("elsewhere", "index.md")).toBe(false);
    expect(starts("handbook", "pages-old", "index.md")).toBe(false);
  });

  it("doesn't start for the output directory, node_modules, or .git", () => {
    expect(starts("handbook", ".ascribe", "build", "site", "plain", "index.md")).toBe(false);
    expect(starts("handbook", "pages", "node_modules", "x", "README.md")).toBe(false);
    expect(starts("code", "node_modules", "x", "index.js")).toBe(false);
    expect(starts("code", ".git", "index")).toBe(false);
  });

  it("doesn't start for an output directory inside a source's folder", () => {
    const inSource: Layout = { ...layout, outputDir: path.join(repo, "code", "out") };
    expect(startsServer(path.join(repo, "code", "out", "a.md"), handbook, inSource, projects)).toBe(
      false,
    );
  });

  it("folds the case of Windows paths", () => {
    const windows = project("C:\\Repo\\handbook");
    const windowsLayout: Layout = {
      contentRoot: "C:\\Repo\\handbook\\pages",
      outputDir: "C:\\Repo\\handbook\\.ascribe\\build",
      sources: [],
      from: [],
    };
    expect(startsServer("c:\\repo\\HANDBOOK\\pages\\a.md", windows, windowsLayout, [windows])).toBe(
      true,
    );
    expect(
      startsServer("c:\\repo\\handbook\\pages\\Node_Modules\\a.md", windows, windowsLayout, [
        windows,
      ]),
    ).toBe(false);
  });
});

describe("changesCheck", () => {
  const repo = path.resolve("/repo");
  const handbook = project(path.join(repo, "handbook"));
  const nested = project(path.join(repo, "handbook", "pages", "nested"));
  const projects = [handbook, nested];
  const layout: Layout = {
    contentRoot: path.join(repo, "handbook", "pages"),
    outputDir: path.join(repo, "handbook", "out"),
    sources: [{ folder: path.join(repo, "code"), include: [], ignore: [] }],
    from: [],
  };
  const changes = (...segments: string[]) =>
    changesCheck(path.join(repo, ...segments), handbook, layout, projects);

  it("counts what startsServer counts", () => {
    expect(changes("handbook", "ascribe.toml")).toBe(true);
    expect(changes("handbook", "pages", "index.md")).toBe(true);
    expect(changes("handbook", "pages", "images", "logo.png")).toBe(true);
    expect(changes("code", "src", "main.rs")).toBe(true);
  });

  it("counts the project's files beside ascribe.toml, which the check reads too", () => {
    for (const file of ["ascribe.nav", "ascribe.permalinks", "ascribe.lock", "README.md"]) {
      expect(changes("handbook", file)).toBe(true);
      expect(startsServer(path.join(repo, "handbook", file), handbook, layout, projects)).toBe(
        false,
      );
    }
    expect(changes("handbook", "memos", "index.md")).toBe(true);
    expect(changes("handbook", "templates", "how-to.md")).toBe(true);
    expect(changes("handbook", "memos")).toBe(true);
  });

  it("doesn't count a nested project's files, or files outside the project and its sources", () => {
    expect(changes("handbook", "pages", "nested", "ascribe.toml")).toBe(false);
    expect(changes("handbook", "pages", "nested", "content", "index.md")).toBe(false);
    expect(changes("elsewhere", "index.md")).toBe(false);
    expect(changes("handbook-old", "ascribe.nav")).toBe(false);
  });

  it("doesn't count the output directory, .ascribe, node_modules, or .git", () => {
    expect(changes("handbook", "out", "site", "index.html")).toBe(false);
    expect(changes("handbook", ".ascribe", "vale", ".vale.ini")).toBe(false);
    expect(changes("handbook", ".ascribe", "cache", "links", "ab", "x.json")).toBe(false);
    expect(changes("handbook", "node_modules", "x", "README.md")).toBe(false);
    expect(changes("handbook", ".git", "index")).toBe(false);
    expect(changes("code", ".git", "index")).toBe(false);
  });

  it("folds the case of Windows paths", () => {
    const windows = project("C:\\Repo\\handbook");
    const windowsLayout: Layout = {
      contentRoot: "C:\\Repo\\handbook\\pages",
      outputDir: "C:\\Repo\\handbook\\.ascribe\\build",
      sources: [],
      from: [],
    };
    expect(changesCheck("c:\\repo\\HANDBOOK\\ascribe.nav", windows, windowsLayout, [windows])).toBe(
      true,
    );
    expect(
      changesCheck("c:\\repo\\handbook\\.Ascribe\\vale\\a.ini", windows, windowsLayout, [windows]),
    ).toBe(false);
  });
});

describe("ChangeBatch", () => {
  let flushed: string[][];
  let batch: ChangeBatch;

  beforeEach(() => {
    vi.useFakeTimers();
    flushed = [];
    batch = new ChangeBatch((files) => flushed.push(files));
  });

  afterEach(() => {
    batch.dispose();
    vi.useRealTimers();
  });

  it("hands over a burst once it goes quiet, each file once", () => {
    for (let i = 0; i < 400; i++) batch.add(path.resolve(`/repo/docs/docs/page-${i % 200}.md`));
    vi.advanceTimersByTime(QUIET_MS - 1);
    expect(flushed).toEqual([]);
    vi.advanceTimersByTime(1);
    expect(flushed.length).toBe(1);
    expect(flushed[0]?.length).toBe(200);
  });

  it("waits again for each change in a burst, up to the longest wait", () => {
    batch.add("/a.md");
    vi.advanceTimersByTime(QUIET_MS - 10);
    batch.add("/b.md");
    vi.advanceTimersByTime(QUIET_MS - 10);
    expect(flushed).toEqual([]);
    vi.advanceTimersByTime(10);
    expect(flushed).toEqual([["/a.md", "/b.md"]]);
  });

  it("hands over a burst that never goes quiet after the longest wait", () => {
    const step = QUIET_MS / 2;
    for (let t = 0; t < MAX_WAIT_MS; t += step) {
      batch.add(`/file-${t}.md`);
      vi.advanceTimersByTime(step);
    }
    expect(flushed.length).toBe(1);
    expect(flushed[0]?.length).toBe(MAX_WAIT_MS / step);
    // The next change starts a new batch.
    batch.add("/later.md");
    vi.advanceTimersByTime(QUIET_MS);
    expect(flushed.at(-1)).toEqual(["/later.md"]);
  });

  it("hands over nothing once disposed", () => {
    batch.add("/a.md");
    batch.dispose();
    vi.advanceTimersByTime(MAX_WAIT_MS);
    expect(flushed).toEqual([]);
  });
});

describe("toldOfChange", () => {
  const root = path.resolve("/work");
  const docs = project(path.join(root, "docs"));
  const api = project(path.join(root, "api"));
  const nested = project(path.join(root, "docs/nested"));
  const projects = [docs, api, nested];
  const at = (file: string) => path.join(root, file);
  const everything = (folder: string): Source => ({ folder, include: [], ignore: [] });

  it("tells a server of its own files", () => {
    expect(toldOfChange(at("docs/content/a.md"), docs, [], projects)).toBe(true);
    expect(toldOfChange(at("docs/ascribe.toml"), docs, [], projects)).toBe(true);
  });

  it("tells a server of a file its `[from]` takes from, in another project", () => {
    const from = [at("api/ascribe.toml")];
    expect(toldOfChange(at("api/ascribe.toml"), docs, [], projects, from)).toBe(true);
    expect(toldOfChange(at("api/ascribe.toml"), docs, [], projects)).toBe(false);
    expect(toldOfChange(at("api/content/a.md"), docs, [], projects, from)).toBe(false);
  });

  it("tells a server of a file in its sources' folder that's in no project", () => {
    const sources = [everything(root)];
    expect(toldOfChange(at("crates/cli/tests/output/a.txt"), docs, sources, projects)).toBe(true);
    expect(toldOfChange(at("examples.observations/default.toml"), docs, sources, projects)).toBe(
      true,
    );
    expect(toldOfChange(at("crates/cli/tests/output/a.txt"), docs, [], projects)).toBe(false);
  });

  it("tells a server of a file in its sources' folder that's in another project", () => {
    const sources = [everything(at("api/code"))];
    expect(toldOfChange(at("api/code/greet.py"), docs, sources, projects)).toBe(true);
    expect(toldOfChange(at("api/content/a.md"), docs, sources, projects)).toBe(false);
    expect(toldOfChange(at("docs/nested/code/x.ts"), docs, [], projects)).toBe(false);
    expect(
      toldOfChange(at("docs/nested/code/x.ts"), docs, [everything(at("docs/nested"))], projects),
    ).toBe(true);
  });

  it("doesn't tell a server of node_modules or .git in its sources' folder", () => {
    const sources = [everything(root)];
    expect(toldOfChange(at("node_modules/a/index.js"), docs, sources, projects)).toBe(false);
    expect(toldOfChange(at(".git/index"), docs, sources, projects)).toBe(false);
  });

  it("tells a server only of the files in its sources' folder that a source can use", () => {
    const sources: Source[] = [
      {
        folder: root,
        include: ["crates/cli/tests/output/**", "examples/{a,b}/*.toml"],
        ignore: ["**/node_modules/**"],
      },
    ];
    const told = (file: string) => toldOfChange(at(file), docs, sources, projects);
    // What `include` covers, and a folder that may hold it.
    expect(told("crates/cli/tests/output/a.txt")).toBe(true);
    expect(told("crates/cli")).toBe(true);
    expect(told("examples/b/x.toml")).toBe(true);
    expect(told("examples/b")).toBe(true);
    // The source's examples, whatever `include` says.
    expect(told("ascribe.examples.toml")).toBe(true);
    expect(told("examples.observations")).toBe(true);
    expect(told("examples.observations/default.toml")).toBe(true);
    // A build's output, and what `include` doesn't cover or `ignore` leaves out.
    expect(told("target")).toBe(false);
    expect(told("target/debug/build/a.d")).toBe(false);
    expect(told("crates/core/src/lib.rs")).toBe(false);
    expect(told("examples/c/x.toml")).toBe(false);
    expect(told("examples/b/x.md")).toBe(false);
    expect(told("examples/b/node_modules")).toBe(false);
    expect(told("docs/nested/ascribe.examples.toml")).toBe(false);
    // Its own files, whatever its sources read.
    expect(told("docs/content/a.md")).toBe(true);
  });

  it("reads a pattern as the content model does", () => {
    const told = (include: string, file: string) =>
      toldOfChange(at(file), docs, [{ folder: root, include: [include], ignore: [] }], projects);
    expect(told("a\\*.txt", "a*.txt")).toBe(true);
    expect(told("a\\*.txt", "ab.txt")).toBe(false);
    expect(told("a?.txt", "ab.txt")).toBe(true);
    expect(told("a?.txt", "a.txt")).toBe(false);
    expect(told("**/*.rs", "src/a/b.rs")).toBe(true);
    expect(told("*.rs", "src/b.rs")).toBe(false);
    expect(told("(a).txt", "(a).txt")).toBe(true);
    // One the content model wouldn't load covers everything.
    expect(told("{a,b", "target/a.d")).toBe(true);
  });
});
