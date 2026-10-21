import * as path from "node:path";
import { describe, expect, it } from "vitest";
import type { Project } from "../../src/projects.js";
import {
  definedProject,
  isAscribeTask,
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

describe("taskLabel", () => {
  it("names the project only when given", () => {
    expect(taskLabel("build")).toBe("Build");
    expect(taskLabel("syncAgentFiles", "handbook")).toBe("Sync Agent Files (handbook)");
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
    expect(isAscribeTask({ type: "ascribe", task: "observeExamples" })).toBe(false);
    expect(isAscribeTask({ type: "ascribe", task: "release" })).toBe(false);
    expect(isAscribeTask({ type: "ascribe", task: "toString" })).toBe(false);
    expect(isAscribeTask({ type: "shell", task: "build" })).toBe(false);
    expect(isAscribeTask({ type: "ascribe", task: "build", project: 1 })).toBe(false);
  });
});
