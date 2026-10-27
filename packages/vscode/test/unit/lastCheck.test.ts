import { mkdirSync, mkdtempSync, realpathSync, rmSync, utimesSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import * as path from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { layoutOf } from "../../src/diskChanges.js";
import { changedSince, projectFiles, readKept, type KeptCheck } from "../../src/lastCheck.js";
import type { Project } from "../../src/projects.js";

const project = (folder: string): Project => ({
  folder,
  config: path.join(folder, "ascribe.toml"),
});

/** A project in docs/ whose code source is the repository, reading only examples/. */
const MODEL = [
  'spec = "0.1"',
  "[project]",
  'content-root = "content"',
  "[sources.code]",
  'path = ".."',
  'include = ["examples/**"]',
].join("\n");

describe("projectFiles", () => {
  let repo: string;
  let docs: Project;
  /** In the past, so every file is older than a check started now. */
  const long = new Date(Date.UTC(2026, 0, 1));

  /** Writes `file` below the repository, dated long ago. */
  function write(file: string, text = ""): string {
    const full = path.join(repo, file);
    mkdirSync(path.dirname(full), { recursive: true });
    writeFileSync(full, text);
    utimesSync(full, long, long);
    return full;
  }

  beforeEach(() => {
    repo = realpathSync(mkdtempSync(path.join(tmpdir(), "last-check-")));
    docs = project(path.join(repo, "docs"));
    write("docs/ascribe.toml", MODEL);
    write("docs/content/index.md", "# Home\n");
    write("docs/memos/index.md");
    write("examples/a/main.rs");
  });

  afterEach(() => {
    rmSync(repo, { recursive: true, force: true });
  });

  const files = (projects = [docs]) => projectFiles(docs, layoutOf(docs, MODEL), projects);

  it("follows the project's own files and those its sources read, and nothing it writes", async () => {
    const before = await files();
    expect(before.count).toBe(4);
    // Its output, its own folder, node_modules, a source's files it doesn't read, and a nested project's.
    write("docs/.ascribe/build/index.html");
    write("docs/.ascribe/cache");
    write("docs/content/node_modules/x/index.js");
    write("target/debug/build.log");
    const nested = project(path.join(repo, "docs/content/nested"));
    write("docs/content/nested/ascribe.toml");
    expect(await files([docs, nested])).toEqual(before);
  });

  it("is a change once a file is modified after the check started, added, or removed", async () => {
    const started = Date.now() - 1000;
    const kept = { started, files: (await files()).list };
    expect(changedSince(kept, await files())).toBe(false);

    // Modified, as a branch switch rewrites it.
    writeFileSync(path.join(repo, "examples/a/main.rs"), "fn main() {}\n");
    expect(changedSince(kept, await files())).toBe(true);
    utimesSync(path.join(repo, "examples/a/main.rs"), long, long);
    expect(changedSince(kept, await files())).toBe(false);

    // Added, even dated before the check.
    const added = write("docs/content/new.md");
    expect(changedSince(kept, await files())).toBe(true);
    rmSync(added);
    expect(changedSince(kept, await files())).toBe(false);

    // Removed.
    rmSync(path.join(repo, "docs/memos/index.md"));
    expect(changedSince(kept, await files())).toBe(true);
  });
});

describe("readKept", () => {
  const kept: KeptCheck = {
    version: "0.4.1",
    started: 5000,
    summary: { errors: 1, warnings: 0, advice: 2 },
    error: null,
    unsaved: 0,
    counted: {
      examples: { count: 0, first: null },
      rechecks: { count: 0, first: null },
      permalinks: { count: 0, first: null },
    },
    problems: [],
    current: true,
    files: "abc",
  };

  it("reads a check the same version kept", () => {
    expect(readKept(JSON.parse(JSON.stringify(kept)), "0.4.1")).toEqual(kept);
  });

  it("ignores one another version kept, or one that can't be read", () => {
    expect(readKept(kept, "0.4.2")).toBeUndefined();
    expect(readKept({ ...kept, summary: null }, "0.4.1")).toBeUndefined();
    expect(readKept({ ...kept, counted: { examples: { count: 1 } } }, "0.4.1")).toBeUndefined();
    expect(readKept("checked", "0.4.1")).toBeUndefined();
    expect(readKept(undefined, "0.4.1")).toBeUndefined();
  });
});
