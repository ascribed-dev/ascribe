// Facts that several files repeat, each checked against its one home: the
// Node version (`engines.node` in package.json), the Rust version
// (rust-toolchain.toml), the glibc floor (`GLIBC` in release-build.yml), and
// the folders the docs take code examples from (`[sources.code] include` in
// docs/ascribe.toml). A copy that can't be generated is compared here, so
// changing one place and not the others fails. The npm platform packages are
// checked the same way in scripts/release/manifests.test.ts.
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "vitest";

const root = fileURLToPath(new URL("../..", import.meta.url));

function read(file: string): string {
  return readFileSync(path.join(root, file), "utf8");
}

/** The tracked files matching git's pathspecs. */
function tracked(...pathspecs: string[]): string[] {
  return execFileSync("git", ["ls-files", "--", ...pathspecs], { cwd: root, encoding: "utf8" })
    .split("\n")
    .filter((file) => file !== "");
}

/** A table's text in a TOML file: from `[name]` to the next table. */
function table(toml: string, name: string): string {
  const escaped = name.replace(/\./g, "\\.");
  const found = new RegExp(`^\\[${escaped}\\]$([\\s\\S]*?)(?=^\\[|(?![\\s\\S]))`, "m").exec(toml);
  if (found === null) throw new Error(`no [${name}]`);
  return found[1] ?? "";
}

/** A string value in a table's text: `key = "value"`. */
function value(text: string, key: string): string | undefined {
  return new RegExp(`^"?${key}"?\\s*=\\s*"([^"]*)"`, "m").exec(text)?.[1];
}

/** A phrase in docs/ascribe.toml. */
function phrase(name: string): string | undefined {
  return value(table(read("docs/ascribe.toml"), "phrases"), name);
}

/** Every capture of a pattern's first group in a text. */
function all(text: string, pattern: RegExp): string[] {
  return Array.from(text.matchAll(pattern), (match) => match[1] ?? "");
}

/** The Node version Ascribe needs: the major in the root package.json's `engines.node` (`>=24`). */
function node(): string {
  const engines = (JSON.parse(read("package.json")) as { engines: { node: string } }).engines;
  const major = /^>=(\d+)$/.exec(engines.node)?.[1];
  if (major === undefined) throw new Error(`package.json's engines.node isn't >=<major>`);
  return major;
}

test("every file that names the Node version names the one in package.json", () => {
  const version = node();
  const found: string[] = [];

  // What each package says it runs on.
  for (const file of tracked("package.json", "**/package.json")) {
    const engine = (JSON.parse(read(file)) as { engines?: { node?: string } }).engines?.node;
    if (engine !== undefined) found.push(`${file}: engines.node ${engine}`);
  }
  // What the workflows, and the version managers, set up.
  for (const file of tracked(".nvmrc", "**/.nvmrc")) found.push(`${file}: ${read(file).trim()}`);
  for (const file of tracked("*.yml", "*.yaml").filter((file) => !file.startsWith("tests/"))) {
    const text = read(file);
    for (const setting of all(text, /^\s*node-version:\s*(\S+)/gm)) {
      found.push(`${file}: node-version ${setting}`);
    }
    for (const setting of all(text, /^\s*node-version-file:\s*(\S+)/gm)) {
      expect(existsSync(path.join(root, setting)), `${file}: ${setting}`).toBe(true);
    }
  }
  // What the docs and the guides for contributors say. The docs write the
  // `{node}` phrase rather than a number.
  found.push(`docs/ascribe.toml: the node phrase ${phrase("node")}`);
  const prose = [
    ...tracked("*.md").filter((file) => !file.includes("/")),
    ...tracked("packages/*/README.md", "docs/content/*.md", ".github/workflows/*.yml"),
  ];
  for (const file of prose.filter((file) => file !== "CHANGELOG.md")) {
    for (const major of all(read(file), /\bNode\.js (\d+)/g))
      found.push(`${file}: Node.js ${major}`);
  }
  expect(read("docs/content/getting-started.md")).toContain(
    "[Node.js](https://nodejs.org) {node} or later",
  );
  for (const file of tracked("docs/content/*.md")) {
    for (const setting of all(read(file), /node-version:\s*(\S+)/g)) {
      expect(setting, `${file}: write the {node} phrase`).toBe("{node}");
    }
  }

  const named = (entry: string): string | undefined => /(\d+)\D*$/.exec(entry)?.[1];
  expect(found.filter((entry) => named(entry) !== version)).toEqual([]);
  // The workflows, both version files, five packages, the phrase, and CONTRIBUTING.md.
  expect(found.length).toBeGreaterThan(15);
});

