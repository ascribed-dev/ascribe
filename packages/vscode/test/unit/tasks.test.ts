import { readFileSync } from "node:fs";
import * as path from "node:path";
import { describe, expect, it } from "vitest";
import type { Project } from "../../src/projects.js";
import {
  definedProject,
  definedSource,
  isAscribeTask,
  projectCommand,
  projectProperty,
  sourceCommand,
  taskLabel,
  testedSources,
} from "../../src/tasks.js";

const project = (folder: string): Project => ({
  folder,
  config: path.join(folder, "ascribe.toml"),
});

const workspace = path.resolve("/repo");
const atRoot = project(workspace);
const docs = project(path.join(workspace, "docs"));

describe("projectCommand", () => {
  it("runs in the project's folder, with no --config when the project is the workspace folder", () => {
    expect(projectCommand("build", atRoot, workspace)).toEqual({ cwd: workspace, args: ["build"] });
    expect(projectCommand("syncAgentFiles", atRoot, workspace)).toEqual({
      cwd: workspace,
      args: ["agents", "sync"],
    });
  });

  it("names the project's ascribe.toml with --config when it isn't the workspace folder", () => {
    expect(projectCommand("build", docs, workspace)).toEqual({
      cwd: docs.folder,
      args: ["build", "--config", docs.config],
    });
    expect(projectCommand("syncAgentFiles", docs, workspace).args).toEqual([
      "agents",
      "sync",
      "--config",
      docs.config,
    ]);
  });

  it("compares folders as paths, so a Windows drive's case doesn't add --config", () => {
    const windows = { folder: "C:\\Repo\\docs", config: "C:\\Repo\\docs\\ascribe.toml" };
    expect(projectCommand("build", windows, "c:\\repo\\docs").args).toEqual(["build"]);
  });
});

describe("Observe Examples", () => {
  const source = (name: string, path: string, more: object = {}) => ({
    name,
    path,
    include: [],
    ignore: [],
    git: null,
    ...more,
  });
  const examples = { valid: true, files: 2, observations: [] };

  it("runs ascribe observe in the source's folder, with no --config, wherever the project is", () => {
    const code = { name: "code", path: "../code" };
    for (const at of [atRoot, docs]) {
      expect(sourceCommand(at, code)).toEqual({
        cwd: path.resolve(at.folder, "../code"),
        args: ["observe"],
      });
    }
    expect(sourceCommand(docs, { name: "here", path: "." }).cwd).toBe(docs.folder);
  });

  it("is offered for each source with tested examples in this repository, as the binary's model says", () => {
    expect(
      testedSources({
        sources: [
          source("code", "code", { examples }),
          source("plain", "plain"),
          source("broken", "broken", { examples: { ...examples, valid: false } }),
          source("api", "sources/api", {
            git: { url: "https://example.com/api.git", branch: null },
            examples,
          }),
        ],
      }),
    ).toEqual([
      { name: "code", path: "code" },
      { name: "broken", path: "broken" },
    ]);
    expect(testedSources({})).toEqual([]);
  });

  it("finds the source a definition names, or the only one when it names none", () => {
    const task = { type: "ascribe", task: "observeExamples" } as const;
    expect(definedSource(task, ["code"])).toBe("code");
    expect(definedSource(task, ["code", "more"])).toBeUndefined();
    expect(definedSource(task, [])).toBeUndefined();
    expect(definedSource({ ...task, source: "more" }, ["code", "more"])).toBe("more");
    expect(definedSource({ ...task, source: "gone" }, ["code"])).toBeUndefined();
  });
});

describe("taskLabel", () => {
  it("names the project and the source only when given", () => {
    expect(taskLabel("build")).toBe("Build");
    expect(taskLabel("syncAgentFiles", "handbook")).toBe("Sync Agent Files (handbook)");
    expect(taskLabel("observeExamples")).toBe("Observe Examples");
    expect(taskLabel("observeExamples", undefined, "code")).toBe("Observe Examples (code)");
    expect(taskLabel("observeExamples", "handbook", "code")).toBe(
      "Observe Examples (handbook, code)",
    );
  });
});

describe("a task's definition", () => {
  it("names a project by its folder from the workspace folder, with forward slashes", () => {
    expect(projectProperty(atRoot, workspace)).toBeUndefined();
    expect(projectProperty(project(path.join(workspace, "a", "b")), workspace)).toBe("a/b");
    expect(projectProperty(project(path.resolve("/elsewhere")), workspace)).toBeUndefined();
  });

  it("finds the project it names, the workspace folder's own when it names none", () => {
    const projects = [atRoot, docs];
    expect(definedProject({ type: "ascribe", task: "build" }, workspace, projects)).toBe(atRoot);
    expect(
      definedProject({ type: "ascribe", task: "build", project: "docs/" }, workspace, projects),
    ).toBe(docs);
    expect(
      definedProject({ type: "ascribe", task: "build", project: "nowhere" }, workspace, projects),
    ).toBeUndefined();
  });

  it("is one of the extension's only with a task it knows and string properties", () => {
    expect(isAscribeTask({ type: "ascribe", task: "build" })).toBe(true);
    expect(isAscribeTask({ type: "ascribe", task: "syncAgentFiles", project: "docs" })).toBe(true);
    expect(isAscribeTask({ type: "ascribe", task: "observeExamples" })).toBe(true);
    expect(isAscribeTask({ type: "ascribe", task: "observeExamples", source: "code" })).toBe(true);
    expect(isAscribeTask({ type: "ascribe", task: "observeExamples", source: 1 })).toBe(false);
    expect(isAscribeTask({ type: "ascribe", task: "release" })).toBe(false);
    expect(isAscribeTask({ type: "ascribe", task: "toString" })).toBe(false);
    expect(isAscribeTask({ type: "shell", task: "build" })).toBe(false);
    expect(isAscribeTask({ type: "ascribe", task: "build", project: 1 })).toBe(false);
  });
});

// VS Code trusts every workspace while it runs extension tests, so Restricted
// Mode can't be opened in an integration test; the provider's source says it
// offers and resolves nothing there.
describe("the task provider in Restricted Mode", () => {
  const provider = readFileSync(new URL("../../src/ui/tasks.ts", import.meta.url), "utf8");

  it("offers no task", () => {
    expect(provider).toMatch(
      /private async provide\(\)[^{]*\{\s*if \(!vscode\.workspace\.isTrusted\) return \[\];/,
    );
  });

  it("resolves none tasks.json names", () => {
    expect(provider).toMatch(
      /private async resolve\([^)]*\)[^{]*\{\s*if \(!vscode\.workspace\.isTrusted \|\|/,
    );
  });

  it("runs none from the Project view, Observe Examples among them", () => {
    expect(provider).toMatch(
      /async run\([^)]*\)[^{]*\{\s*if \(!vscode\.workspace\.isTrusted\) return;/,
    );
  });
});
