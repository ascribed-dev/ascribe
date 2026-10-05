import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { outsideAssets, SeenFiles } from "../src/dev.js";
import { readProject } from "../src/project.js";

function project(): string {
  const dir = mkdtempSync(path.join(tmpdir(), "ascribe-astro-dev-"));
  writeFileSync(path.join(dir, "ascribe.toml"), 'spec = "0.1"\n');
  return dir;
}

function manifest(dir: string, files: unknown): void {
  const buildDir = path.join(dir, ".ascribe", "build", "site");
  mkdirSync(buildDir, { recursive: true });
  writeFileSync(
    path.join(buildDir, "site.manifest.json"),
    JSON.stringify({ format: "ascribe-manifest", version: 1, files }),
  );
}

describe("outsideAssets", () => {
  it("lists the assets copied from outside the content root", () => {
    const dir = project();
    manifest(dir, [
      { path: "index.md", kind: "page", source: "index.md" },
      { path: "img/a.png", kind: "asset", source: "img/a.png" },
      { path: "_ascribe/up/shared/logo.png", kind: "asset", source: "../shared/logo.png" },
      {
        path: "_ascribe/files/_ascribe/up/spec.pdf",
        kind: "asset",
        source: "../spec.pdf",
        url: "/_ascribe/files/_ascribe/up/spec.pdf",
      },
      { path: "_ascribe/schema.ts", kind: "generated" },
    ]);
    expect(outsideAssets(readProject(dir), "site")).toEqual([
      path.join(dir, "shared", "logo.png"),
      path.join(dir, "spec.pdf"),
    ]);
  });

  it("gives none without a manifest, or with one it can't read", () => {
    const dir = project();
    expect(outsideAssets(readProject(dir), "site")).toEqual([]);
    manifest(dir, "not a list");
    expect(outsideAssets(readProject(dir), "site")).toEqual([]);
  });
});

describe("SeenFiles", () => {
  it("tells an edit from a late report of one already seen", async () => {
    const dir = mkdtempSync(path.join(tmpdir(), "ascribe-seen-"));
    const file = path.join(dir, "a.md");
    writeFileSync(file, "one");
    const seen = new SeenFiles();
    seen.prime([dir]);
    expect(seen.changed(file)).toBe(false);
    writeFileSync(file, "two, longer");
    expect(seen.changed(file)).toBe(true);
    expect(seen.changed(file)).toBe(false);
    // The same size, written later.
    await new Promise((resolve) => setTimeout(resolve, 20));
    writeFileSync(file, "three, more");
    expect(seen.changed(file)).toBe(true);
    // A new file, and a removed one.
    const added = path.join(dir, "b.md");
    writeFileSync(added, "b");
    expect(seen.changed(added)).toBe(true);
    rmSync(added);
    expect(seen.changed(added)).toBe(true);
    rmSync(dir, { recursive: true });
  });
});