test("the Rust version in Cargo.toml is rust-toolchain.toml's", () => {
  const channel = value(table(read("rust-toolchain.toml"), "toolchain"), "channel");
  const minimum = value(table(read("Cargo.toml"), "workspace.package"), "rust-version");
  // `rust-version` is the toolchain's major and minor. (crates/comrak-tessera
  // keeps its own: it's a fork, and states what upstream supports.)
  expect(channel).toMatch(/^\d+\.\d+\.\d+$/);
  expect(minimum).toBe(channel?.replace(/\.\d+$/, ""));
});

test("the glibc floor is release-build.yml's everywhere it's written", () => {
  const workflow = read(".github/workflows/release-build.yml");
  const floor = /^\s*GLIBC:\s*"([\d.]+)"/m.exec(workflow)?.[1];
  expect(floor).toMatch(/^\d+\.\d+$/);

  expect(phrase("glibc"), "docs/ascribe.toml's glibc phrase").toBe(floor);
  expect(read("docs/content/getting-started.md")).toContain("glibc {glibc} or later");
  for (const file of [
    ".github/workflows/release-build.yml",
    "RELEASING.md",
    "packages/cli/README.md",
  ]) {
    const named = all(read(file), /\bglibc (\d+\.\d+)/g);
    expect(named.length, `${file} names the floor`).toBeGreaterThan(0);
    expect(new Set(named), file).toEqual(new Set([floor]));
  }
  expect(read("RELEASING.md")).toContain(`--target <triple>.${floor}`);
});

/** The files the docs take with `@snippet` or `@include` from `code:`, outside code blocks. */
function codeExamples(): string[] {
  const files = new Set<string>();
  for (const page of tracked("docs/content/*.md")) {
    let fence: string | undefined;
    for (const line of read(page).split("\n")) {
      const marker = /^\s*(`{3,}|~{3,})/.exec(line)?.[1];
      if (marker !== undefined) {
        if (fence === undefined) fence = marker;
        else if (marker[0] === fence[0] && marker.length >= fence.length) fence = undefined;
        continue;
      }
      if (fence !== undefined) continue;
      const file = /^@(?:snippet|include)\b.*:\s*code:([^#\s]+)/.exec(line)?.[1];
      if (file !== undefined) files.add(file);
    }
  }
  return [...files].sort();
}

test("the docs' source folders are docs/ascribe.toml's everywhere a build is started for them", () => {
  const include = all(
    /^include\s*=\s*\[([\s\S]*?)\]/m.exec(table(read("docs/ascribe.toml"), "sources.code"))?.[1] ??
      "",
    /"([^"]+)"/g,
  );
  expect(include.length).toBeGreaterThan(0);
  const folders = include.map((glob) => {
    expect(glob, "an include is a whole folder").toMatch(/^[^*]+\/\*\*$/);
    return glob.replace(/\/\*\*$/, "");
  });

  // site-npm.yml builds production's site when one of them changes on `main`.
  const npm = read(".github/workflows/site-npm.yml");
  const paths = /^ {4}paths:\n((?: {6}.*\n)+)/m.exec(npm)?.[1] ?? "";
  expect(all(paths, /^\s*- "([^"]+)"/gm).sort()).toEqual(["docs/**", "site/**", ...include].sort());

  // Netlify skips a push to `main` when none of them changed.
  const netlify = /^\s*ignore = "git diff .* -- (.*)"$/m.exec(read("site/netlify.toml"))?.[1];
  expect(netlify?.split(" ").sort()).toEqual(
    [".", "../docs", ...folders.map((folder) => `../${folder}`)].sort(),
  );

  // CI's site job runs when a file the docs take an example from changed.
  // Its pattern names only the workflows the docs read, not the whole folder.
  const ci = /^\s*site=false\n[\s\S]*?grep -qE '([^']+)'/m.exec(read(".github/workflows/ci.yml"));
  const site = new RegExp(ci?.[1] ?? "(?!)");
  const examples = codeExamples();
  expect(examples.length).toBeGreaterThan(10);
  for (const file of examples) {
    expect(
      folders.some((folder) => file.startsWith(`${folder}/`)),
      `${file} is in [sources.code] include`,
    ).toBe(true);
    expect(site.test(file), `ci.yml's site pattern matches ${file}`).toBe(true);
  }
  expect(site.test("docs/content/index.md")).toBe(true);
});
