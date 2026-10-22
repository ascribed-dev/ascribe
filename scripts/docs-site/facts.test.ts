// Facts that several files repeat, each checked against its one home: the
// Node version (`engines.node` in package.json), the Rust version
// (rust-toolchain.toml), the glibc floor (`GLIBC` in release-build.yml), and
// the folders the docs take code examples from (`[sources.code] include` in
// docs/ascribe.toml) and the changelog the site renders (`changelogFile` in
// site/src/changelog.ts), the tables site/home/ascribe.toml copies from
// docs/ascribe.toml, the docs' address the Report workflow checks, and the
// install commands the landing page and the README repeat from getting
// started, and the language server's own requests the editor's language
// server page lists. A copy that can't be generated is compared here, so
// changing one place and not the others fails. The npm platform packages are
// checked the same way in scripts/release/manifests.test.ts.
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "smol-toml";
import { expect, test } from "vitest";
import { changesDir } from "../release/changes.ts";

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
  // Each place, and the setting or sentence there that names a version.
  const found: [string, string][] = [];

  // What each package says it runs on.
  for (const file of tracked("package.json", "**/package.json")) {
    const engine = (JSON.parse(read(file)) as { engines?: { node?: string } }).engines?.node;
    if (engine !== undefined) found.push([file, `engines.node ${engine}`]);
  }
  // What the workflows, and the version managers, set up.
  for (const file of tracked(".nvmrc", "**/.nvmrc")) found.push([file, read(file).trim()]);
  for (const file of tracked("*.yml", "*.yaml").filter((file) => !file.startsWith("tests/"))) {
    const text = read(file);
    // A step that sets up another Node on purpose says so on its line, with
    // `# Not Ascribe's Node:` and why, and isn't counted.
    for (const setting of all(text, /^\s*node-version:\s*(\S+)(?!.*# Not Ascribe's Node:)/gm)) {
      found.push([file, `node-version ${setting}`]);
    }
    for (const setting of all(text, /^\s*node-version-file:\s*(\S+)/gm)) {
      expect(existsSync(path.join(root, setting)), `${file}: ${setting}`).toBe(true);
    }
  }
  // What the docs and the guides for contributors say. The docs write the
  // `{node}` phrase rather than a number.
  found.push(["docs/ascribe.toml", `the node phrase ${phrase("node")}`]);
  const prose = [
    ...tracked("*.md").filter((file) => !file.includes("/")),
    ...tracked("packages/*/README.md", "docs/content/*.md", ".github/workflows/*.yml"),
  ];
  for (const file of prose.filter((file) => file !== "CHANGELOG.md")) {
    for (const major of all(read(file), /\bNode\.js (\d+)/g))
      found.push([file, `Node.js ${major}`]);
  }
  expect(read("docs/content/getting-started.md")).toContain(
    "[Node.js](https://nodejs.org) {node} or later",
  );
  for (const file of tracked("docs/content/*.md")) {
    for (const setting of all(read(file), /node-version:\s*(\S+)/g)) {
      expect(setting, `${file}: write the {node} phrase`).toBe("{node}");
    }
  }

  // The major: the first number in the setting (`>=24`, `24.11.0`).
  const major = (setting: string): string | undefined => /\d+/.exec(setting)?.[0];
  expect(
    found
      .filter(([, setting]) => major(setting) !== version)
      .map(([file, setting]) => `${file}: ${setting}`),
  ).toEqual([]);
  // The workflows, both version files, five packages, the phrase, and CONTRIBUTING.md.
  expect(found.length).toBeGreaterThan(15);
});

test("the Rust version in Cargo.toml is rust-toolchain.toml's", () => {
  const channel = value(table(read("rust-toolchain.toml"), "toolchain"), "channel");
  const minimum = value(table(read("Cargo.toml"), "workspace.package"), "rust-version");
  // `rust-version` is the toolchain's major and minor. (crates/comrak-ascribe
  // keeps its own: it's a fork, and states what upstream supports.)
  expect(channel).toMatch(/^\d+\.\d+\.\d+$/);
  expect(minimum).toBe(channel?.replace(/\.\d+$/, ""));
});

