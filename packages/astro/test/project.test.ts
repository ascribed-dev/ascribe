import { mkdtempSync, mkdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { consumerMismatches, normalizeBase, readProject } from "../src/project.js";

function project(toml: string): string {
  const dir = mkdtempSync(path.join(tmpdir(), "ascribe-astro-"));
  writeFileSync(path.join(dir, "ascribe.toml"), toml);
  return dir;
}

describe("readProject", () => {
  it("reads the output directory and consumer settings", () => {
    const dir = project(`
spec = "0.1"
[project]
output-dir = "out"
[consumer]
site = "https://docs.example.com"
base-path = "/docs"
trailing-slash = "never"
`);
    const info = readProject(dir);
    expect(info.consumer).toEqual({
      site: "https://docs.example.com",
      basePath: "/docs/",
      trailingSlash: "never",
    });
    expect(info.siteRoot("site")).toBe(path.join(dir, "out", "site", "site"));
  });

  it("needs no [builds] table: the implicit `site` build has the same output layout", () => {
    const info = readProject(project('spec = "0.1"\n[project]\ncontent-root = "docs"\n'));
    expect(info.siteRoot("site")).toBe(path.join(info.dir, ".ascribe", "build", "site", "site"));
  });

  it("uses the profile's defaults", () => {
    const info = readProject(project('spec = "0.1"\n'));
    expect(info.contentRoot).toBe(path.join(info.dir, "docs"));
    expect(info.consumer).toEqual({ site: undefined, basePath: "/", trailingSlash: "always" });
    expect(info.siteRoot("site")).toBe(path.join(info.dir, ".ascribe", "build", "site", "site"));
  });

  it("says which file it couldn't read", () => {
    const dir = mkdtempSync(path.join(tmpdir(), "ascribe-astro-"));
    mkdirSync(path.join(dir, "empty"));
    expect(() => readProject(path.join(dir, "empty"))).toThrow(/ascribe\.toml/);
  });
});

describe("consumerMismatches", () => {
  const info = readProject(
    project(
      '[consumer]\nsite = "https://a.example"\nbase-path = "/docs/"\ntrailing-slash = "never"\n',
    ),
  );

  it("accepts the same routing however the base path is written", () => {
    expect(
      consumerMismatches(info, {
        base: "/docs",
        trailingSlash: "never",
        site: "https://a.example/",
      }),
    ).toEqual([]);
    expect(
      consumerMismatches(info, { base: "/docs/", trailingSlash: "ignore", site: undefined }),
    ).toEqual([]);
  });

  it("names each difference", () => {
    const problems = consumerMismatches(info, {
      base: "/",
      trailingSlash: "always",
      site: "https://b.example",
    });
    expect(problems).toHaveLength(3);
    expect(problems.join("\n")).toMatch(/base-path.*"\/docs\/".*"\/"/);
    expect(problems.join("\n")).toMatch(/trailing-slash.*"never".*"always"/);
    expect(problems.join("\n")).toMatch(/site/);
  });
});

describe("normalizeBase", () => {
  it("adds the missing slashes", () => {
    expect(normalizeBase("docs")).toBe("/docs/");
    expect(normalizeBase("/")).toBe("/");
  });
});
