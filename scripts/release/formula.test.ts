// The Homebrew formula formula.ts writes from a release's SHA256SUMS. CI
// installs the last release's formula with Homebrew (ci.yml, `install`).
import { readFileSync } from "node:fs";
import path from "node:path";
import { expect, test } from "vitest";
import { PLATFORMS, formula, readSums } from "./formula.ts";

const hash = (n: number) => String(n).repeat(64);
const sums = [
  `${hash(1)}  THIRD-PARTY-NOTICES`,
  `${hash(2)}  github/ascribe-1.2.3-darwin-arm64.tar.gz`,
  `${hash(3)}  github/ascribe-1.2.3-linux-arm64.tar.gz`,
  `${hash(4)}  github/ascribe-1.2.3-linux-x64.tar.gz`,
  `${hash(5)}  github/ascribe-1.2.3-win32-x64.zip`,
  `${hash(6)}  npm/ascribed-cli-1.2.3.tgz`,
  "",
].join("\n");

test("reads SHA256SUMS by file name, without the folder", () => {
  const read = readSums(sums);
  expect(read.get("ascribe-1.2.3-linux-x64.tar.gz")).toBe(hash(4));
  expect(read.get("THIRD-PARTY-NOTICES")).toBe(hash(1));
  expect(read.size).toBe(6);
});

test("names each platform's archive at the release's address, with its hash", () => {
  const text = formula("1.2.3", sums);
  expect(text).toContain('version "1.2.3"');
  for (const [target, n] of [
    ["darwin-arm64", 2],
    ["linux-arm64", 3],
    ["linux-x64", 4],
  ] as const) {
    const url = `https://github.com/ascribed-dev/ascribe/releases/download/v1.2.3/ascribe-1.2.3-${target}.tar.gz`;
    expect(text).toContain(`url "${url}"\n      sha256 "${hash(n)}"`);
  }
  expect(text).not.toContain("win32");
  expect(text).toMatch(/on_macos do\n {4}depends_on arch: :arm64\n {4}on_arm do/);
  expect(text).toMatch(
    /on_linux do\n {4}on_arm do[\s\S]*linux-arm64[\s\S]*on_intel do[\s\S]*linux-x64/,
  );
});

test("stops when SHA256SUMS lacks a platform, or the version isn't one", () => {
  expect(() => formula("1.2.3", sums.replace(/.*linux-arm64.*\n/, ""))).toThrow(
    "SHA256SUMS has no ascribe-1.2.3-linux-arm64.tar.gz",
  );
  expect(() => formula("1.2.4", sums)).toThrow(
    "SHA256SUMS has no ascribe-1.2.4-darwin-arm64.tar.gz",
  );
  expect(() => formula("main", sums)).toThrow("isn't a version");
});

test("has a block for each target a release builds, but Windows's", () => {
  const workflow = readFileSync(
    path.join(import.meta.dirname, "../../.github/workflows/release-build.yml"),
    "utf8",
  );
  const build = workflow.slice(workflow.indexOf("  build:"), workflow.indexOf("  pack:"));
  const released = Array.from(build.matchAll(/\{ target: ([\w-]+),/g), ([, target = ""]) => target)
    .filter((target) => !target.startsWith("win32"))
    .sort((a, b) => a.localeCompare(b));
  expect(released.length).toBeGreaterThan(0);
  expect(PLATFORMS.map((p) => p.target).sort((a, b) => a.localeCompare(b))).toEqual(released);
});