test("the glibc floor is release-build.yml's everywhere it's written", () => {
  const workflow = read(".github/workflows/release-build.yml");
  const floor = /^\s*GLIBC:\s*"([\d.]+)"/m.exec(workflow)?.[1];
  expect(floor).toMatch(/^\d+\.\d+$/);

  expect(phrase("glibc"), "docs/ascribe.toml's glibc phrase").toBe(floor);
  expect(read("docs/content/install.md")).toContain("glibc {glibc} or later");
  // The directives reference and SPEC.md use the floor as their example of
  // prose that differs by release, so it moves with the floor too.
  for (const file of [
    ".github/workflows/release-build.yml",
    "RELEASING.md",
    "packages/cli/README.md",
    "docs/content/writing/directives.md",
    "SPEC.md",
  ]) {
    const named = all(read(file), /\bglibc (\d+\.\d+)/g);
    expect(named.length, `${file} names the floor`).toBeGreaterThan(0);
    expect(new Set(named), file).toEqual(new Set([floor]));
  }
  expect(read("RELEASING.md")).toContain(`--target <triple>.${floor}`);
  expect(read("site/public/install.sh")).toContain(`glibc_floor="${floor}"`);
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

test("the docs' source folders and the changelog are named everywhere a build is started for them", () => {
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
  // The changelog the site renders, at the repository's root.
  const changelog =
    /^export const changelogFile = "([^"]+)";$/m.exec(read("site/src/changelog.ts"))?.[1] ?? "";
  expect(changelog).toBe("CHANGELOG.md");
  // The entries not yet gathered into it, which the site renders in its
  // unreleased section, and the script it gathers them with.
  const gather = "scripts/release/changes.ts";
  expect(read("site/src/changelog.ts")).toContain(`"../../${gather}"`);

  // A folder inside site/, such as the landing page's demonstration, is
  // covered by the site's own pattern in both of the next two.
  const outside = include.filter((glob) => !glob.startsWith("site/"));
  // What the examples were tested with: the source's ascribe.examples.toml,
  // at its root, and the observations beside it, which a page's tested line
  // comes from.
  expect(read("docs/ascribe.toml")).toMatch(/^\[sources\.code\]\npath = "\.\." /m);
  const examplesConfig = "ascribe.examples.toml";
  const tested = [examplesConfig, "examples.observations/**"];
  expect(
    existsSync(path.join(root, examplesConfig)),
    `${examplesConfig} is at the repository's root`,
  ).toBe(true);

  // site-npm.yml builds production's site when one of them changes on `main`.
  const npm = read(".github/workflows/site-npm.yml");
  const paths = /^ {4}paths:\n((?: {6}.*\n)+)/m.exec(npm)?.[1] ?? "";
  expect(all(paths, /^\s*- "([^"]+)"/gm).sort()).toEqual(
    ["docs/**", "site/**", ...outside, ...tested, changelog, `${changesDir}/**`, gather].sort(),
  );

  // Netlify skips a push to `main` when none of them changed.
  const netlify = /^\s*ignore = "git diff .* -- (.*)"$/m.exec(read("site/netlify.toml"))?.[1];
  expect(netlify?.split(" ").sort()).toEqual(
    [
      ".",
      "../docs",
      ...[...outside, ...tested].map((glob) => `../${glob.replace(/\/\*\*$/, "")}`),
      `../${changelog}`,
      `../${changesDir}`,
      `../${gather}`,
    ].sort(),
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
  for (const file of [
    changelog,
    `${changesDir}/an-entry.md`,
    gather,
    examplesConfig,
    "examples.observations/default.toml",
  ]) {
    expect(site.test(file), `ci.yml's site pattern matches ${file}`).toBe(true);
  }
});

