import * as path from "node:path";
import { describe, expect, it } from "vitest";
import type { Project } from "../../src/projects.js";
import {
  definedProject,
  examplesCommands,
  isAscribeTask,
  localSources,
  observeCommand,
  observeQuestion,
  projectCommand,
  projectProperty,
  taskLabel,
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

describe("observeCommand", () => {
  it("runs ascribe observe in the source's folder, which takes no --config", () => {
    const folder = path.join(workspace, "code");
    expect(observeCommand({ name: "code", folder })).toEqual({ cwd: folder, args: ["observe"] });
  });
});

describe("localSources", () => {
  it("reads the sources with a path, resolved from the project's folder, and leaves out those in another repository", () => {
    const text = [
      "[sources.code]",
      'path = ".."',
      "[sources.api]",
      'git = "https://github.com/acme/api.git"',
      'path = "api"',
      "[sources.cli]",
      'path = "tools/cli"',
    ].join("\n");
    expect(localSources(docs, text)).toEqual([
      { name: "code", folder: workspace },
      { name: "cli", folder: path.join(docs.folder, "tools", "cli") },
    ]);
  });

  it("gives none for a model that doesn't parse, or has no sources", () => {
    expect(localSources(docs, "[sources.code\n")).toEqual([]);
    expect(localSources(docs, 'spec = "0.1"')).toEqual([]);
  });
});

describe("examplesCommands", () => {
  it("lists the test, then the environment's commands, in order, as observe runs them", () => {
    const text = [
      'include = ["examples/**"]',
      'test = "node --test"',
      "[environment]",
      'node = { label = "Node", command = "node --version" }',
      'npm = { label = "npm", command = "npm --version" }',
    ].join("\n");
    expect(examplesCommands(text)).toEqual(["node --test", "node --version", "npm --version"]);
  });

  it("gives none for a config without commands, and undefined for one that isn't TOML", () => {
    expect(examplesCommands('include = ["examples/**"]')).toEqual([]);
    expect(examplesCommands("test = ")).toBeUndefined();
  });
});

describe("observeQuestion", () => {
  it("names the file and shows each command on a line", () => {
    expect(
      observeQuestion("code/ascribe.examples.toml", ["node --version", "node --test"]),
    ).toEqual({
      message:
        "Observe Examples runs the commands code/ascribe.examples.toml names, as you. Run them?",
      detail: "node --version\nnode --test",
    });
  });

  it("says so when there's no command, or they can't be read", () => {
    expect(observeQuestion("ascribe.examples.toml", []).detail).toBe("It names no command.");
    expect(observeQuestion("ascribe.examples.toml", undefined).detail).toMatch(
      /couldn't read its commands/,
    );
  });
});

describe("taskLabel", () => {
  it("names the project and the source only when given", () => {
    expect(taskLabel("build")).toBe("Build");
    expect(taskLabel("syncAgentFiles", "handbook")).toBe("Sync Agent Files (handbook)");
    expect(taskLabel("observeExamples", undefined, "code")).toBe("Observe Examples (code)");
    expect(taskLabel("observeExamples", "docs", "code")).toBe("Observe Examples (docs, code)");
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
    expect(isAscribeTask({ type: "ascribe", task: "observeExamples", source: "code" })).toBe(true);
    expect(isAscribeTask({ type: "ascribe", task: "release" })).toBe(false);
    expect(isAscribeTask({ type: "ascribe", task: "toString" })).toBe(false);
    expect(isAscribeTask({ type: "shell", task: "build" })).toBe(false);
    expect(isAscribeTask({ type: "ascribe", task: "build", project: 1 })).toBe(false);
  });
});
