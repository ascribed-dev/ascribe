// The generated paths ARCHITECTURE.md's tables name: each matches a file git
// tracks, and the lists other files keep (the mark's assets, the stylesheets
// with generated blocks, `docs/content/_generated/`) are all in them.
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";
import { matches, readWriters, writers, writersOf } from "./generated.ts";

const root = fileURLToPath(new URL("../..", import.meta.url));

const tracked = execFileSync("git", ["ls-files", "-z"], { cwd: root, encoding: "utf8" })
  .split("\0")
  .filter((file) => file !== "");

const all = writers();

/** The names of the writers that take a file. */
const by = (file: string): string[] => writersOf(file, all).map((writer) => writer.name);

test("the tables name the bless tests and the other writers", () => {
  expect(all.length).toBeGreaterThan(15);
  expect(all.map((writer) => writer.name)).toContain("tests/conformance/tests/docs.rs");
  expect(all.map((writer) => writer.name)).toContain("cargo insta review");
  for (const writer of all) expect(writer.paths, writer.name).not.toEqual([]);
});

test("every path the tables name matches a file git tracks", () => {
  const empty = all.flatMap((writer) =>
    [...writer.paths, ...writer.except]
      .filter((pattern) => !tracked.some((file) => matches(pattern, file)))
      .map((pattern) => `${writer.name}: ${pattern}`),
  );
  expect(empty).toEqual([]);
});

test("every file in docs/content/_generated/ has one writer", () => {
  const files = tracked.filter((file) => file.startsWith("docs/content/_generated/"));
  expect(files.length).toBeGreaterThan(50);
  expect(files.filter((file) => by(file).length !== 1)).toEqual([]);
});

test("every image design/assets.json lists is the assets command's", () => {
  const assets = JSON.parse(readFileSync(`${root}/design/assets.json`, "utf8")) as {
    command: string;
    assets: { path: string }[];
  };
  for (const { path } of assets.assets) expect(by(path), path).toEqual([assets.command]);
});

test("every stylesheet with generated blocks is the tokens test's", () => {
  const tokens = readFileSync(`${root}/design/tokens.toml`, "utf8");
  const sheets = new Set(Array.from(tokens.matchAll(/^\[emit\."([^"]+)"\./gm), (m) => m[1] ?? ""));
  for (const sheet of sheets) expect(by(sheet), sheet).toEqual(["scripts/design/tokens.test.ts"]);
});

test("a hand-written file is no writer's", () => {
  for (const file of ["plugins/ascribe/README.md", "ARCHITECTURE.md", "tests/render/README.md"]) {
    expect(by(file), file).toEqual([]);
  }
  expect(by("crates/ascribe-cli/tests/output/check.txt")).toEqual([
    "crates/ascribe-cli/tests/all/output.rs",
  ]);
  expect(by("crates/ascribe-cli/tests/output/agents/quill/AGENTS.md")).toEqual([
    "crates/ascribe-cli/tests/all/agents.rs",
  ]);
});

describe("matches", () => {
  test("takes a file, a folder, and the wildcards", () => {
    expect(matches("design/specimen.html", "design/specimen.html")).toBe(true);
    expect(matches("design/specimen.html", "design/specimen.htm")).toBe(false);
    expect(matches("schemas/", "schemas/a/b.json")).toBe(true);
    expect(matches("schemas/", "schemasx/a.json")).toBe(false);
    expect(matches("a/*.md", "a/b.md")).toBe(true);
    expect(matches("a/*.md", "a/b/c.md")).toBe(false);
    expect(matches("**/snapshots/*.snap", "crates/x/tests/all/snapshots/y.snap")).toBe(true);
    expect(matches("**/snapshots/*.snap", "snapshots/y.snap")).toBe(true);
    expect(matches("**/snapshots/*.snap", "crates/x/snapshot/y.snap")).toBe(false);
    expect(matches("src/pages/[...slug].astro", "src/pages/[...slug].astro")).toBe(true);
    expect(matches("src/pages/[...slug].astro", "src/pages/x.astro")).toBe(false);
  });
});

describe("readWriters", () => {
  test("reads the tables with a Paths column in the section, and their exceptions", () => {
    const markdown = [
      "## Tests",
      "",
      "| Kind | Where |",
      "|---|---|",
      "| Unit | `src/` |",
      "",
      "### `ASCRIBE_BLESS=1`",
      "",
      "| Test | Rewrites | Paths |",
      "|---|---|---|",
      "| `a.rs` | The plugin, all but its README | `p/`, `!p/README.md` |",
      "",
      "| Command | Writes | Paths |",
      "|---|---|---|",
      "| `cargo insta review` | Snapshots | `**/*.snap` |",
      "",
      "## CI",
      "",
      "| Job | Paths |",
      "|---|---|",
      "| `x` | `y/` |",
    ].join("\n");
    expect(readWriters(markdown)).toEqual([
      { name: "a.rs", paths: ["p/"], except: ["p/README.md"] },
      { name: "cargo insta review", paths: ["**/*.snap"], except: [] },
    ]);
  });
});