test("the site's own pages share the docs' release tables, sources, and address, and the report checks the docs", () => {
  type Toml = Record<string, unknown>;
  const docs = parse(read("docs/ascribe.toml")) as Toml;
  const home = parse(read("site/home/ascribe.toml")) as Toml;
  const at = (toml: Toml, ...keys: string[]): unknown =>
    keys.reduce<unknown>((value, key) => (value as Toml | undefined)?.[key], toml);

  // What the pages' availability marks need, to mark a claim as the docs do.
  for (const keys of [
    ["dimensions", "release"],
    ["releases"],
    ["phrases", "version"],
    ["consumer", "site"],
    ["consumer", "trailing-slash"],
    ["consumer", "permalink-path"],
  ]) {
    expect(at(home, ...keys), keys.join(".")).toEqual(at(docs, ...keys));
    expect(at(docs, ...keys), keys.join(".")).toBeDefined();
  }
  // The `site` phrase, which links into the docs start with.
  expect(at(home, "phrases", "site")).toBe(at(docs, "consumer", "site"));

  // The same files for code examples, from each project's own folder.
  const sources = (toml: Toml, folder: string) => {
    const code = at(toml, "sources", "code") as Toml;
    return { ...code, path: path.resolve(root, folder, String(code["path"])) };
  };
  expect(sources(home, "site/home")).toEqual(sources(docs, "docs"));

  // The nightly report checks the published docs, at their address and base path.
  const report = /--site (\S+)/.exec(read(".github/workflows/report.yml"))?.[1];
  expect(report).toBe(
    `${String(at(docs, "consumer", "site"))}${String(at(docs, "consumer", "base-path"))}`,
  );

  // A preview is built with its own address in each file that names the site's.
  const preview = /^\s*sed -i ".*" (.+)$/m.exec(read(".github/workflows/site.yml"))?.[1];
  expect(preview?.split(" ").sort()).toEqual(
    ["docs/ascribe.toml", "site/home/ascribe.toml"].sort(),
  );
});

/** The text of a Markdown file's `## heading` section, to the next level-2 heading. */
function section(markdown: string, heading: string): string {
  const start = markdown.indexOf(`\n## ${heading}\n`);
  if (start === -1) throw new Error(`no "## ${heading}"`);
  const end = markdown.indexOf("\n## ", start + 1);
  return markdown.slice(start, end === -1 ? undefined : end);
}

/** The lines of each ```sh block in a text. */
function shell(text: string): string[][] {
  return Array.from(text.matchAll(/^```sh\n([\s\S]*?)^```$/gm), ([, block = ""]) =>
    block.trim().split("\n"),
  );
}

test("the landing page and the README install what getting started's first two sections do", () => {
  // Getting started's home for each: its section's first command, the npm arm's.
  const gettingStarted = read("docs/content/getting-started.md");
  const installs = ["Install the command", "Install the editor"].map((heading) => {
    const command = shell(section(gettingStarted, heading))[0]?.[0];
    if (command === undefined) throw new Error(`"${heading}" has no command`);
    return command;
  });
  expect(installs[0]).toMatch(/^npm install /);

  // The landing page shows the two, each in a block of its own.
  const landing = shell(section(read("site/home/content/index.md"), "Start"));
  expect(landing).toEqual(installs.map((command) => [command]));

  // The README's "Get started" block starts with them, then checks.
  const readme = shell(section(read("README.md"), "Get started"))[0];
  expect(readme).toEqual([...installs, "npx ascribe check"]);
});

test("the language server's page lists each request of its own, with its schema", () => {
  const declared = tracked("crates/ascribe-lsp/src/*.rs")
    .flatMap((file) => all(read(file), /pub const \w*METHOD: &str = "(ascribe\/[^"]+)";/g))
    .sort();
  expect(declared.length).toBeGreaterThan(0);
  const page = section(read("docs/content/editor/language-server.md"), "Ascribe's own requests");
  const rows = Array.from(
    page.matchAll(/^\| `(ascribe\/[^`]+)` \|.*\| `([^`]+\.schema\.json)` \|$/gm),
    (row) => ({
      request: row[1] ?? "",
      schema: row[2] ?? "",
    }),
  );
  expect(rows.map((row) => row.request).sort()).toEqual(declared);
  for (const { schema } of rows)
    expect(existsSync(path.join(root, "schemas", schema)), schema).toBe(true);
});
