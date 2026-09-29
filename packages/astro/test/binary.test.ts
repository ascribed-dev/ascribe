import { chmodSync, mkdirSync, mkdtempSync, utimesSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it, vi } from "vitest";
import { findBinary } from "../src/binary.js";

const EXE = process.platform === "win32" ? "ascribe.exe" : "ascribe";

function touch(file: string, seconds = 0): string {
  mkdirSync(path.dirname(file), { recursive: true });
  writeFileSync(file, "");
  chmodSync(file, 0o755);
  utimesSync(file, seconds, seconds);
  return file;
}

afterEach(() => vi.unstubAllEnvs());

describe("findBinary", () => {
  it("prefers the option, then ASCRIBE_BIN, then the workspace's target", () => {
    const root = mkdtempSync(path.join(tmpdir(), "ascribe-astro-"));
    const workspace = touch(path.join(root, "target", "debug", EXE));
    const project = path.join(root, "docs", "site");
    mkdirSync(project, { recursive: true });
    const viaOption = touch(path.join(root, "bin", "from-option"));
    const viaEnv = touch(path.join(root, "bin", "from-env"));

    vi.stubEnv("ASCRIBE_BIN", "");
    expect(findBinary({ projectDir: project, root })).toBe(workspace);
    vi.stubEnv("ASCRIBE_BIN", viaEnv);
    expect(findBinary({ projectDir: project, root })).toBe(viaEnv);
    expect(findBinary({ binary: "bin/from-option", projectDir: project, root })).toBe(viaOption);
  });

  it("takes the newer of the release and debug builds", () => {
    const root = mkdtempSync(path.join(tmpdir(), "ascribe-astro-"));
    touch(path.join(root, "target", "debug", EXE), 2_000);
    const release = touch(path.join(root, "target", "release", EXE), 3_000);
    vi.stubEnv("ASCRIBE_BIN", "");
    expect(findBinary({ projectDir: root, root })).toBe(release);
  });

  it("says how to point at one when there is none, or the named one is missing", () => {
    const root = mkdtempSync(path.join(tmpdir(), "ascribe-astro-"));
    vi.stubEnv("ASCRIBE_BIN", "");
    // A temporary directory has no `target/` in any parent.
    expect(() => findBinary({ projectDir: root, root })).toThrow(/cargo build -p tessera-cli/);
    expect(() => findBinary({ binary: "nope", projectDir: root, root })).toThrow(/`binary` option/);
    vi.stubEnv("ASCRIBE_BIN", path.join(root, "nope"));
    expect(() => findBinary({ projectDir: root, root })).toThrow(/ASCRIBE_BIN/);
  });
});
