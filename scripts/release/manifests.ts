// The files that carry Ascribe's version, and what the release scripts share.
// One version covers the binary (the Cargo workspace), every published npm
// package, and the VS Code extension.
import { readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const root = resolve(fileURLToPath(import.meta.url), "..", "..", "..");

/** A supported platform. */
export interface Target {
  /** npm's `os-cpu`, which is also `vsce --target`'s name. */
  target: string;
  /** The Rust target triple. */
  rust: string;
  /** The binary's file name. */
  exe: string;
}

/** A published npm package: a platform's binary (with its `target`), or a JavaScript package. */
export interface NpmPackage {
  name: string;
  dir: string;
  target?: string;
}

/** The fields of a `package.json` the release scripts read or write. */
export interface PackageManifest {
  name: string;
  version: string;
  publisher?: string;
  repository?: { url?: string };
  ascribe?: { minServerVersion?: string };
  /** The commit a canary was built from; npm shows it as the version's `gitHead`. */
  gitHead?: string;
}

/** The supported platforms. */
export const targets: readonly Target[] = [
  { target: "darwin-arm64", rust: "aarch64-apple-darwin", exe: "ascribe" },
  { target: "linux-arm64", rust: "aarch64-unknown-linux-gnu", exe: "ascribe" },
  { target: "linux-x64", rust: "x86_64-unknown-linux-gnu", exe: "ascribe" },
  { target: "win32-x64", rust: "x86_64-pc-windows-msvc", exe: "ascribe.exe" },
];

/**
 * The published npm packages, in the order they're published: a package
 * comes after everything it depends on.
 */
export const npmPackages: readonly NpmPackage[] = [
  ...targets.map(({ target }) => ({
    name: `@ascribed/cli-${target}`,
    dir: `packages/cli/platforms/cli-${target}`,
    target,
  })),
  { name: "@ascribed/cli", dir: "packages/cli" },
  { name: "@ascribed/elements", dir: "packages/elements" },
  { name: "@ascribed/review", dir: "packages/review" },
  { name: "@ascribed/astro", dir: "packages/astro" },
];

export const extension = { id: "Ascribe.ascribe-vscode", dir: "packages/vscode" };

/** `x.y.z` or `x.y.z-pre.n`: what the scripts accept as a version. */
export const VERSION = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/;

/** A nightly canary's version: `x.y.z-next.n`. */
export const CANARY = /^\d+\.\d+\.\d+-next\.\d+$/;

export function readJson(path: string): PackageManifest {
  return JSON.parse(readFileSync(join(root, path), "utf8")) as PackageManifest;
}

export function writeJson(path: string, value: unknown): void {
  writeFileSync(join(root, path), `${JSON.stringify(value, null, 2)}\n`);
}

/** The Cargo workspace's version, from `[workspace.package]`. */
export function cargoVersion(): string {
  const toml = readFileSync(join(root, "Cargo.toml"), "utf8");
  const version = /\[workspace\.package\][^[]*?\nversion = "([^"]+)"/.exec(toml)?.[1];
  if (version === undefined) throw new Error("Cargo.toml has no [workspace.package] version");
  return version;
}

/**
 * The skill `@ascribed/cli` ships, whose metadata names the version. The
 * binary writes the same file, and `crates/ascribe-cli/src/agents/skill.rs`
 * fails when this copy differs.
 */
export const skill = "packages/cli/skills/ascribe/SKILL.md";

/**
 * The plugin's copy of the skill, and its two manifests, which name the
 * version too. The binary writes them, and
 * `crates/ascribe-cli/src/agents/plugin.rs` fails when they differ.
 */
export const pluginSkill = "plugins/ascribe/skills/ascribe/SKILL.md";
export const pluginManifests = [
  "plugins/ascribe/.claude-plugin/plugin.json",
  "plugins/ascribe/plugin.json",
];

/** The skill's version line, with the version as its one group. */
export const SKILL_VERSION = /^ {2}ascribe-version: "([^"]+)"$/m;

/** A file that carries the version, and the version it says. */
export interface FoundVersion {
  file: string;
  version: string | undefined;
}

