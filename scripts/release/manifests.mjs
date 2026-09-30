// The files that carry Ascribe's version, and what the release scripts share.
// One version covers the binary (the Cargo workspace), every published npm
// package, and the VS Code extension.
import { readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const root = resolve(fileURLToPath(import.meta.url), "..", "..", "..");

/** The supported platforms: npm's `os-cpu`, which is also `vsce --target`'s name. */
export const targets = [
  { target: "darwin-arm64", rust: "aarch64-apple-darwin", exe: "ascribe" },
  { target: "darwin-x64", rust: "x86_64-apple-darwin", exe: "ascribe" },
  { target: "linux-arm64", rust: "aarch64-unknown-linux-gnu", exe: "ascribe" },
  { target: "linux-x64", rust: "x86_64-unknown-linux-gnu", exe: "ascribe" },
  { target: "win32-x64", rust: "x86_64-pc-windows-msvc", exe: "ascribe.exe" },
];

/**
 * The published npm packages, in the order they're published: a package
 * comes after everything it depends on.
 */
export const npmPackages = [
  ...targets.map(({ target }) => ({
    name: `@ascribed/cli-${target}`,
    dir: `packages/cli/platforms/cli-${target}`,
    target,
  })),
  { name: "@ascribed/cli", dir: "packages/cli" },
  { name: "@ascribed/elements", dir: "packages/elements" },
  { name: "@ascribed/astro", dir: "packages/astro" },
];

export const extension = { id: "Ascribe.ascribe-vscode", dir: "packages/vscode" };

/** `x.y.z` or `x.y.z-pre.n`: what the scripts accept as a version. */
export const VERSION = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/;

export function readJson(path) {
  return JSON.parse(readFileSync(join(root, path), "utf8"));
}

export function writeJson(path, value) {
  writeFileSync(join(root, path), `${JSON.stringify(value, null, 2)}\n`);
}

/** The Cargo workspace's version, from `[workspace.package]`. */
export function cargoVersion() {
  const toml = readFileSync(join(root, "Cargo.toml"), "utf8");
  const match = /\[workspace\.package\][^[]*?\nversion = "([^"]+)"/.exec(toml);
  if (!match) throw new Error("Cargo.toml has no [workspace.package] version");
  return match[1];
}

/** Every place the version is written, and what each says now. */
export function versions() {
  const found = [{ file: "Cargo.toml", version: cargoVersion() }];
  for (const { dir } of [...npmPackages, extension]) {
    const file = `${dir}/package.json`;
    found.push({ file, version: readJson(file).version });
  }
  const vscode = readJson(`${extension.dir}/package.json`);
  found.push({
    file: `${extension.dir}/package.json (ascribe.minServerVersion)`,
    version: vscode.ascribe?.minServerVersion,
  });
  return found;
}

/**
 * Checks that every file carries the same version, that it's a version, that
 * the changelog has a section for it, and, given a tag, that the tag is
 * `v<version>`. Returns the version and the problems found.
 */
export function checkVersion(tag) {
  const found = versions();
  const version = found[0].version;
  const problems = found
    .filter((entry) => entry.version !== version)
    .map((entry) => `${entry.file} says ${entry.version}, but Cargo.toml says ${version}`);
  if (!VERSION.test(version)) problems.push(`${version} isn't a version (x.y.z or x.y.z-pre)`);
  if (tag !== undefined && tag !== `v${version}`) {
    problems.push(`the tag is ${tag}, but the version is ${version}; tag the release v${version}`);
  }
  if (changelogSection(version) === undefined) {
    problems.push(`CHANGELOG.md has no "## ${version}" section`);
  }
  return { version, problems };
}

/** The changelog's section for a version, without its heading, or undefined. */
export function changelogSection(version) {
  const text = readFileSync(join(root, "CHANGELOG.md"), "utf8");
  const lines = text.split("\n");
  const start = lines.findIndex((line) => line.startsWith(`## ${version}`));
  if (start === -1) return undefined;
  const end = lines.findIndex((line, i) => i > start && line.startsWith("## "));
  return lines
    .slice(start + 1, end === -1 ? undefined : end)
    .join("\n")
    .trim();
}
