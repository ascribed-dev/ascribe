import { readFileSync } from "node:fs";
import * as path from "node:path";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  ChangeBatch,
  layoutOf,
  MAX_WAIT_MS,
  PROJECT_DEFAULTS,
  QUIET_MS,
  startsServer,
  type Layout,
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
        "[sources.api]",
        'git = "https://github.com/acme/api.git"',
      ].join("\n"),
    );
    expect(layout).toEqual({
      contentRoot: path.resolve("/repo/docs/pages"),
      outputDir: path.resolve("/repo/out"),
      sources: [path.resolve("/repo")],
    });
  });

  it("gives the defaults for what's missing, and for a model that doesn't parse", () => {
    const defaults: Layout = {
      contentRoot: path.resolve("/repo/docs/docs"),
      outputDir: path.resolve("/repo/docs/.ascribe/build"),
      sources: [],
    };
    expect(layoutOf(docs, 'spec = "0.1"\n')).toEqual(defaults);
    expect(layoutOf(docs, "[project\ncontent-root = ")).toEqual(defaults);
    expect(layoutOf(docs, "[project]\ncontent-root = 3\nsources = 1\n")).toEqual(defaults);
  });

  it("has the content model reference's defaults", () => {
    const reference = readFileSync(
      new URL("../../../../docs/content/reference/content-model.md", import.meta.url),
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
    sources: [path.join(repo, "code")],
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