/** Every place the version is written, and what each says now. */
export function versions(): FoundVersion[] {
  const found: FoundVersion[] = [{ file: "Cargo.toml", version: cargoVersion() }];
  for (const { dir } of [...npmPackages, extension]) {
    const file = `${dir}/package.json`;
    found.push({ file, version: readJson(file).version });
  }
  const vscode = readJson(`${extension.dir}/package.json`);
  found.push({
    file: `${extension.dir}/package.json (ascribe.minServerVersion)`,
    version: vscode.ascribe?.minServerVersion,
  });
  for (const file of [skill, pluginSkill]) {
    found.push({
      file,
      version: SKILL_VERSION.exec(readFileSync(join(root, file), "utf8"))?.[1],
    });
  }
  for (const file of pluginManifests) {
    found.push({ file, version: readJson(file).version });
  }
  return found;
}

/**
 * Checks that every file carries the same version, that it's a version, that
 * the changelog has a section for it, and, given a tag, that the tag is
 * `v<version>`. Returns the version and the problems found.
 */
export function checkVersion(tag?: string): { version: string; problems: string[] } {
  const found = versions();
  const version = cargoVersion();
  const problems = found
    .filter((entry) => entry.version !== version)
    .map((entry) => `${entry.file} says ${entry.version}, but Cargo.toml says ${version}`);
  if (!VERSION.test(version)) problems.push(`${version} isn't a version (x.y.z or x.y.z-pre)`);
  if (tag !== undefined && tag !== `v${version}`) {
    problems.push(`the tag is ${tag}, but the version is ${version}; tag the release v${version}`);
  }
  // A canary ships what's unreleased, so the unreleased section stands in for its own.
  if (CANARY.test(version)) {
    if (unreleasedSection(readChangelog()) === undefined) {
      problems.push(`CHANGELOG.md has no "## Unreleased" section, which a canary needs`);
    }
  } else if (changelogSection(version) === undefined) {
    problems.push(`CHANGELOG.md has no "## ${version}" section`);
  }
  return { version, problems };
}

function readChangelog(): string {
  return readFileSync(join(root, "CHANGELOG.md"), "utf8");
}

/**
 * The changelog's unreleased section: `## Unreleased`, or `## x.y.z
 * (unreleased)` when it names the version it will be released as. Returns the
 * version it names, if any, or undefined when there's no such section.
 */
export function unreleasedSection(changelog: string): { version: string | undefined } | undefined {
  for (const line of changelog.split("\n")) {
    const match = /^## (?:Unreleased|(\d+\.\d+\.\d+) \(unreleased\))\s*$/i.exec(line);
    if (match) return { version: match[1] };
  }
  return undefined;
}

/**
 * A canary's version: the version after `released` (the workspace's version,
 * the latest release), then `-next.<run>`. The next version is the one the
 * changelog's unreleased section names, or else a patch bump.
 */
export function canaryVersion(released: string, changelog: string, run: number): string {
  const parts = /^(\d+)\.(\d+)\.(\d+)$/.exec(released);
  if (!parts) throw new Error(`${released} isn't a released version (x.y.z)`);
  if (!Number.isSafeInteger(run) || run < 1) throw new Error(`${run} isn't a run number`);
  const unreleased = unreleasedSection(changelog);
  if (unreleased === undefined) throw new Error(`CHANGELOG.md has no "## Unreleased" section`);
  const [, major, minor, patch] = parts.map(Number);
  const next = unreleased.version ?? `${major}.${minor}.${Number(patch) + 1}`;
  return `${next}-next.${run}`;
}

/**
 * Cargo.lock with the workspace's crates (the packages without a `source`)
 * moved from one version to another, as `cargo update --workspace` would,
 * without needing Cargo or the registry. Line endings are kept, so it works on
 * a Windows checkout too.
 */
export function relockWorkspace(lock: string, from: string, to: string): string {
  const version = new RegExp(`^version = "${from.replaceAll(".", "\\.")}"(?=\r?$)`, "m");
  return lock
    .split(/(?=^\[\[package\]\]\r?$)/m)
    .map((entry) =>
      /^source = /m.test(entry) ? entry : entry.replace(version, `version = "${to}"`),
    )
    .join("");
}

/** The changelog's section for a version, without its heading, or undefined. */
export function changelogSection(version: string): string | undefined {
  const lines = readChangelog().split("\n");
  const start = lines.findIndex((line) => line.startsWith(`## ${version}`));
  if (start === -1) return undefined;
  const end = lines.findIndex((line, i) => i > start && line.startsWith("## "));
  return lines
    .slice(start + 1, end === -1 ? undefined : end)
    .join("\n")
    .trim();
}
