import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const targets = [
  ["cli-darwin-arm64", "darwin", "arm64"],
  ["cli-darwin-x64", "darwin", "x64"],
  ["cli-linux-arm64", "linux", "arm64"],
  ["cli-linux-x64", "linux", "x64"],
  ["cli-win32-x64", "win32", "x64"],
] as const;

describe("platform package manifests", () => {
  it("declare the supported operating system and CPU", () => {
    for (const [directory, os, cpu] of targets) {
      const manifest = JSON.parse(
        readFileSync(new URL(`../platforms/${directory}/package.json`, import.meta.url), "utf8"),
      ) as { name: string; os: string[]; cpu: string[]; exports: Record<string, string> };
      expect(manifest.name).toBe(`@ascribed/${directory}`);
      expect(manifest.os).toEqual([os]);
      expect(manifest.cpu).toEqual([cpu]);
      expect(Object.keys(manifest.exports)).toEqual([
        os === "win32" ? "./bin/ascribe.exe" : "./bin/ascribe",
      ]);
    }
  });

  it("keeps installation free of lifecycle scripts", () => {
    const manifest = JSON.parse(
      readFileSync(new URL("../package.json", import.meta.url), "utf8"),
    ) as { scripts: Record<string, string> };
    expect(manifest.scripts.postinstall).toBeUndefined();
    expect(manifest.scripts.install).toBeUndefined();
    expect(manifest.scripts.preinstall).toBeUndefined();
  });
});
