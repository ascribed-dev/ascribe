import { chmodSync, mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it, vi } from "vitest";
import { findBinary } from "../src/binary.js";

function touch(file: string): string {
  mkdirSync(path.dirname(file), { recursive: true });
  writeFileSync(file, "");
  chmodSync(file, 0o755);
  return file;
}

afterEach(() => vi.unstubAllEnvs());

describe("findBinary", () => {
  it("prefers the option, then ASCRIBE_BIN, then the installed platform package", () => {
    const root = mkdtempSync(path.join(tmpdir(), "ascribe-astro-"));
    const viaOption = touch(path.join(root, "bin", "from-option"));
    const viaEnv = touch(path.join(root, "bin", "from-env"));

    vi.stubEnv("ASCRIBE_BIN", "");
    expect(findBinary({ root })).toMatch(/ascribe(?:\.exe)?$/);
    vi.stubEnv("ASCRIBE_BIN", viaEnv);
    expect(findBinary({ root })).toBe(viaEnv);
    expect(findBinary({ binary: "bin/from-option", root })).toBe(viaOption);
  });

  it("reports named binaries that do not exist", () => {
    const root = mkdtempSync(path.join(tmpdir(), "ascribe-astro-"));
    vi.stubEnv("ASCRIBE_BIN", "");
    expect(() => findBinary({ binary: "nope", root })).toThrow(/`binary` option/);
    vi.stubEnv("ASCRIBE_BIN", path.join(root, "nope"));
    expect(() => findBinary({ root })).toThrow(/ASCRIBE_BIN/);
  });
});
