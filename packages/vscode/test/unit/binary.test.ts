import * as path from "node:path";
import { describe, expect, it } from "vitest";
import {
  ancestorsWithin,
  bundledDirectory,
  executableName,
  projectBinaryName,
  resolveBinary,
  type BinaryEnvironment,
  type ResolveOptions,
} from "../../src/binary.js";
import { parseVersion } from "../../src/version.js";

/** A machine whose files are a map from path to what `--version` prints (or a failure). */
function machine(
  files: Record<string, string | Error>,
  platform: NodeJS.Platform = "linux",
  arch = "x64",
): BinaryEnvironment & { ran: string[] } {
  const ran: string[] = [];
  return {
    ran,
    platform,
    arch,
    fileExists: async (file) => file in files,
    runVersion: async (file) => {
      ran.push(file);
      const result = files[file];
      if (result === undefined) throw new Error("ENOENT");
      if (result instanceof Error) throw result;
      return result;
    },
  };
}

function failed(): never {
  throw new Error("test version didn't parse");
}

const extensionPath = path.resolve("/ext");
const project = path.resolve("/work/docs");
const projectBinary = path.join(project, "node_modules", ".bin", "ascribe");
const bundledBinary = path.join(bundledDirectory(extensionPath, "linux", "x64"), "ascribe");

function options(env: BinaryEnvironment, overrides: Partial<ResolveOptions> = {}): ResolveOptions {
  return {
    setting: "",
    projectRoots: [project],
    extensionPath,
    minVersion: parseVersion("0.2.0") ?? failed(),
    env,
    ...overrides,
  };
}

