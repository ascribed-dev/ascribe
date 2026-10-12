// The credits page (site/home/content/credits.md) against what it's about:
// each project it names as a dependency is one, in a Cargo.toml or a
// package.json, so removing a dependency fails here until the page follows;
// the comrak release it names is FORK.md's; and each link it makes to a file
// of the repository on GitHub, which the site's link test doesn't follow,
// leads to a file and a heading that are there. The reverse isn't required:
// the page names the dependencies worth naming, not all of them.
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "smol-toml";
import { expect, test } from "vitest";

const root = fileURLToPath(new URL("../..", import.meta.url));
const PAGE = "site/home/content/credits.md";
const REPO = "https://github.com/ascribed-dev/ascribe";

function read(file: string): string {
  return readFileSync(path.join(root, file), "utf8");
}

/** The tracked files matching git's pathspecs. */
function tracked(...pathspecs: string[]): string[] {
  return execFileSync("git", ["ls-files", "--", ...pathspecs], { cwd: root, encoding: "utf8" })
    .split("\n")
    .filter((file) => file !== "");
}

/** A Markdown text's sections at a heading level: each heading's text, and what follows it. */
function sections(markdown: string, level: number): Map<string, string> {
  const marker = "#".repeat(level);
  const found = new Map<string, string>();
  const parts = markdown.split(new RegExp(`^${marker} (.+)$`, "m"));
  for (let i = 1; i < parts.length; i += 2) found.set(parts[i] ?? "", parts[i + 1] ?? "");
  return found;
}

/** Every inline link in a text: its text and destination. */
function links(markdown: string): { text: string; url: string }[] {
  return Array.from(markdown.matchAll(/\[([^\]]+)\]\(([^)\s]+)\)/g), (match) => ({
    text: match[1] ?? "",
    url: match[2] ?? "",
  }));
}

/** A heading's anchor on GitHub. */
function githubSlug(heading: string): string {
  return heading
    .trim()
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\s_-]/gu, "")
    .replace(/\s/g, "-");
}

/** Every dependency's name in the repository's manifests, Rust's and npm's. */
function dependencies(): Set<string> {
  const names = new Set<string>();
  const tables = ["dependencies", "dev-dependencies", "build-dependencies"];
  type Table = Record<string, unknown>;
  for (const file of tracked("Cargo.toml", "**/Cargo.toml")) {
    const manifest = parse(read(file)) as Table;
    const holders = [manifest, (manifest.workspace ?? {}) as Table];
    for (const target of Object.values((manifest.target ?? {}) as Table)) {
      holders.push(target as Table);
    }
    for (const holder of holders) {
      for (const table of tables) {
        for (const name of Object.keys((holder[table] ?? {}) as Table)) names.add(name);
      }
    }
  }
  for (const file of tracked("package.json", "**/package.json")) {
    const manifest = JSON.parse(read(file)) as Record<string, Record<string, string> | undefined>;
    for (const table of [
      "dependencies",
      "devDependencies",
      "peerDependencies",
      "optionalDependencies",
    ]) {
      for (const name of Object.keys(manifest[table] ?? {})) names.add(name);
    }
  }
  return names;
}

/**
 * The package a project the page names is, where it isn't the name in
 * lowercase. comrak is the fork's crate, not upstream's, which only the
 * CommonMark suite uses.
 */
const PACKAGES: Record<string, string> = {
  comrak: "comrak-ascribe",
  Playwright: "playwright-core",
  Sätteri: "satteri",
};

test("each project the credits page names as a dependency is one", () => {
  const builtOn = sections(read(PAGE), 2).get("What Ascribe is built on");
  if (builtOn === undefined) throw new Error(`${PAGE} has no "What Ascribe is built on"`);
  const groups = sections(builtOn, 3);
  // The programs a user installs aren't dependencies; the other groups are.
  const programs = "Programs Ascribe runs and doesn't ship";
  expect([...groups.keys()]).toEqual([
    "Inside the binary",
    "In the npm packages",
    programs,
    "Building Ascribe and its site",
  ]);

  const known = dependencies();
  const named: string[] = [];
  const missing: string[] = [];
  for (const [group, text] of groups) {
    if (group === programs) continue;
    for (const { text: name, url } of links(text)) {
      if (url.startsWith(REPO)) continue;
      named.push(name);
      const pkg = PACKAGES[name] ?? name.toLowerCase();
      if (!known.has(pkg)) missing.push(`${name} (${pkg}), in "${group}"`);
    }
  }
  expect(named).toContain("comrak");
  expect(named.length).toBeGreaterThan(10);
  expect(missing, "in no Cargo.toml or package.json: change the page").toEqual([]);
});

test("the credits page's comrak release is FORK.md's", () => {
  const release = /^\| Release \| \*\*(v[^*]+)\*\*/m.exec(read("crates/comrak-ascribe/FORK.md"));
  if (release === null) throw new Error("FORK.md names no release");
  const phrases = (parse(read("site/home/ascribe.toml")) as { phrases: Record<string, string> })
    .phrases;
  expect(phrases["comrak-release"], "site/home/ascribe.toml's comrak-release").toBe(release[1]);
  expect(read(PAGE)).toContain("{comrak-release}");
});

test("each link the credits page makes to a file of the repository leads to it", () => {
  const files = new Set(tracked());
  const prefix = `${REPO}/blob/main/`;
  const found = links(read(PAGE)).filter(({ url }) => url.startsWith(prefix));
  expect(found.length).toBeGreaterThan(5);
  const broken: string[] = [];
  for (const { url } of found) {
    const [file = "", fragment] = url.slice(prefix.length).split("#");
    if (!files.has(file)) {
      broken.push(`${url} (no such file)`);
      continue;
    }
    if (fragment === undefined) continue;
    const headings = Array.from(read(file).matchAll(/^#{1,6} (.+)$/gm), (m) =>
      githubSlug(m[1] ?? ""),
    );
    if (!headings.includes(fragment)) broken.push(`${url} (no heading "${fragment}")`);
  }
  expect(broken).toEqual([]);
});