describe("resolveBinary", () => {
  it("prefers the ascribe.path setting over everything", async () => {
    const env = machine({
      "/custom/ascribe": "ascribe 0.3.0\n",
      [projectBinary]: "ascribe 0.3.0\n",
      [bundledBinary]: "ascribe 0.3.0\n",
    });
    const result = await resolveBinary(options(env, { setting: "/custom/ascribe" }));
    expect(result).toMatchObject({
      kind: "found",
      binary: { path: "/custom/ascribe", source: "setting" },
    });
    expect(env.ran).toEqual(["/custom/ascribe"]);
  });

  it("treats a blank setting as unset", async () => {
    const env = machine({ [projectBinary]: "ascribe 0.3.0\n" });
    const result = await resolveBinary(options(env, { setting: "   " }));
    expect(result).toMatchObject({ kind: "found", binary: { source: "project" } });
  });

  it("fails, rather than falling through, when the setting names no working binary", async () => {
    const env = machine({ [projectBinary]: "ascribe 0.3.0\n", [bundledBinary]: "ascribe 0.3.0\n" });
    const result = await resolveBinary(options(env, { setting: "/nope/ascribe" }));
    expect(result.kind).toBe("missing");
    if (result.kind !== "missing") return;
    expect(result.error.message).toContain("ascribe.path");
    expect(result.error.message).toContain("/nope/ascribe");
    expect(result.error.tried).toEqual(["/nope/ascribe: not found"]);
    expect(env.ran).toEqual([]);
  });

  it("fails when the setting names a file that can't report a version", async () => {
    const env = machine({ "/custom/ascribe": new Error("Exec format error") });
    const result = await resolveBinary(options(env, { setting: "/custom/ascribe" }));
    expect(result.kind).toBe("missing");
    if (result.kind !== "missing") return;
    expect(result.error.tried[0]).toContain("Exec format error");
  });

  it("uses the project's node_modules/.bin/ascribe when the setting is unset", async () => {
    const env = machine({ [projectBinary]: "ascribe 0.3.0\n", [bundledBinary]: "ascribe 0.3.0\n" });
    const result = await resolveBinary(options(env));
    expect(result).toMatchObject({
      kind: "found",
      binary: { path: projectBinary, source: "project" },
    });
  });

  it("tries each project root in order", async () => {
    const parent = path.resolve("/work");
    const parentBinary = path.join(parent, "node_modules", ".bin", "ascribe");
    const env = machine({ [parentBinary]: "ascribe 0.3.0\n" });
    const result = await resolveBinary(options(env, { projectRoots: [project, parent] }));
    expect(result).toMatchObject({ kind: "found", binary: { path: parentBinary } });
  });

  it("skips a project binary that won't run and uses the bundled one", async () => {
    const env = machine({
      [projectBinary]: new Error("EACCES"),
      [bundledBinary]: "ascribe 0.3.0\n",
    });
    const result = await resolveBinary(options(env));
    expect(result).toMatchObject({
      kind: "found",
      binary: { path: bundledBinary, source: "bundled" },
    });
  });

  it("falls back to the bundled binary for the platform and architecture", async () => {
    const macBinary = path.join(bundledDirectory(extensionPath, "darwin", "arm64"), "ascribe");
    const env = machine({ [macBinary]: "ascribe 0.3.0\n" }, "darwin", "arm64");
    const result = await resolveBinary(options(env, { projectRoots: [] }));
    expect(result).toMatchObject({ kind: "found", binary: { path: macBinary, source: "bundled" } });
    expect(macBinary).toContain("darwin-arm64");
  });

  it("names the Windows files: ascribe.exe bundled, ascribe.cmd in node_modules/.bin", async () => {
    expect(executableName("win32")).toBe("ascribe.exe");
    expect(projectBinaryName("win32")).toBe("ascribe.cmd");
    expect(executableName("linux")).toBe("ascribe");
    const winBundled = path.join(bundledDirectory(extensionPath, "win32", "x64"), "ascribe.exe");
    const env = machine({ [winBundled]: "ascribe 0.3.0\n" }, "win32", "x64");
    const result = await resolveBinary(options(env, { projectRoots: [] }));
    expect(result).toMatchObject({ kind: "found", binary: { path: winBundled } });
  });

  it("reports every place it looked, with next steps, when nothing is found", async () => {
    const env = machine({});
    const result = await resolveBinary(options(env));
    expect(result.kind).toBe("missing");
    if (result.kind !== "missing") return;
    expect(result.error.message).toContain("Couldn't find the Ascribe binary");
    expect(result.error.message).toContain("npm install");
    expect(result.error.message).toContain("ascribe.path");
    expect(result.error.tried).toEqual([
      `${projectBinary}: not found`,
      `${bundledBinary}: not found`,
    ]);
  });

  it("treats output without a version number as unusable", async () => {
    const env = machine({ [projectBinary]: "hello\n" });
    const result = await resolveBinary(options(env, { projectRoots: [project] }));
    expect(result.kind).toBe("missing");
    if (result.kind !== "missing") return;
    expect(result.error.tried[0]).toContain("no version number");
  });

  describe("version", () => {
    it("reads the version the binary prints", async () => {
      const env = machine({ [projectBinary]: "ascribe 1.4.2\n" });
      const result = await resolveBinary(options(env));
      expect(result).toMatchObject({ kind: "found", binary: { version: { parts: [1, 4, 2] } } });
    });

    it("warns when the project's binary is older than the extension expects", async () => {
      const env = machine({ [projectBinary]: "ascribe 0.1.9\n" });
      const result = await resolveBinary(options(env));
      expect(result.kind).toBe("found");
      if (result.kind !== "found") return;
      expect(result.binary.warning).toContain("0.1.9");
      expect(result.binary.warning).toContain("0.2.0");
    });

    it("doesn't warn for the expected version or a newer one", async () => {
      for (const printed of ["ascribe 0.2.0", "ascribe 0.2.1", "ascribe 1.0.0"]) {
        const env = machine({ [projectBinary]: printed });
        const result = await resolveBinary(options(env));
        expect(result).toMatchObject({ kind: "found" });
        if (result.kind === "found") expect(result.binary.warning).toBeUndefined();
      }
    });

    it("warns about an older binary named by the setting, too", async () => {
      const env = machine({ "/custom/ascribe": "ascribe 0.0.1" });
      const result = await resolveBinary(options(env, { setting: "/custom/ascribe" }));
      expect(result.kind === "found" && result.binary.warning).toContain("0.0.1");
    });

    it("counts a pre-release as older than its release", async () => {
      const env = machine({ [projectBinary]: "ascribe 0.2.0-beta.1" });
      const result = await resolveBinary(options(env));
      expect(result.kind === "found" && result.binary.warning).toContain("0.2.0-beta.1");
    });
  });
});

describe("ancestorsWithin", () => {
  it("lists the directory and its parents up to the boundary", () => {
    const boundary = path.resolve("/repo");
    const dir = path.join(boundary, "docs", "site");
    expect(ancestorsWithin(dir, boundary)).toEqual([dir, path.join(boundary, "docs"), boundary]);
  });

  it("is just the directory when it is the boundary", () => {
    const boundary = path.resolve("/repo");
    expect(ancestorsWithin(boundary, boundary)).toEqual([boundary]);
  });

  it("is just the directory when it lies outside the boundary", () => {
    const outside = path.resolve("/elsewhere/docs");
    expect(ancestorsWithin(outside, path.resolve("/repo"))).toEqual([outside]);
    // A sibling whose name starts with the boundary's is not inside it.
    const sibling = path.resolve("/repository/docs");
    expect(ancestorsWithin(sibling, path.resolve("/repo"))).toEqual([sibling]);
  });
});
